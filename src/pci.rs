// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! What PCI configuration space says is on the machine.
//!
//! Reads through `EFI_PCI_ROOT_BRIDGE_IO_PROTOCOL`, so the firmware decides
//! whether that means MMCONFIG or the legacy ports and neither has to be
//! reached for directly. Checks that need a device identify it out of here.

use core::fmt;

use log::debug;
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
    pub const fn address(self) -> PciIoAddress {
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

/// Upper bound on what one scan records. QEMU's q35 answers for about ten
/// functions and a large server for a few hundred, so this is a guard
/// against a runaway scan rather than a limit anything should meet.
const MAX_FUNCTIONS: usize = 256;

/// One function that answered, with the identity checks match on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Device {
    pub function: Function,
    pub vendor: u16,
    pub device: u16,
    /// Base class and subclass, which say what a function is without
    /// naming a model, so a check can match a device nobody listed.
    pub class: u8,
    pub subclass: u8,
    /// Which root bridge handle found it, the one to read it through.
    bridge: usize,
}

/// Fills the unused tail of a [`Devices`], which `count` keeps anything
/// from reading.
const ABSENT: Device = Device {
    function: Function {
        segment: 0,
        bus: 0,
        device: 0,
        function: 0,
    },
    vendor: 0,
    device: 0,
    class: 0,
    subclass: 0,
    bridge: 0,
};

/// What one walk of configuration space found.
///
/// Built once by [`scan`] and handed to every check, so the walk that costs
/// something happens once however many checks read configuration space.
pub struct Devices {
    entries: [Device; MAX_FUNCTIONS],
    len: usize,
}

impl Devices {
    /// Records `device`, unless an earlier bridge already answered for the
    /// same function. Returns whether it was new.
    fn push(&mut self, device: Device) -> bool {
        if self.iter().any(|seen| seen.function == device.function) {
            return false;
        }
        if self.len < MAX_FUNCTIONS {
            self.entries[self.len] = device;
            self.len += 1;
        }
        true
    }

    pub fn iter(&self) -> impl Iterator<Item = &Device> {
        self.entries[..self.len].iter()
    }

    pub fn count(&self) -> usize {
        self.len
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

/// Opens every PCI root bridge in turn and calls `visit` with it, its
/// segment and its position among the handles.
fn for_each_bridge(mut visit: impl FnMut(&PciIoAccessPci<'_>, u32, usize)) {
    let handles = match boot::locate_handle_buffer(SearchType::ByProtocol(&PciRootBridgeIo::GUID)) {
        Ok(handles) => handles,
        Err(error) => {
            check::warn!(
                "pci.root_bridge",
                status = check::text!("{:?}", error.status());
                "protocol not published ({:?}), so nothing can be read out \
                 of configuration space",
                error.status()
            );
            return;
        }
    };
    for (index, handle) in handles.iter().enumerate() {
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
                    status = check::text!("{:?}", error.status());
                    "one bridge would not open ({:?})",
                    error.status()
                );
                continue;
            }
        };
        let segment = bridge.segment_nr();
        visit(&bridge.pci(), segment, index);
    }
}

/// Finds every function every root bridge answers for.
///
/// Every bus each bridge decodes, not just bus 0. An AMD server has several
/// IOMMUs, on buses of their own rather than all on bus 0. That is what
/// makes this the expensive part, and why one scan is shared by every check
/// rather than run once per check.
pub fn scan() -> Devices {
    let mut devices = Devices {
        entries: [ABSENT; MAX_FUNCTIONS],
        len: 0,
    };
    let mut bridges = 0;
    let mut repeated = 0;
    for_each_bridge(|pci, segment, bridge| {
        bridges += 1;
        let mut record = |device| {
            if !devices.push(device) {
                repeated += 1;
            }
        };
        for bus in 0..=u8::MAX {
            for device in 0..32u8 {
                let zero = Function {
                    segment,
                    bus,
                    device,
                    function: 0,
                };
                let Some(entry) = probe(pci, zero, bridge) else {
                    continue;
                };
                record(entry);
                let header = pci
                    .read_one::<u8>(zero.address().with_register(REG_HEADER_TYPE))
                    .unwrap_or(0);
                if header & HEADER_TYPE_MULTI_FUNCTION == 0 {
                    continue;
                }
                for function in 1..8u8 {
                    if let Some(entry) = probe(pci, Function { function, ..zero }, bridge) {
                        record(entry);
                    }
                }
            }
        }
    });
    debug!("[SCAN] PCI root bridges: {bridges}");
    // Firmware that does not hold each bridge to its own buses answers for
    // every function through every bridge. Only the first answer is kept.
    if repeated > 0 {
        debug!("[SCAN] functions answering through more than one bridge: {repeated}");
    }

    if devices.count() == MAX_FUNCTIONS {
        check::warn!(
            "pci.scan",
            "stopped at {MAX_FUNCTIONS} functions, so every check reading \
             configuration space saw only that many"
        );
    }
    devices
}

/// Calls `visit` for every function `devices` holds, with the root bridge
/// that answers for it open. Probes nothing: [`scan`] already did.
pub fn for_each_device(devices: &Devices, mut visit: impl FnMut(&PciIoAccessPci<'_>, &Device)) {
    for_each_bridge(|pci, _, bridge| {
        for device in devices.iter() {
            if device.bridge == bridge {
                visit(pci, device);
            }
        }
    });
}

/// Reads `function`'s identity, or `None` where nothing answers.
///
/// An absent function reads all ones, the bus returning nothing rather than
/// a device answering.
fn probe(pci: &PciIoAccessPci<'_>, function: Function, bridge: usize) -> Option<Device> {
    let id = pci
        .read_one::<u32>(function.address().with_register(REG_ID))
        .ok()?;
    if id == u32::MAX {
        return None;
    }
    // Only for a function that answered, so the scan's cost is unchanged
    // for the addresses where nothing is.
    let class = pci
        .read_one::<u32>(function.address().with_register(REG_CLASS))
        .unwrap_or(0);
    Some(Device {
        function,
        vendor: id as u16,
        device: (id >> 16) as u16,
        class: (class >> 24) as u8,
        subclass: (class >> 16) as u8,
        bridge,
    })
}
