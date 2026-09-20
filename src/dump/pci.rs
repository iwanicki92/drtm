// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Every function the scan found, its configuration space, and the MMIO
//! registers of the IOMMUs firmware enabled.

use alloc::vec::Vec;

use uefi::proto::pci::root_bridge::PciIoAccessPci;

use crate::amd::psp;
use crate::check;
use crate::logging::record;
use crate::pci::{self, Devices, Function};

const VENDOR_AMD: u16 = 0x1022;

const REG_CLASS: u8 = 0x08;

/// How much of each function to read: the standard header for everyone, all
/// of legacy space for AMD's own functions, and extended space for the ones
/// a launch deals with.
const HEADER: u32 = 0x40;
const LEGACY: u32 = 0x100;
const EXTENDED: u32 = 0x1000;

/// The host bridge, whose registers include the northbridge's own.
const HOST_BRIDGE: Function = Function {
    segment: 0,
    bus: 0,
    device: 0,
    function: 0,
};

/// The host bridge's index/data pairs' data halves (SMN and HSMP in Linux's
/// `arch/x86/kernel/amd_node.c`, and the PSP's SMN pair). A read reaches
/// whatever register firmware last selected, so these are left unread.
const HOST_BRIDGE_DATA: [u32; 3] = [0x64, 0xBC, 0xC8];

/// Capability ID and type of an AMD IOMMU, and its base address registers
/// (_AMD IOMMU specification, section 3.2_).
const CAPABILITY_SECURE_DEVICE: u8 = 0x0F;
const CAPABILITY_TYPE_IOMMU: u8 = 3;
const CAPABILITY_BASE_LOW: u8 = 0x04;
const CAPABILITY_BASE_HIGH: u8 = 0x08;
const BASE_LOW_ENABLE: u32 = 1;
const BASE_LOW_ADDRESS: u32 = !0x3FFF;

/// IOMMU MMIO registers, all 64-bit and read without side effects
/// (_AMD IOMMU specification, section 3.4_).
const IOMMU_REGISTERS: [(usize, &str); 13] = [
    (0x00, "DEVICE_TABLE_BASE"),
    (0x08, "COMMAND_BUFFER_BASE"),
    (0x10, "EVENT_LOG_BASE"),
    (0x18, "CONTROL"),
    (0x20, "EXCLUSION_BASE"),
    (0x28, "EXCLUSION_LIMIT"),
    (0x30, "EXTENDED_FEATURE"),
    (0x38, "PPR_LOG_BASE"),
    (0x2000, "COMMAND_HEAD"),
    (0x2008, "COMMAND_TAIL"),
    (0x2010, "EVENT_HEAD"),
    (0x2018, "EVENT_TAIL"),
    (0x2020, "STATUS"),
];

fn is_iommu(pci: &PciIoAccessPci<'_>, function: Function) -> Option<u8> {
    let mut found = None;
    pci::for_each_capability(pci, function, |capability| {
        if capability.id == CAPABILITY_SECURE_DEVICE && capability.kind == CAPABILITY_TYPE_IOMMU {
            found = Some(capability.offset);
        }
    });
    found
}

/// What each function is, what its capability list holds, and its
/// configuration space.
///
/// The identity and the capabilities were once `[SCAN]` prose. They say what
/// is on the machine rather than what a check concluded from it, so they
/// belong here, where a reader can take the values rather than the sentence.
pub fn dump(devices: &Devices) {
    pci::for_each_device(devices, |pci, device| {
        let function = device.function;
        let class = pci
            .read_one::<u32>(function.address().with_register(REG_CLASS))
            .unwrap_or(0)
            >> 8;
        record(
            "device",
            &[
                ("fn", check::text!("{function}")),
                ("vendor", check::hex(device.vendor)),
                ("device", check::hex(device.device)),
                ("class", check::hex(class)),
            ],
        );
        pci::for_each_capability(pci, function, |capability| {
            record(
                "capability",
                &[
                    ("fn", check::text!("{function}")),
                    ("id", check::hex(capability.id)),
                    ("type", check::count(capability.kind)),
                    ("at", check::hex(capability.offset)),
                ],
            );
        });
        config(pci, device.function, size(pci, device));
    });
}

/// How far into a function's configuration space to read. Extended space is
/// four times legacy and most of it is zero, so only the functions a launch
/// deals with are worth it.
fn size(pci: &PciIoAccessPci<'_>, device: &pci::Device) -> u32 {
    if device.function == HOST_BRIDGE
        || psp::is_secure_processor(device)
        || is_iommu(pci, device.function).is_some()
    {
        EXTENDED
    } else if device.vendor == VENDOR_AMD {
        LEGACY
    } else {
        HEADER
    }
}

fn config(pci: &PciIoAccessPci<'_>, function: Function, size: u32) {
    let mut bytes = Vec::with_capacity(size as usize);
    for offset in (0..size).step_by(4) {
        if function == HOST_BRIDGE && HOST_BRIDGE_DATA.contains(&offset) {
            record(
                "config",
                &[
                    ("fn", check::text!("{function}")),
                    ("at", check::hex(offset)),
                    ("read", check::flag(false)),
                ],
            );
            bytes.extend_from_slice(&[0; 4]);
            continue;
        }
        let address = function.address().with_extended_register(offset);
        match pci.read_one::<u32>(address) {
            Ok(value) => bytes.extend_from_slice(&value.to_le_bytes()),
            Err(error) => {
                record(
                    "config",
                    &[
                        ("fn", check::text!("{function}")),
                        ("at", check::hex(offset)),
                        ("status", check::text!("{:?}", error.status())),
                    ],
                );
                break;
            }
        }
    }
    // Unimplemented space reads as all ones, and a function that ends well
    // before its window is mostly zeros, so the tail goes in as the byte it
    // repeats. `len` stays what was read, so the rest is recoverable.
    let fill = bytes
        .last()
        .copied()
        .filter(|&byte| byte == 0 || byte == 0xFF);
    let kept = match fill {
        Some(fill) => bytes.len() - bytes.iter().rev().take_while(|&&byte| byte == fill).count(),
        None => bytes.len(),
    };
    let head: [(&str, check::Value); 3] = [
        ("fn", check::text!("{function}")),
        ("len", check::count(bytes.len() as u64)),
        ("hex", check::bytes(&bytes[..kept])),
    ];
    match fill.filter(|_| kept < bytes.len()) {
        None => record("config", &head),
        Some(fill) => {
            let [name, len, hex] = head;
            record("config", &[name, len, hex, ("fill", check::hex(fill))]);
        }
    }
}

/// Where firmware put each enabled IOMMU's registers, and what they hold.
pub fn dump_iommu_registers(devices: &Devices) {
    pci::for_each_device(devices, |pci, device| {
        let function = device.function;
        let Some(capability) = is_iommu(pci, function) else {
            return;
        };
        // A capability near the end of the header would put these registers
        // past it, which reads as 0 rather than overflowing.
        let read = |register: u8| {
            capability
                .checked_add(register)
                .and_then(|offset| {
                    pci.read_one::<u32>(function.address().with_register(offset))
                        .ok()
                })
                .unwrap_or(0)
        };
        let (low, high) = (read(CAPABILITY_BASE_LOW), read(CAPABILITY_BASE_HIGH));
        let base = (u64::from(high) << 32) | u64::from(low & BASE_LOW_ADDRESS);
        if low & BASE_LOW_ENABLE == 0 || base == 0 {
            record(
                "iommu",
                &[
                    ("fn", check::text!("{function}")),
                    ("base", check::hex(base)),
                    ("enabled", check::flag(false)),
                ],
            );
            return;
        }
        let Ok(base) = usize::try_from(base) else {
            return;
        };
        for (offset, name) in IOMMU_REGISTERS {
            // Safety: firmware enabled the IOMMU at this base, which maps its
            // register block there, and these offsets are 64-bit registers
            // whose reads change nothing.
            let value = unsafe { ((base + offset) as *const u64).read_volatile() };
            record(
                "iommu",
                &[
                    ("fn", check::text!("{function}")),
                    ("name", check::text!("{name}")),
                    ("value", check::hex(value)),
                ],
            );
        }
    });
}
