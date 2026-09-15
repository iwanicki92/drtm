// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! What PCI configuration space says is on the machine.
//!
//! Reads through `EFI_PCI_ROOT_BRIDGE_IO_PROTOCOL`, so the firmware decides
//! whether that means MMCONFIG or the legacy ports and neither has to be
//! reached for directly. Checks that need a device identify it out of here.

use core::fmt;

use log::{debug, trace};
use uefi::Identify;
use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams, SearchType};
use uefi::proto::pci::PciIoAddress;
use uefi::proto::pci::root_bridge::{PciIoAccessPci, PciRootBridgeIo};

use crate::check;

/// Configuration space offsets, all within the first 256 bytes so the legacy
/// window is enough (PCI Local Bus Specification 3.0, section 6.1).
const REG_ID: u8 = 0x00;
const REG_STATUS: u8 = 0x06;
const REG_CLASS: u8 = 0x08;
const REG_HEADER_TYPE: u8 = 0x0E;
const REG_CAPABILITY_POINTER: u8 = 0x34;

/// Status bit 4. The capability pointer holds nothing readable unless it is
/// set, so a device without it has no list to walk.
const STATUS_CAPABILITY_LIST: u16 = 1 << 4;

/// Header type bit 7. Clear means function 0 is the only function, so the
/// other seven need not be probed.
const HEADER_TYPE_MULTI_FUNCTION: u8 = 1 << 7;

/// A malformed list can point at itself. Devices have 48 dwords of space
/// after the header, so nothing legitimate needs more hops than that.
const MAX_CAPABILITIES: u32 = 48;

/// One function that answered in configuration space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Function {
    pub segment: u32,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl Function {
    const fn address(self) -> PciIoAddress {
        PciIoAddress::new(self.bus, self.device, self.function)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04x}:{:02x}:{:02x}.{}",
            self.segment, self.bus, self.device, self.function
        )
    }
}

/// One entry of a function's capability list.
///
/// `kind` is bits 18:16 of the header, which AMD uses to tell apart the two
/// things it puts under capability ID `0x0F`, the IOMMU and the DEV.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capability {
    pub offset: u8,
    pub id: u8,
    pub kind: u8,
}

/// Calls `visit` for every capability in `function`'s list.
pub fn for_each_capability(
    pci: &PciIoAccessPci<'_>,
    function: Function,
    mut visit: impl FnMut(Capability),
) {
    let address = function.address();
    let status = pci
        .read_one::<u16>(address.with_register(REG_STATUS))
        .unwrap_or(0);
    if status & STATUS_CAPABILITY_LIST == 0 {
        return;
    }
    // The low two bits of a pointer are reserved and read as zero on
    // conforming devices, so mask rather than trust them.
    let mut offset = pci
        .read_one::<u8>(address.with_register(REG_CAPABILITY_POINTER))
        .unwrap_or(0)
        & 0xFC;
    let mut seen = 0;
    while offset != 0 && seen < MAX_CAPABILITIES {
        let header = pci
            .read_one::<u32>(address.with_register(offset))
            .unwrap_or(0);
        visit(Capability {
            offset,
            id: header as u8,
            kind: ((header >> 16) & 0x7) as u8,
        });
        offset = ((header >> 8) as u8) & 0xFC;
        seen += 1;
    }
}

/// Calls `visit` for every function every root bridge answers for.
///
/// Every bus each bridge decodes, not just bus 0. An AMD server has
/// several IOMMUs, on buses of their own rather than all on bus 0.
pub fn for_each_function(mut visit: impl FnMut(&PciIoAccessPci<'_>, Function)) {
    let handles = match boot::locate_handle_buffer(SearchType::ByProtocol(&PciRootBridgeIo::GUID)) {
        Ok(handles) => handles,
        Err(error) => {
            check::warn!(
                "pci.root_bridge",
                "protocol not published ({:?}), so nothing can be read out \
                 of configuration space",
                error.status()
            );
            return;
        }
    };
    debug!("[SCAN] PCI root bridges: {}", handles.len());

    for handle in handles.iter() {
        let params = OpenProtocolParams {
            handle: *handle,
            agent: boot::image_handle(),
            controller: None,
        };
        // Safety: the handle came from `locate_handle_buffer` for this very
        // protocol. `GetProtocol` leaves the firmware's PCI bus driver
        // attached, where an exclusive open disconnects it and takes the
        // console down with it for the rest of the boot.
        let bridge = unsafe {
            boot::open_protocol::<PciRootBridgeIo>(params, OpenProtocolAttributes::GetProtocol)
        };
        let mut bridge = match bridge {
            Ok(bridge) => bridge,
            Err(error) => {
                check::warn!(
                    "pci.root_bridge",
                    "one bridge would not open ({:?})",
                    error.status()
                );
                continue;
            }
        };
        let segment = bridge.segment_nr();
        let pci = bridge.pci();
        for bus in 0..=u8::MAX {
            for device in 0..32u8 {
                let zero = Function {
                    segment,
                    bus,
                    device,
                    function: 0,
                };
                if !present(&pci, zero) {
                    continue;
                }
                visit(&pci, zero);
                let header = pci
                    .read_one::<u8>(zero.address().with_register(REG_HEADER_TYPE))
                    .unwrap_or(0);
                if header & HEADER_TYPE_MULTI_FUNCTION == 0 {
                    continue;
                }
                for function in 1..8u8 {
                    let other = Function { function, ..zero };
                    if present(&pci, other) {
                        visit(&pci, other);
                    }
                }
            }
        }
    }
}

/// Whether anything answers at `function`.
///
/// An absent function reads all ones, the bus returning nothing rather than
/// a device answering.
fn present(pci: &PciIoAccessPci<'_>, function: Function) -> bool {
    match pci.read_one::<u32>(function.address().with_register(REG_ID)) {
        Ok(id) => id != u32::MAX,
        Err(_) => false,
    }
}

/// Reports every function the root bridges answer for.
pub fn report() {
    for_each_function(|pci, function| {
        let address = function.address();
        let id = pci
            .read_one::<u32>(address.with_register(REG_ID))
            .unwrap_or(0);
        let class = pci
            .read_one::<u32>(address.with_register(REG_CLASS))
            .unwrap_or(0)
            >> 8;
        trace!(
            "[SCAN] {function} id {:04x}:{:04x} class {class:06x}",
            id as u16,
            (id >> 16) as u16
        );
        for_each_capability(pci, function, |capability| {
            trace!(
                "[SCAN]   capability {:02x} type {} at {:02x}",
                capability.id, capability.kind, capability.offset
            );
        });
    });
}
