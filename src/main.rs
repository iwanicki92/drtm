// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

#![no_main]
#![no_std]

mod amd;

use amd::checks::TpmProbe;
use amd::memory::SlbRegion;
use log::{error, info, warn};
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    info!("drtm: hello from UEFI");

    if amd::checks::skinit_supported() {
        info!("drtm: [PASS] SKINIT CPUID bit set (CPUID.80000001:ECX[12])");
    } else if amd::checks::cpu_is_amd() {
        error!(
            "drtm: [FAIL] SKINIT CPUID bit not set (CPUID.80000001:ECX[12]), \
             SKINIT unsupported on this CPU"
        );
    } else {
        error!(
            "drtm: [FAIL] SKINIT CPUID bit not set (CPUID.80000001:ECX[12]), \
             and this CPU does not report itself as AMD, SKINIT is an AMD \
             extension"
        );
    }

    match amd::checks::probe_tpm() {
        TpmProbe::NotPresent => {
            error!("drtm: [FAIL] no TPM answered at locality 0 (0xFED40000)");
        }
        TpmProbe::Present { family, interface } => {
            info!("drtm: [PASS] TPM present at locality 0: {family:?} {interface:?}");
            warn!(
                "drtm: [WARN] software cannot tell a discrete TPM from a \
                 firmware TPM, and SKINIT requires a discrete one, verify \
                 manually"
            );
        }
    }

    match amd::memory::probe_slb_region() {
        SlbRegion::Available(address) => {
            info!(
                "drtm: [PASS] 64 KiB-aligned SLB region available below 4 GiB \
                 (probed at {address:#x})"
            );
        }
        SlbRegion::Unavailable(status) => {
            error!(
                "drtm: [FAIL] no 64 KiB-aligned SLB region available below \
                 4 GiB ({status:?})"
            );
        }
    }

    // Terminal marker. The QEMU test harness reads serial output until this
    // line, so it knows it has every check's result and not a partial boot.
    info!("drtm: checks complete");

    Status::SUCCESS
}
