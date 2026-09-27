// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Whether the platform describes AMD's DRTM ACPI device.
//!
//! Reported rather than judged: firmware other than AMD's need not publish
//! it, and UEFI exposes no namespace to ask whether it reports it enabled.
//! Its absence is also a hint when the PSP service reports DRTM off.

use crate::acpi::{self, AML};
use crate::check;

/// The device's hardware ID, which AML carries as a plain string. What a
/// board calls the device and which table it puts it in are firmware's, so
/// this is the part worth matching on.
const HID: &[u8] = b"DRTM0002";

/// Header fields naming who wrote the table, as corroboration for a reader
/// comparing one machine against another (ACPI 6.5, section 5.2.6).
const OEM_ID: usize = 10;
const OEM_ID_LEN: usize = 6;
const OEM_TABLE_ID: usize = 16;
const OEM_TABLE_ID_LEN: usize = 8;

/// Reports which table describes the device, or that none does.
pub fn report() {
    let Ok(tables) = acpi::tables() else {
        check::info!(
            "amd.drtm_device",
            "no ACPI tables, so nothing describes a DRTM device"
        );
        return;
    };

    let Some(table) = find(tables) else {
        check::info!(
            "amd.drtm_device",
            found = check::flag(false);
            "no ACPI table describes a DRTM0002 device"
        );
        return;
    };

    let signature = text(table, 0, 4);
    let oem = text(table, OEM_ID, OEM_ID_LEN);
    let oem_table = text(table, OEM_TABLE_ID, OEM_TABLE_ID_LEN);
    check::info!(
        "amd.drtm_device",
        found = check::flag(true),
        table = check::text!("{signature}"),
        oem = check::text!("{oem}"),
        oem_table = check::text!("{oem_table}");
        "DRTM0002 device in the {signature} from {oem} {oem_table}, \
         _STA not evaluated"
    );
}

/// Whether any ACPI table describes the device, or `None` without tables.
pub fn described() -> Option<bool> {
    acpi::tables().ok().map(|tables| find(tables).is_some())
}

/// The first AML table carrying the hardware ID.
fn find(tables: impl IntoIterator<Item = &'static [u8]>) -> Option<&'static [u8]> {
    // The DSDT hangs off the FADT rather than the root table, and is where a
    // firmware that does not give the device a table of its own would put it.
    tables
        .into_iter()
        .chain(acpi::dsdt())
        .filter(|table| AML.iter().any(|signature| table[..4] == **signature))
        .find(|table| describes_device(table))
}

/// Whether the table's AML carries the hardware ID anywhere.
///
/// A byte search rather than a walk of the namespace: the encoding around
/// the ID differs with how firmware declares the device, while the ID itself
/// is the same eight bytes either way.
fn describes_device(table: &[u8]) -> bool {
    table.windows(HID.len()).any(|window| window == HID)
}

/// A header field as text, without the padding firmware writes it out to.
fn text(table: &[u8], at: usize, len: usize) -> &str {
    table
        .get(at..at + len)
        .and_then(|bytes| str::from_utf8(bytes).ok())
        .unwrap_or("?")
        .trim_end_matches([' ', '\0'])
}
