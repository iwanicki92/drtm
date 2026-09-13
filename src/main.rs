// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

#![no_main]
#![no_std]

mod amd;

use log::{error, info};
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    info!("drtm: hello from UEFI");

    if amd::checks::skinit_supported() {
        info!("drtm: [PASS] SKINIT CPUID bit set (CPUID.80000001:ECX[12])");
    } else {
        error!(
            "drtm: [FAIL] SKINIT CPUID bit not set (CPUID.80000001:ECX[12]), \
             SKINIT unsupported on this CPU"
        );
    }

    Status::SUCCESS
}
