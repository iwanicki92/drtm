// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! What `VM_CR` says about SVM and about launch state left on the core.
//!
//! Firmware disables SVM by setting `SVMDIS`, usually with `LOCK` (_APM
//! Vol 2, section 15.30.1_). That leaves `SKINIT` usable while CPUID still
//! reports it, so it is a fact for a reader of a failed `amd.skinit`, not
//! a verdict. `R_INIT` is the one bit that would change a launch: while it
//! is set an INIT arrives as `#SX`, so parking the other processors with
//! INIT would not park them.

use core::arch::x86_64::__cpuid;

use crate::{check, msr};

/// `CPUID.80000001:ECX[2]`, SVM, which is what defines the MSR.
const CPUID_SVM: u32 = 1 << 2;

/// `CPUID.8000000A:EDX[2]`, SVM lock, which decides whether a locked
/// `SVMDIS` can be undone with a key rather than a firmware setting
/// (_APM Vol 2, section 15.4_).
const CPUID_SVM_LEAF: u32 = 0x8000_000A;
const CPUID_SVML: u32 = 1 << 2;

/// `MSR 0xC0010114` and its five bits (_APM Vol 2, section 15.30.1_).
const VM_CR: u32 = 0xC001_0114;
const DPD: u64 = 1 << 0;
const R_INIT: u64 = 1 << 1;
const DIS_A20M: u64 = 1 << 2;
const LOCK: u64 = 1 << 3;
const SVMDIS: u64 = 1 << 4;

/// Reports the register, decoded.
pub fn report() {
    if __cpuid(0x8000_0000).eax < 0x8000_0001 || __cpuid(0x8000_0001).ecx & CPUID_SVM == 0 {
        check::skip!(
            "amd.vm_cr",
            "no SVM (CPUID.80000001:ECX[2] clear), so VM_CR is not defined"
        );
        return;
    }
    // Safety: CPUID reports SVM, which is what defines this MSR.
    let value = unsafe { msr::read(VM_CR) };
    let set = |bit| value & bit != 0;

    let svm = match (set(SVMDIS), set(LOCK)) {
        (false, false) => "SVM not disabled, unlocked",
        (false, true) => "SVM not disabled, locked by firmware",
        (true, false) => "SVM disabled, not locked",
        (true, true) => {
            let unlockable = __cpuid(0x8000_0000).eax >= CPUID_SVM_LEAF
                && __cpuid(CPUID_SVM_LEAF).edx & CPUID_SVML != 0;
            if unlockable {
                "SVM disabled and locked, undoable with a key"
            } else {
                "SVM disabled and locked, undoable only by a firmware setting"
            }
        }
    };

    if set(R_INIT) {
        check::warn!(
            "amd.vm_cr",
            value = check::hex(value),
            svmdis = check::flag(set(SVMDIS)),
            lock = check::flag(set(LOCK)),
            r_init = check::flag(true),
            dis_a20m = check::flag(set(DIS_A20M)),
            dpd = check::flag(set(DPD));
            "VM_CR reads {value:#x} with R_INIT set, so an INIT arrives as \
             #SX and would not park a processor until a launch clears it \
             ({svm})"
        );
        return;
    }
    check::info!(
        "amd.vm_cr",
        value = check::hex(value),
        svmdis = check::flag(set(SVMDIS)),
        lock = check::flag(set(LOCK)),
        r_init = check::flag(false),
        dis_a20m = check::flag(set(DIS_A20M)),
        dpd = check::flag(set(DPD));
        "VM_CR reads {value:#x}: {svm}, R_INIT clear{}{}",
        if set(DIS_A20M) { ", DIS_A20M set" } else { "" },
        if set(DPD) { ", DPD set" } else { "" }
    );
}
