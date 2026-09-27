// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! What the PSP path needs before `SKINIT`, as the guide's pre-`SKINIT`
//! module does it (AMD publication 58453, section 2): the service found,
//! kicked and asked what it offers. The TMR around everything the launch
//! and the probe touch waits for the preamble, after `ExitBootServices()`.

use alloc::format;
use alloc::string::String;

use common::params::MailboxSource;
use common::psp::{self, Command, Mailbox};

use crate::amd::psp as secure_processor;
use crate::amd::{aspt, drtm_device};
use crate::pci::Devices;
use crate::tpm::{self, Family, Probe};

/// `PSP_BASE_ADDR_LO` and `_HI` on SMN (58453, appendix G.2), read through
/// the host bridge's index and data pair at configuration offsets `0xB8`
/// and `0xBC`, as GRUB and SKL read them.
const SMN_PSP_BASE_LO: u32 = 0x13B1_02E0;
const SMN_PSP_BASE_HI: u32 = 0x13B1_02E4;
const SMN_INDEX: u8 = 0xB8;
const SMN_DATA: u8 = 0xBC;
const PSP_BASE_MASK: u32 = 0xFFF0_0000;

/// The TMR: index 0 over the low 4 GiB, which holds the SLB, any region the
/// service hands back below 4 GiB, and the interrupt window the probe needs.
pub const TMR_BASE: u64 = 0;
pub const TMR_SIZE: u64 = 1 << 32;

/// The service, kicked and offering DRTM.
pub struct Service {
    pub mailbox: Mailbox,
    pub source: MailboxSource,
    pub capability: [u32; 3],
}

impl Service {
    /// Sets TMR 0 up over the low 4 GiB. Only after `ExitBootServices()`:
    /// devices cannot reach that memory from here, and firmware drivers
    /// stopping theirs would hang.
    pub fn setup_tmr(&self) -> Result<psp::Answer, psp::Error> {
        let units = (TMR_SIZE / psp::TMR_UNIT) as u32;
        let base = [units, TMR_BASE as u32, (TMR_BASE >> 32) as u32];
        self.mailbox
            .command(Command::TmrSetup, 0, base, psp::TIMEOUT_MS)
    }
}

/// A PCI configuration dword of 0:0.0 through ports `0xCF8` and `0xCFC`.
///
/// # Safety
///
/// Ports `0xCF8` and `0xCFC` must be the PC configuration mechanism, and
/// nothing else may use them meanwhile.
unsafe fn host_bridge_write(offset: u8, value: u32) {
    let address = 0x8000_0000u32 | u32::from(offset & 0xFC);
    // Safety: the caller's contract.
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") 0xCF8u16, in("eax") address,
                         options(nomem, nostack, preserves_flags));
        core::arch::asm!("out dx, eax", in("dx") 0xCFCu16, in("eax") value,
                         options(nomem, nostack, preserves_flags));
    }
}

/// # Safety
///
/// As for [`host_bridge_write`].
unsafe fn host_bridge_read(offset: u8) -> u32 {
    let address = 0x8000_0000u32 | u32::from(offset & 0xFC);
    let value: u32;
    // Safety: the caller's contract.
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") 0xCF8u16, in("eax") address,
                         options(nomem, nostack, preserves_flags));
        core::arch::asm!("in eax, dx", in("dx") 0xCFCu16, out("eax") value,
                         options(nomem, nostack, preserves_flags));
    }
    value
}

fn smn_read(address: u32) -> u32 {
    // Safety: the gate let an AMD CPU through, whose host bridge takes an
    // SMN address at 0xB8 and answers at 0xBC, and boot services run on
    // this processor alone.
    unsafe {
        host_bridge_write(SMN_INDEX, address);
        host_bridge_read(SMN_DATA)
    }
}

/// `C2PMSG_72`'s address from the PSP base on SMN.
fn smn_mailbox() -> Option<u64> {
    let low = smn_read(SMN_PSP_BASE_LO) & PSP_BASE_MASK;
    let high = smn_read(SMN_PSP_BASE_HI) & 0xFFFF;
    let base = (u64::from(high) << 32) | u64::from(low);
    (base != 0 && low != PSP_BASE_MASK).then_some(base + psp::C2PMSG_72_OFFSET)
}

/// Kicks the service and asks what it offers, or says why the PSP path is
/// not open. Changes PSP state from the kick on, so it runs after the
/// checks.
pub fn prepare(devices: &Devices, tsc_per_ms: u64) -> Result<Service, String> {
    match tpm::probe() {
        Probe::Present {
            family: Family::V2_0,
            ..
        } => {}
        Probe::Present { .. } => {
            return Err("the TPM is 1.2, and the service extends through TPM 2.0 commands".into());
        }
        _ => return Err("no TPM answers".into()),
    }

    let (address, source) = match aspt::drtm_mailbox() {
        Some(address) => (address as u64, MailboxSource::Aspt),
        None if secure_processor::has_mailbox(devices).is_some() => match smn_mailbox() {
            Some(address) => (address, MailboxSource::Smn),
            None => return Err("no ASPT, and SMN gives no PSP base".into()),
        },
        None => {
            return Err("no Secure Processor offering the mailbox interface".into());
        }
    };
    if address + 0x60 > 1 << 32 {
        return Err(format!(
            "C2PMSG_72 at {address:#x} is above 4 GiB, where the SL's page tables do not reach"
        ));
    }
    // Safety: the ASPT or SMN says the service's registers are here, below
    // 4 GiB, which UEFI identity-maps and the SL maps.
    let mailbox = unsafe { Mailbox::new(address as usize, tsc_per_ms) };
    let register = mailbox.register();
    if register & psp::READY == 0 {
        return Err(format!(
            "the service is not ready, C2PMSG_72 at {address:#x} reads {register:#010x}"
        ));
    }

    mailbox
        .kick()
        .map_err(|error| format!("the kick got no answer ({error:?})"))?;
    let capability = mailbox
        .command(Command::GetCapability, 0, [0; 3], psp::TIMEOUT_MS)
        .map_err(|error| format!("GET_CAPABILITY got no answer ({error:?})"))?;
    if !capability.ok() || capability.results[0] & psp::CAP_ENABLED == 0 {
        // Turin firmware publishes the device only with its DRTM Virtual
        // Device Support option on, and its service reports DRTM off without.
        let hint = match drtm_device::described() {
            Some(false) => {
                ", and no ACPI table describes a DRTM0002 device, so a DRTM \
                 Virtual Device Support option in firmware setup may be off"
            }
            _ => "",
        };
        return Err(format!(
            "GET_CAPABILITY answers {} with {:#x}, DRTM not enabled{hint}",
            psp::status_name(capability.status),
            capability.results[0]
        ));
    }
    Ok(Service {
        mailbox,
        source,
        capability: capability.results,
    })
}
