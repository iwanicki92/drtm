// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The AMD launch: the gate, the choice of path, then the preamble, which
//! ends in `SKINIT` and the Secure Loader built from `sl/`, which hands off
//! to the DLME built from `dlme/`.

mod dlme;
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

/// A Secure Loader image and what `build.rs` computed from it.
pub struct Image {
    pub bytes: &'static [u8],
    /// Bytes `SKINIT` measures.
    pub measured: usize,
    /// Where the parameter page goes in the SLB, and its size.
    pub params_offset: usize,
    pub params_size: usize,
    /// PCR 17 after `SKINIT`, per bank.
    pub expected_sha1: [u8; 20],
    pub expected_sha256: [u8; 32],
}

/// The DLME and what `build.rs` computed from it.
pub struct DlmeImage {
    /// The measured image, in whole pages. Its parameter page follows.
    pub bytes: &'static [u8],
    /// Where its scratch memory ends, from the block's start.
    pub end: usize,
    pub sha256: [u8; 32],
    /// PCR 17 after the classic SL's `SKINIT` and its extend of the DLME.
    pub pcr17_sha1: [u8; 20],
    pub pcr17_sha256: [u8; 32],
}

/// A signed SL built elsewhere, which `launch=foreign` places in the SLB
/// without drtm's parameter page. One whose header carries AMD's GUID gets
/// its `AmdSlInputParameters`, naming drtm's DLME as its OS SL.
pub struct Foreign {
    pub image: Image,
    /// The key its token names.
    pub key_id: [u8; 16],
    /// Where in the SLB its `AmdSlInputParameters` go, when it takes them.
    pub parameters: Option<usize>,
    /// SHA-256 PCR 17, all but the last extend, and PCR 18 after its
    /// `LAUNCH`.
    pub pcr17: [u8; 32],
    pub pcr18: [u8; 32],
}

/// Whether this build embeds a foreign SL for `launch=foreign`.
pub const HAS_FOREIGN: bool = image::FOREIGN.is_some();

/// The SL built for each path, `CLASSIC` and `PSP`, the PCRs the PSP
/// service's `LAUNCH` should leave, the `DLME`, and `FOREIGN` where the
/// build has one.
mod image {
    use super::{DlmeImage, Foreign, Image};

    include!(concat!(env!("OUT_DIR"), "/sl.rs"));
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

    let foreign = match config.launch {
        Launch::Foreign => image::FOREIGN.as_ref(),
        _ => None,
    };
    let service = match config.launch {
        Launch::Skinit => None,
        _ => match psp::prepare(devices, tsc_per_ms) {
            Ok(service) => Some(service),
            Err(why) if matches!(config.launch, Launch::Psp | Launch::Foreign) => {
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
        (Some(_), Launch::Foreign) => (
            "foreign",
            "the foreign SL through the PSP DRTM service, which reports only \
             through port 0x80 until it jumps to the DLME",
        ),
        (Some(_), _) => ("psp", "through the PSP DRTM service"),
        (None, Launch::Skinit) => ("classic", "classic SKINIT, as launch=skinit asks"),
        (None, _) => ("classic", "classic SKINIT"),
    };
    match foreign {
        Some(foreign) => check::after_summary(
            Tag::Info,
            "launch.start",
            &[
                ("path", Value::Text(format_args!("{path}"))),
                ("key_id", Value::Bytes(&foreign.key_id)),
            ],
            format_args!("launching {why}, the DLME writes to {}", sinks.source()),
        ),
        None => check::after_summary(
            Tag::Info,
            "launch.start",
            &[("path", Value::Text(format_args!("{path}")))],
            format_args!(
                "launching {why}, the SL and the DLME write to {}",
                sinks.source()
            ),
        ),
    }
    preamble::launch(config, sinks, service, foreign, tsc_per_ms);
}
