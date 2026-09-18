// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Every ACPI table, whole, plus the two the FADT points at and any boot
//! errors firmware recorded.
//!
//! No allow-list and no size cap beyond the one `acpi::table` applies: a
//! table costs two bytes of log per byte, and deciding on the machine which
//! ones matter is how the first run came back missing the interesting one.

use core::slice;

use crate::acpi;
use crate::check;
use crate::logging::record;

/// The RSDP's 2.0 form.
const RSDP_LEN: usize = 36;

/// FADT fields locating the FACS and DSDT, 32-bit first, then the 64-bit
/// ones that override them where set, as Linux's `acpi_table_fadt` lays them
/// out (`include/acpi/actbl.h`). A wrong offset here reads some other field
/// as an address and can fault.
const FADT_FIRMWARE_CTRL: usize = 36;
const FADT_DSDT: usize = 40;
const FADT_X_FIRMWARE_CTRL: usize = 132;
const FADT_X_DSDT: usize = 140;

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
/// recompute from the bytes.
fn emit(table: &[u8]) {
    record(
        "acpi",
        &[
            ("sig", check::text!("{}", signature(table))),
            ("addr", check::hex(table.as_ptr() as u64)),
            ("len", check::count(table.len() as u64)),
            ("csum", check::flag(acpi::checksum_ok(table))),
            ("hex", check::bytes(table)),
        ],
    );
}

/// A table's four signature bytes. Always ASCII in practice, and a table
/// whose header is unreadable is worth saying so about rather than hiding.
fn signature(table: &[u8]) -> &str {
    str::from_utf8(&table[..4]).unwrap_or("?")
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}

/// The 64-bit field where set and present, else the 32-bit one.
fn fadt_address(fadt: &[u8], wide: usize, narrow: usize) -> Option<usize> {
    let address = read_u64(fadt, wide)
        .filter(|&address| address != 0)
        .or_else(|| read_u32(fadt, narrow).map(u64::from))?;
    usize::try_from(address)
        .ok()
        .filter(|&address| address != 0)
}

/// The FACS and DSDT, which the FADT points to rather than the root table.
fn fadt_targets(fadt: &[u8]) {
    if let Some(address) = fadt_address(fadt, FADT_X_FIRMWARE_CTRL, FADT_FIRMWARE_CTRL) {
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
    let Some(address) = fadt_address(fadt, FADT_X_DSDT, FADT_DSDT) else {
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
