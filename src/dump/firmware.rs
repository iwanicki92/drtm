// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! What the firmware says about itself: its own version, what it published
//! to us, and SMBIOS.

use core::slice;

use uefi::boot::{self, ScopedProtocol};
use uefi::proto::loaded_image::LoadedImage;
use uefi::runtime::{self, VariableVendor};
use uefi::table::cfg::ConfigTableEntry;
use uefi::{CStr16, cstr16, system};

use crate::check;
use crate::logging::record;

/// SMBIOS entry point fields, where Linux's `dmi_present()` and
/// `dmi_smbios3_present()` read them (`drivers/firmware/dmi_scan.c`).
const SMBIOS3_ANCHOR: &[u8; 5] = b"_SM3_";
const SMBIOS3_LEN: usize = 24;
const SMBIOS3_MAX_SIZE: usize = 0x0C;
const SMBIOS3_ADDRESS: usize = 0x10;
const SMBIOS2_ANCHOR: &[u8; 4] = b"_SM_";
const SMBIOS2_LEN: usize = 31;
const SMBIOS2_LENGTH: usize = 0x16;
const SMBIOS2_ADDRESS: usize = 0x18;

/// Likely far above any SMBIOS table, so a corrupt size is refused.
const SMBIOS_MAX_SIZE: usize = 1 << 20;

/// The type that ends the table.
const SMBIOS_END: u8 = 127;

/// Room for any of [`VARIABLES`]. `BootOrder` holds two bytes per entry, so
/// this fits 512 of them.
const VARIABLE_BUFFER_LEN: usize = 1024;

/// Variables that describe how the platform boots.
const VARIABLES: [&CStr16; 6] = [
    cstr16!("SecureBoot"),
    cstr16!("SetupMode"),
    cstr16!("AuditMode"),
    cstr16!("DeployedMode"),
    cstr16!("BootCurrent"),
    cstr16!("BootOrder"),
];

pub fn dump() {
    record(
        "uefi",
        &[
            ("vendor", check::text!("{}", system::firmware_vendor())),
            ("revision", check::hex(system::firmware_revision())),
            ("spec", check::text!("{}", system::uefi_revision())),
        ],
    );
    let image: uefi::Result<ScopedProtocol<LoadedImage>> =
        boot::open_protocol_exclusive(boot::image_handle());
    if let Ok(image) = image {
        let (base, size) = image.info();
        record(
            "image",
            &[
                ("base", check::hex(base as usize as u64)),
                ("len", check::count(size)),
            ],
        );
    }
    system::with_config_table(|entries| {
        for entry in entries {
            record(
                "configtable",
                &[
                    ("guid", check::text!("{}", entry.guid)),
                    ("addr", check::hex(entry.address as usize as u64)),
                ],
            );
        }
    });
    for name in VARIABLES {
        let mut buffer = [0u8; VARIABLE_BUFFER_LEN];
        match runtime::get_variable(name, &VariableVendor::GLOBAL_VARIABLE, &mut buffer) {
            Ok((value, attributes)) => record(
                "variable",
                &[
                    ("name", check::text!("{name}")),
                    ("attrs", check::text!("{attributes:?}")),
                    ("hex", check::bytes(value)),
                ],
            ),
            Err(error) => record(
                "variable",
                &[
                    ("name", check::text!("{name}")),
                    ("status", check::text!("{:?}", error.status())),
                ],
            ),
        }
    }
    dump_smbios();
}

/// The SMBIOS table's address and size, from the 3.x entry point where there
/// is one.
fn smbios_table() -> Option<(usize, usize)> {
    let (v3, v2) = system::with_config_table(|entries| {
        let find = |guid| {
            entries
                .iter()
                .find(|entry| entry.guid == guid)
                .map(|entry| entry.address as usize)
        };
        (
            find(ConfigTableEntry::SMBIOS3_GUID),
            find(ConfigTableEntry::SMBIOS_GUID),
        )
    });
    if let Some(entry) = v3 {
        // Safety: firmware published this address as a 3.x entry point, which
        // is 24 bytes long.
        let bytes = unsafe { slice::from_raw_parts(entry as *const u8, SMBIOS3_LEN) };
        entry_point(3, entry, bytes);
        if bytes[..5] == *SMBIOS3_ANCHOR {
            let size = u32::from_le_bytes(bytes[SMBIOS3_MAX_SIZE..][..4].try_into().ok()?);
            let address = u64::from_le_bytes(bytes[SMBIOS3_ADDRESS..][..8].try_into().ok()?);
            if let Some(table) = plausible(address, size.into()) {
                return Some(table);
            }
        }
    }
    let entry = v2?;
    // Safety: firmware published this address as a 2.x entry point, which is
    // 31 bytes long.
    let bytes = unsafe { slice::from_raw_parts(entry as *const u8, SMBIOS2_LEN) };
    entry_point(2, entry, bytes);
    if bytes[..4] != *SMBIOS2_ANCHOR {
        return None;
    }
    let size = u16::from_le_bytes([bytes[SMBIOS2_LENGTH], bytes[SMBIOS2_LENGTH + 1]]);
    let address = u32::from_le_bytes(bytes[SMBIOS2_ADDRESS..][..4].try_into().ok()?);
    plausible(address.into(), size.into())
}

fn entry_point(version: u64, address: usize, bytes: &[u8]) {
    record(
        "smbios",
        &[
            ("entry", check::count(version)),
            ("addr", check::hex(address as u64)),
            ("hex", check::bytes(bytes)),
        ],
    );
}

/// An SMBIOS table's address and size, unless either is zero or the size is
/// past [`SMBIOS_MAX_SIZE`].
fn plausible(address: u64, size: u64) -> Option<(usize, usize)> {
    let address = usize::try_from(address).ok().filter(|&a| a != 0)?;
    let size = usize::try_from(size)
        .ok()
        .filter(|&s| s != 0 && s <= SMBIOS_MAX_SIZE)?;
    Some((address, size))
}

/// Every structure, whole, with its string set: both are bytes a reader
/// decodes, and picking types here is how the useful one goes missing.
fn dump_smbios() {
    let Some((address, size)) = smbios_table() else {
        record("smbios", &[("found", check::flag(false))]);
        return;
    };
    // Safety: the entry point gives the table's address and an upper bound on
    // its size, and firmware keeps it in place while boot services run.
    let table = unsafe { slice::from_raw_parts(address as *const u8, size) };
    let mut offset = 0;
    while let Some(header) = table.get(offset..offset + 4) {
        let (kind, length) = (header[0], usize::from(header[1]));
        let handle = u16::from_le_bytes([header[2], header[3]]);
        let Some(strings) = table.get(offset + length..) else {
            break;
        };
        // The string set ends at the first double NUL, which a structure
        // without strings still carries.
        let Some(end) = strings.windows(2).position(|pair| pair == [0, 0]) else {
            break;
        };
        let whole = offset + length + end + 2;
        record(
            "smbios",
            &[
                ("type", check::count(kind)),
                ("handle", check::hex(handle)),
                ("len", check::count(length as u64)),
                ("hex", check::bytes(&table[offset..whole])),
            ],
        );
        if kind == SMBIOS_END {
            break;
        }
        offset = whole;
    }
}
