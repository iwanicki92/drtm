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
    .code64
drtm_trampoline_start:
    /* CR0.PG cannot be cleared with PCIDE set. */
    mov rax, cr4
    and rax, ~(1 << 17)
    mov cr4, rax

    lgdt [rdi]
    mov ebx, edx
    mov ebp, ecx
    lea rax, [rip + drtm_trampoline_32]
    push 0x08
    push rax
    retfq

    .code32
drtm_trampoline_32:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    /* Paging off, then long mode off, then PAE. */
    mov eax, cr0
    and eax, 0x7FFFFFFF
    mov cr0, eax
    mov ecx, 0xC0000080
    rdmsr
    and eax, 0xFFFFFEFF
    wrmsr
    mov eax, cr4
    and eax, 0xFFFFFFDF
    mov cr4, eax

    /* One marker byte: this far, the launch got. */
    cmp ebx, 1
    jne 1f
    mov edx, ebp
    mov al, 0x53
    out dx, al
    jmp 3f
1:
    cmp ebx, 2
    jne 2f
    mov byte ptr [ebp], 0x53
    jmp 3f
2:
    cmp ebx, 3
    jne 3f
    mov dword ptr [ebp], 0x53
3:
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

/// Where in the page the GDT and its descriptor go, after the code.
const GDT: usize = 0x800;
const GDTR: usize = 0x820;

/// A flat 32-bit code segment at 0x08, as `SKINIT` itself loads, and flat
/// data at 0x10.
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
