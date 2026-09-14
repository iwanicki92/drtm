// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

#![no_main]
#![no_std]

mod amd;
mod cpu_state;
mod logging;
mod msr;
mod platform;
mod tpm;

use log::{info, warn};
use platform::Vendor;
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    logging::init();

    info!("hello from UEFI");

    // Phase 1: identify the platform, which decides what else can run.
    let cpu = platform::identify();
    info!("CPU vendor {}", cpu.name());

    // Phase 2: checks that hold regardless of which DRTM path applies.
    cpu_state::report();
    tpm::report();

    // Phase 3: the vendor's own preconditions.
    match cpu.vendor {
        Vendor::Amd => amd::report(),
        Vendor::Intel => warn!(
            "[SKIP] AMD checks, this CPU is {}, and the Intel TXT path \
             is not implemented",
            cpu.name()
        ),
        Vendor::Other => warn!(
            "[SKIP] AMD checks, this CPU is {}, which is neither AMD \
             nor Intel, so no DRTM path applies",
            cpu.name()
        ),
    }

    // Terminal marker. The QEMU test harness reads serial output until this
    // line, so it knows it has every check's result and not a partial boot.
    info!("checks complete");

    Status::SUCCESS
}
