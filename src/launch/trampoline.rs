// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The last code before `SKINIT`: out of long mode and into flat 32-bit
//! protected mode with paging off, which `SKINIT` requires.
//!
//! It runs from a page below 4 GiB, since the firmware may have loaded
//! drtm above, and is position-independent so it runs wherever that page
//! is. What it needs arrives in registers: `RDI` the GDTR, `ESI` the SLB,
//! `EDX` the marker UART's kind and `ECX` its address.

use core::arch::{asm, global_asm};
use core::ptr;

use common::uart::{Space, Uart};

global_asm!(
    r#"
    .section .text.drtm_trampoline, "ax"
    .global drtm_trampoline_start
    .global drtm_trampoline_end

    /* The control register and MSR bits changed here. */
    .set CR0_PG, 1 << 31            /* paging */
    .set CR4_PAE, 1 << 5            /* page table entries of 64 bits */
    .set CR4_PCIDE, 1 << 17         /* process-context identifiers */
    .set CR4_CET, 1 << 23           /* control-flow enforcement */
    .set MSR_EFER, 0xC0000080       /* extended feature enable register */
    .set EFER_LME, 1 << 8           /* long mode enable */

    /* Segment selectors: byte offsets into the GDT `install` writes. */
    .set CODE32, 0x08
    .set DATA32, 0x10

    /* How to reach the marker UART, the kind `enter` passes in EDX. */
    .set MARKER_PORT, 1             /* an I/O port */
    .set MARKER_MMIO8, 2            /* a byte-wide MMIO register */
    .set MARKER_MMIO32, 3           /* a dword-wide MMIO register */
    .set MARKER_BYTE, 0x53          /* ASCII 'S' */

    .code64
drtm_trampoline_start:
    /*
     * The CPU is in 64-bit long mode on the firmware's page tables,
     * which map this page at its own address. Turn off two CR4
     * features first: paging cannot be turned off with PCIDE set, and
     * with CET's shadow stack on, the far return below would fault.
     */
    mov rax, cr4
    and rax, ~(CR4_PCIDE | CR4_CET)
    mov cr4, rax

    /*
     * Load the GDT, the table the CPU reads segment descriptors from,
     * so the flat 32-bit segments below exist. RDMSR further down
     * overwrites ECX and EDX, so the marker's kind and address move to
     * EBX and EBP.
     */
    lgdt [rdi]
    mov ebx, edx
    mov ebp, ecx

    /*
     * Switch to 32-bit code. A far return pops an address and then a
     * code segment selector, so they are pushed in reverse. CODE32 is a
     * 32-bit segment, so execution goes on at drtm_trampoline_32 in
     * compatibility mode, long mode's 32-bit submode. The address is
     * taken relative to RIP, which keeps it right wherever the page is.
     */
    lea rax, [rip + drtm_trampoline_32]
    push CODE32
    push rax
    retfq

    .code32
drtm_trampoline_32:
    /*
     * Load the flat data segment, base 0 and limit 4 GiB, into every
     * data segment register, so memory addresses below are physical
     * once paging is off. Nothing uses the stack from here on, so it
     * does not matter where RSP pointed.
     */
    mov ax, DATA32
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /*
     * Leave long mode. Clearing CR0.PG turns paging off, which also
     * ends long mode, and execution goes on because this page's
     * virtual and physical addresses are the same. Then clear
     * EFER.LME, which would bring long mode back with paging, and
     * CR4.PAE. RDMSR reads the MSR that ECX names into EDX:EAX, and
     * WRMSR writes EDX:EAX back to it. What is left is 32-bit protected
     * mode without paging, which SKINIT requires.
     */
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

    /*
     * Write one 'S' to the UART, so a serial log shows the launch got
     * this far. EBX says how to reach it and EBP is its address. An
     * EBX of 0, no UART, matches no case and skips the write. It does
     * not wait for the UART to be ready. "1f" is a jump forward to the
     * next label "1:".
     */
    cmp ebx, MARKER_PORT
    jne 1f
    mov edx, ebp                    /* OUT takes the port in DX */
    mov al, MARKER_BYTE
    out dx, al
    jmp 3f
1:
    cmp ebx, MARKER_MMIO8
    jne 2f
    mov byte ptr [ebp], MARKER_BYTE
    jmp 3f
2:
    cmp ebx, MARKER_MMIO32
    jne 3f
    mov dword ptr [ebp], MARKER_BYTE
3:
    /*
     * SKINIT takes the SLB's physical address in EAX. It measures the
     * Secure Loader into the TPM and jumps to it, so it never returns.
     * The loop only stops the CPU should that go wrong: CLI blocks
     * interrupts, HLT waits for one, and the jump halts again if
     * something wakes it anyway.
     */
    mov eax, esi
    skinit eax
4:
    cli
    hlt
    jmp 4b
drtm_trampoline_end:
    .code64
    "#
);

unsafe extern "C" {
    static drtm_trampoline_start: u8;
    static drtm_trampoline_end: u8;
}

/// Where in the page the GDT and its descriptor go, after the code. The
/// descriptor is the 10 bytes `LGDT` reads: the table's size in bytes
/// minus one, then its address.
const GDT: usize = 0x800;
const GDTR: usize = 0x820;

/// The GDT: the null descriptor the CPU requires first, then a flat 32-bit
/// code segment at selector 0x08, as `SKINIT` itself loads, and flat data
/// at 0x10. Both have base 0 and limit 0xFFFFF in 4 KiB units, so they
/// cover 4 GiB. Access byte 0x9A is present, ring 0, code, readable, and
/// 0x92 present, ring 0, data, writable. The 0xC nibble sets the 4 KiB
/// units and 32-bit operands.
const DESCRIPTORS: [u64; 3] = [0, 0x00CF_9A00_0000_FFFF, 0x00CF_9200_0000_FFFF];

/// Copies the trampoline and its GDT into `page`.
///
/// # Safety
///
/// `page` must be 4 KiB below 4 GiB that nothing else uses, executable in
/// the firmware's page tables.
pub unsafe fn install(page: *mut u8) {
    let (start, end) = (
        &raw const drtm_trampoline_start,
        &raw const drtm_trampoline_end,
    );
    let len = end as usize - start as usize;
    assert!(len <= GDT, "the trampoline outgrew its space");
    // Safety: the code is `len` bytes, the page takes it before `GDT`, and
    // the GDT and its descriptor fit behind it. The caller vouches for the
    // page.
    unsafe {
        ptr::copy_nonoverlapping(start, page, len);
        let gdt = page.add(GDT).cast::<[u64; 3]>();
        gdt.write_unaligned(DESCRIPTORS);
        let gdtr = page.add(GDTR);
        gdtr.cast::<u16>()
            .write_unaligned((size_of_val(&DESCRIPTORS) - 1) as u16);
        gdtr.add(2).cast::<u64>().write_unaligned(gdt as u64);
    }
}

/// Jumps into the installed trampoline, which ends in `SKINIT` with `slb`.
///
/// # Safety
///
/// `page` must hold what [`install`] put there, and everything `SKINIT`
/// needs must be in place: boot services gone, APs parked, interrupts off,
/// the SLB written.
pub unsafe fn enter(page: *mut u8, slb: u32, marker: Option<Uart>) -> ! {
    let (kind, address) = match marker {
        Some(uart) => match (uart.space(), uart.width()) {
            (Space::Io, _) => (1u32, uart.base() as u32),
            (Space::Mmio, 4) => (3, uart.base() as u32),
            (Space::Mmio, _) => (2, uart.base() as u32),
        },
        None => (0, 0),
    };
    // Safety: the caller's contract. The trampoline never returns.
    unsafe {
        // A plain jump into the page, each argument in the register the
        // trampoline reads it from.
        asm!(
            "jmp {entry}",
            entry = in(reg) page,
            in("rdi") page.add(GDTR),
            in("esi") slb,
            in("edx") kind,
            in("ecx") address,
            options(noreturn),
        )
    }
}
