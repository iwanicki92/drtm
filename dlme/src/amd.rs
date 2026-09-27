// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! What AMD's SL leaves the OS SL, in the area `OsSlParametersOffset`
//! names: a header, then the tables it names by offsets from itself.
//!
//! No public document gives the layout. It is read from AMD's SL, see
//! `docs/foreign-sl.md`, so everything here is reported as `info`.

use core::slice;

use common::out::Out;
use common::params::{DLME_BLOCK_SIZE, Params};
use common::record::{Tag, Value};
use common::tcg_log;
use common::tpm::DrtmPcrs;

unsafe extern "C" {
    static __dlme_start: u8;
    static __dlme_end: u8;
}

/// What the DLME reads of the header, which is `0x54` bytes in the EPYC
/// SL's version 2 and `0x7c` in the client SL's version 4, and each
/// descriptor's length. The header's `Size` covers it and its tables.
const HEADER_LEN: usize = 0x40;
const DESCRIPTOR_LEN: usize = 0x20;

/// A descriptor's type word, for a TMR.
const TMR: u32 = 2;

/// The area, from the end of the DLME's scratch memory to the end of its
/// block, where the preamble told the SL to put it.
fn area(block: u64) -> &'static [u8] {
    // Both are symbols `dlme.ld` defines, and only their addresses are
    // taken.
    let at = &raw const __dlme_end as usize - &raw const __dlme_start as usize;
    let len = DLME_BLOCK_SIZE as usize - at;
    // Safety: the block lies below 4 GiB, which the DLME identity maps, and
    // nothing writes it while the DLME runs.
    unsafe { slice::from_raw_parts((block as usize + at) as *const u8, len) }
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

/// `count` items of `len` bytes, or a size in bytes where `len` is 1, at
/// `offset` from the header, where they fit the header's `size`.
fn table(area: &[u8], size: u32, offset: u32, count: u32, len: usize) -> Option<&[u8]> {
    let start = offset as usize;
    let end = start.checked_add((count as usize).checked_mul(len)?)?;
    (start >= HEADER_LEN && end <= size as usize).then_some(())?;
    area.get(start..end)
}

/// Reports the header, the TMR descriptors, and whether the log replays to
/// `pcrs`.
pub fn report(out: &mut Out, params: &Params, block: u64, pcrs: Option<&DrtmPcrs>) {
    let area = area(block);
    let header = &area[..HEADER_LEN];
    let (version, size) = (u32_at(header, 0), u32_at(header, 4));
    if version == 0 || !(HEADER_LEN..=area.len()).contains(&(size as usize)) {
        out.check(
            Tag::Info,
            "launch.amd_handoff",
            &[
                ("version", Value::Hex(version.into())),
                ("size", Value::Hex(size.into())),
            ],
            format_args!("the SL left no header the DLME knows in its area"),
        );
        return;
    }
    let (memory_at, memory_count) = (u32_at(header, 0x10), u32_at(header, 0x14));
    let (tmrs_at, tmrs_count) = (u32_at(header, 0x18), u32_at(header, 0x1c));
    let (log_at, log_size) = (u32_at(header, 0x20), u32_at(header, 0x24));
    let acpi_size = u32_at(header, 0x2c);
    let smm_size = u32_at(header, 0x34);
    let mailbox = u64_at(header, 0x38);
    let preamble = params.psp.map_or(0, |psp| psp.mailbox);
    out.check(
        Tag::Info,
        "launch.amd_handoff",
        &[
            ("version", Value::Hex(version.into())),
            ("size", Value::Hex(size.into())),
            ("features", Value::Hex(u64_at(header, 0x08))),
            ("memory_descriptors", Value::Count(memory_count.into())),
            ("tmr_descriptors", Value::Count(tmrs_count.into())),
            ("tcg_log", Value::Hex(log_size.into())),
            ("acpi_tables", Value::Hex(acpi_size.into())),
            ("smm", Value::Hex(smm_size.into())),
            ("mailbox", Value::Hex(mailbox)),
            ("preamble_mailbox", Value::Hex(preamble)),
        ],
        format_args!(
            "the SL left {memory_count} memory and {tmrs_count} TMR descriptors, \
             a {log_size:#x}-byte TCG log, {acpi_size:#x} bytes of ACPI tables and \
             {smm_size:#x} of SMM configuration, and names the mailbox at {mailbox:#x} \
             (the preamble's is at {preamble:#x})"
        ),
    );

    if let Some(memory) = table(area, size, memory_at, memory_count, DESCRIPTOR_LEN) {
        for descriptor in memory.as_chunks::<DESCRIPTOR_LEN>().0 {
            let (base, len) = (u64_at(descriptor, 0x10), u64_at(descriptor, 0x18));
            out.progress(
                "launch.amd_memory",
                &[
                    ("attribute", Value::Hex(u32_at(descriptor, 0x0c).into())),
                    ("base", Value::Hex(base)),
                    ("size", Value::Hex(len)),
                ],
                format_args!("memory [{base:#x}, {:#x})", base.saturating_add(len)),
            );
        }
    }
    match table(area, size, tmrs_at, tmrs_count, DESCRIPTOR_LEN) {
        Some(tmrs) => {
            for descriptor in tmrs.as_chunks::<DESCRIPTOR_LEN>().0 {
                let kind = u32_at(descriptor, 4);
                let index = u32_at(descriptor, 0x0c);
                let (base, len) = (u64_at(descriptor, 0x10), u64_at(descriptor, 0x18));
                out.check(
                    Tag::Info,
                    "launch.amd_tmr",
                    &[
                        ("type", Value::Hex(kind.into())),
                        ("index", Value::Count(index.into())),
                        ("base", Value::Hex(base)),
                        ("size", Value::Hex(len)),
                    ],
                    format_args!(
                        "TMR {index}{} over [{base:#x}, {:#x})",
                        if kind == TMR {
                            ""
                        } else {
                            ", of an unknown type,"
                        },
                        base.saturating_add(len)
                    ),
                );
            }
        }
        None => out.check(
            Tag::Info,
            "launch.amd_tmr",
            &[("count", Value::Count(tmrs_count.into()))],
            format_args!("the TMR descriptors at {tmrs_at:#x} do not fit the area"),
        ),
    }

    let Some(log) = table(area, size, log_at, log_size, 1) else {
        out.check(
            Tag::Info,
            "launch.amd_log",
            &[("size", Value::Hex(log_size.into()))],
            format_args!("the TCG log at {log_at:#x} does not fit the area"),
        );
        return;
    };
    let replayed = tcg_log::replay(log);
    let matches = match (&replayed, pcrs) {
        (Ok(replay), Some(now)) => replay[0] == now[0].as_bytes() && replay[1] == now[1].as_bytes(),
        _ => false,
    };
    let detail = match (&replayed, pcrs) {
        (Err(_), _) => "does not parse",
        (Ok(_), None) => "parses, and the PCRs did not read to replay it against",
        (Ok(_), Some(_)) if matches => "replays to PCR 17 and 18",
        (Ok(_), Some(_)) => "does not replay to PCR 17 and 18",
    };
    out.check(
        Tag::Info,
        "launch.amd_log",
        &[
            ("size", Value::Hex(log_size.into())),
            ("replays", Value::Flag(matches)),
        ],
        format_args!("the TCG log the SL left {detail}"),
    );
}
