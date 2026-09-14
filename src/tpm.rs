// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! TPM presence and interface detection.
//!
//! Not specific to any DRTM path. Every vendor's dynamic launch needs a
//! discrete TPM, and the register layout here is the TCG PC Client Platform
//! TPM Profile, not an AMD interface.

use log::{error, info, warn};

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

/// Runs the TPM check and reports it.
pub fn report() {
    match probe() {
        Probe::NotPresent => {
            error!("[FAIL] no TPM answered at locality 0 (0xFED40000)");
        }
        Probe::Present { family, interface } => {
            info!("[PASS] TPM present at locality 0: {family:?} {interface:?}");
            warn!(
                "[WARN] software cannot tell a discrete TPM from a \
                 firmware TPM, and SKINIT requires a discrete one, verify \
                 manually"
            );
        }
    }
}
