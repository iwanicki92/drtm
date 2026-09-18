// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Raw platform state for a hardware run, as records rather than prose.
//!
//! Only reads, like the checks. Ordered from the reads least likely to fault
//! to the most, with a marker record before each section, so a machine that
//! dies mid-dump says where. Within a section the reads follow the order the
//! source lists them in, which names the one that did it.

mod acpi;
mod cpu;
mod firmware;
mod pci;
mod psp;
mod tpm;

use log::LevelFilter;

use crate::check;
use crate::logging::{self, Format};
use crate::pci::Devices;
use crate::platform::Cpu;

/// Gathers everything, if the run asked for it.
///
/// Both conditions, not either: the dump has no prose form to print, and a
/// run left at `info` wants results rather than a megabyte of hexadecimal.
pub fn run(cpu: &Cpu, devices: &Devices) {
    if logging::format() != Format::Json || log::max_level() < LevelFilter::Trace {
        return;
    }
    at("firmware");
    firmware::dump();
    at("cpuid");
    cpu::dump_cpuid(cpu);
    at("processors");
    cpu::dump_processors();
    at("acpi");
    acpi::dump();
    at("tpm");
    tpm::dump();
    at("pci");
    pci::dump(devices);
    at("msr");
    cpu::dump_msrs(cpu);
    at("iommu");
    pci::dump_iommu_registers(devices);
    at("psp");
    psp::dump(devices);
    at("end");
}

/// Says which section comes next, or that the last one finished.
fn at(section: &str) {
    logging::record("dump", &[("at", check::text!("{section}"))]);
}
