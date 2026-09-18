// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Raw platform state for a hardware run, as records rather than prose.
//!
//! Only reads, like the checks. Ordered from the reads least likely to fault
//! to the most, and each risky read is announced before it happens, so a
//! machine that dies mid-dump still says what killed it.

mod acpi;
mod cpu;

use log::LevelFilter;

use crate::check;
use crate::logging::{self, Format};
use crate::platform::Cpu;

/// Gathers everything, if the run asked for it.
///
/// Both conditions, not either: the dump has no prose form to print, and a
/// run left at `info` wants results rather than a megabyte of hexadecimal.
pub fn run(cpu: &Cpu) {
    if logging::format() != Format::Json || log::max_level() < LevelFilter::Trace {
        return;
    }
    logging::record("dump", &[("at", check::text!("start"))]);
    cpu::dump_cpuid(cpu);
    cpu::dump_processors();
    acpi::dump();
    cpu::dump_msrs(cpu);
    logging::record("dump", &[("at", check::text!("end"))]);
}
