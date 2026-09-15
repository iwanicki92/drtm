// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Which AMD Secure Processor the platform has, if any.
//!
//! Reported rather than judged. Classic `SKINIT` does not go through the
//! PSP, and which platforms need the PSP path instead is not something this
//! can answer, so every outcome here is a fact and none is a verdict.

use core::fmt;

use crate::check;
use crate::pci::{Devices, Function};

/// AMD's PCI vendor ID, the only one any Secure Processor answers with.
const VENDOR_AMD: u16 = 0x1022;

/// What the loaders' device table says a Secure Processor offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drtm {
    /// An AMD Secure Processor with no DRTM service. GRUB and the Secure
    /// Kernel Loader drop these in `is_drtm_device()` and carry on as if
    /// nothing answered.
    None,
    V1,
    V2,
    V3,
    /// The Turin generation's CCP/ASP, which drives the same mailbox as v2
    /// and v3 under a name of its own.
    Asp,
}

impl fmt::Display for Drtm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "no DRTM service"),
            Self::V1 => write!(f, "PSP v1"),
            Self::V2 => write!(f, "PSP v2"),
            Self::V3 => write!(f, "PSP v3"),
            Self::Asp => write!(f, "ASP"),
        }
    }
}

/// Every Secure Processor the loaders know, by device ID. GRUB, the Secure
/// Kernel Loader and Linux carry the same table (`psp_devs_list` in each of
/// `grub-core/loader/slaunch/psp.c`, `secure-kernel-loader:psp.c` and
/// Linux's `slaunch.c`), and the last two entries are what
/// `meta-trenchboot` patches into all three.
const DEVICES: [(u16, Drtm); 9] = [
    (0x1537, Drtm::None),
    (0x1456, Drtm::V1),
    (0x1468, Drtm::None),
    (0x1486, Drtm::V2),
    (0x15DF, Drtm::V3),
    (0x1649, Drtm::V2),
    (0x14CA, Drtm::V3),
    (0x15C7, Drtm::V3),
    (0x156E, Drtm::Asp),
];

fn lookup(vendor: u16, device: u16) -> Option<Drtm> {
    if vendor != VENDOR_AMD {
        return None;
    }
    DEVICES
        .iter()
        .find(|(id, _)| *id == device)
        .map(|(_, drtm)| *drtm)
}

/// Reports every AMD Secure Processor configuration space offers.
///
/// Never a pass or a fail. The device ID says which mailbox a loader would
/// drive, not whether the PSP's firmware would answer: that takes a mailbox
/// command, which would change platform state rather than observe it.
pub fn report(devices: &Devices) {
    let mut found = 0;
    for entry in devices.iter() {
        let Some(drtm) = lookup(entry.vendor, entry.device) else {
            continue;
        };
        found += 1;
        describe(entry.function, entry.device, drtm);
    }

    if found == 0 {
        check::info!(
            "amd.psp",
            "no AMD Secure Processor in configuration space (isn't required \
             by classic SKINIT)"
        );
    }
}

/// Says what one match is, and what a loader would do with it.
fn describe(function: Function, device: u16, drtm: Drtm) {
    match drtm {
        // The three versions `init_drtm_interface()` maps mailbox registers
        // for, and no other (`secure-kernel-loader:psp.c`).
        Drtm::V2 | Drtm::V3 | Drtm::Asp => check::info!(
            "amd.psp",
            "{VENDOR_AMD:04x}:{device:04x} at {function}, {drtm}, the \
             interface a PSP launch would use"
        ),
        Drtm::V1 => check::info!(
            "amd.psp",
            "{VENDOR_AMD:04x}:{device:04x} at {function}, {drtm}, which no \
             loader maps a mailbox for"
        ),
        Drtm::None => check::info!(
            "amd.psp",
            "{VENDOR_AMD:04x}:{device:04x} at {function}, {drtm}"
        ),
    }
}
