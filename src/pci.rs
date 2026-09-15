// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! What PCI configuration space says is on the machine.
//!
//! Reads through `EFI_PCI_ROOT_BRIDGE_IO_PROTOCOL`, so the firmware decides
//! whether that is MMCONFIG or the legacy ports and neither the MMIO window
//! nor `0xCF8` has to be reached for. The IOMMU and PSP checks both identify
//! their device out of this scan.

use log::{debug, trace, warn};
use uefi::Identify;
use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams, SearchType};
use uefi::proto::pci::PciIoAddress;
use uefi::proto::pci::root_bridge::{PciIoAccessPci, PciRootBridgeIo};

/// Configuration space offsets, all within the first 256 bytes so the legacy
/// window is enough (PCI Local Bus Specification 3.0, section 6.1).
const REG_ID: u8 = 0x00;
const REG_STATUS: u8 = 0x06;
const REG_CLASS: u8 = 0x08;
const REG_CAPABILITY_POINTER: u8 = 0x34;

/// Status bit 4. The capability pointer holds nothing readable unless it is
/// set, so a device without it has no list to walk.
const STATUS_CAPABILITY_LIST: u16 = 1 << 4;

/// A malformed list can point at itself. Devices have 48 dwords of space
/// after the header, so nothing legitimate needs more hops than that.
const MAX_CAPABILITIES: u32 = 48;

fn scan_function(pci: &PciIoAccessPci<'_>, segment: u32, bus: u8, dev: u8, fun: u8) {
    let addr = PciIoAddress::new(bus, dev, fun);
    let Ok(id) = pci.read_one::<u32>(addr.with_register(REG_ID)) else {
        return;
    };
    // An absent function reads all ones, the bus returning nothing rather
    // than a device answering.
    if id == u32::MAX {
        return;
    }
    let class = pci
        .read_one::<u32>(addr.with_register(REG_CLASS))
        .unwrap_or(0)
        >> 8;
    let status = pci
        .read_one::<u16>(addr.with_register(REG_STATUS))
        .unwrap_or(0);
    trace!(
        "[SCAN] {segment:04x}:{bus:02x}:{dev:02x}.{fun} id {:04x}:{:04x} class {class:06x}",
        id as u16,
        (id >> 16) as u16
    );

    if status & STATUS_CAPABILITY_LIST == 0 {
        return;
    }
    // The low two bits of a pointer are reserved and read as zero on
    // conforming devices, so mask rather than trust them.
    let mut next = pci
        .read_one::<u8>(addr.with_register(REG_CAPABILITY_POINTER))
        .unwrap_or(0)
        & 0xFC;
    let mut seen = 0;
    while next != 0 && seen < MAX_CAPABILITIES {
        let header = pci.read_one::<u32>(addr.with_register(next)).unwrap_or(0);
        trace!(
            "[SCAN]   capability {:02x} type {} at {next:02x}",
            header as u8,
            (header >> 16) & 0x7
        );
        next = ((header >> 8) as u8) & 0xFC;
        seen += 1;
    }
}

/// Reports every function the root bridges answer for.
///
/// Bus 0 only, which is where both devices the checks want sit. A machine
/// with devices behind a bridge would have them missed, so this grows a
/// recursion when something needs one.
pub fn report() {
    let handles = match boot::locate_handle_buffer(SearchType::ByProtocol(&PciRootBridgeIo::GUID)) {
        Ok(handles) => handles,
        Err(error) => {
            warn!(
                "[WARN] no PCI root bridge protocol ({:?}), so nothing can be \
                 read out of configuration space",
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
                warn!(
                    "[WARN] a PCI root bridge would not open ({:?})",
                    error.status()
                );
                continue;
            }
        };
        let segment = bridge.segment_nr();
        let pci = bridge.pci();
        for dev in 0..32u8 {
            for fun in 0..8u8 {
                scan_function(&pci, segment, 0, dev, fun);
            }
        }
    }
}
