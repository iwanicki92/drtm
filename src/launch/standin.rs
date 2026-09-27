// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! What a foreign SL with an `AMD_SL_HDR_V1` header expects from the OS
//! loader: `AmdSlInputParameters` at `SLB + TotalLength` (AMD publication
//! 58453, appendix B.3), naming an OS Secure Loader for it to measure and
//! jump to. Here that OS SL is a stand-in that only writes a POST code.
//!
//! No public document gives these layouts. They are read from AMD's SL,
//! see `docs/foreign-sl.md`.

use core::ptr;

/// The OS SL block: the stand-in's image, then the area the SL fills with
/// what it hands the OS SL. 64 MiB aligned, as `EXTEND_OSSL_DIGEST` wants
/// the image it measures.
pub const BLOCK_SIZE: usize = 1 << 20;
const IMAGE_SIZE: usize = 0x1000;

/// Below 1 MiB, where the SL puts the APs' reset vector.
pub const AP_STARTUP_SIZE: usize = 0x4000;

/// The SL's heap, page tables and stacks.
pub const RESERVED_SIZE: usize = 2 << 20;

/// What the stand-in writes to port 0x80 once the SL has jumped to it.
pub const REACHED: u32 = 0xD7D7_0001;

/// `OS_SL_DESCRIPTOR_V1`, which the SL finds by this GUID in the image.
const DESCRIPTOR_GUID: [u8; 16] = [
    0xe5, 0xc7, 0x04, 0x68, 0x83, 0x42, 0x1a, 0x1d, 0xd0, 0xcc, 0xd3, 0x98, 0x12, 0x88, 0x1d, 0x48,
];
const DESCRIPTOR_VERSION: u32 = 1;
const ENTRY: usize = 0x40;
const AP_ENTRY: usize = 0x50;

/// `cli`, then `hlt` in a loop, which decodes the same in any mode.
const HALT: [u8; 4] = [0xfa, 0xf4, 0xeb, 0xfd];

/// `AmdSlInputParameters` as the SL reads it.
const PARAMETERS_VERSION: u32 = 1;
pub const PARAMETERS_LEN: usize = 0x4c;

/// Where the stand-in and the SL's working memory are.
pub struct Regions {
    pub block: u64,
    pub ap_startup: u32,
    pub reserved: u64,
}

/// Writes the stand-in's image to the start of the block and zeroes the
/// rest.
///
/// # Safety
///
/// `block` must be [`BLOCK_SIZE`] writable bytes used by nothing else.
pub unsafe fn install(block: *mut u8) {
    let mut image = [0u8; IMAGE_SIZE];
    image[..16].copy_from_slice(&DESCRIPTOR_GUID);
    image[0x10..0x14].copy_from_slice(&DESCRIPTOR_VERSION.to_le_bytes());
    image[0x14..0x1c].copy_from_slice(&(ENTRY as u64).to_le_bytes());
    image[0x1c..0x24].copy_from_slice(&(AP_ENTRY as u64).to_le_bytes());
    // The SL jumps here in 32-bit flat mode: `mov eax, REACHED`,
    // `out 0x80, eax`, then halt.
    image[ENTRY] = 0xb8;
    image[ENTRY + 1..ENTRY + 5].copy_from_slice(&REACHED.to_le_bytes());
    image[ENTRY + 5..ENTRY + 7].copy_from_slice(&[0xe7, 0x80]);
    image[ENTRY + 7..ENTRY + 11].copy_from_slice(&HALT);
    image[AP_ENTRY..AP_ENTRY + 4].copy_from_slice(&HALT);
    // Safety: the caller's contract.
    unsafe {
        ptr::write_bytes(block, 0, BLOCK_SIZE);
        ptr::copy_nonoverlapping(image.as_ptr(), block, IMAGE_SIZE);
    }
}

/// `AmdSlInputParameters` naming `regions`.
pub fn parameters(regions: &Regions) -> [u8; PARAMETERS_LEN] {
    let mut page = [0u8; PARAMETERS_LEN];
    let mut put = |at: usize, bytes: &[u8]| page[at..at + bytes.len()].copy_from_slice(bytes);
    put(0x00, &PARAMETERS_VERSION.to_le_bytes());
    put(0x04, &(PARAMETERS_LEN as u32).to_le_bytes());
    put(0x08, &regions.block.to_le_bytes());
    put(0x10, &(BLOCK_SIZE as u64).to_le_bytes());
    // OsSlImageOffset and OsSlImageSize, then OsSlParametersOffset and
    // OsSlParametersSize, both relative to the block.
    put(0x18, &0u64.to_le_bytes());
    put(0x20, &(IMAGE_SIZE as u64).to_le_bytes());
    put(0x28, &(IMAGE_SIZE as u64).to_le_bytes());
    put(0x30, &((BLOCK_SIZE - IMAGE_SIZE) as u64).to_le_bytes());
    put(0x38, &regions.ap_startup.to_le_bytes());
    put(0x3c, &(AP_STARTUP_SIZE as u32).to_le_bytes());
    put(0x40, &regions.reserved.to_le_bytes());
    put(0x48, &(RESERVED_SIZE as u32).to_le_bytes());
    page
}
