// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! CPUID leaves, the processors firmware started, and MSRs.

use core::arch::x86_64::{__cpuid, __cpuid_count, CpuidResult};

use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams};
use uefi::proto::pi::mp::MpServices;

use crate::check;
use crate::cpu_state::read_cr;
use crate::logging::record;
use crate::msr;
use crate::platform::{Cpu, Vendor};

/// Leaves whose output depends on the subleaf in `ECX`. Each is read for
/// subleaves 0 to 31 and recorded where not all zero.
const SUBLEAF_LEAVES: [u32; 11] = [
    0x4,
    0x7,
    0xB,
    0xD,
    0xF,
    0x10,
    0x14,
    0x1F,
    0x8000_001D,
    0x8000_0020,
    0x8000_0026,
];

/// A range bound that stops a garbage maximum leaf from reading forever.
const MAX_LEAVES: u32 = 0x100;

const HYPERVISOR_BASE: u32 = 0x4000_0000;
const EXTENDED_BASE: u32 = 0x8000_0000;

/// `CPUID.1:ECX` bit 31, set under a hypervisor, as in the QEMU runs.
const CPUID_1_ECX_HYPERVISOR: u32 = 1 << 31;

/// `CPUID.1:EDX`: `rdmsr` itself, then MTRRs and PAT.
const CPUID_1_EDX_MSR: u32 = 1 << 5;
const CPUID_1_EDX_APIC: u32 = 1 << 9;
const CPUID_1_EDX_MTRR: u32 = 1 << 12;
const CPUID_1_EDX_PAT: u32 = 1 << 16;

/// `CPUID.80000001:ECX`: SVM and OS-visible workarounds.
const CPUID_EXT1_ECX_SVM: u32 = 1 << 2;
const CPUID_EXT1_ECX_OSVW: u32 = 1 << 9;

/// `CPUID.8000001F:EAX` bit 4, SEV-SNP, which brings the RMP MSRs.
const CPUID_SEV_EAX_SNP: u32 = 1 << 4;

/// `IA32_MTRRCAP`: bits 7:0 count the variable ranges, bit 8 is fixed ranges.
const MTRRCAP: u32 = 0xFE;
const MTRRCAP_VCNT: u64 = 0xFF;
const MTRRCAP_FIX: u64 = 1 << 8;
const MTRR_PHYS_BASE0: u32 = 0x200;
const MTRR_FIXED: [u32; 11] = [
    0x250, 0x258, 0x259, 0x268, 0x269, 0x26A, 0x26B, 0x26C, 0x26D, 0x26E, 0x26F,
];

/// AMD MSRs the family 1Ah model 02h PPR defines as readable (_57238 rev
/// 0.49, volume 1_), which covers the EPYC 9015.
const AMD_MSRS: [(u32, &str); 9] = [
    (0x8B, "PATCH_LEVEL"),
    (0xC000_0080, "EFER"),
    (0xC001_0010, "SYSCFG"),
    (0xC001_0015, "HWCR"),
    (0xC001_001D, "TOP_MEM2"),
    (0xC001_0058, "MMIO_CONF_BASE"),
    (0xC001_0111, "SMM_BASE"),
    (0xC001_0112, "SMM_ADDR"),
    (0xC001_0113, "SMM_MASK"),
];

/// The family that PPR covers. Nothing says another family implements the
/// same set, and a missing MSR raises `#GP`, which hangs the machine.
const AMD_MSR_FAMILY: u32 = 0x1A;

const SVM_MSRS: [(u32, &str); 2] = [(0xC001_0114, "VM_CR"), (0xC001_0117, "VM_HSAVE_PA")];
const OSVW_MSRS: [(u32, &str); 2] = [
    (0xC001_0140, "OSVW_ID_LENGTH"),
    (0xC001_0141, "OSVW_STATUS"),
];
const SNP_MSRS: [(u32, &str); 2] = [(0xC001_0132, "RMP_BASE"), (0xC001_0133, "RMP_END")];

fn emit(leaf: u32, subleaf: u32, result: CpuidResult) {
    record(
        "cpuid",
        &[
            ("leaf", check::hex(leaf)),
            ("subleaf", check::count(subleaf)),
            ("eax", check::hex(result.eax)),
            ("ebx", check::hex(result.ebx)),
            ("ecx", check::hex(result.ecx)),
            ("edx", check::hex(result.edx)),
        ],
    );
}

fn is_zero(result: &CpuidResult) -> bool {
    result.eax | result.ebx | result.ecx | result.edx == 0
}

/// Every leaf from `base` to the maximum `base` reports.
fn dump_range(base: u32) {
    let max = __cpuid(base).eax;
    if max < base || max - base > MAX_LEAVES {
        emit(base, 0, __cpuid(base));
        return;
    }
    for leaf in base..=max {
        if SUBLEAF_LEAVES.contains(&leaf) {
            for subleaf in 0..32 {
                let result = __cpuid_count(leaf, subleaf);
                if !is_zero(&result) {
                    emit(leaf, subleaf, result);
                }
            }
        } else {
            emit(leaf, 0, __cpuid(leaf));
        }
    }
}

pub fn dump_cpuid(cpu: &Cpu) {
    // The fold of the extended family and model fields, which leaf 1 below
    // carries raw and which decides what the MSR list applies to.
    record(
        "cpu",
        &[
            ("vendor", check::text!("{}", cpu.name())),
            ("family", check::hex(cpu.family)),
            ("model", check::hex(cpu.model)),
        ],
    );
    dump_range(0);
    if __cpuid(1).ecx & CPUID_1_ECX_HYPERVISOR != 0 {
        dump_range(HYPERVISOR_BASE);
    }
    dump_range(EXTENDED_BASE);
    for register in [0, 3, 4] {
        record(
            "cr",
            &[
                ("name", check::text!("cr{register}")),
                ("value", check::hex(read_cr(register))),
            ],
        );
    }
}

/// Every processor MP services knows, with its APIC ID and location.
pub fn dump_processors() {
    let Ok(handle) = boot::get_handle_for_protocol::<MpServices>() else {
        record("processors", &[("found", check::flag(false))]);
        return;
    };
    // Safety: the handle came from `get_handle_for_protocol` for this
    // protocol, and `GetProtocol` leaves its driver alone.
    let mp = unsafe {
        boot::open_protocol::<MpServices>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    };
    let Ok(mp) = mp else {
        record("processors", &[("opened", check::flag(false))]);
        return;
    };
    let Ok(count) = mp.get_number_of_processors() else {
        record("processors", &[("counted", check::flag(false))]);
        return;
    };
    record(
        "processors",
        &[
            ("total", check::count(count.total as u64)),
            ("enabled", check::count(count.enabled as u64)),
            ("self", check::count(mp.who_am_i().unwrap_or(0) as u64)),
        ],
    );
    for number in 0..count.total {
        match mp.get_processor_info(number) {
            Ok(info) => record(
                "processor",
                &[
                    ("number", check::count(number as u64)),
                    ("apic", check::hex(info.processor_id)),
                    ("package", check::count(info.location.package)),
                    ("core", check::count(info.location.core)),
                    ("thread", check::count(info.location.thread)),
                    ("bsp", check::flag(info.is_bsp())),
                    ("enabled", check::flag(info.is_enabled())),
                    ("healthy", check::flag(info.is_healthy())),
                ],
            ),
            Err(error) => record(
                "processor",
                &[
                    ("number", check::count(number as u64)),
                    ("status", check::text!("{:?}", error.status())),
                ],
            ),
        }
    }
}

/// Reads `msr` and records it.
///
/// # Safety
///
/// As [`msr::read`]: the MSR must exist, or the read raises `#GP`.
unsafe fn read(number: u32, name: &str) {
    // Safety: the caller's contract.
    let value = unsafe { msr::read(number) };
    record(
        "msr",
        &[
            ("id", check::hex(number)),
            ("name", check::text!("{name}")),
            ("value", check::hex(value)),
        ],
    );
}

pub fn dump_msrs(cpu: &Cpu) {
    let leaf1 = __cpuid(1);
    if leaf1.edx & CPUID_1_EDX_MSR == 0 {
        record("msr", &[("supported", check::flag(false))]);
        return;
    }
    if leaf1.edx & CPUID_1_EDX_APIC != 0 {
        // Safety: CPUID reports a local APIC, which defines this MSR.
        unsafe { read(0x1B, "APIC_BASE") };
    }
    if leaf1.edx & CPUID_1_EDX_PAT != 0 {
        // Safety: CPUID reports PAT.
        unsafe { read(0x277, "PAT") };
    }
    if leaf1.edx & CPUID_1_EDX_MTRR != 0 {
        dump_mtrrs();
    }
    if cpu.vendor != Vendor::Amd {
        return;
    }
    let max_extended = __cpuid(EXTENDED_BASE).eax;
    let ext1 = __cpuid(EXTENDED_BASE + 1).ecx;
    if ext1 & CPUID_EXT1_ECX_SVM != 0 {
        for (number, name) in SVM_MSRS {
            // Safety: CPUID reports SVM, which defines these.
            unsafe { read(number, name) };
        }
    }
    if ext1 & CPUID_EXT1_ECX_OSVW != 0 {
        for (number, name) in OSVW_MSRS {
            // Safety: CPUID reports OSVW, which defines these.
            unsafe { read(number, name) };
        }
    }
    if max_extended >= 0x8000_001F && __cpuid(0x8000_001F).eax & CPUID_SEV_EAX_SNP != 0 {
        for (number, name) in SNP_MSRS {
            // Safety: CPUID reports SEV-SNP, which defines these.
            unsafe { read(number, name) };
        }
    }
    if cpu.family != AMD_MSR_FAMILY {
        record(
            "msr",
            &[
                ("family", check::hex(cpu.family)),
                ("documented", check::flag(false)),
            ],
        );
        return;
    }
    for (number, name) in AMD_MSRS {
        // Safety: an AMD CPU of the family whose PPR defines these.
        unsafe { read(number, name) };
    }
}

fn dump_mtrrs() {
    // Safety: the caller checked CPUID reports MTRRs, which define these.
    let cap = unsafe {
        read(MTRRCAP, "MTRRCAP");
        msr::read(MTRRCAP)
    };
    // Safety: as above.
    unsafe { read(0x2FF, "MTRR_DEF_TYPE") };
    for index in 0..(cap & MTRRCAP_VCNT) as u32 {
        // Safety: MTRRCAP counts this many variable ranges.
        unsafe {
            read(MTRR_PHYS_BASE0 + 2 * index, "MTRR_PHYS_BASE");
            read(MTRR_PHYS_BASE0 + 2 * index + 1, "MTRR_PHYS_MASK");
        }
    }
    if cap & MTRRCAP_FIX != 0 {
        for number in MTRR_FIXED {
            // Safety: MTRRCAP reports the fixed ranges.
            unsafe { read(number, "MTRR_FIX") };
        }
    }
}
