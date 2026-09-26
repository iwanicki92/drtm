// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The PSP DRTM service's mailbox (AMD publication 58453, sections 3 and
//! 4): `C2PMSG_72` carries the command and the status, `C2PMSG_93` to `_95`
//! the arguments and results.

use core::ptr;

use crate::clock::Deadline;

/// `C2PMSG_72`'s offset from the PSP's register base (58453, appendix G.2).
pub const C2PMSG_72_OFFSET: u64 = 0x10A20;

/// `C2PMSG_93`, `_94` and `_95`, as offsets from `C2PMSG_72`.
const ARGUMENTS: [usize; 3] = [(93 - 72) * 4, (94 - 72) * 4, (95 - 72) * 4];

/// Bit 31, which the service sets once it will take a command and x86
/// clears by writing one. Bits 27:24 hold the TMR index, 23:16 the
/// command and 15:0 the status.
pub const READY: u32 = 1 << 31;
const INDEX_SHIFT: u32 = 24;
const COMMAND_SHIFT: u32 = 16;
const STATUS_MASK: u32 = 0xFFFF;

/// How long a command may take: the 5 s GRUB and SKL allow, and two
/// minutes for the extend, which hashed at about 85 KB/s on hardware.
pub const TIMEOUT_MS: u64 = 5000;
pub const EXTEND_TIMEOUT_MS: u64 = 120_000;

/// `GET_CAPABILITY`'s first result word: bit 0 DRTM enabled.
pub const CAP_ENABLED: u32 = 1 << 0;

/// A TMR's size comes in these units, and descriptors are this long.
pub const TMR_UNIT: u64 = 0x10000;
pub const TMR_DESCRIPTOR_LEN: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Command {
    GetCapability = 0x1,
    TmrSetup = 0x2,
    TmrRelease = 0x3,
    Launch = 0x4,
    GetTcgLogs = 0x7,
    TpmLocalityAccess = 0x8,
    GetTmrDescriptors = 0x9,
    AllocateSharedMemory = 0xA,
    ExtendOsslDigest = 0xB,
    GetIvrsTableInfo = 0xC,
}

impl Command {
    pub const fn name(self) -> &'static str {
        match self {
            Self::GetCapability => "GET_CAPABILITY",
            Self::TmrSetup => "TMR_SETUP",
            Self::TmrRelease => "TMR_RELEASE",
            Self::Launch => "LAUNCH",
            Self::GetTcgLogs => "GET_TCG_LOGS",
            Self::TpmLocalityAccess => "TPM_LOCALITY_ACCESS",
            Self::GetTmrDescriptors => "GET_TMR_DESCRIPTORS",
            Self::AllocateSharedMemory => "ALLOCATE_SHARED_MEMORY",
            Self::ExtendOsslDigest => "EXTEND_OSSL_DIGEST",
            Self::GetIvrsTableInfo => "GET_IVRS_TABLE_INFO",
        }
    }

    /// The word written to `C2PMSG_72`, with the ready bit clear.
    pub const fn word(self, index: u8) -> u32 {
        ((index as u32 & 0xF) << INDEX_SHIFT) | ((self as u32) << COMMAND_SHIFT)
    }
}

pub const NO_ERROR: u16 = 0x0;
pub const NOT_SUPPORTED: u16 = 0x1;
pub const LAUNCH_ERROR: u16 = 0x2;
pub const GET_TCG_LOGS_FAILED: u16 = 0x7;
pub const OUT_OF_RESOURCES: u16 = 0x8;
pub const GET_IVRS_TABLE_FAILED: u16 = 0x10;

/// The status's name, as the guide's section 3.1 defines it.
pub const fn status_name(status: u16) -> &'static str {
    match status {
        0x0 => "DRTM_NO_ERROR",
        0x1 => "DRTM_NOT_SUPPORTED",
        0x2 => "DRTM_LAUNCH_ERROR",
        0x3 => "DRTM_TMR_SETUP_FAILED_ERROR",
        0x4 => "DRTM_TMR_DESTROY_FAILED_ERROR",
        0x7 => "DRTM_GET_TCG_LOGS_FAILED",
        0x8 => "DRTM_OUT_OF_RESOURCES_ERROR",
        0x9 => "DRTM_GENERIC_ERROR",
        0xA => "DRTM_INVALID_SERVICE_ID_ERROR",
        0xB => "DRTM_MEMORY_UNALIGNED_ERROR",
        0xC => "DRTM_MINIMUM_SIZE_ERROR",
        0xD => "DRTM_GET_TMR_DESCRIPTOR_FAILED",
        0xE => "DRTM_EXTEND_OSSL_DIGEST_FAILED",
        0xF => "DRTM_TMR_SETUP_NOT_ALLOWED",
        0x10 => "DRTM_GET_IVRS_TABLE_FAILED",
        _ => "an undefined status",
    }
}

/// What a command came back with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    /// `C2PMSG_93` to `_95` as the service left them.
    pub results: [u32; 3],
}

impl Answer {
    pub fn ok(&self) -> bool {
        self.status == NO_ERROR
    }

    /// A region's size in `C2PMSG_93` and its address in `_94` and `_95`.
    pub fn region(&self) -> (u64, u32) {
        let base = u64::from(self.results[1]) | (u64::from(self.results[2]) << 32);
        (base, self.results[0])
    }
}

/// Why a command got no answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The ready bit was clear before the command could be written.
    Busy,
    /// No status within the command's time.
    Timeout,
}

impl Error {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Busy => "the ready bit stayed clear before it",
            Self::Timeout => "no status in time",
        }
    }
}

/// The mailbox at a known `C2PMSG_72` address.
pub struct Mailbox {
    address: usize,
    tsc_per_ms: u64,
}

impl Mailbox {
    /// # Safety
    ///
    /// `address` must be the service's `C2PMSG_72`, mapped as device memory
    /// with `C2PMSG_95` behind it, for as long as the mailbox is used.
    pub unsafe fn new(address: usize, tsc_per_ms: u64) -> Self {
        Self {
            address,
            tsc_per_ms,
        }
    }

    pub fn address(&self) -> usize {
        self.address
    }

    fn read(&self, offset: usize) -> u32 {
        // Safety: `new`'s caller vouched for the register block.
        unsafe { ptr::read_volatile((self.address + offset) as *const u32) }
    }

    fn write(&self, offset: usize, value: u32) {
        // Safety: as for `read`.
        unsafe { ptr::write_volatile((self.address + offset) as *mut u32, value) }
    }

    /// What `C2PMSG_72` holds now.
    pub fn register(&self) -> u32 {
        self.read(0)
    }

    fn wait_ready(&self, ms: u64) -> bool {
        let deadline = Deadline::after(ms, self.tsc_per_ms);
        loop {
            if self.register() & READY != 0 {
                return true;
            }
            if deadline.passed() {
                return self.register() & READY != 0;
            }
            core::hint::spin_loop();
        }
    }

    /// The zero write the service needs before it answers anything, then
    /// the wait for it to say it is ready.
    pub fn kick(&self) -> Result<(), Error> {
        self.write(0, 0);
        if self.wait_ready(TIMEOUT_MS) {
            Ok(())
        } else {
            Err(Error::Timeout)
        }
    }

    /// Issues `command` with `arguments` in `C2PMSG_93` to `_95` and waits
    /// up to `timeout_ms` for the status.
    pub fn command(
        &self,
        command: Command,
        index: u8,
        arguments: [u32; 3],
        timeout_ms: u64,
    ) -> Result<Answer, Error> {
        if !self.wait_ready(TIMEOUT_MS) {
            return Err(Error::Busy);
        }
        for (offset, value) in ARGUMENTS.into_iter().zip(arguments) {
            self.write(offset, value);
        }
        self.write(0, command.word(index));
        if !self.wait_ready(timeout_ms) {
            return Err(Error::Timeout);
        }
        Ok(Answer {
            status: (self.register() & STATUS_MASK) as u16,
            results: ARGUMENTS.map(|offset| self.read(offset)),
        })
    }
}

/// One `TMR_DESCRIPTOR_RESPONSE` (58453, section 4.5): the base, then a
/// size the guide gives no unit for.
pub fn tmr_descriptor(bytes: &[u8]) -> Option<(u64, u32)> {
    let word = |at: usize| -> Option<u32> {
        Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
    };
    Some((u64::from(word(0)?) | (u64::from(word(4)?) << 32), word(8)?))
}

/// How a descriptor's size matched a TMR of `size` bytes, if it did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TmrSizeUnit {
    Bytes,
    Units64K,
}

pub fn tmr_matches(descriptor: (u64, u32), base: u64, size: u64) -> Option<TmrSizeUnit> {
    let (at, len) = descriptor;
    if at != base {
        return None;
    }
    if u64::from(len) == size {
        Some(TmrSizeUnit::Bytes)
    } else if u64::from(len) * TMR_UNIT == size {
        Some(TmrSizeUnit::Units64K)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_commands() {
        // What GRUB writes for TMR_SETUP of index 0, and SKL for LAUNCH.
        assert_eq!(Command::TmrSetup.word(0), 0x0002_0000);
        assert_eq!(Command::Launch.word(0), 0x0004_0000);
        assert_eq!(Command::ExtendOsslDigest.word(0), 0x000B_0000);
        assert_eq!(Command::TmrSetup.word(7), 0x0702_0000);
        assert_eq!(Command::GetIvrsTableInfo.word(0) & READY, 0);
    }

    #[test]
    fn names_statuses() {
        assert_eq!(status_name(0), "DRTM_NO_ERROR");
        assert_eq!(status_name(0x2), "DRTM_LAUNCH_ERROR");
        assert_eq!(status_name(0xE), "DRTM_EXTEND_OSSL_DIGEST_FAILED");
        assert_eq!(status_name(0x10), "DRTM_GET_IVRS_TABLE_FAILED");
        assert_eq!(status_name(0x5), "an undefined status");
    }

    #[test]
    fn reads_regions() {
        let answer = Answer {
            status: 0,
            results: [0x4ea, 0xac04_00a0, 0x1],
        };
        assert_eq!(answer.region(), (0x1_ac04_00a0, 0x4ea));
    }

    #[test]
    fn matches_tmrs_in_either_unit() {
        let mut bytes = [0; TMR_DESCRIPTOR_LEN];
        bytes[8..12].copy_from_slice(&0x10000u32.to_le_bytes());
        let descriptor = tmr_descriptor(&bytes).unwrap();
        assert_eq!(descriptor, (0, 0x10000));
        // [0, 4 GiB) in 64 KiB units, and 64 KiB in bytes.
        assert_eq!(
            tmr_matches(descriptor, 0, 1 << 32),
            Some(TmrSizeUnit::Units64K)
        );
        assert_eq!(
            tmr_matches(descriptor, 0, 0x10000),
            Some(TmrSizeUnit::Bytes)
        );
        assert_eq!(tmr_matches(descriptor, 0x10000, 1 << 32), None);
        assert_eq!(tmr_matches(descriptor, 0, 1 << 31), None);
        assert_eq!(tmr_descriptor(&bytes[..11]), None);
    }
}
