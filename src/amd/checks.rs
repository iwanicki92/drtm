// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Individual AMD DRTM precondition checks.
//!
//! These run only after the CPU has been identified as AMD, so nothing
//! here re-tests the vendor.

use core::arch::x86_64::__cpuid;

/// `CPUID.80000001:ECX` bit 12.
///
/// APM Vol 2, section 15.4: `SKINIT`/`STGI` are usable when either
/// `EFER.SVME` is 1 or this bit is 1, so this CPUID check alone is
/// sufficient without also reading `EFER`.
const SKINIT_CPUID_BIT: u32 = 1 << 12;

/// Whether the CPU advertises `SKINIT` support via CPUID.
///
/// This is a hard gate: without it, no other AMD DRTM check matters.
pub fn skinit_supported() -> bool {
    // An unimplemented CPUID leaf returns the highest implemented leaf's
    // values instead of zero, so check leaf 0x80000001 exists before reading
    // it. Every x86-64 CPU implements it, but reading a leaf unguarded is how
    // garbage gets mistaken for a feature bit.
    if __cpuid(0x8000_0000).eax < 0x8000_0001 {
        return false;
    }
    let result = __cpuid(0x8000_0001);
    result.ecx & SKINIT_CPUID_BIT != 0
}
