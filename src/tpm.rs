// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! TPM presence and interface detection.
//!
//! Not specific to any DRTM path. Every vendor's dynamic launch needs a
//! discrete TPM, and the register layout here is the TCG PC Client Platform
//! TPM Profile, not an AMD interface.

use common::tpm::{self, Bank, DRTM_PCR_COUNT, DrtmPcrs, FIRST_DRTM_PCR, ReadError};
use log::trace;
use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi::proto::tcg::v2::Tcg;

use crate::check;

/// Base of the TPM's locality 0 MMIO window on the PC platform. Fixed by
/// the chipset, not discovered (TCG PC Client Platform TPM Profile for
/// TPM 2.0, section 6.5.2). GRUB and Xen both hardcode this constant the
/// same way (`grub-core/commands/i386/tpm.c`, `xen/arch/x86/tpm.c`).
const TPM_LOCALITY_0_BASE: usize = 0xFED4_0000;

/// `TPM_ACCESS_0` under the FIFO layout, `TPM_LOC_STATE_0` under CRB (PTP
/// table 20). Both carry `tpmRegValidSts` in bit 7, so this offset can be
/// read before the active interface is known.
const TPM_ACCESS_OFFSET: usize = 0x00;

/// `TPM_INTF_CAPABILITY_0`, but only under the FIFO layout. The CRB layout
/// puts `TPM_DATA_CSUM_0` at this offset instead (PTP table 20), so this must
/// not be read until `INTERFACE_TYPE` says FIFO is active.
const TPM_INTF_CAPABILITY_OFFSET: usize = 0x14;

/// `TPM_INTERFACE_ID_0` under FIFO, `TPM_CRB_INTF_ID_0` under CRB (PTP table
/// 20). Both carry `InterfaceType` in bits 3:0, so this offset can be read
/// before the active interface is known.
const TPM_INTERFACE_ID_OFFSET: usize = 0x30;

/// `TPM_ACCESS.tpmRegValidSts`, bit 7 (PTP table 31, and table 37 for the CRB
/// register aliased at the same offset): set once the rest of the register
/// holds valid values.
const TPM_ACCESS_VALID_STS: u8 = 1 << 7;

/// `TPM_INTF_CAPABILITY.InterfaceVersion`, bits 30:28 (PTP table 34). `011`
/// is the only TPM 2.0 value; a TPM 1.2 device reports `000` or `010` here.
const INTF_VERSION_SHIFT: u32 = 28;
const INTF_VERSION_MASK: u32 = 0x7;
const INTF_VERSION_TPM_1_2: [u32; 2] = [0x0, 0x2];

/// `TPM_INTERFACE_ID.InterfaceType`, bits 3:0 (PTP table 23). `0000` is FIFO
/// active, `0001` CRB active, `1111` a TPM built to an earlier interface
/// specification (i.e. TPM 1.2).
const INTERFACE_TYPE_MASK: u32 = 0xF;
const INTERFACE_TYPE_CRB: u32 = 0x1;

/// `TPM_INTERFACE_ID.CapLocality`, bit 8 (PTP tables 23 and 26). Clear means
/// locality 0 and nothing else, set means all five.
const CAP_LOCALITY_ALL: u32 = 1 << 8;

/// The locality the DCE runs at, and the one this check reads to confirm the
/// window is decoded. Each locality has a 4 KiB window of its own, at
/// `base + (locality << 12)` (PTP section 6.5.2).
const DCE_LOCALITY: usize = 2;

/// Start of `locality`'s own MMIO window.
pub const fn locality_base(locality: usize) -> usize {
    TPM_LOCALITY_0_BASE + (locality << 12)
}

const SHA256_LEN: usize = 32;

/// What the PSP DRTM service leaves in PCR 18 to 20 when a launch fails
/// after `SKINIT`: one all-ones extend into the reset value, so
/// SHA-256 of 32 zero bytes followed by 32 `0xFF` bytes (58453, section 2,
/// step 4). The DLME may boot on regardless, which is why it is worth
/// telling apart from a launch that never happened.
const PSP_CAPPED_PCR: [u8; SHA256_LEN] = [
    0xbb, 0xa9, 0x1c, 0xa8, 0x5d, 0xc9, 0x14, 0xb2, 0xec, 0x3e, 0xfb, 0x9e, 0x16, 0xe7, 0x26, 0x7b,
    0xf9, 0x19, 0x3b, 0x14, 0x35, 0x0d, 0x20, 0xfb, 0xa8, 0xa8, 0xb4, 0x06, 0x73, 0x0a, 0xe3, 0x0a,
];

/// # Safety
///
/// `offset` must be within the TPM's locality 0 MMIO window (`0x000`-`0xFFF`
/// from `TPM_LOCALITY_0_BASE`) and aligned to 4 bytes.
unsafe fn read_tpm_reg32(offset: usize) -> u32 {
    let addr = (TPM_LOCALITY_0_BASE + offset) as *const u32;
    // Safety: the caller guarantees the offset is in the window and aligned,
    // which is this function's whole contract.
    unsafe { core::ptr::read_volatile(addr) }
}

/// # Safety
///
/// `offset` must be within the TPM's locality 0 MMIO window (`0x000`-`0xFFF`
/// from `TPM_LOCALITY_0_BASE`).
unsafe fn read_tpm_reg8(offset: usize) -> u8 {
    let addr = (TPM_LOCALITY_0_BASE + offset) as *const u8;
    // Safety: the caller guarantees the offset is in the window, which is
    // this function's whole contract. `read_volatile` also wants the pointer
    // aligned, which any address is for a `u8`.
    unsafe { core::ptr::read_volatile(addr) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    V1_2,
    V2_0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    Tis,
    Crb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Probe {
    /// No TPM answered at locality 0.
    NotPresent,
    Present {
        family: Family,
        interface: Interface,
    },
}

/// Probes for a TPM at locality 0 and, if one answers, its family and
/// active interface.
///
/// This only confirms *a* TPM answered, not that it's a discrete TPM.
/// `SKINIT` transmits the Secure Loader image to the TPM over the LPC link
/// using transactions unique to `SKINIT` (APM Vol 2, sections 15.27.4 and
/// 15.27.5); a firmware TPM running inside the processor complex cannot
/// take part in that protocol, so AMD requires a discrete TPM
/// (`TrenchBoot.github.io:docs/user-docs/requirements.md`). Software has
/// no way to tell the two apart from here.
pub fn probe() -> Probe {
    // Safety: a valid byte register at this offset under either interface
    // layout (PTP table 20).
    let access = unsafe { read_tpm_reg8(TPM_ACCESS_OFFSET) };
    // PTP, TPM_ACCESS_x.tpmEstablishment: "reading from a device that
    // doesn't exist ... returns a value of all ones" (real hardware bus
    // abort). Some platforms instead leave the window unbacked and reading
    // it just returns zero; TPM_ACCESS_VALID_STS clear catches that case
    // too, since a real TPM sets it once initialized.
    if access == 0xFF || access & TPM_ACCESS_VALID_STS == 0 {
        return Probe::NotPresent;
    }

    // The active interface has to be established first. TPM_INTF_CAPABILITY
    // is only valid when InterfaceType is 0000 or 1111 (PTP section 6.5.2.7);
    // under CRB its offset holds TPM_DATA_CSUM, which would read back as a
    // plausible-looking InterfaceVersion.
    //
    // Safety: same MMIO window, 4-byte aligned offset valid under either
    // interface layout.
    let interface_id = unsafe { read_tpm_reg32(TPM_INTERFACE_ID_OFFSET) };
    if interface_id & INTERFACE_TYPE_MASK == INTERFACE_TYPE_CRB {
        // CRB was introduced with TPM 2.0, so the family follows.
        return Probe::Present {
            family: Family::V2_0,
            interface: Interface::Crb,
        };
    }

    // FIFO is active, or this is a TPM built to an earlier interface
    // specification. TPM_INTF_CAPABILITY is valid in both cases and is what
    // separates 1.2 from 2.0.
    //
    // Safety: same MMIO window, offset valid now that FIFO is known active.
    let intf_capability = unsafe { read_tpm_reg32(TPM_INTF_CAPABILITY_OFFSET) };
    let version = (intf_capability >> INTF_VERSION_SHIFT) & INTF_VERSION_MASK;
    let family = if INTF_VERSION_TPM_1_2.contains(&version) {
        Family::V1_2
    } else {
        Family::V2_0
    };
    Probe::Present {
        family,
        interface: Interface::Tis,
    }
}

/// Whether the TPM implements the localities a dynamic launch needs.
fn localities(family: Family) -> Localities {
    if family == Family::V1_2 {
        return Localities::Unknown;
    }
    // Safety: same MMIO window as the probe, 4-byte aligned offset valid
    // under either interface layout.
    let interface_id = unsafe { read_tpm_reg32(TPM_INTERFACE_ID_OFFSET) };
    if interface_id & CAP_LOCALITY_ALL == 0 {
        return Localities::ZeroOnly;
    }
    // A TPM that has them is not the same as a platform that decodes them:
    // the chipset owns the address range, and only locality 0 is open under
    // all circumstances (_Intel TXT Software Development Guide, table 25_).
    let addr = (locality_base(DCE_LOCALITY) + TPM_ACCESS_OFFSET) as *const u8;
    // Safety: the locality 2 window is 4 KiB of MMIO fixed by the chipset,
    // and this offset is a valid byte register under either layout. An
    // undecoded window reads all ones rather than faulting.
    let access = unsafe { addr.read_volatile() };
    if access == 0xFF || access & TPM_ACCESS_VALID_STS == 0 {
        return Localities::NoAnswer;
    }
    Localities::All
}

/// Logs `TPM_ACCESS` in each locality's window, so a run shows which ones
/// answer and which read all ones. Reading it requests nothing (PTP table 31).
fn trace_access() {
    let access: [u8; 5] = core::array::from_fn(|locality| {
        let addr = (locality_base(locality) + TPM_ACCESS_OFFSET) as *const u8;
        // Safety: each window is 4 KiB of MMIO fixed by the chipset, this is
        // a byte register under the FIFO layout, and an undecoded window
        // reads all ones rather than faulting.
        unsafe { addr.read_volatile() }
    });
    trace!("[SCAN] TPM_ACCESS by locality: {access:02x?}");
}

/// What the TPM and the chipset between them offer above locality 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Localities {
    All,
    /// The TPM answers for locality 0 alone, which a firmware TPM also does.
    ZeroOnly,
    /// The TPM claims five, but locality 2 does not answer before `SKINIT`.
    NoAnswer,
    /// A TPM 1.2 device, which predates the register this reads.
    Unknown,
}

/// Reports whether the localities a launch needs are there.
fn report_localities(found: Localities) {
    match found {
        Localities::All => check::pass!(
            "tpm.locality",
            "five localities, and locality {DCE_LOCALITY} answers"
        ),
        // The one thing software can prove here. SKINIT measures through
        // locality 4 and the DCE extends PCR 17 and 18 from locality 2, so
        // neither can happen on this TPM however the rest of the platform
        // looks.
        Localities::ZeroOnly => check::fail!(
            "tpm.locality",
            "locality 0 only, so no dynamic launch can measure anything here"
        ),
        // A platform launching through the PSP can read all ones here, so
        // this says what was read and leaves the conclusion open.
        Localities::NoAnswer => check::info!(
            "tpm.locality",
            address = check::hex(locality_base(DCE_LOCALITY) as u64);
            "the TPM reports five localities but locality {DCE_LOCALITY} \
             ({:#x}) reads all ones before SKINIT, which may mean the platform \
             does not decode it or that it is locked until a launch",
            locality_base(DCE_LOCALITY)
        ),
        Localities::Unknown => check::skip!(
            "tpm.locality",
            "a TPM 1.2 device predates the register that reports locality \
             support"
        ),
    }
}

/// Why the DRTM PCRs could not be read.
enum PcrReadError {
    /// The firmware publishes no TCG2 protocol, so there is no driver to
    /// carry a command to the TPM from here.
    NoProtocol,
    /// The protocol is there but refused the command.
    Submit(uefi::Status),
    /// The TPM answered with a response code other than success.
    Response(u32),
    /// The response did not parse, or did not carry all six digests.
    Malformed,
}

/// The firmware's TCG2 protocol, if it publishes one.
fn tcg() -> Option<ScopedProtocol<Tcg>> {
    let handle = boot::get_handle_for_protocol::<Tcg>().ok()?;
    // Safety: the handle came from `get_handle_for_protocol` for this
    // protocol, and `GetProtocol` leaves its driver alone.
    unsafe {
        boot::open_protocol::<Tcg>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    }
    .ok()
}

/// The active PCR banks, as `EFI_TCG2_EVENT_ALGORITHM_BITMAP` flags, where
/// the TCG2 protocol says.
pub fn active_banks() -> Option<u32> {
    let banks = tcg()?.get_active_pcr_banks().ok()?;
    (!banks.is_empty()).then(|| banks.bits())
}

/// Reads the SHA-256 bank of PCR 17 to 22 through the firmware's TCG2
/// protocol. A read changes nothing in the TPM.
fn read_drtm_pcrs() -> Result<DrtmPcrs, PcrReadError> {
    let mut tcg = tcg().ok_or(PcrReadError::NoProtocol)?;
    // Header, update counter, the echoed selection and six 32-byte digests
    // come to 232 bytes. Twice that leaves room for a TPM that pads.
    let mut response = [0u8; 512];
    tcg.submit_command(&tpm::pcr_read_command(Bank::Sha256), &mut response)
        .map_err(|e| PcrReadError::Submit(e.status()))?;
    let size = tpm::response_size(&response)
        .unwrap_or(0)
        .min(response.len());
    trace!("[SCAN] TPM2_PCR_Read response: {:02x?}", &response[..size]);
    tpm::parse_pcr_read(&response[..size], Bank::Sha256).map_err(|error| match error {
        ReadError::Response(rc) => PcrReadError::Response(rc),
        ReadError::Malformed => PcrReadError::Malformed,
    })
}

/// Reports what the DRTM PCRs hold before any launch this boot could make.
///
/// All ones is the reset value and the expected state. Anything else is
/// reported rather than judged: a launch resets these PCRs again, so an
/// earlier one in this TPM power cycle stops nothing, and PCR 18 at the
/// PSP's cap value is the one state that says what happened.
fn report_drtm_pcrs(family: Family) {
    if family == Family::V1_2 {
        check::skip!(
            "tpm.drtm_pcrs",
            "a TPM 1.2 reads its PCRs with a different command, which is \
             not implemented"
        );
        return;
    }
    let pcrs = match read_drtm_pcrs() {
        Ok(pcrs) => pcrs,
        Err(PcrReadError::NoProtocol) => {
            check::info!(
                "tpm.drtm_pcrs",
                "the firmware publishes no TCG2 protocol, so the DRTM PCRs \
                 were not read, which a launch does not need"
            );
            return;
        }
        Err(PcrReadError::Submit(status)) => {
            check::info!(
                "tpm.drtm_pcrs",
                status = check::text!("{status:?}");
                "the TCG2 protocol refused TPM2_PCR_Read with {status:?}"
            );
            return;
        }
        Err(PcrReadError::Response(rc)) => {
            check::info!(
                "tpm.drtm_pcrs",
                rc = check::hex(rc);
                "the TPM answered TPM2_PCR_Read with response code {rc:#x}"
            );
            return;
        }
        Err(PcrReadError::Malformed) => {
            check::info!(
                "tpm.drtm_pcrs",
                "the TPM2_PCR_Read response did not carry the six SHA-256 \
                 digests asked for"
            );
            return;
        }
    };

    let at_reset = pcrs.map(|pcr| pcr.all(0xFF));
    let last = FIRST_DRTM_PCR + DRTM_PCR_COUNT - 1;
    if at_reset.iter().all(|&reset| reset) {
        check::pass!(
            "tpm.drtm_pcrs",
            pcr17 = check::bytes(pcrs[0].as_bytes()),
            pcr18 = check::bytes(pcrs[1].as_bytes()),
            pcr19 = check::bytes(pcrs[2].as_bytes()),
            pcr20 = check::bytes(pcrs[3].as_bytes()),
            pcr21 = check::bytes(pcrs[4].as_bytes()),
            pcr22 = check::bytes(pcrs[5].as_bytes());
            "PCR {FIRST_DRTM_PCR} to {last} at the reset value, no dynamic \
             launch since the TPM started"
        );
    } else if pcrs[1].as_bytes() == PSP_CAPPED_PCR {
        check::info!(
            "tpm.drtm_pcrs",
            pcr17 = check::bytes(pcrs[0].as_bytes()),
            pcr18 = check::bytes(pcrs[1].as_bytes()),
            pcr19 = check::bytes(pcrs[2].as_bytes()),
            pcr20 = check::bytes(pcrs[3].as_bytes()),
            pcr21 = check::bytes(pcrs[4].as_bytes()),
            pcr22 = check::bytes(pcrs[5].as_bytes());
            "PCR 18 holds the value the PSP leaves after a launch that failed \
             after SKINIT, so one ran since the TPM started"
        );
    } else {
        let changed = at_reset.iter().filter(|&&reset| !reset).count();
        check::info!(
            "tpm.drtm_pcrs",
            pcr17 = check::bytes(pcrs[0].as_bytes()),
            pcr18 = check::bytes(pcrs[1].as_bytes()),
            pcr19 = check::bytes(pcrs[2].as_bytes()),
            pcr20 = check::bytes(pcrs[3].as_bytes()),
            pcr21 = check::bytes(pcrs[4].as_bytes()),
            pcr22 = check::bytes(pcrs[5].as_bytes());
            "{changed} of PCR {FIRST_DRTM_PCR} to {last} not at the reset \
             value, so a dynamic launch already ran since the TPM started, \
             or the TPM does not start them at all ones"
        );
    }
}

/// Runs the TPM check and reports it.
pub fn report() {
    match probe() {
        Probe::NotPresent => {
            check::fail!("tpm.present", "nothing answered at locality 0 (0xFED40000)");
            check::skip!(
                "tpm.locality",
                "no TPM answered, so there is nothing to ask"
            );
            check::skip!(
                "tpm.drtm_pcrs",
                "no TPM answered, so there is nothing to read"
            );
        }
        Probe::Present { family, interface } => {
            check::pass!(
                "tpm.present",
                family = check::text!("{family:?}"),
                interface = check::text!("{interface:?}");
                "locality 0 answered, {family:?} {interface:?}"
            );
            if interface == Interface::Tis {
                trace_access();
            }
            report_localities(localities(family));
            report_drtm_pcrs(family);
            check::warn!(
                "tpm.discrete",
                "software cannot tell a discrete TPM from a firmware TPM, \
                 and SKINIT requires a discrete one, verify manually"
            );
        }
    }
}
