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
//! and a UART, and map the 2 MiB page the SLB starts at `SL_VBASE`, which
//! is why the preamble puts the SLB on a 2 MiB boundary.

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

    /* The 2 MiB page below maps the SLB only from such a boundary. */
    test ebp, 0x1FFFFF
    jnz 7f

    /*
     * EBX: the PML4, whose first and last entries share one PDPT, then
     * that PDPT and a PD. The PDPT's first four entries are the low
     * 4 GiB in 1 GiB pages, entry 510 the PD, whose first entry is the
     * 2 MiB page at the SLB.
     */
    lea ebx, [ebp + __page_tables_offset]
    lea eax, [ebx + 0x1003]
    mov [ebx], eax
    mov [ebx + 511 * 8], eax
    mov dword ptr [ebx + 0x1000], 0x00000083
    mov dword ptr [ebx + 0x1008], 0x40000083
    mov dword ptr [ebx + 0x1010], 0x80000083
    mov dword ptr [ebx + 0x1018], 0xC0000083
    lea eax, [ebx + 0x2003]
    mov [ebx + 0x1000 + 510 * 8], eax
    lea eax, [ebp + 0x83]
    mov [ebx + 0x2000], eax

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

    /*
     * The accessed bits are preset, or loading a segment would set them
     * and change the signed image before the service checks it.
     */
    .balign 8
sl_gdt:
    .quad 0
    .quad 0x00AF9B000000FFFF    /* 0x08: 64-bit code */
    .quad 0x00CF93000000FFFF    /* 0x10: flat data */

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

    .code32
7:
    cli
    hlt
    jmp 7b
    "#
);
