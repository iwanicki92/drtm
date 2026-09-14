// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Checks on the state of the processor we are running on.
//!
//! Not specific to any DRTM path. Every vendor's dynamic launch is issued by
//! the bootstrap processor, and the registers read here are architectural.

use core::arch::x86_64::__cpuid;

use log::{error, info, warn};

use crate::msr;

/// `CPUID.1:EDX` bit 9. A local APIC is present, which is what makes
/// `IA32_APIC_BASE` readable.
const CPUID_APIC_BIT: u32 = 1 << 9;

/// `IA32_APIC_BASE`, whose bit 8 is set only on the bootstrap processor.
const IA32_APIC_BASE: u32 = 0x1B;
const APIC_BASE_BSP: u64 = 1 << 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bsp {
    Yes,
    No,
    /// No local APIC, so `IA32_APIC_BASE` does not exist to be read.
    Unknown,
}

/// Whether this is the bootstrap processor.
pub fn on_bsp() -> Bsp {
    if __cpuid(0x1).edx & CPUID_APIC_BIT == 0 {
        return Bsp::Unknown;
    }
    // Safety: CPUID reports a local APIC, which is what defines this MSR.
    let apic_base = unsafe { msr::read(IA32_APIC_BASE) };
    if apic_base & APIC_BASE_BSP != 0 {
        Bsp::Yes
    } else {
        Bsp::No
    }
}

/// Runs every processor-state check and reports it.
pub fn report() {
    match on_bsp() {
        Bsp::Yes => {
            info!("[PASS] running on the bootstrap processor (IA32_APIC_BASE[8])");
        }
        Bsp::No => {
            error!(
                "[FAIL] not running on the bootstrap processor \
                 (IA32_APIC_BASE[8] clear), a dynamic launch must be issued \
                 by the BSP"
            );
        }
        Bsp::Unknown => {
            warn!(
                "[WARN] no local APIC (CPUID.1:EDX[9] clear), so whether this \
                 is the bootstrap processor cannot be read"
            );
        }
    }
}
