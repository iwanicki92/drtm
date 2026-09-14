// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Checks on the state of the processor we are running on.
//!
//! Not specific to any DRTM path. Every vendor's dynamic launch is issued by
//! the bootstrap processor, and the registers read here are architectural.

use core::arch::asm;
use core::arch::x86_64::__cpuid;

use log::{error, info, warn};

use crate::msr;

/// `CPUID.1:EDX` bit 9. A local APIC is present, which is what makes
/// `IA32_APIC_BASE` readable.
const CPUID_APIC_BIT: u32 = 1 << 9;

/// `CPUID.1:EDX` bits 7 and 14: the machine-check exception and the
/// machine-check architecture. Both are needed before the MSRs below exist.
const CPUID_MCE_BIT: u32 = 1 << 7;
const CPUID_MCA_BIT: u32 = 1 << 14;

/// `IA32_APIC_BASE`, whose bit 8 is set only on the bootstrap processor.
const IA32_APIC_BASE: u32 = 0x1B;
const APIC_BASE_BSP: u64 = 1 << 8;

/// Machine-check MSRs. `IA32_MCG_CAP` bits 7:0 hold the number of reporting
/// banks, and bank `i`'s status register is `IA32_MC0_STATUS + 4 * i`.
const IA32_MCG_CAP: u32 = 0x179;
const IA32_MCG_STATUS: u32 = 0x17A;
const IA32_MC0_STATUS: u32 = 0x401;
const MCG_CAP_BANK_COUNT: u64 = 0xFF;
const MCG_STATUS_MCIP: u64 = 1 << 2;
const MCI_STATUS_VALID: u64 = 1 << 63;

/// `CR0` and `EFLAGS` bits a dynamic launch constrains: caches on, native FPU
/// error reporting, and not in virtual-8086 mode.
const CR0_CD: u64 = 1 << 30;
const CR0_NW: u64 = 1 << 29;
const CR0_NE: u64 = 1 << 5;
const EFLAGS_VM: u64 = 1 << 17;

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

fn read_cr0() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov {}, cr0", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

fn read_eflags() -> u64 {
    let value: u64;
    // No `nostack`: `pushfq` writes through the stack pointer.
    unsafe {
        asm!("pushfq", "pop {}", out(reg) value, options(preserves_flags));
    }
    value
}

fn report_bsp() {
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

/// Reports the `CR0`/`EFLAGS` state a launch requires.
///
/// A deviation is not a failure: the launch preamble sets these bits itself,
/// unconditionally. It is still worth saying, because firmware leaving caches
/// off is a sign of something else being wrong.
fn report_control_registers() {
    let cr0 = read_cr0();
    let eflags = read_eflags();
    let mut deviations = 0;
    for (deviates, description) in [
        (cr0 & CR0_CD != 0, "CR0.CD set, caches disabled"),
        (cr0 & CR0_NW != 0, "CR0.NW set, write-through disabled"),
        (
            cr0 & CR0_NE == 0,
            "CR0.NE clear, FPU errors not reported natively",
        ),
        (eflags & EFLAGS_VM != 0, "EFLAGS.VM set, virtual-8086 mode"),
    ] {
        if deviates {
            warn!("[WARN] {description}, a launch would have to correct this");
            deviations += 1;
        }
    }
    if deviations == 0 {
        info!(
            "[PASS] CR0 and EFLAGS already as a launch needs them \
             (CD, NW, VM clear, NE set)"
        );
    }
}

/// Reports whether the machine-check state permits a launch.
///
/// Unlike the control registers this is not something a launch may correct:
/// an error in progress or logged is a real condition on the machine.
fn report_machine_check() {
    let features = __cpuid(0x1).edx;
    if features & CPUID_MCE_BIT == 0 || features & CPUID_MCA_BIT == 0 {
        warn!(
            "[WARN] no machine-check architecture (CPUID.1:EDX[7] or [14] \
             clear), so the machine-check preconditions cannot be read"
        );
        return;
    }

    // Safety: CPUID reports MCA, which is what defines these MSRs. The bank
    // count from IA32_MCG_CAP is exactly how many banks exist to be read.
    let status = unsafe { msr::read(IA32_MCG_STATUS) };
    let banks = (unsafe { msr::read(IA32_MCG_CAP) } & MCG_CAP_BANK_COUNT) as u32;

    let in_progress = status & MCG_STATUS_MCIP != 0;
    if in_progress {
        error!("[FAIL] machine check in progress (IA32_MCG_STATUS.MCIP set)");
    }

    let mut logged = 0;
    for bank in 0..banks {
        let bank_status = unsafe { msr::read(IA32_MC0_STATUS + 4 * bank) };
        if bank_status & MCI_STATUS_VALID != 0 {
            error!("[FAIL] IA32_MC{bank}_STATUS holds a logged error ({bank_status:#018x})");
            logged += 1;
        }
    }

    if !in_progress && logged == 0 {
        info!("[PASS] no machine check in progress, all {banks} status banks clear");
    }
}

/// Runs every processor-state check and reports it.
pub fn report() {
    report_bsp();
    report_control_registers();
    report_machine_check();
}
