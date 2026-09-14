// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! AMD-specific DRTM (`SKINIT`) precondition checks.

pub mod checks;
pub mod memory;
pub mod mmconfig;

use log::{error, info};
use memory::SlbRegion;

/// Runs every AMD check and reports it.
///
/// Only called once the CPU has been identified as AMD, so a failure here
/// means an AMD machine that cannot do a dynamic launch, not a machine on
/// the wrong path.
pub fn report() {
    if checks::skinit_supported() {
        info!("[PASS] SKINIT CPUID bit set (CPUID.80000001:ECX[12])");
    } else {
        error!(
            "[FAIL] SKINIT CPUID bit not set (CPUID.80000001:ECX[12]), \
             SKINIT unsupported on this CPU"
        );
    }

    match memory::probe_slb_region() {
        SlbRegion::Available(address) => {
            info!(
                "[PASS] 64 KiB-aligned SLB region available below 4 GiB \
                 (probed at {address:#x})"
            );
        }
        SlbRegion::Unavailable(status) => {
            error!(
                "[FAIL] no 64 KiB-aligned SLB region available below \
                 4 GiB ({status:?})"
            );
        }
    }

    mmconfig::report();
}
