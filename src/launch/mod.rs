// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The classic AMD `SKINIT` launch: the gate, then the preamble, which ends
//! in `SKINIT` and the Secure Loader built from `sl/`.

mod gate;
mod preamble;
mod trampoline;

use crate::config::{Config, Launch};
use crate::platform::Cpu;

/// The Secure Loader image and what `build.rs` computed from it.
mod image {
    include!(concat!(env!("OUT_DIR"), "/sl.rs"));

    pub const BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sl.bin"));
}

/// Launches, if `drtm.conf` and the checks allow it. Returns where they do
/// not, or where the preamble failed before leaving boot services.
pub fn run(config: &Config, cpu: &Cpu) {
    if config.launch == Launch::Off {
        return;
    }
    if let Some(sinks) = gate::check(cpu) {
        preamble::launch(config, sinks);
    }
}
