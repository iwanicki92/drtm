// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Individual AMD DRTM precondition checks.

use core::arch::x86_64::__cpuid;

/// `CPUID.80000001:ECX` bit 12.
///
/// APM Vol 2, section 15.4: `SKINIT`/`STGI` are usable when either
/// `EFER.SVME` is 1 or this bit is 1, so this CPUID check alone is
/// sufficient without also reading `EFER`.
const SKINIT_CPUID_BIT: u32 = 1 << 12;

/// Whether the CPU advertises `SKINIT` support via CPUID.
///
/// This is a hard gate: without it, no other AMD DRTM check matters.
pub fn skinit_supported() -> bool {
    let result = __cpuid(0x8000_0001);
    result.ecx & SKINIT_CPUID_BIT != 0
}

/// Base of the TPM's locality 0 MMIO window on the PC platform. Fixed by
/// the chipset, not discovered (TCG PC Client Platform TPM Profile for
/// TPM 2.0, section 6.5.2). GRUB and Xen both hardcode this constant the
/// same way (`grub-core/commands/i386/tpm.c`, `xen/arch/x86/tpm.c`).
const TPM_LOCALITY_0_BASE: usize = 0xFED4_0000;

const TPM_ACCESS_OFFSET: usize = 0x00;
const TPM_INTF_CAPABILITY_OFFSET: usize = 0x14;
const TPM_INTERFACE_ID_OFFSET: usize = 0x30;

/// `TPM_ACCESS.tpmRegValidSts`, bit 7 (PTP table 31): set once the rest of
/// the register holds valid values.
const TPM_ACCESS_VALID_STS: u8 = 1 << 7;

/// `TPM_INTF_CAPABILITY.InterfaceVersion`, bits 30:28 (PTP table 34). `011`
/// is the only TPM 2.0 value; a TPM 1.2 device reports `000` or `010` here.
const INTF_VERSION_SHIFT: u32 = 28;
const INTF_VERSION_MASK: u32 = 0x7;
const INTF_VERSION_TPM_1_2: [u32; 2] = [0x0, 0x2];

/// `TPM_INTERFACE_ID.InterfaceType`, bits 3:0 (PTP table 23).
const INTERFACE_TYPE_MASK: u32 = 0xF;
const INTERFACE_TYPE_CRB: u32 = 0x1;

/// # Safety
///
/// `offset` must be within the TPM's locality 0 MMIO window (`0x000`-`0xFFF`
/// from `TPM_LOCALITY_0_BASE`) and aligned to 4 bytes.
unsafe fn read_tpm_reg32(offset: usize) -> u32 {
    let addr = (TPM_LOCALITY_0_BASE + offset) as *const u32;
    unsafe { core::ptr::read_volatile(addr) }
}

/// # Safety
///
/// `offset` must be within the TPM's locality 0 MMIO window (`0x000`-`0xFFF`
/// from `TPM_LOCALITY_0_BASE`).
unsafe fn read_tpm_reg8(offset: usize) -> u8 {
    let addr = (TPM_LOCALITY_0_BASE + offset) as *const u8;
    unsafe { core::ptr::read_volatile(addr) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpmFamily {
    V1_2,
    V2_0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpmInterface {
    Tis,
    Crb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TpmProbe {
    /// No TPM answered at locality 0.
    NotPresent,
    Present {
        family: TpmFamily,
        interface: TpmInterface,
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
pub fn probe_tpm() -> TpmProbe {
    // Safety: TPM_ACCESS_0 is a valid byte register at this offset in the
    // locality 0 window (PTP table 20).
    let access = unsafe { read_tpm_reg8(TPM_ACCESS_OFFSET) };
    // PTP, TPM_ACCESS_x.tpmEstablishment: "reading from a device that
    // doesn't exist ... returns a value of all ones" (real hardware bus
    // abort). Some platforms instead leave the window unbacked and reading
    // it just returns zero; TPM_ACCESS_VALID_STS clear catches that case
    // too, since a real TPM sets it once initialized.
    if access == 0xFF || access & TPM_ACCESS_VALID_STS == 0 {
        return TpmProbe::NotPresent;
    }

    // Safety: same MMIO window as above, offset is within range.
    let intf_capability = unsafe { read_tpm_reg32(TPM_INTF_CAPABILITY_OFFSET) };
    let version = (intf_capability >> INTF_VERSION_SHIFT) & INTF_VERSION_MASK;
    if INTF_VERSION_TPM_1_2.contains(&version) {
        // TPM 1.2 predates CRB, so FIFO (TIS) is the only interface.
        return TpmProbe::Present {
            family: TpmFamily::V1_2,
            interface: TpmInterface::Tis,
        };
    }

    // Safety: same MMIO window as above, offset is within range.
    let interface_id = unsafe { read_tpm_reg32(TPM_INTERFACE_ID_OFFSET) };
    let interface = if interface_id & INTERFACE_TYPE_MASK == INTERFACE_TYPE_CRB {
        TpmInterface::Crb
    } else {
        TpmInterface::Tis
    };
    TpmProbe::Present {
        family: TpmFamily::V2_0,
        interface,
    }
}
