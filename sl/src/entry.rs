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

    /* The control register and MSR bits changed here. */
    .set CR0_PG, 1 << 31            /* paging */
    .set CR4_PAE, 1 << 5            /* page table entries of 64 bits */
    .set MSR_EFER, 0xC0000080       /* extended feature enable register */
    .set EFER_LME, 1 << 8           /* long mode enable */
    .set MSR_VM_CR, 0xC0010114      /* SVM control */
    .set VM_CR_DIS_A20M, 1 << 2     /* ignore the legacy A20 mask */

    /* Segment selectors: byte offsets into sl_gdt. */
    .set CODE64, 0x08
    .set DATA, 0x10

    /*
     * The three page tables, as offsets from __page_tables, and the bits
     * of their entries: present, writable, and for LARGE a page mapped
     * directly (1 GiB in a PDPT, 2 MiB in a PD) rather than a table.
     */
    .set PML4, 0
    .set PDPT, 0x1000
    .set PD, 0x2000
    .set TABLE, (1 << 0) | (1 << 1)
    .set LARGE, TABLE | (1 << 7)
    .set GIB, 1 << 30

sl_entry32:
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

    /*
     * Load sl_gdt, again through a descriptor built on the stack, then
     * the flat data segment into every data segment register. From here
     * on memory can be addressed without "ss:".
     */
    lea eax, [esi + GDT_OFFSET]
    sub esp, 8
    mov word ptr ss:[esp], 3 * 8 - 1        /* limit: 3 descriptors */
    mov dword ptr ss:[esp + 2], eax         /* address */
    lgdt ss:[esp]
    add esp, 8
    mov ax, DATA
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /*
     * Zero .bss, the stack and the page tables. REP STOSD writes EAX
     * to ECX dwords starting at EDI, upwards since CLD clears the
     * direction flag.
     */
    lea edi, [ebp + __bss_offset]
    mov ecx, offset __bss_dwords
    xor eax, eax
    cld
    rep stosd

    /*
     * The 2 MiB page below starts at the SLB, which works only if the
     * SLB sits on a 2 MiB boundary. Halt at 7 if it does not.
     */
    test ebp, 0x1FFFFF
    jnz 7f

    /*
     * Build the page tables at EBX. Long mode translates an address in
     * up to four steps (PML4, PDPT, PD, PT), each entry pointing to the
     * next table or, with LARGE, mapping a page itself. The high dwords
     * of the entries stay zero from the clearing above.
     *
     * PML4 entry 0 covers virtual addresses from 0 and entry 511 the
     * top 512 GiB, where SL_VBASE is. Both point to the one PDPT. Its
     * entries 0 to 3 map the low 4 GiB at their own addresses in 1 GiB
     * pages, for the TPM and the UART. Its entry 510 points to the PD,
     * whose entry 0 maps the 2 MiB at the SLB at SL_VBASE.
     */
    lea ebx, [ebp + __page_tables_offset]
    lea eax, [ebx + PDPT + TABLE]
    mov [ebx + PML4], eax
    mov [ebx + PML4 + 511 * 8], eax
    mov dword ptr [ebx + PDPT + 0 * 8], 0 * GIB + LARGE
    mov dword ptr [ebx + PDPT + 1 * 8], 1 * GIB + LARGE
    mov dword ptr [ebx + PDPT + 2 * 8], 2 * GIB + LARGE
    mov dword ptr [ebx + PDPT + 3 * 8], 3 * GIB + LARGE
    lea eax, [ebx + PD + TABLE]
    mov [ebx + PDPT + 510 * 8], eax
    lea eax, [ebp + LARGE]
    mov [ebx + PD], eax

    /*
     * Enter long mode, in the order the CPU requires: PAE on, CR3 at
     * the PML4, EFER.LME set, then paging on. The CPU is now in
     * compatibility mode, since CS is still a 32-bit segment. This code
     * keeps running because its physical addresses are mapped to
     * themselves.
     */
    mov eax, cr4
    or eax, CR4_PAE
    mov cr4, eax
    mov cr3, ebx
    mov ecx, MSR_EFER
    rdmsr
    or eax, EFER_LME
    wrmsr
    mov eax, cr0
    or eax, CR0_PG
    mov cr0, eax

    /*
     * Load the 64-bit code segment with a far return, which pops an
     * address and then a selector, and go on at sl_long's physical
     * address. ESP is back at the SLB's end, so the two pushes land in
     * the PD's last entry, which nothing uses. sl_long is 8-byte
     * aligned, so that entry's present bit stays clear.
     */
    lea eax, [esi + LONG_OFFSET]
    push CODE64
    push eax
    retf

    /*
     * A null descriptor, the 64-bit code segment and flat data. The
     * accessed bits are preset, or loading a segment would set them and
     * change the signed image before the service checks it.
     */
    .balign 8
sl_gdt:
    .quad 0
    .quad 0x00AF9B000000FFFF    /* 0x08: 64-bit code */
    .quad 0x00CF93000000FFFF    /* 0x10: flat data */

    .code64
sl_long:
    /*
     * Jump from the physical address to the one the SL is linked at,
     * under SL_VBASE. The address has 64 bits, so it goes through RAX.
     */
    movabs rax, offset sl_high
    jmp rax
sl_high:
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

    .code32
7:
    /* Stops the CPU: no interrupts, and halt again if woken anyway. */
    cli
    hlt
    jmp 7b
    "#
);
