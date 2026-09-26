/*
 * SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

/*
 * The Secure Loader Block, 64 KiB. The measured image comes first, opened
 * by the header SKINIT reads, then the signature block the PSP DRTM service
 * checks, then the parameter page the preamble writes, then what the SL
 * zeroes for itself: .bss, the page tables and the stack.
 *
 * The SL runs at SL_VBASE, where its page tables map the SLB, so its
 * absolute addresses are fixed at link time wherever the SLB lands and
 * nothing needs relocating.
 */

ENTRY(sl_entry32)

SL_VBASE = 0xFFFFFFFF80000000;
SLB_SIZE = 0x10000;
PAGE_TABLES = 3 * 4096;
STACK_SIZE = 6144;

SECTIONS
{
    . = SL_VBASE;
    __sl_start = .;

    /*
     * AMD's two words (APM Vol 2, section 15.27.2), then TrenchBoot's
     * layout up to bootloader_data_offset at byte 68.
     */
    .header : {
        SHORT(sl_entry32 - __sl_start)
        SHORT(__image_end - __sl_start)
        QUAD(0) QUAD(0) QUAD(0) QUAD(0) QUAD(0) QUAD(0) QUAD(0)
        LONG(0) SHORT(0)
        SHORT(0)
        SHORT(__params - __sl_start)
    }

    .text : {
        KEEP(*(.sl.entry))
        *(.text .text.*)
    }
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }

    /*
     * SKINIT measures a multiple of 4 bytes, and the signature header sits
     * at the measured length rounded up to 16.
     */
    . = ALIGN(16);
    __image_end = .;

    /* The $AS1 header, signature and key token, written by build.rs. */
    .signature (NOLOAD) : {
        . += 0x740;
    }

    .params (NOLOAD) : ALIGN(16) {
        KEEP(*(.sl.params))
    }

    /*
     * The page tables go last, on the page boundaries they need, and the
     * stack right below them.
     */
    .bss (NOLOAD) : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(16);
        __bss_data_end = .;
    }

    .stack SL_VBASE + SLB_SIZE - PAGE_TABLES - STACK_SIZE (NOLOAD) : {
        __stack_bottom = .;
        . += STACK_SIZE;
        __stack_top = .;
        __page_tables = .;
        . += PAGE_TABLES;
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.eh_frame*)
        *(.comment)
        *(.note*)
    }
}

/* What the 32-bit entry uses before paging, as offsets into the SLB. */
__bss_offset = __bss_start - __sl_start;
__bss_dwords = (__bss_end - __bss_start) / 4;
__page_tables_offset = __page_tables - __sl_start;

ASSERT(__bss_data_end <= __stack_bottom, "the SL does not fit in the SLB")
ASSERT((__page_tables - __sl_start) % 4096 == 0, "the page tables are not page aligned")
ASSERT(__image_end - __sl_start <= 61440, "the measured image exceeds 60 KiB")
