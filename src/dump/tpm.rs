// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! The TPM's interface registers, under whichever layout is active.
//!
//! Sends the TPM no command, and reads only registers a read cannot change.
//! The firmware event log is not gathered: asking for it makes firmware
//! start keeping its final events table, which is a change to the platform.

use crate::check;
use crate::logging::record;
use crate::tpm::{self, Interface, Probe};

/// Registers safe to read under the FIFO layout, with their widths (PTP
/// table 20). `TPM_DATA_FIFO` is left out, since reading it takes a byte.
const FIFO_REGISTERS: [(usize, usize, &str); 10] = [
    (0x00, 1, "ACCESS"),
    (0x08, 4, "INT_ENABLE"),
    (0x0C, 1, "INT_VECTOR"),
    (0x10, 4, "INT_STATUS"),
    (0x14, 4, "INTF_CAPABILITY"),
    (0x18, 4, "STS"),
    (0x1C, 4, "INTF_CAPABILITYX"),
    (0x30, 4, "INTERFACE_ID"),
    (0xF00, 4, "DID_VID"),
    (0xF04, 1, "RID"),
];

/// The CRB layout's registers, all plain reads (PTP table 20), each read at
/// its own width.
const CRB_REGISTERS: [(usize, usize, &str); 15] = [
    (0x00, 4, "LOC_STATE"),
    (0x0C, 4, "LOC_STS"),
    (0x1C, 4, "INTF_CAPABILITYX"),
    (0x30, 8, "CRB_INTF_ID"),
    (0x40, 4, "CRB_CTRL_REQ"),
    (0x44, 4, "CRB_CTRL_STS"),
    (0x48, 4, "CRB_CTRL_CANCEL"),
    (0x4C, 4, "CRB_CTRL_START"),
    (0x50, 4, "CRB_INT_ENABLE"),
    (0x54, 4, "CRB_INT_STS"),
    (0x58, 4, "CRB_CTRL_CMD_SIZE"),
    (0x5C, 4, "CRB_CTRL_CMD_LADDR"),
    (0x60, 4, "CRB_CTRL_CMD_HADDR"),
    (0x64, 4, "CRB_CTRL_RSP_SIZE"),
    (0x68, 8, "CRB_CTRL_RSP_ADDR"),
];

pub fn dump() {
    let Probe::Present { interface, .. } = tpm::probe() else {
        record("tpm", &[("found", check::flag(false))]);
        return;
    };
    let registers: &[_] = match interface {
        Interface::Tis => &FIFO_REGISTERS,
        Interface::Crb => &CRB_REGISTERS,
    };
    for &(offset, width, name) in registers {
        let address = tpm::locality_base(0) + offset;
        // Safety: the offset is inside locality 0's 4 KiB window and names a
        // register of this width under the active layout, whose reads have no
        // side effect.
        let value = unsafe {
            match width {
                1 => u64::from((address as *const u8).read_volatile()),
                4 => u64::from((address as *const u32).read_volatile()),
                _ => (address as *const u64).read_volatile(),
            }
        };
        record(
            "tpm",
            &[
                ("interface", check::text!("{interface:?}")),
                ("name", check::text!("{name}")),
                ("value", check::hex(value)),
            ],
        );
    }
}
