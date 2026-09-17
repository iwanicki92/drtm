// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

#![no_main]
#![no_std]

extern crate alloc;

mod acpi;
mod amd;
mod check;
mod config;
mod cpu_state;
mod logging;
mod msr;
mod pci;
mod platform;
mod tpm;

use log::info;
use platform::Vendor;
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    logging::init();
    let config = config::Config::load();
    log::set_max_level(config.level);
    logging::set_format(config.format);
    config.report();

    info!("hello from UEFI");

    // Phase 1: identify the platform, which decides what else can run.
    let cpu = platform::identify();
    info!("CPU vendor {}", cpu.name());

    // Phase 2: checks that hold regardless of which DRTM path applies.
    cpu_state::report();
    tpm::report();
    let devices = pci::scan();
    pci::report(&devices);

    // Phase 3: the vendor's own preconditions.
    match cpu.vendor {
        Vendor::Amd => amd::report(&devices),
        Vendor::Intel => check::skip!(
            "amd.checks",
            "this CPU is {}, and the Intel TXT path is not implemented",
            cpu.name()
        ),
        Vendor::Other => check::skip!(
            "amd.checks",
            "this CPU is {}, which is neither AMD nor Intel, so no DRTM \
             path applies",
            cpu.name()
        ),
    }

    // Terminal marker. The QEMU test harness reads serial output until this
    // line, so it knows it has every check's result and not a partial boot.
    info!("checks complete");

    Status::SUCCESS
}
