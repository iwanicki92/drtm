// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! AMD-specific DRTM (`SKINIT`) precondition checks.

pub mod aspt;
pub mod checks;
pub mod drtm_device;
pub mod iommu;
pub mod ivrs;
pub mod memory;
pub mod mmconfig;
pub mod psp;

use memory::SlbRegion;

use crate::check;
use crate::pci::Devices;

/// Runs every AMD check and reports it.
///
/// Only called once the CPU has been identified as AMD, so a failure here
/// means an AMD machine that cannot do a dynamic launch, not a machine on
/// the wrong path.
pub fn report(devices: &Devices) {
    if checks::skinit_supported() {
        check::pass!("amd.skinit", "CPUID.80000001:ECX[12] set");
    } else {
        check::fail!(
            "amd.skinit",
            "CPUID.80000001:ECX[12] clear, SKINIT unsupported on this CPU"
        );
    }

    match memory::probe_slb_region() {
        SlbRegion::Available(address) => {
            check::pass!(
                "amd.slb",
                address = check::hex(address);
                "64 KiB-aligned region available below 4 GiB \
                 (probed at {address:#x})"
            );
        }
        SlbRegion::Unavailable(status) => {
            check::fail!(
                "amd.slb",
                "no 64 KiB-aligned region available below 4 GiB ({status:?})"
            );
        }
    }

    mmconfig::report();
    let iommus = iommu::report(devices);
    ivrs::report(&iommus);
    psp::report(devices);
    aspt::report(devices);
    drtm_device::report();
}
