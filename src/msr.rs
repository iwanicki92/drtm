// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Reading model-specific registers.
//!
//! `rdmsr` needs ring 0, which a UEFI application has. What it does not have
//! is any guarantee the MSR exists, so guarding is the caller's job.

use core::arch::asm;

/// Reads the 64-bit value of `msr`.
///
/// # Safety
///
/// The MSR must be implemented on this CPU. Reading one that is not raises
/// `#GP`, so callers first test whatever implies it exists: a CPUID feature
/// bit, or the vendor for a vendor-specific MSR.
pub unsafe fn read(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    // Safety: the caller guarantees the MSR exists, which is this function's
    // whole contract. `rdmsr` needs ring 0, which a UEFI application has.
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags),
        );
    }
    (u64::from(high) << 32) | u64::from(low)
}
