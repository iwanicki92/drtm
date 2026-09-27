// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! From `SKINIT` to `sl_main`.
//!
//! `SKINIT` enters at the header's entry offset in flat 32-bit protected
//! mode, paging off, `EAX` the SLB base, `CS` and `SS` flat and the other
//! segment registers still real mode (APM Vol 2, section 15.27.5). This
//! sets up what `common/src/long_mode.s` takes, which builds page tables in
//! the SLB, enters long mode and comes back to `entry_high` at the address
//! the SL is linked at, `sl.ld`'s `SL_VBASE`. That maps the 2 MiB page the
//! SLB starts at, which is why the preamble puts the SLB on a 2 MiB
//! boundary.

use core::arch::global_asm;

global_asm!(
    r#"
    .section .sl.entry, "ax"
    .code32
    .global entry32

    .set MSR_VM_CR, 0xC0010114      /* SVM control */
    .set VM_CR_DIS_A20M, 1 << 2     /* ignore the legacy A20 mask */

entry32:
    /*
     * SKINIT enters here in 32-bit protected mode with paging off, EAX
     * the SLB's physical address and ESP its end, 64 KiB up. EBP keeps
     * the base for the whole entry. The SLB can be anywhere, so every
     * address before paging is computed from it.
     */
    mov ebp, eax

    /*
     * Load an IDT, the table of exception and interrupt handlers, of
     * limit 0. Any exception then finds no handler, which escalates to
     * a shutdown instead of running code from the firmware's IDT. LIDT
     * reads a 2-byte limit and a 4-byte address from memory, here two
     * zero dwords pushed on the stack. The segment override "ss:" is
     * there because SS is the only data segment SKINIT made flat.
     */
    push 0
    push 0
    lidt ss:[esp]
    add esp, 8

    /*
     * SKINIT set VM_CR.DIS_A20M, which makes the CPU ignore the legacy
     * A20 mask. Clear it again, as TrenchBoot's SKL does. R_INIT, which
     * SKINIT also set, stays set: an INIT signal to this CPU raises an
     * exception rather than resetting it. Nothing sends one, since no other
     * CPU is woken.
     */
    mov ecx, MSR_VM_CR
    rdmsr
    and eax, ~VM_CR_DIS_A20M
    wrmsr

    /*
     * ESI: where this code is. The SL header's first word is the entry
     * point's offset into the SLB.
     */
    movzx esi, word ptr ss:[ebp]
    add esi, ebp
"#,
    include_str!("../../common/src/long_mode.s"),
    r#"
    .code64
entry_high:
    /*
     * Point the stack at the top of the SL's stack area (it grows down)
     * and call sl_main(slb), whose first argument goes in EDI.
     */
    lea rsp, [rip + __stack_top]
    mov edi, ebp
    call sl_main
6:
    cli
    hlt
    jmp 6b
"#
);
