// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Whether to launch at all, from the results the checks recorded.
//!
//! Blocks only where a launch is impossible or unsafe. Any other failure is
//! launched through, and the Secure Loader's verdicts say what it cost.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::arch::x86_64::__cpuid;
use core::fmt;

use common::params::MAX_SINKS;
use common::record::{Tag, Value};
use common::spcr::{self, Unusable};
use common::uart::{self, Uart};

use crate::check;
use crate::platform::{Cpu, Vendor};
use crate::{acpi, logging};

/// `CPUID.80000001:EDX[26]`, 1 GiB pages, which the Secure Loader's page
/// tables map the low 4 GiB with.
const CPUID_PAGE_1GB: u32 = 1 << 26;

/// Where the Secure Loader will write, and what decided it.
pub struct Sinks {
    pub uarts: [Option<Uart>; MAX_SINKS],
    source: Source,
}

impl Sinks {
    /// What decided them, as prose.
    pub fn source(&self) -> &impl fmt::Display {
        &self.source
    }
}

enum Source {
    Spcr,
    /// No usable SPCR, and COM1 answered or UEFI has a serial device. The
    /// reason SPCR was not used, if it was there.
    Com1(Option<Unusable>),
    None(Option<Unusable>),
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spcr = |f: &mut fmt::Formatter<'_>, why: &Option<Unusable>| match why {
            Some(why) => write!(f, "SPCR names a UART the SL cannot use ({why:?})"),
            None => write!(f, "no SPCR table"),
        };
        match self {
            Self::Spcr => write!(f, "the UART SPCR names"),
            Self::Com1(why) => {
                spcr(f, why)?;
                write!(f, ", so COM1 at 0x3f8")
            }
            Self::None(why) => {
                spcr(f, why)?;
                write!(f, ", and nothing answers at COM1")
            }
        }
    }
}

/// The sink list: the UART SPCR names, or else COM1 where something says
/// it is there.
fn sinks() -> Sinks {
    let mut uarts = [None; MAX_SINKS];
    let why = match acpi::find(b"SPCR") {
        Ok(Some(table)) => match spcr::decode(table) {
            Ok(found) => {
                uarts[0] = Some(found);
                return Sinks {
                    uarts,
                    source: Source::Spcr,
                };
            }
            Err(why) => Some(why),
        },
        _ => None,
    };
    if logging::has_serial() || uart::COM1.answers() {
        uarts[0] = Some(uart::COM1);
        Sinks {
            uarts,
            source: Source::Com1(why),
        }
    } else {
        Sinks {
            uarts,
            source: Source::None(why),
        }
    }
}

/// Decides whether to launch, and reports it where it will not. Returns the
/// sinks when it will.
pub fn check(cpu: &Cpu) -> Option<Sinks> {
    let mut blockers: Vec<String> = Vec::new();
    if cpu.vendor != Vendor::Amd {
        blockers.push(format!("the CPU is {}, not AMD", cpu.name()));
    } else if !check::reported("amd.skinit", Tag::Pass) {
        blockers.push("amd.skinit did not pass, so SKINIT raises #UD".into());
    }
    if check::reported("cpu.machine_check", Tag::Fail) {
        blockers.push("cpu.machine_check failed".into());
    }
    if check::reported("cpu.bsp", Tag::Fail) {
        blockers.push("cpu.bsp failed, this is not the bootstrap processor".into());
    }
    if check::reported("amd.slb", Tag::Fail) {
        blockers.push("amd.slb failed, no SLB below 4 GiB".into());
    }
    if check::reported("tpm.present", Tag::Fail) {
        blockers.push("tpm.present failed, no TPM answers".into());
    }
    if cpu.vendor == Vendor::Amd && __cpuid(0x8000_0001).edx & CPUID_PAGE_1GB == 0 {
        blockers.push(
            "no 1 GiB pages (CPUID.80000001:EDX[26] clear), which the Secure \
             Loader maps with"
                .into(),
        );
    }
    // Last, since it writes to COM1's scratch register where nothing else
    // says a UART is there, and a blocked launch has no use for it.
    let sinks = blockers.is_empty().then(sinks);
    if let Some(Sinks {
        source: source @ Source::None(_),
        ..
    }) = &sinks
    {
        blockers.push(format!("no output sink the SL can use: {source}"));
    }

    if !blockers.is_empty() {
        check::after_summary(
            Tag::Fail,
            "launch.gate",
            &[("blockers", Value::Count(blockers.len() as u64))],
            format_args!("not launching: {}", blockers.join(", ")),
        );
        return None;
    }
    sinks
}

/// Reports a launch blocked after the gate, by what `launch=psp` needs.
pub fn block(why: &str) {
    check::after_summary(
        Tag::Fail,
        "launch.gate",
        &[("blockers", Value::Count(1))],
        format_args!("not launching: {why}"),
    );
}
