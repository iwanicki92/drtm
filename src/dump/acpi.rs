// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Every ACPI table, whole, plus the two the FADT points at and any boot
//! errors firmware recorded.
//!
//! No allow-list: choosing on the machine which tables matter is how a run
//! comes back missing the one that mattered. The single exception is AML past
//! [`AML_CAP`], which outweighs every other table together.

use core::slice;

use crate::acpi::{self, AML, FADT_DSDT, FADT_FIRMWARE_CTRL, FADT_X_DSDT, FADT_X_FIRMWARE_CTRL};
use crate::bytes::{read_u32, read_u64};
use crate::check;
use crate::logging::record;

/// The RSDP's 2.0 form.
const RSDP_LEN: usize = 36;

/// How much AML goes in whole. A DSDT runs to a hundred times this, and a
/// prefix of one is no use to a disassembler, so past it only the header
/// goes in. `acpidump` on a booted OS is where the rest belongs.
const AML_CAP: usize = 4 << 10;

/// Where the BERT puts the boot error region, and the region's own data
/// length, as `acpi_table_bert` and `acpi_bert_region` lay them out
/// (`include/acpi/actbl1.h`).
const BERT_REGION_LENGTH: usize = 36;
const BERT_REGION_ADDRESS: usize = 40;
const ERROR_STATUS_LEN: usize = 20;
const ERROR_STATUS_DATA_LENGTH: usize = 12;

/// A bound on the boot error region, whose length firmware supplies and
/// nothing else checks.
const MAX_REGION: usize = 64 << 10;

pub fn dump() {
    let Some(rsdp) = acpi::rsdp() else {
        record(
            "acpi",
            &[("sig", check::text!("RSDP")), ("found", check::flag(false))],
        );
        return;
    };
    // Safety: firmware published this address as the RSDP. The 2.0 form is
    // 36 bytes, and past a 20-byte 1.0 one UEFI's identity map still covers
    // the rest.
    let bytes = unsafe { slice::from_raw_parts(rsdp as *const u8, RSDP_LEN) };
    record(
        "acpi",
        &[
            ("sig", check::text!("RSDP")),
            ("addr", check::hex(rsdp as u64)),
            ("len", check::count(RSDP_LEN as u64)),
            ("hex", check::bytes(bytes)),
        ],
    );
    let Ok(tables) = acpi::tables() else {
        record(
            "acpi",
            &[("sig", check::text!("RSDT")), ("found", check::flag(false))],
        );
        return;
    };
    for table in tables {
        emit(table);
        match &table[..4] {
            b"FACP" => fadt_targets(table),
            b"BERT" => boot_errors(table),
            _ => {}
        }
    }
}

/// One table, with the checksum verdict a reader would otherwise have to
/// recompute from the bytes. An oversized AML table is marked `cut`, its
/// `hex` then being the header alone while `len` stays the whole size.
fn emit(table: &[u8]) {
    let cut = table.len() > AML_CAP && AML.iter().any(|sig| table[..4] == **sig);
    let kept = if cut { acpi::HEADER_LEN } else { table.len() };
    let head: [(&str, check::Value); 4] = [
        ("sig", check::text!("{}", signature(table))),
        ("addr", check::hex(table.as_ptr() as u64)),
        ("len", check::count(table.len() as u64)),
        ("csum", check::flag(acpi::checksum_ok(table))),
    ];
    let hex = ("hex", check::bytes(&table[..kept]));
    let [sig, addr, len, csum] = head;
    if cut {
        record(
            "acpi",
            &[sig, addr, len, csum, ("cut", check::flag(true)), hex],
        );
    } else {
        record("acpi", &[sig, addr, len, csum, hex]);
    }
}

/// A table's four signature bytes. Always ASCII in practice, and a table
/// whose header is unreadable is worth saying so about rather than hiding.
fn signature(table: &[u8]) -> &str {
    str::from_utf8(&table[..4]).unwrap_or("?")
}

/// The FACS and DSDT, which the FADT points to rather than the root table.
fn fadt_targets(fadt: &[u8]) {
    if let Some(address) = acpi::fadt_address(fadt, FADT_X_FIRMWARE_CTRL, FADT_FIRMWARE_CTRL) {
        // Safety: the FADT gives this as the FACS's address.
        match unsafe { acpi::table(address) } {
            // The FACS carries no checksum field, so it goes in without the
            // verdict every other table gets.
            Some(facs) => record(
                "acpi",
                &[
                    ("sig", check::text!("FACS")),
                    ("addr", check::hex(address as u64)),
                    ("len", check::count(facs.len() as u64)),
                    ("hex", check::bytes(facs)),
                ],
            ),
            None => record(
                "acpi",
                &[
                    ("sig", check::text!("FACS")),
                    ("addr", check::hex(address as u64)),
                    ("found", check::flag(false)),
                ],
            ),
        }
    }
    let Some(address) = acpi::fadt_address(fadt, FADT_X_DSDT, FADT_DSDT) else {
        record(
            "acpi",
            &[("sig", check::text!("DSDT")), ("found", check::flag(false))],
        );
        return;
    };
    // Safety: the FADT gives this as the DSDT's address.
    match unsafe { acpi::table(address) } {
        Some(dsdt) => emit(dsdt),
        None => record(
            "acpi",
            &[
                ("sig", check::text!("DSDT")),
                ("addr", check::hex(address as u64)),
                ("found", check::flag(false)),
            ],
        ),
    }
}

/// The errors firmware recorded from the previous boot, if any.
fn boot_errors(bert: &[u8]) {
    let (Some(length), Some(address)) = (
        read_u32(bert, BERT_REGION_LENGTH),
        read_u64(bert, BERT_REGION_ADDRESS),
    ) else {
        return;
    };
    let Ok(address @ 1..) = usize::try_from(address) else {
        record("bert", &[("found", check::flag(false))]);
        return;
    };
    let length = (length as usize).min(MAX_REGION);
    if length < ERROR_STATUS_LEN {
        record(
            "bert",
            &[
                ("addr", check::hex(address as u64)),
                ("len", check::count(length as u64)),
            ],
        );
        return;
    }
    // Safety: the BERT gives this region's address and length, and firmware
    // reserves it for the OS to read.
    let region = unsafe { slice::from_raw_parts(address as *const u8, length) };
    let data = read_u32(region, ERROR_STATUS_DATA_LENGTH).unwrap_or(0) as usize;
    let shown = (ERROR_STATUS_LEN + data).min(length);
    record(
        "bert",
        &[
            ("addr", check::hex(address as u64)),
            ("len", check::count(shown as u64)),
            ("hex", check::bytes(&region[..shown])),
        ],
    );
}
