// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! From the SL to the DLME: its measurement, then the jump.
//!
//! The DLME's entry takes flat 32-bit protected mode with paging off, as
//! AMD's SL leaves it, so the SL leaves long mode the way the preamble's
//! trampoline does before `SKINIT`.

use core::arch::global_asm;
use core::slice;

use common::out::Out;
use common::params::{self, Handoff, Params};
use common::record::{Tag, Value};
#[cfg(feature = "classic")]
use common::sha1;
use common::sha256;

global_asm!(
    r#"
    .section .text.hand_off, "ax"
    .global sl_hand_off
    .global sl_hand_off32

    .set CR0_PG, 1 << 31
    .set CR4_PAE, 1 << 5
    .set MSR_EFER, 0xC0000080
    .set EFER_LME, 1 << 8
    .set DATA, 0x10
    .set CODE32, 0x18

    .code64
sl_hand_off:
    /*
     * RDI: the DLME's entry, RSI: a stack below 4 GiB, RDX: where
     * sl_hand_off32 is. All three are physical, and the low 4 GiB are
     * identity mapped, so each stays valid once paging is off. A far
     * return to the flat 32-bit code segment goes on in compatibility
     * mode.
     */
    mov rsp, rsi
    push CODE32
    push rdx
    retfq

    .code32
sl_hand_off32:
    /*
     * Flat data everywhere, then out of long mode: paging off, which
     * ends it, then EFER.LME and CR4.PAE clear. What is left is what
     * the DLME's entry takes.
     */
    mov ax, DATA
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov eax, cr0
    and eax, ~CR0_PG
    mov cr0, eax
    mov ecx, MSR_EFER
    rdmsr
    and eax, ~EFER_LME
    wrmsr
    mov eax, cr4
    and eax, ~CR4_PAE
    mov cr4, eax
    jmp edi
    .code64
"#
);

unsafe extern "C" {
    static sl_hand_off: u8;
    static sl_hand_off32: u8;
    static __sl_start: u8;
    static __stack_top: u8;
}

/// `OS_SL_DESCRIPTOR_V1`'s GUID at the start of the DLME, and where its
/// entry offset sits in it.
const DESCRIPTOR_GUID: [u8; 16] = [
    0xe5, 0xc7, 0x04, 0x68, 0x83, 0x42, 0x1a, 0x1d, 0xd0, 0xcc, 0xd3, 0x98, 0x12, 0x88, 0x1d, 0x48,
];
const ENTRY_OFFSET: usize = 0x14;

/// The DLME's image as it lies in memory, and its digests.
pub struct Measured {
    #[cfg(feature = "classic")]
    pub sha1: [u8; sha1::LEN],
    pub sha256: [u8; sha256::LEN],
    /// The SHA-256 is the one the build computed.
    pub matches: bool,
}

/// The DLME's measured image.
pub fn image(params: &Params) -> &'static [u8] {
    // Safety: the parameter page checked the block lies below 4 GiB, which
    // the SL identity maps, and nothing writes it while the SL runs.
    unsafe {
        slice::from_raw_parts(
            params.dlme.block as usize as *const u8,
            params.dlme.len as usize,
        )
    }
}

/// Kept out of line, so the hashes' state is not in the callers' frames.
#[inline(never)]
pub fn measure(params: &Params) -> Measured {
    let image = image(params);
    let sha256 = sha256::digest(image);
    Measured {
        #[cfg(feature = "classic")]
        sha1: sha1::digest(image),
        sha256,
        matches: sha256 == params.dlme.sha256,
    }
}

/// Completes the DLME's page with `handoff` and jumps to the DLME, or
/// reports why not and returns. Kept out of line, so its frame is not in
/// `sl_main`'s while the SL runs.
#[inline(never)]
pub fn jump(out: &mut Out, params: &Params, handoff: Handoff, measured: &Measured, slb: u32) {
    let image = image(params);
    let fields = [
        ("block", Value::Hex(params.dlme.block)),
        ("sha256", Value::Bytes(&measured.sha256)),
        ("expected", Value::Bytes(&params.dlme.sha256)),
    ];
    if !measured.matches || image[..16] != DESCRIPTOR_GUID {
        out.check(
            Tag::Fail,
            "launch.handoff",
            &fields,
            format_args!(
                "the DLME at {:#x} is not the one the build made, so the SL does not jump to it",
                params.dlme.block
            ),
        );
        return;
    }
    let entry = u64::from_le_bytes(image[ENTRY_OFFSET..ENTRY_OFFSET + 8].try_into().unwrap());
    if entry >= u64::from(params.dlme.len) {
        out.check(
            Tag::Fail,
            "launch.handoff",
            &fields,
            format_args!("the DLME's entry offset {entry:#x} lies outside its image"),
        );
        return;
    }
    let entry = params.dlme.block + entry;
    out.progress(
        "launch.handoff",
        &[("entry", Value::Hex(entry))],
        format_args!("jumping to the DLME at {entry:#x}"),
    );

    let handoff = Handoff {
        counts: out.counts(),
        ..handoff
    };
    let page = (params.dlme.block + u64::from(params.dlme.len)) as usize as *mut [u8; params::SIZE];
    // What sl.ld places in the SLB, by its offset from the SL's start.
    let physical =
        |symbol: *const u8| u64::from(slb) + (symbol as u64 - &raw const __sl_start as u64);
    // Safety: the page follows the DLME's image in its block, which the
    // parameter page checked is below 4 GiB and the SL identity maps. The
    // jump leaves the SL for good: the DLME is the one the build made, its
    // entry inside it, and the stack the SL's own, at its physical address.
    unsafe {
        handoff.write(&mut *page);
        core::arch::asm!(
            "jmp {hand_off}",
            hand_off = in(reg) &raw const sl_hand_off,
            in("rdi") entry,
            in("rsi") physical(&raw const __stack_top),
            in("rdx") physical(&raw const sl_hand_off32),
            options(noreturn),
        )
    }
}
