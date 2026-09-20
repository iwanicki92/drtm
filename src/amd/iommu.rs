// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Whether the platform has an AMD IOMMU, and where.
//!
//! Found by its PCI capability rather than a device ID, since AMD publishes
//! no list of IOMMU device IDs and the capability offset is what anything
//! programming the IOMMU works from.

use alloc::vec::Vec;

use crate::check;
use crate::pci::{self, Devices, Function};

/// One IOMMU as configuration space shows it, which is also how the IVRS
/// table names one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Iommu {
    pub function: Function,
    pub capability: u16,
}

/// AMD puts both the IOMMU and the older Device Exclusion Vector under
/// capability ID `0x0F`, so the type field is what separates them
/// (_AMD I/O Virtualization Technology Specification, section 3.7.1_).
const CAPABILITY_ID_SECURE_DEVICE: u8 = 0x0F;
const CAPABILITY_TYPE_IOMMU: u8 = 3;

/// Bit 0 of the IOMMU Base Address Low register, capability offset `0x04`.
/// Firmware sets it once the MMIO base is programmed, which locks the block
/// (_AMD IOMMU specification, section 3.2_).
const CAPABILITY_REG_BASE_LOW: u8 = 0x04;
const BASE_LOW_ENABLE: u32 = 1 << 0;

/// Where the Secure Kernel Loader expects the IOMMU
/// (`secure-kernel-loader:include/iommu.h`). No specification fixes an
/// address, so this is reported and never judged.
const SKL_IOMMU: Function = Function {
    segment: 0,
    bus: 0,
    device: 0,
    function: 2,
};

/// Reports every AMD IOMMU configuration space offers, and returns them.
///
/// Not a gate. A missing or firmware-disabled IOMMU costs DMA protection,
/// which is a weaker launch rather than no launch, so it warns where a real
/// precondition would fail. Where a particular loader looks is only noted.
pub fn report(devices: &Devices) -> Vec<Iommu> {
    let mut found = Vec::new();
    let mut at_skl_address = false;
    pci::for_each_device(devices, |pci, device| {
        let function = device.function;
        pci::for_each_capability(pci, function, |capability| {
            if capability.id != CAPABILITY_ID_SECURE_DEVICE
                || capability.kind != CAPABILITY_TYPE_IOMMU
            {
                return;
            }
            found.push(Iommu {
                function,
                capability: capability.offset.into(),
            });
            at_skl_address |= function == SKL_IOMMU;
            // A block that would run past the header's 256 bytes is malformed
            // and counts as disabled, rather than overflowing the offset.
            let enabled = capability
                .offset
                .checked_add(CAPABILITY_REG_BASE_LOW)
                .and_then(|register| {
                    pci.read_one::<u32>(function.address().with_register(register))
                        .ok()
                })
                .is_some_and(|base_low| base_low & BASE_LOW_ENABLE != 0);
            if enabled {
                check::pass!(
                    "amd.iommu",
                    function = check::text!("{function}"),
                    capability = check::hex(capability.offset);
                    "at {function}, capability block at {:#04x}",
                    capability.offset
                );
            } else {
                check::warn!(
                    "amd.iommu",
                    function = check::text!("{function}"),
                    capability = check::hex(capability.offset);
                    "at {function}, capability block at {:#04x}, enable bit \
                     clear, so firmware left it unconfigured and it protects \
                     nothing until software programs it",
                    capability.offset
                );
            }
        });
    });

    if found.is_empty() {
        check::warn!(
            "amd.iommu",
            "none in configuration space, so a launch here would have no \
             IOMMU to protect memory with"
        );
        return found;
    }
    if !at_skl_address {
        check::info!(
            "amd.iommu",
            "none at {SKL_IOMMU}, where the Secure Kernel Loader expects one"
        );
    }
    if found.len() > 1 {
        check::info!(
            "amd.iommu",
            found = check::count(found.len() as u64);
            "{} found, while the Secure Kernel Loader programs only the one \
             at {SKL_IOMMU}",
            found.len()
        );
    }
    found
}
