// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! The Secure Processor registers the ASPT names, and the DRTM mailbox.
//!
//! Every one is a status or command register whose read issues nothing: a
//! command is a write. The DRTM service's argument registers C2PMSG_93 to
//! 95 are left out, since they hold a response and there is none until a
//! command has been issued.

use core::fmt;

use crate::acpi;
use crate::amd::{aspt, psp};
use crate::check;
use crate::logging::record;
use crate::pci::Devices;

/// The register offsets each structure type names, and where in the
/// structure each offset sits (_AMD publication 58193, tables 5 to 9 and
/// table 23_). Type 4 names a memory buffer address rather than a register,
/// so nothing here reads it.
const REGISTERS: [(u16, &[(usize, &str)]); 4] = [
    (0, &[(8, "FEATURE"), (12, "INT_ENABLE"), (16, "INT_STATUS")]),
    (
        1,
        &[
            (8, "SEV_CMD_RESP"),
            (12, "SEV_CMD_BUF_LO"),
            (16, "SEV_CMD_BUF_HI"),
        ],
    ),
    (2, &[(8, "ACPI_CMD_RESP")]),
    (
        3,
        &[
            (8, "TEE_CMDRESP"),
            (12, "TEE_CMDBUF_LO"),
            (16, "TEE_CMDBUF_HI"),
        ],
    ),
];

/// What the table declares, then each register it names, then the DRTM
/// mailbox, which it does not name.
///
/// Both the Secure Processor and the table are needed: the table alone
/// would have this reading an address with nothing behind it.
pub fn dump(devices: &Devices) {
    if !psp::has_mailbox(devices) {
        record("psp", &[("found", check::flag(false))]);
        return;
    }
    let Ok(Some(table)) = acpi::find(aspt::SIGNATURE) else {
        record("psp", &[("table", check::flag(false))]);
        return;
    };
    let (Some((base, pages)), Some(structures)) = (aspt::block(table), aspt::structures(table))
    else {
        record("psp", &[("table", check::flag(false))]);
        return;
    };
    record(
        "psp",
        &[
            ("base", check::hex(base as u64)),
            ("pages", check::count(pages as u64)),
            ("structures", check::count(structures.len() as u64)),
        ],
    );

    for structure in &structures {
        let named = REGISTERS
            .iter()
            .find(|(kind, _)| *kind == structure.kind)
            .map(|(_, named)| *named)
            .unwrap_or_default();
        for &(at, name) in named {
            // A zero offset names no register, which is what the ACPI
            // mailbox structure's reserved tail reads as.
            match aspt::field(structure.bytes, at).filter(|offset| *offset != 0) {
                None => continue,
                Some(offset) => read(table, offset, name),
            }
        }
    }
    if let Some(offset) = mailbox_offset(table, base) {
        read(table, offset, "DRTM_CMD_RESP");
    }
}

/// A register's name together with the mailbox number it is usually called
/// by, like `FEATURE (C2PMSG_63)`. The interrupt registers sit below the
/// mailbox block and have no number, so they print their name alone.
struct Named {
    name: &'static str,
    c2pmsg: Option<usize>,
}

impl fmt::Display for Named {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.c2pmsg {
            Some(number) => write!(f, "{} (C2PMSG_{number})", self.name),
            None => f.write_str(self.name),
        }
    }
}

/// Where the DRTM mailbox is, as an offset into the block, so it is dumped
/// the same way as the registers the table names.
fn mailbox_offset(table: &[u8], base: usize) -> Option<u32> {
    u32::try_from(aspt::mailbox(table)?.checked_sub(base)?).ok()
}

/// One register, or a record saying it was left alone. An offset the table's
/// own page count does not cover is not read, since what is there is then
/// whatever else firmware put at that address.
fn read(table: &[u8], offset: u32, name: &'static str) {
    let name = Named {
        name,
        c2pmsg: aspt::c2pmsg(table, offset),
    };
    let Some(address) = aspt::register(table, offset) else {
        record(
            "psp",
            &[
                ("name", check::text!("{name}")),
                ("at", check::hex(offset)),
                ("read", check::flag(false)),
            ],
        );
        return;
    };
    // Safety: the ASPT says the Secure Processor's registers are here, UEFI
    // identity-maps what firmware reports, and these are 32-bit status and
    // command registers whose reads issue nothing.
    let value = unsafe { (address as *const u32).read_volatile() };
    record(
        "psp",
        &[
            ("name", check::text!("{name}")),
            ("at", check::hex(offset)),
            ("value", check::hex(value)),
        ],
    );
}
