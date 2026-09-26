// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! From `SKINIT` to `sl_main`.
//!
//! `SKINIT` enters at the header's entry offset in flat 32-bit protected
//! mode, paging off, `EAX` the SLB base, `CS` and `SS` flat and the other
//! segment registers still real mode (APM Vol 2, section 15.27.5). This
//! builds page tables in the SLB, enters long mode and calls `sl_main` at
//! the address the SL is linked at, `sl.ld`'s `SL_VBASE`.
//!
//! The page tables identity-map the low 4 GiB in 1 GiB pages, for the TPM
//! and a UART, and map the SLB's 16 pages at `SL_VBASE`.

use core::arch::global_asm;

global_asm!(
    r#"
    .section .sl.entry, "ax"
    .code32
    .global sl_entry32
    .set GDT_OFFSET, sl_gdt - sl_entry32
    .set LONG_OFFSET, sl_long - sl_entry32
sl_entry32:
    mov ebp, eax

    /* An IDT of limit 0, so any exception shuts down. */
    push 0
    push 0
    lidt ss:[esp]
    add esp, 8

    /* SKINIT disabled A20 masking; R_INIT stays set, nothing wakes APs. */
    mov ecx, 0xC0010114
    rdmsr
    and eax, 0xFFFFFFFB
    wrmsr

    /* ESI: where this code is. Its offset is the header's first word. */
    movzx esi, word ptr ss:[ebp]
    add esi, ebp

    /* Our GDT, through SS while DS is still a real-mode segment. */
    lea eax, [esi + GDT_OFFSET]
    sub esp, 8
    mov word ptr ss:[esp], 3 * 8 - 1
    mov dword ptr ss:[esp + 2], eax
    lgdt ss:[esp]
    add esp, 8
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* Zero .bss, the page tables and the stack. */
    lea edi, [ebp + __bss_offset]
    mov ecx, offset __bss_dwords
    xor eax, eax
    cld
    rep stosd

    /* EBX: PML4, then the low PDPT, the high PDPT, one PD and one PT. */
    lea ebx, [ebp + __page_tables_offset]
    lea eax, [ebx + 0x1003]
    mov [ebx], eax
    lea eax, [ebx + 0x2003]
    mov [ebx + 511 * 8], eax
    mov dword ptr [ebx + 0x1000], 0x00000083
    mov dword ptr [ebx + 0x1008], 0x40000083
    mov dword ptr [ebx + 0x1010], 0x80000083
    mov dword ptr [ebx + 0x1018], 0xC0000083
    lea eax, [ebx + 0x3003]
    mov [ebx + 0x2000 + 510 * 8], eax
    lea eax, [ebx + 0x4003]
    mov [ebx + 0x3000], eax
    lea eax, [ebp + 3]
    lea edi, [ebx + 0x4000]
    mov ecx, 16
1:
    mov [edi], eax
    add eax, 0x1000
    add edi, 8
    loop 1b

    /* Long mode: PAE, CR3, EFER.LME, then paging. */
    mov eax, cr4
    or eax, 0x20
    mov cr4, eax
    mov cr3, ebx
    mov ecx, 0xC0000080
    rdmsr
    or eax, 0x100
    wrmsr
    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax

    /* Into the 64-bit code segment, still at the physical address. */
    lea eax, [esi + LONG_OFFSET]
    push 0x08
    push eax
    retf

    .balign 8
sl_gdt:
    .quad 0
    .quad 0x00AF9A000000FFFF    /* 0x08: 64-bit code */
    .quad 0x00CF92000000FFFF    /* 0x10: flat data */

    .code64
sl_long:
    movabs rax, offset sl_high
    jmp rax
sl_high:
    lea rsp, [rip + __stack_top]
    mov edi, ebp
    call sl_main
6:
    cli
    hlt
    jmp 6b
    "#
);
