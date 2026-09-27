/*
 * SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

/*
 * The start of the DLME's 1 MiB block. The measured image comes first,
 * opened by OS_SL_DESCRIPTOR_V1 and padded to whole pages, then the
 * parameter page the preamble writes and drtm's SL completes, then what
 * the DLME zeroes for itself: .bss, the stack and the page tables. The
 * rest of the block, from __dlme_end, is left to a foreign SL.
 *
 * Like the SL, the DLME runs where its page tables map its block, at
 * DLME_VBASE, so nothing needs relocating.
 */

ENTRY(entry32)

DLME_VBASE = 0xFFFFFFFF80000000;
PAGE_TABLES = 3 * 4096;
STACK_SIZE = 16384;

SECTIONS
{
    . = DLME_VBASE;
    __dlme_start = .;

    .text : {
        KEEP(*(.dlme.entry))
        *(.text .text.*)
    }
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }

    . = ALIGN(4096);
    __image_end = .;

    .params (NOLOAD) : {
        KEEP(*(.dlme.params))
    }

    .bss (NOLOAD) : ALIGN(4096) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
    }

    .stack (NOLOAD) : ALIGN(4096) {
        __stack_bottom = .;
        . += STACK_SIZE;
        __stack_top = .;
    }

    .page_tables (NOLOAD) : ALIGN(4096) {
        __page_tables = .;
        . += PAGE_TABLES;
    }
    __dlme_end = .;

    /DISCARD/ : {
        *(.eh_frame*)
        *(.comment)
        *(.note*)
    }
}

/* What the 32-bit entry uses before paging, as offsets into the block. */
__bss_offset = __bss_start - __dlme_start;
__bss_dwords = (__dlme_end - __bss_start) / 4;
__page_tables_offset = __page_tables - __dlme_start;
__stack_top_offset = __stack_top - __dlme_start;
__entry32_offset = entry32 - __dlme_start;
__here_offset = entry_here - __dlme_start;

ASSERT(__image_end - __dlme_start == ADDR(.params) - __dlme_start, "the parameter page does not follow the image")
ASSERT(__dlme_end - __dlme_start <= 0x40000, "the DLME leaves less than 768 KiB of its block")
