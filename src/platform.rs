// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! CPU identification, which decides the DRTM path to take.
//!
//! Runs before any vendor-specific check, so those never have to re-test
//! the vendor or guard a vendor-specific MSR against faulting.

use core::arch::x86_64::__cpuid;
use core::str;

/// CPUID leaf 0 vendor strings, in the `EBX`, `EDX`, `ECX` order the leaf
/// returns them.
const VENDOR_AMD: [u32; 3] = [0x6874_7541, 0x6974_6E65, 0x444D_4163];
const VENDOR_INTEL: [u32; 3] = [0x756E_6547, 0x4965_6E69, 0x6C65_746E];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    Amd,
    Intel,
    Other,
}

pub struct Cpu {
    pub vendor: Vendor,
    /// Family and model, each already folded together with its extended
    /// field. A vendor's register list holds for the families its
    /// documentation covers, so anything reading MSRs by name needs these.
    pub family: u32,
    pub model: u32,
    name: [u8; 12],
}

impl Cpu {
    /// The raw 12-character vendor string, e.g. `AuthenticAMD`.
    ///
    /// Reported alongside the verdict so an unrecognised CPU says what it
    /// actually is rather than just "not AMD".
    pub fn name(&self) -> &str {
        // CPUID vendor strings are ASCII, so this cannot fail in practice.
        str::from_utf8(&self.name).unwrap_or("<not ASCII>")
    }
}

/// Reads the CPU's vendor string and classifies it.
pub fn identify() -> Cpu {
    let result = __cpuid(0x0);
    let words = [result.ebx, result.edx, result.ecx];
    let vendor = match words {
        VENDOR_AMD => Vendor::Amd,
        VENDOR_INTEL => Vendor::Intel,
        _ => Vendor::Other,
    };
    let mut name = [0u8; 12];
    for (i, word) in words.iter().enumerate() {
        name[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    let (family, model) = family_model(__cpuid(0x1).eax);
    Cpu {
        vendor,
        family,
        model,
        name,
    }
}

/// Splits `CPUID.1:EAX` into family and model.
///
/// Each has an extended field that only counts from a certain family up, so
/// folding them in unconditionally would renumber every older CPU. Both
/// vendors number the same way here, as Linux's `x86_family`/`x86_model`
/// read them (`arch/x86/include/asm/cpu_device_id.h`).
fn family_model(signature: u32) -> (u32, u32) {
    let mut family = (signature >> 8) & 0xF;
    if family == 0xF {
        family += (signature >> 20) & 0xFF;
    }
    let mut model = (signature >> 4) & 0xF;
    if family >= 0x6 {
        model |= ((signature >> 16) & 0xF) << 4;
    }
    (family, model)
}
