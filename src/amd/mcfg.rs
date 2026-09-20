// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Whether the ACPI MCFG places the configuration window where the MSR
//! says it is.
//!
//! An OS reaches configuration space through the table and a loader that
//! runs after `SKINIT` through the register, so the two naming different
//! windows means one of them reads something other than configuration
//! space.

use crate::acpi;
use crate::amd::mmconfig::{self, MmioConfig};
use crate::bytes::{read_u16, read_u64};
use crate::check;

const SIGNATURE: &[u8; 4] = b"MCFG";

/// The header is followed by 8 reserved bytes, then one 16-byte allocation
/// per segment: the base address, the segment, the first and last bus it
/// covers (PCI Firmware Specification 3.2, section 4.1.2).
const FIRST_ALLOCATION: usize = acpi::HEADER_LEN + 8;
const ALLOCATION_LEN: usize = 16;
const ALLOCATION_BASE: usize = 0;
const ALLOCATION_SEGMENT: usize = 8;
const ALLOCATION_START_BUS: usize = 10;
const ALLOCATION_END_BUS: usize = 11;

/// The MSR's window covers buses from 0 in segment 0, so that is the
/// allocation to hold it against.
const SEGMENT_0: u16 = 0;

/// Segment 0's allocation: its base and the last bus it covers.
fn segment_0(table: &[u8]) -> Option<(u64, u8)> {
    table
        .get(FIRST_ALLOCATION..)?
        .as_chunks::<ALLOCATION_LEN>()
        .0
        .iter()
        .map(|allocation| allocation.as_slice())
        .find(|allocation| {
            read_u16(allocation, ALLOCATION_SEGMENT) == Some(SEGMENT_0)
                && allocation[ALLOCATION_START_BUS] == 0
        })
        .map(|allocation| {
            (
                read_u64(allocation, ALLOCATION_BASE).unwrap_or(0),
                allocation[ALLOCATION_END_BUS],
            )
        })
}

/// Reports whether the table agrees with the window `amd.mmconfig` read.
pub fn report() {
    let MmioConfig::Enabled { base, buses } = mmconfig::probe() else {
        check::skip!(
            "amd.mmconfig_table",
            "MSR 0xC0010058 places no window, so there is nothing for the \
             MCFG to agree with"
        );
        return;
    };

    let table = match acpi::find_or(SIGNATURE, "no MCFG") {
        Ok(table) => table,
        Err(why) => {
            check::info!(
                "amd.mmconfig_table",
                msr_base = check::hex(base);
                "{why}, so an OS reaches configuration space through port I/O \
                 rather than the window at {base:#x}"
            );
            return;
        }
    };
    if !acpi::checksum_ok(table) {
        check::warn!(
            "amd.mmconfig_table",
            "MCFG checksum is wrong, so an OS discards the table"
        );
        return;
    }
    let Some((mcfg_base, end_bus)) = segment_0(table) else {
        check::warn!(
            "amd.mmconfig_table",
            msr_base = check::hex(base);
            "MCFG has no allocation for segment 0 from bus 0, where MSR \
             0xC0010058 puts a window at {base:#x}"
        );
        return;
    };

    let mcfg_buses = u64::from(end_bus) + 1;
    if mcfg_base != base {
        check::warn!(
            "amd.mmconfig_table",
            mcfg_base = check::hex(mcfg_base),
            msr_base = check::hex(base);
            "MCFG places segment 0's window at {mcfg_base:#x} while MSR \
             0xC0010058 says {base:#x}, so an OS and a loader reading the \
             register reach different windows"
        );
    } else if mcfg_buses != buses {
        check::info!(
            "amd.mmconfig_table",
            mcfg_base = check::hex(mcfg_base),
            msr_base = check::hex(base),
            mcfg_buses = check::count(mcfg_buses),
            msr_buses = check::count(buses);
            "MCFG places segment 0's window at {mcfg_base:#x}, matching MSR \
             0xC0010058, but covers {mcfg_buses} buses where the register \
             covers {buses}"
        );
    } else {
        check::info!(
            "amd.mmconfig_table",
            mcfg_base = check::hex(mcfg_base),
            msr_base = check::hex(base),
            mcfg_buses = check::count(mcfg_buses);
            "MCFG places segment 0's window at {mcfg_base:#x} for \
             {mcfg_buses} buses, matching MSR 0xC0010058"
        );
    }
}
