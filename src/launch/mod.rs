// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The AMD launch: the gate, the choice of path, then the preamble, which
//! ends in `SKINIT` and the Secure Loader built from `sl/`.

mod gate;
mod pit;
mod preamble;
mod psp;
mod trampoline;

use common::record::{Tag, Value};

use crate::check;
use crate::config::{Config, Launch};
use crate::pci::Devices;
use crate::platform::Cpu;

/// The Secure Loader image and what `build.rs` computed from it.
mod image {
    include!(concat!(env!("OUT_DIR"), "/sl.rs"));

    pub const BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sl.bin"));
}

/// Launches, if `drtm.conf` and the checks allow it. Returns where they do
/// not, or where the preamble failed before leaving boot services.
pub fn run(config: &Config, cpu: &Cpu, devices: &Devices) {
    if config.launch == Launch::Off {
        return;
    }
    let Some(sinks) = gate::check(cpu) else {
        return;
    };
    let tsc_per_ms = preamble::calibrate_tsc();

    let service = match config.launch {
        Launch::Skinit => None,
        _ => match psp::prepare(devices, tsc_per_ms) {
            Ok(service) => Some(service),
            Err(why) if config.launch == Launch::Psp => {
                gate::block(&why);
                return;
            }
            Err(why) => {
                check::after_summary(
                    Tag::Info,
                    "launch.path",
                    &[],
                    format_args!("not through the PSP DRTM service: {why}, so classic SKINIT"),
                );
                None
            }
        },
    };
    let (path, why) = match (&service, config.launch) {
        (Some(_), _) => ("psp", "through the PSP DRTM service"),
        (None, Launch::Skinit) => ("classic", "classic SKINIT, as launch=skinit asks"),
        (None, _) => ("classic", "classic SKINIT"),
    };
    check::after_summary(
        Tag::Info,
        "launch.start",
        &[("path", Value::Text(format_args!("{path}")))],
        format_args!("launching {why}, the SL writes to {}", sinks.source()),
    );
    preamble::launch(config, sinks, service, tsc_per_ms);
}
