// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Waiting without a firmware timer: the time-stamp counter, at a rate the
//! preamble measured against UEFI's `Stall()` while it still could.

use core::arch::x86_64::_rdtsc;

pub fn tsc() -> u64 {
    // Safety: `rdtsc` is available on every x86-64 processor and reads a
    // counter, touching no memory.
    unsafe { _rdtsc() }
}

/// Whole milliseconds since the counter read `start`.
pub fn ms_since(start: u64, tsc_per_ms: u64) -> u64 {
    tsc().saturating_sub(start) / tsc_per_ms.max(1)
}

/// A point in time `ms` milliseconds from its creation.
pub struct Deadline(u64);

impl Deadline {
    pub fn after(ms: u64, tsc_per_ms: u64) -> Self {
        Self(tsc().saturating_add(ms.saturating_mul(tsc_per_ms)))
    }

    pub fn passed(&self) -> bool {
        tsc() >= self.0
    }
}

/// Spins for `ms` milliseconds.
pub fn delay(ms: u64, tsc_per_ms: u64) {
    let deadline = Deadline::after(ms, tsc_per_ms);
    while !deadline.passed() {
        core::hint::spin_loop();
    }
}
