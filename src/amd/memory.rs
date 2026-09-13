// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! Availability of the memory `SKINIT` needs for the Secure Loader Block.

use uefi::Status;
use uefi::boot::{self, AllocateType, MemoryType, PAGE_SIZE};
use uefi::data_types::PhysicalAddress;

/// The Secure Loader Block is 64 KiB and must start on a 64 KiB boundary
/// (APM Vol 2, section 15.27.3).
const SLB_SIZE: u64 = 64 * 1024;

/// Highest byte address the SLB may occupy. `SKINIT` takes its physical
/// address in `EAX`, so the whole block has to fit in 32 bits.
const SLB_MAX_ADDRESS: u64 = u32::MAX as u64;

/// Pages to ask UEFI for. `AllocatePages` works in 4 KiB pages and
/// guarantees no coarser alignment, so the 64 KiB-aligned block has to be
/// carved out of a larger one: aligning a page-aligned base up costs at most
/// `SLB_SIZE - PAGE_SIZE`, hence `2 * SLB_SIZE - PAGE_SIZE` (124 KiB) always
/// contains a suitably aligned `SLB_SIZE` window.
const PROBE_PAGES: usize = ((2 * SLB_SIZE - PAGE_SIZE as u64) / PAGE_SIZE as u64) as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlbRegion {
    /// A suitable region was obtained at this address, then released.
    Available(PhysicalAddress),
    /// UEFI could not supply one, and answered with this status.
    Unavailable(Status),
}

/// Checks that a region meeting the SLB's constraints can actually be
/// allocated, by allocating one and freeing it again.
///
/// This answers "the memory is there and our placement arithmetic is right",
/// not "the memory is reserved". Nothing holds the region afterwards, so an
/// [`SlbRegion::Available`] address is evidence, not a claim on memory.
pub fn probe_slb_region() -> SlbRegion {
    let block = match boot::allocate_pages(
        AllocateType::MaxAddress(SLB_MAX_ADDRESS),
        MemoryType::LOADER_DATA,
        PROBE_PAGES,
    ) {
        Ok(block) => block,
        Err(error) => return SlbRegion::Unavailable(error.status()),
    };

    let aligned = (block.as_ptr() as u64).next_multiple_of(SLB_SIZE);

    // Safety: nothing was written through `block` and no reference into the
    // allocation outlives this call.
    //
    // A failed free would only leak the probe block, and the question this
    // check exists to answer is already settled by then, so the status is of
    // no further use.
    let _ = unsafe { boot::free_pages(block, PROBE_PAGES) };

    SlbRegion::Available(aligned)
}
