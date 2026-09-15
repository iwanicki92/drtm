// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Whether the platform has an AMD IOMMU, and where.
//!
//! Found by its PCI capability rather than a device ID, since AMD publishes
//! no list of IOMMU device IDs and the capability offset is what anything
//! programming the IOMMU works from.

use log::{info, warn};

use crate::pci::{self, Function};

/// AMD puts both the IOMMU and the older Device Exclusion Vector under
/// capability ID `0x0F`, so the type field is what separates them
/// (_AMD I/O Virtualization Technology Specification, section 3.7.1_).
const CAPABILITY_ID_SECURE_DEVICE: u8 = 0x0F;
const CAPABILITY_TYPE_IOMMU: u8 = 3;

/// The Secure Kernel Loader sets up the IOMMU at this fixed address and no
/// other, whatever else the scan turns up
/// (`secure-kernel-loader:include/iommu.h`).
const SKL_IOMMU: Function = Function {
    segment: 0,
    bus: 0,
    device: 0,
    function: 2,
};

/// Reports every AMD IOMMU configuration space offers.
///
/// Not a gate. A missing IOMMU costs DMA protection, which is a weaker
/// launch rather than no launch, so it warns where a real precondition
/// would fail.
pub fn report() {
    let mut found = 0;
    let mut at_skl_address = false;
    pci::for_each_function(|pci, function| {
        pci::for_each_capability(pci, function, |capability| {
            if capability.id != CAPABILITY_ID_SECURE_DEVICE
                || capability.kind != CAPABILITY_TYPE_IOMMU
            {
                return;
            }
            found += 1;
            at_skl_address |= function == SKL_IOMMU;
            info!(
                "[PASS] AMD IOMMU at {function}, capability block at \
                 {:#04x}",
                capability.offset
            );
        });
    });

    if found == 0 {
        warn!(
            "[WARN] no AMD IOMMU in configuration space, so a launch here \
             would have no IOMMU to protect memory with"
        );
        return;
    }
    if !at_skl_address {
        warn!(
            "[WARN] no IOMMU at {SKL_IOMMU}, the fixed address the Secure \
             Kernel Loader sets one up at, so it would configure none"
        );
    }
    if found > 1 {
        warn!(
            "[WARN] {found} IOMMUs. The Secure Kernel Loader configures \
             only the one at {SKL_IOMMU}, so its deny-all device table \
             reaches no other"
        );
    }
}
