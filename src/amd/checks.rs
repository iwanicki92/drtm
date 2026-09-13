// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Individual AMD DRTM precondition checks.

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
    let result = __cpuid(0x8000_0001);
    result.ecx & SKINIT_CPUID_BIT != 0
}
