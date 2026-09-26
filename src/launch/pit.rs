// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Where the PIT's interrupt reaches an IO-APIC, from the MADT, for the
//! Secure Loader's TMR probe.
//!
//! The PIT is ISA IRQ 0. An interrupt source override moves it to another
//! GSI and may set its polarity and trigger (ACPI 6.5, section 5.2.12.5),
//! and the IO-APIC whose GSI base is the highest at or below that GSI has
//! it on the pin the difference names.

use common::params::Pit;

use crate::acpi;
use crate::bytes::{read_u16, read_u32};

/// MADT entries start after the header, the local APIC address and flags.
const ENTRIES: usize = acpi::HEADER_LEN + 8;
const IO_APIC: u8 = 1;
const OVERRIDE: u8 = 2;
const ISA_PIT: u8 = 0;

/// MPS INTI flags: polarity in bits 1:0, trigger in 3:2, 3 meaning active
/// low and level. 0 is the bus's default, which for ISA is high and edge.
const ACTIVE_LOW: u16 = 0b11;
const LEVEL: u16 = 0b11 << 2;

pub fn find() -> Option<Pit> {
    let madt = acpi::find(b"APIC").ok()??;
    let mut gsi = u32::from(ISA_PIT);
    let mut flags = 0;
    let mut ioapics = [(0u32, 0u32); 16];
    let mut count = 0;
    let mut at = ENTRIES;
    while at + 2 <= madt.len() {
        let (kind, len) = (madt[at], usize::from(madt[at + 1]));
        let entry = madt.get(at..at + len)?;
        if len < 2 {
            return None;
        }
        match kind {
            IO_APIC if len >= 12 && count < ioapics.len() => {
                ioapics[count] = (read_u32(entry, 4)?, read_u32(entry, 8)?);
                count += 1;
            }
            OVERRIDE if len >= 10 && entry[2] == 0 && entry[3] == ISA_PIT => {
                gsi = read_u32(entry, 4)?;
                flags = read_u16(entry, 8)?;
            }
            _ => {}
        }
        at += len;
    }
    let &(ioapic, base) = ioapics[..count]
        .iter()
        .filter(|&&(_, base)| base <= gsi)
        .max_by_key(|&&(_, base)| base)?;
    Some(Pit {
        ioapic,
        pin: u8::try_from(gsi - base).ok()?,
        active_low: flags & ACTIVE_LOW == ACTIVE_LOW,
        level: flags & LEVEL == LEVEL,
    })
}
