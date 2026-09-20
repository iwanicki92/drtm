// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Where the platform puts PCI MMIO configuration space.
//!
//! Read from an AMD-specific MSR, so this runs only after the vendor test.
//! Reported rather than judged: where the window sits constrains how a
//! launch reaches it, not whether the platform can launch.

use crate::{check, msr};

/// `MSR 0xC0010058`, the MMIO configuration base address register. Bit 0
/// enables the window and bits 5:2 select its size. Linux has carried it
/// since family 10h as `MSR_FAM10H_MMIO_CONF_BASE`, but only the Processor
/// Programming Reference gives the base field's width. Reading it on a
/// non-AMD CPU raises `#GP` on silicon, though not under QEMU, whose MSR
/// helpers switch on the register number and know nothing of the vendor.
const MMIO_CONFIG_BASE_MSR: u32 = 0xC001_0058;
const MMIO_CONFIG_ENABLE: u64 = 1;
const BUS_RANGE_SHIFT: u32 = 2;
const BUS_RANGE_MASK: u64 = 0xF;

/// Bits 51:20, the widest base field any PPR documents (family 1Ah model
/// 02h, PPR 57238). Model 70h stops at 47:20 and reserves 51:48, which read
/// zero, so the wider mask is correct there too and no model table is
/// needed. Either way a base this high already fails the 4 GiB test.
const BASE_ADDRESS_MASK: u64 = 0x000F_FFFF_FFF0_0000;

/// One bus is 32 devices of 8 functions of 4 KiB, so 1 MiB of window each.
/// `BusRange` counts buses as `2^n` up to 8h, then adds segments of 256, and
/// `2^n` stays the total across them, so one shift covers the whole field.
const BYTES_PER_BUS: u64 = 1 << 20;
const FOUR_GIB: u64 = 1 << 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmioConfig {
    /// The enable bit is clear, so the platform describes the window some
    /// other way, such as a chipset register plus ACPI MCFG. Not a failure.
    Disabled,
    Enabled {
        base: u64,
        buses: u64,
    },
}

/// Reads where the MMIO configuration window is, if the MSR places it.
pub fn probe() -> MmioConfig {
    // Safety: everything in this module runs only once the CPU is known to
    // be AMD, which is what defines this MSR.
    let value = unsafe { msr::read(MMIO_CONFIG_BASE_MSR) };
    if value & MMIO_CONFIG_ENABLE == 0 {
        return MmioConfig::Disabled;
    }
    MmioConfig::Enabled {
        base: value & BASE_ADDRESS_MASK,
        buses: 1 << ((value >> BUS_RANGE_SHIFT) & BUS_RANGE_MASK),
    }
}

/// Runs the MMIO configuration space check and reports it.
pub fn report() {
    match probe() {
        MmioConfig::Disabled => {
            check::skip!(
                "amd.mmconfig",
                "MSR 0xC0010058 has its enable bit clear, so this platform \
                 places the window some other way (a chipset register plus \
                 ACPI MCFG)"
            );
        }
        MmioConfig::Enabled { base, buses } => {
            let end = base + buses * BYTES_PER_BUS;
            if end <= FOUR_GIB {
                check::info!(
                    "amd.mmconfig",
                    base = check::hex(base),
                    end = check::hex(end),
                    buses = check::count(buses);
                    "window at {base:#x}..{end:#x} ({buses} buses), below 4 GiB"
                );
            } else {
                // SKINIT leaves the CPU in 32-bit mode with paging off (_APM
                // Vol 2, section 15.27_), and a loader unaware of this reads
                // memory instead, as `secure-kernel-loader:pci.c` did.
                check::info!(
                    "amd.mmconfig",
                    base = check::hex(base),
                    end = check::hex(end),
                    buses = check::count(buses);
                    "window at {base:#x}..{end:#x} ({buses} buses), not \
                     entirely below 4 GiB, which the 32-bit unpaged state \
                     SKINIT leaves cannot address, so port I/O is needed"
                );
            }
        }
    }
}
