// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The northbridge function a launch releases `SL_DEV` through.
//!
//! Reported rather than judged. On family 17h and later the control is
//! `MEMPROT_CR` at offset `0x384` of function 0 of each node, which the
//! Secure Kernel Loader walks from `00:18.0` (`secure-kernel-loader:dev.c`).
//! No public AMD document defines the register, so what it holds before a
//! launch is a fact to record, not a precondition to hold the platform to.

use crate::amd::psp::VENDOR_AMD;
use crate::check;
use crate::pci::{self, Devices, Function};

/// Node 0's function 0. Node `n` is device `0x18 + n`, and the device field
/// ends at `0x1F`, so at most eight nodes.
const NODE_0: Function = Function {
    segment: 0,
    bus: 0,
    device: 0x18,
    function: 0,
};
const LAST_NODE_DEVICE: u8 = 0x1F;

/// The register and its enable bit, as SKL's `include/dev.h` has them. Past
/// the first 256 bytes, so reading it takes extended configuration space.
const MEMPROT_CR: u32 = 0x384;
const MEMPROT_EN: u32 = 1 << 0;

/// Whether `device` is function 0 of a node, where the walk looks.
fn is_node(device: &pci::Device) -> bool {
    let function = device.function;
    device.vendor == VENDOR_AMD
        && function.segment == NODE_0.segment
        && function.bus == NODE_0.bus
        && function.function == NODE_0.function
        && (NODE_0.device..=LAST_NODE_DEVICE).contains(&function.device)
}

/// Reports each node's function 0 and what its `MEMPROT_CR` holds.
pub fn report(devices: &Devices) {
    let mut found = 0;
    pci::for_each_device(devices, |pci, device| {
        if !is_node(device) {
            return;
        }
        found += 1;
        let function = device.function;
        let id = check::text!("{VENDOR_AMD:04x}:{:04x}", device.device);
        let address = function.address().with_extended_register(MEMPROT_CR);
        match pci.read_one::<u32>(address) {
            Ok(memprot) => {
                let enabled = memprot & MEMPROT_EN != 0;
                check::info!(
                    "amd.northbridge",
                    function = check::text!("{function}"),
                    device = id,
                    memprot = check::hex(memprot),
                    memprot_en = check::flag(enabled);
                    "{VENDOR_AMD:04x}:{:04x} at {function}, MEMPROT_CR at \
                     {MEMPROT_CR:#x} reads {memprot:#x}, MEMPROT_EN {}",
                    device.device,
                    if enabled {
                        "set before any launch"
                    } else {
                        "clear, as before a launch"
                    }
                );
            }
            Err(error) => {
                check::info!(
                    "amd.northbridge",
                    function = check::text!("{function}"),
                    device = id,
                    status = check::text!("{:?}", error.status());
                    "{VENDOR_AMD:04x}:{:04x} at {function}, but the root \
                     bridge would not read offset {MEMPROT_CR:#x} ({:?}), so \
                     MEMPROT_CR is out of reach through this protocol",
                    device.device,
                    error.status()
                );
            }
        }
    });

    if found == 0 {
        check::info!(
            "amd.northbridge",
            "no AMD function at {NODE_0}, where the walk that releases \
             SL_DEV after a launch starts"
        );
    }
}
