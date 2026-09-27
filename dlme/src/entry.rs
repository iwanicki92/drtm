// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! `OS_SL_DESCRIPTOR_V1`, then the entry both kinds of SL jump to.
//!
//! drtm's SLs and AMD's all jump to `entry32` in flat 32-bit protected mode
//! with paging off and a usable stack. The entry finds its own address and
//! relies on no other register, then goes through `common/src/long_mode.s`
//! to `dlme_main` at the address the DLME is linked at.

use core::arch::global_asm;

global_asm!(
    r#"
    .section .dlme.entry, "ax"
    .code32
    .global entry32
    .global entry_here

    /* What the DLME writes to port 0x80 before anything else. */
    .set REACHED, 0xD7D70001

    /*
     * OS_SL_DESCRIPTOR_V1, which AMD's SL finds by its GUID: the GUID,
     * version 1, then the entry and AP entry as offsets from here.
     */
descriptor:
    .byte 0xe5, 0xc7, 0x04, 0x68, 0x83, 0x42, 0x1a, 0x1d
    .byte 0xd0, 0xcc, 0xd3, 0x98, 0x12, 0x88, 0x1d, 0x48
    .long 1
    .quad entry32 - descriptor
    .quad ap_entry32 - descriptor

    /* The APs stay halted: interrupts off, and halt again if woken. */
    .balign 16
ap_entry32:
    cli
    hlt
    jmp ap_entry32

entry32:
    /*
     * Say on port 0x80 that the DLME was reached, for a debug card, before
     * anything that could fail. Then find where this code is: CALL pushes
     * the address after it, and POP takes it back.
     */
    cli
    mov eax, REACHED
    out 0x80, eax
    call entry_here
entry_here:
    pop ebp
    sub ebp, offset __here_offset
    lea esi, [ebp + __entry32_offset]

    /*
     * Move to the DLME's own stack, which the caller's may not be, and
     * load an IDT of limit 0, so any exception shuts the machine down
     * rather than running the caller's handlers.
     */
    lea esp, [ebp + __stack_top_offset]
    push 0
    push 0
    lidt ss:[esp]
    add esp, 8
"#,
    include_str!("../../common/src/long_mode.s"),
    r#"
    .code64
entry_high:
    /* The stack at its linked address, then dlme_main(base) in EDI. */
    lea rsp, [rip + __stack_top]
    mov edi, ebp
    call dlme_main
6:
    cli
    hlt
    jmp 6b
"#
);
