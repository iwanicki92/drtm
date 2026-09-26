// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Whether the Secure Processor's DRTM service says it will take a command.
//!
//! Reported rather than judged, as the Secure Processor itself is. The
//! register says what the service's state is now, not whether the launch
//! this platform needs goes through it.

use alloc::vec::Vec;
use core::sync::atomic::{AtomicU8, Ordering};

use common::params::PspDrtm;

use log::debug;

use crate::acpi;
use crate::amd::psp;
use crate::bytes::{read_u16, read_u32, read_u64};
use crate::check;
use crate::pci::Devices;

pub const SIGNATURE: &[u8; 4] = b"ASPT";

/// What the report below read of the service, for the launch to pass on:
/// 0 nothing, 1 not ready, 2 ready.
static STATE: AtomicU8 = AtomicU8::new(0);

/// What `amd.psp_drtm` read of the DRTM service this run.
pub fn drtm_state() -> PspDrtm {
    match STATE.load(Ordering::Relaxed) {
        1 => PspDrtm::NotReady,
        2 => PspDrtm::Ready,
        _ => PspDrtm::Unknown,
    }
}

/// The header is followed by where the Secure Processor's registers are and
/// how many pages of them there are, then by the structures saying what is
/// in that block (_AMD publication 58193, section 2_).
const REGISTER_BASE: usize = acpi::HEADER_LEN;
const PAGES: usize = REGISTER_BASE + 8;
const STRUCTURE_COUNT: usize = PAGES + 4;
const FIRST_STRUCTURE: usize = STRUCTURE_COUNT + 4;
const PAGE: usize = 4096;

/// Every structure opens with its type and, at offset 2, its length. Type 0
/// is the global one, which is where the feature register's offset is. What
/// the fields after that are is the type's own business.
const STRUCTURE_HEADER: usize = 4;
const STRUCTURE_LENGTH: usize = 2;
const STRUCTURE_GLOBAL: u16 = 0;
const GLOBAL_FEATURE: usize = 8;

/// The feature register is C2PMSG_63 and the DRTM service's command and
/// response register C2PMSG_72, with four bytes per mailbox (_AMD
/// publication 58453, section 3.1_).
const FEATURE_MAILBOX: usize = 63;
const DRTM_MAILBOX: usize = 72;
const MAILBOX_WIDTH: usize = 4;

/// Bit 31 of that register, which the service sets while it will take a
/// command.
const READY: u32 = 1 << 31;

/// Reports what the DRTM service's register holds, if the platform says
/// where it is.
///
/// The ASPT is the only route to that address that observes the platform:
/// the register block sits behind an SMN access otherwise, and an SMN read
/// is a write to the host bridge's index register.
pub fn report(devices: &Devices) {
    if !psp::has_mailbox(devices) {
        check::skip!(
            "amd.psp_drtm",
            "no Secure Processor offering the mailbox interface, so no DRTM \
             service to ask"
        );
        return;
    }

    let table = match acpi::find_or(SIGNATURE, "no ASPT") {
        Ok(table) => table,
        Err(why) => {
            check::info!(
                "amd.psp_drtm",
                "{why}, so nothing says where the Secure Processor's \
                 registers are"
            );
            return;
        }
    };

    if !acpi::checksum_ok(table) {
        check::info!(
            "amd.psp_drtm",
            "ASPT checksum is wrong, so an OS discards the table and the \
             register addresses with it"
        );
        return;
    }
    let Some(mailbox) = mailbox(table) else {
        check::info!(
            "amd.psp_drtm",
            "ASPT names no global register structure, or puts the DRTM \
             mailbox outside the register block it declares"
        );
        return;
    };

    // Printed before the read, so a machine that dies on it says which
    // address it was reading.
    debug!("[PSP] DRTM mailbox at {mailbox:#x}");
    // Safety: the ASPT says the Secure Processor's registers are here, UEFI
    // identity-maps what firmware reports, and this is a 32-bit register
    // whose read issues nothing: a command is a write.
    let value = unsafe { (mailbox as *const u32).read_volatile() };
    let ready = value & READY != 0;
    STATE.store(if ready { 2 } else { 1 }, Ordering::Relaxed);
    check::info!(
        "amd.psp_drtm",
        mailbox = check::hex(mailbox as u64),
        value = check::hex(value),
        ready = check::flag(ready);
        "DRTM service {}, C2PMSG_72 at {mailbox:#x} reads {value:#010x}",
        if ready { "ready" } else { "not ready" }
    );
}

/// One register structure, as its type and its own bytes. What the bytes
/// hold depends on the type, so reading a field is the caller's to do.
pub struct Structure<'a> {
    pub kind: u16,
    pub bytes: &'a [u8],
}

/// Where the Secure Processor's registers are and how many 4 KiB pages of
/// them the table declares.
pub fn block(table: &[u8]) -> Option<(usize, usize)> {
    let base = usize::try_from(read_u64(table, REGISTER_BASE)?).ok()?;
    let pages = read_u32(table, PAGES)? as usize;
    (base != 0).then_some((base, pages))
}

/// The address of the register at `offset` in that block, if the pages the
/// table declares cover it.
pub fn register(table: &[u8], offset: u32) -> Option<usize> {
    let (base, pages) = block(table)?;
    let offset = usize::try_from(offset).ok()?;
    if offset.checked_add(MAILBOX_WIDTH)? > pages.checked_mul(PAGE)? {
        return None;
    }
    base.checked_add(offset)
}

/// A register offset a structure names, at `at` bytes into it.
pub fn field(structure: &[u8], at: usize) -> Option<u32> {
    read_u32(structure, at)
}

/// Every register structure the table holds. A structure whose length does
/// not fit stops the walk, since the one after it is wherever that length
/// says.
pub fn structures(table: &[u8]) -> Option<Vec<Structure<'_>>> {
    let mut found = Vec::new();
    let mut offset = FIRST_STRUCTURE;
    for _ in 0..read_u32(table, STRUCTURE_COUNT)? {
        let structure = table.get(offset..)?;
        let len = usize::from(read_u16(structure, STRUCTURE_LENGTH)?);
        if len < STRUCTURE_HEADER || len > structure.len() {
            return None;
        }
        found.push(Structure {
            kind: read_u16(structure, 0)?,
            bytes: &structure[..len],
        });
        offset += len;
    }
    Some(found)
}

/// Where C2PMSG_72 is, from the block the table declares and the offset it
/// gives for C2PMSG_63. Both move between Secure Processor generations, so
/// the table is what says where they are on this one.
pub fn mailbox(table: &[u8]) -> Option<usize> {
    let feature = feature_offset(table)?;
    let past = u32::try_from((DRTM_MAILBOX - FEATURE_MAILBOX) * MAILBOX_WIDTH).ok()?;
    register(table, feature.checked_add(past)?)
}

/// Which C2PMSG mailbox the register at `offset` is, counted from the one
/// the table names. An offset below the mailbox block, which the interrupt
/// registers are, is not one of them and has no number.
pub fn c2pmsg(table: &[u8], offset: u32) -> Option<usize> {
    let width = i64::try_from(MAILBOX_WIDTH).ok()?;
    let distance = i64::from(offset) - i64::from(feature_offset(table)?);
    if distance % width != 0 {
        return None;
    }
    let feature = i64::try_from(FEATURE_MAILBOX).ok()?;
    usize::try_from(feature + distance / width).ok()
}

/// The feature register's offset into that block, from the table's global
/// structure.
fn feature_offset(table: &[u8]) -> Option<u32> {
    let global = structures(table)?
        .into_iter()
        .find(|structure| structure.kind == STRUCTURE_GLOBAL)?;
    field(global.bytes, GLOBAL_FEATURE)
}
