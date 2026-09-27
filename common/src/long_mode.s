/*
 * SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

/*
 * From flat 32-bit protected mode with paging off to long mode at the
 * address the image is linked at, for the SL and the DLME. Each includes
 * this after an entry of its own, which leaves:
 *
 *   EBP  the image's physical base, on a 2 MiB boundary
 *   ESI  the physical address of the label entry32
 *
 * and defines entry_high, where this jumps once in long mode. The linker
 * script defines __bss_offset, __bss_dwords and __page_tables_offset.
 *
 * The page tables identity-map the low 4 GiB in 1 GiB pages, for the TPM,
 * the UARTs and the mailbox, and map the 2 MiB page the image starts at to
 * 0xFFFFFFFF80000000, where both are linked.
 */

    /* The control register and MSR bits changed here. */
    .set CR0_PG, 1 << 31            /* paging */
    .set CR4_PAE, 1 << 5            /* page table entries of 64 bits */
    .set CR4_LA57, 1 << 12          /* five-level paging */
    .set CR4_PCIDE, 1 << 17         /* process-context identifiers */
    .set CR4_CET, 1 << 23           /* control-flow enforcement */
    .set MSR_EFER, 0xC0000080       /* extended feature enable register */
    .set EFER_LME, 1 << 8           /* long mode enable */

    /* Segment selectors: byte offsets into long_mode_gdt. */
    .set CODE64, 0x08
    .set DATA, 0x10
    .set CODE32, 0x18

    .set GDT_OFFSET, long_mode_gdt - entry32
    .set LONG_OFFSET, long_mode_long - entry32

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

    .code32
    /*
     * Load long_mode_gdt through a descriptor built on the stack, then
     * the flat data segment into every data segment register. From here
     * on memory can be addressed without "ss:".
     */
    lea eax, [esi + GDT_OFFSET]
    sub esp, 8
    mov word ptr ss:[esp], 4 * 8 - 1        /* limit: 4 descriptors */
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
     * The 2 MiB page below starts at the image, which works only if the
     * image sits on a 2 MiB boundary. Halt at 7 if it does not.
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
     * top 512 GiB, where the link address is. Both point to the one
     * PDPT. Its entries 0 to 3 map the low 4 GiB at their own addresses
     * in 1 GiB pages. Its entry 510 points to the PD, whose entry 0 maps
     * the 2 MiB at the image at the link address.
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
    /* Four-level tables, whatever the code before this left in CR4. */
    mov eax, cr4
    and eax, ~(CR4_LA57 | CR4_PCIDE | CR4_CET)
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
     * address and then a selector, and go on at long_mode_long's
     * physical address. In the SL, ESP is back at the SLB's end, so the
     * two pushes land in the PD's last entry, which nothing uses.
     * long_mode_long follows the GDT and so is 8-byte aligned, which
     * keeps that entry's present bit clear.
     */
    lea eax, [esi + LONG_OFFSET]
    push CODE64
    push eax
    retf

    /*
     * A null descriptor, the 64-bit code segment, flat data, and the
     * flat 32-bit code segment the SL hands off to the DLME in. The
     * accessed bits are preset, or loading a segment would set them and
     * change the image before it is measured or checked.
     */
    .balign 8
long_mode_gdt:
    .quad 0
    .quad 0x00AF9B000000FFFF    /* 0x08: 64-bit code */
    .quad 0x00CF93000000FFFF    /* 0x10: flat data */
    .quad 0x00CF9B000000FFFF    /* 0x18: flat 32-bit code */

    .code64
long_mode_long:
    /*
     * Jump from the physical address to the one the image is linked at.
     * The address has 64 bits, so it goes through RAX.
     */
    movabs rax, offset entry_high
    jmp rax

    .code32
7:
    /* Stops the CPU: no interrupts, and halt again if woken anyway. */
    cli
    hlt
    jmp 7b
