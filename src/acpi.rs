// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Finding ACPI tables through the root pointer UEFI hands over.
//!
//! Reads the tables in place, where firmware left them, and writes nothing.

use core::ptr;
use core::slice;

use uefi::system;
use uefi::table::cfg::ConfigTableEntry;

/// Root System Description Pointer fields (ACPI 6.5, section 5.2.5.3).
const RSDP_SIGNATURE: [u8; 8] = *b"RSD PTR ";
const RSDP_REVISION: usize = 15;
const RSDP_RSDT: usize = 16;
const RSDP_XSDT: usize = 24;

/// Every table opens with a 36-byte header holding its signature and, at
/// offset 4, its length (ACPI 6.5, section 5.2.6).
pub const HEADER_LEN: usize = 36;
const HEADER_LENGTH: usize = 4;

/// Likely far above any table firmware ships, so a corrupt length is refused
/// rather than read.
const MAX_TABLE_LEN: usize = 1 << 20;

/// Neither UEFI configuration table entry leads to a valid root table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoAcpi;

/// Reads a `T` at `base + offset`.
///
/// # Safety
///
/// Those bytes must be mapped and readable. UEFI identity-maps memory, so a
/// physical address firmware reported qualifies.
unsafe fn read<T: Copy>(base: usize, offset: usize) -> T {
    // Safety: the caller vouches for the range, and `read_unaligned` needs
    // nothing more.
    unsafe { ptr::read_unaligned((base + offset) as *const T) }
}

/// The root table's address and how wide each of its entries is: the XSDT
/// with 8-byte entries where ACPI 2.0 provides one, the RSDT's 4 otherwise.
fn root() -> Option<(usize, usize)> {
    let (acpi2, acpi1) = system::with_config_table(|entries| {
        let find = |guid| {
            entries
                .iter()
                .find(|entry| entry.guid == guid)
                .map(|entry| entry.address as usize)
        };
        (
            find(ConfigTableEntry::ACPI2_GUID),
            find(ConfigTableEntry::ACPI_GUID),
        )
    });
    let rsdp = acpi2.or(acpi1)?;
    // Safety: firmware published this address as the RSDP, which is at least
    // 20 bytes, enough for the signature and revision.
    let (signature, revision) =
        unsafe { (read::<[u8; 8]>(rsdp, 0), read::<u8>(rsdp, RSDP_REVISION)) };
    if signature != RSDP_SIGNATURE {
        return None;
    }
    if revision >= 2 {
        // Safety: revision 2 and later define the 36-byte form, which holds
        // the XSDT address.
        let xsdt = unsafe { read::<u64>(rsdp, RSDP_XSDT) };
        if let Ok(xsdt @ 1..) = usize::try_from(xsdt) {
            return Some((xsdt, 8));
        }
    }
    // Safety: every revision carries the RSDT address within its 20 bytes.
    let rsdt = unsafe { read::<u32>(rsdp, RSDP_RSDT) };
    (rsdt != 0).then_some((rsdt as usize, 4))
}

/// The whole table at `address`, if its header gives a plausible length.
fn table(address: usize) -> Option<&'static [u8]> {
    // Safety: `address` came from the RSDP or a root table, both of which
    // point at a table header.
    let len = unsafe { read::<u32>(address, HEADER_LENGTH) } as usize;
    if !(HEADER_LEN..=MAX_TABLE_LEN).contains(&len) {
        return None;
    }
    // Safety: the header says the table spans `len` bytes, and firmware keeps
    // ACPI memory in place while boot services run, which covers our run.
    Some(unsafe { slice::from_raw_parts(address as *const u8, len) })
}

/// The first table carrying `signature`, or `Ok(None)` if the root table
/// lists none.
pub fn find(signature: &[u8; 4]) -> Result<Option<&'static [u8]>, NoAcpi> {
    let (address, width) = root().ok_or(NoAcpi)?;
    let root = table(address).ok_or(NoAcpi)?;
    for entry in root[HEADER_LEN..].chunks_exact(width) {
        let mut raw = [0u8; 8];
        raw[..width].copy_from_slice(entry);
        let Ok(address @ 1..) = usize::try_from(u64::from_le_bytes(raw)) else {
            continue;
        };
        if let Some(table) = table(address).filter(|table| table[..4] == *signature) {
            return Ok(Some(table));
        }
    }
    Ok(None)
}

/// Whether the table's bytes sum to zero, as every ACPI table's must.
pub fn checksum_ok(table: &[u8]) -> bool {
    table.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)) == 0
}
