// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Whether the ACPI IVRS table describes the IOMMUs configuration space shows.
//!
//! An OS finds AMD IOMMUs through that table alone, so one it leaves out
//! goes unused after a launch, however firmware configured it.

use alloc::vec::Vec;

use crate::acpi;
use crate::amd::iommu::Iommu;
use crate::check;
use crate::pci::Function;

const SIGNATURE: &[u8; 4] = b"IVRS";

/// The header and IVinfo are followed by 8 reserved bytes, then the blocks
/// (_AMD IOMMU specification, section 5.2_).
const FIRST_BLOCK: usize = acpi::HEADER_LEN + 12;

/// Every block opens with its type and, at offset 2, its length. The three
/// IVHD types each describe one IOMMU, and firmware may list the same one
/// under several of them. IVMD blocks share the opening and are skipped.
const BLOCK_LENGTH: usize = 2;
const IVHD_TYPES: [u8; 3] = [0x10, 0x11, 0x40];
const IVHD_DEVICE_ID: usize = 4;
const IVHD_CAPABILITY: usize = 6;
const IVHD_SEGMENT: usize = 16;
const IVHD_MIN_LEN: usize = 24;

/// Reports whether the table lists the IOMMUs `found`, and only those.
///
/// A missing entry warns rather than fails, as a missing IOMMU does. An
/// entry with no IOMMU behind it likely points at a firmware bug, and warns
/// too.
pub fn report(found: &[Iommu]) {
    let table = match acpi::find(SIGNATURE) {
        Ok(Some(table)) => table,
        missing => {
            let why = if missing.is_err() {
                "no ACPI tables"
            } else {
                "no IVRS table"
            };
            if found.is_empty() {
                check::skip!("amd.iommu_table", "{why}, and no IOMMU for one to describe");
            } else {
                check::warn!(
                    "amd.iommu_table",
                    "{why}, so an OS uses none of the IOMMUs in configuration \
                     space ({})",
                    found.len()
                );
            }
            return;
        }
    };

    if !acpi::checksum_ok(table) {
        check::warn!(
            "amd.iommu_table",
            "IVRS checksum is wrong, so an OS discards the table"
        );
        return;
    }
    let Some(listed) = listed(table) else {
        check::warn!(
            "amd.iommu_table",
            "IVRS has a block too short for its type or running past the \
             table, so an OS may reject or misread it"
        );
        return;
    };

    let mut matched = true;
    for iommu in found.iter().filter(|iommu| !listed.contains(iommu)) {
        matched = false;
        check::warn!(
            "amd.iommu_table",
            "IVRS does not list {} (capability block at {:#04x}), so an OS \
             leaves it unused",
            iommu.function,
            iommu.capability
        );
    }
    for iommu in listed.iter().filter(|iommu| !found.contains(iommu)) {
        matched = false;
        check::warn!(
            "amd.iommu_table",
            "IVRS lists {} (capability block at {:#04x}), which configuration \
             space does not show",
            iommu.function,
            iommu.capability
        );
    }
    if matched {
        check::info!(
            "amd.iommu_table",
            "IOMMUs listed in IVRS: {}, matching configuration space",
            listed.len()
        );
    }
}

/// Every distinct IOMMU the table's IVHD blocks name, or `None` if a block
/// is malformed.
fn listed(table: &[u8]) -> Option<Vec<Iommu>> {
    let mut listed = Vec::new();
    let mut offset = FIRST_BLOCK;
    while offset < table.len() {
        let block = &table[offset..];
        let len = usize::from(read_u16(block, BLOCK_LENGTH)?);
        if len < BLOCK_LENGTH + 2 || len > block.len() {
            return None;
        }
        if IVHD_TYPES.contains(&block[0]) {
            if len < IVHD_MIN_LEN {
                return None;
            }
            let id = read_u16(block, IVHD_DEVICE_ID)?;
            let iommu = Iommu {
                function: Function {
                    segment: read_u16(block, IVHD_SEGMENT)?.into(),
                    bus: (id >> 8) as u8,
                    device: ((id >> 3) & 0x1F) as u8,
                    function: (id & 0x7) as u8,
                },
                capability: read_u16(block, IVHD_CAPABILITY)?,
            };
            if !listed.contains(&iommu) {
                listed.push(iommu);
            }
        }
        offset += len;
    }
    Some(listed)
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let raw = bytes.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([raw[0], raw[1]]))
}
