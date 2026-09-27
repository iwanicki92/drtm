// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! drtm's DLME in its block, which every launch path ends in, and what a
//! foreign SL with an `AMD_SL_HDR_V1` header expects from the OS loader:
//! `AmdSlInputParameters` at `SLB + TotalLength` (AMD publication 58453,
//! appendix B.3), naming the DLME as the OS Secure Loader it jumps to.
//!
//! No public document gives these layouts. They are read from AMD's SL,
//! see `docs/foreign-sl.md`.

use core::ptr;

use common::params;

use super::image::DLME;

/// The block: the DLME's image, its parameter page and scratch memory,
/// then the area a foreign SL fills for it. 64 MiB aligned on the PSP
/// paths, as `EXTEND_OSSL_DIGEST` wants the image it measures.
pub const BLOCK_SIZE: usize = params::DLME_BLOCK_SIZE as usize;
const PAGE: usize = 0x1000;

/// Below 1 MiB, where a foreign SL puts the APs' reset vector.
pub const AP_STARTUP_SIZE: usize = 0x4000;

/// A foreign SL's heap, page tables and stacks.
pub const RESERVED_SIZE: usize = 2 << 20;

/// What the DLME writes to port 0x80 first, which `dlme/src/entry.rs` holds
/// too.
pub const REACHED: u32 = 0xD7D7_0001;

/// `AmdSlInputParameters` as AMD's SL reads it.
const PARAMETERS_VERSION: u32 = 1;
pub const PARAMETERS_LEN: usize = 0x4c;

/// The DLME's measured length, which the parameter page carries.
pub fn len() -> u32 {
    DLME.bytes.len() as u32
}

/// Where a foreign SL's area starts in the block: after the DLME's scratch.
fn area() -> usize {
    DLME.end.next_multiple_of(PAGE)
}

/// Where a foreign SL's working memory is.
pub struct Regions {
    pub ap_startup: u32,
    pub reserved: u64,
}

/// Writes the DLME's image and `page` to the start of `block` and zeroes the
/// rest.
///
/// # Safety
///
/// `block` must be [`BLOCK_SIZE`] writable bytes used by nothing else.
pub unsafe fn install(block: *mut u8, page: &[u8; params::SIZE]) {
    // Safety: the caller's contract, and the image and its page lie inside
    // the block, which `build.rs` checked.
    unsafe {
        ptr::write_bytes(block, 0, BLOCK_SIZE);
        ptr::copy_nonoverlapping(DLME.bytes.as_ptr(), block, DLME.bytes.len());
        ptr::copy_nonoverlapping(page.as_ptr(), block.add(DLME.bytes.len()), params::SIZE);
    }
}

/// `AmdSlInputParameters` naming the DLME in `block`, and `regions`.
pub fn parameters(block: u64, regions: &Regions) -> [u8; PARAMETERS_LEN] {
    let mut page = [0u8; PARAMETERS_LEN];
    let mut put = |at: usize, bytes: &[u8]| page[at..at + bytes.len()].copy_from_slice(bytes);
    put(0x00, &PARAMETERS_VERSION.to_le_bytes());
    put(0x04, &(PARAMETERS_LEN as u32).to_le_bytes());
    put(0x08, &block.to_le_bytes());
    put(0x10, &(BLOCK_SIZE as u64).to_le_bytes());
    // OsSlImageOffset and OsSlImageSize, then OsSlParametersOffset and
    // OsSlParametersSize, both relative to the block.
    put(0x18, &0u64.to_le_bytes());
    put(0x20, &u64::from(len()).to_le_bytes());
    put(0x28, &(area() as u64).to_le_bytes());
    put(0x30, &((BLOCK_SIZE - area()) as u64).to_le_bytes());
    put(0x38, &regions.ap_startup.to_le_bytes());
    put(0x3c, &(AP_STARTUP_SIZE as u32).to_le_bytes());
    put(0x40, &regions.reserved.to_le_bytes());
    put(0x48, &(RESERVED_SIZE as u32).to_le_bytes());
    page
}
