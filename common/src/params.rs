// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The parameter page: what the preamble tells the Secure Loader, written
//! into the SLB behind the measured image at the header's
//! `bootloader_data_offset`.
//!
//! `SL_DEV` keeps devices away from it but the measurement does not cover
//! it, so the SL validates it before use and trusts it after, which is
//! enough for a loader whose only output is a report. The layout is fixed
//! little-endian fields rather than a Rust struct, so both sides agree on
//! it by construction and the host tests read the same bytes.

use crate::record::Format;
use crate::tpm::Bank;
use crate::transport::Interface;
use crate::uart::{Space, Uart};

pub const MAGIC: [u8; 8] = *b"DRTMSLPP";
pub const VERSION: u32 = 1;
pub const MAX_SINKS: usize = 4;

const SINK_SIZE: usize = 16;
const SINKS: usize = 40;
const SHA1: usize = SINKS + MAX_SINKS * SINK_SIZE;
const SHA256: usize = SHA1 + 20;

/// Bytes the page takes.
pub const SIZE: usize = SHA256 + 32;

/// The TPM family, which decides the command set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    V1_2,
    V2_0,
}

/// What the checks read of the PSP's DRTM service before the launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PspDrtm {
    /// No service, or nothing said where its register is.
    Unknown,
    NotReady,
    /// The service is on, which keeps localities 1 to 4 locked until a PSP
    /// launch that the classic path never issues.
    Ready,
}

/// Everything the page carries, decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Params {
    pub format: Format,
    /// As `log::LevelFilter` numbers it: 0 is off, 5 is trace.
    pub level: u8,
    pub family: Family,
    pub interface: Interface,
    /// The active PCR banks, as [`Bank::bit`] flags.
    pub banks: u32,
    pub psp_drtm: PspDrtm,
    /// Time-stamp counter ticks per millisecond.
    pub tsc_per_ms: u64,
    pub sinks: [Option<Uart>; MAX_SINKS],
    /// PCR 17 after the launch, per bank computed at build time.
    pub expected_sha1: [u8; 20],
    pub expected_sha256: [u8; 32],
}

/// Why a page was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid {
    Magic,
    Version(u32),
    Size(u32),
    Field(&'static str),
}

impl Params {
    /// The expected PCR 17 for `bank`, where the build computed one.
    pub fn expected(&self, bank: Bank) -> Option<&[u8]> {
        match bank {
            Bank::Sha1 => Some(&self.expected_sha1),
            Bank::Sha256 => Some(&self.expected_sha256),
            _ => None,
        }
    }

    pub fn encode(&self) -> [u8; SIZE] {
        let mut page = [0; SIZE];
        page[..8].copy_from_slice(&MAGIC);
        page[8..12].copy_from_slice(&VERSION.to_le_bytes());
        page[12..16].copy_from_slice(&(SIZE as u32).to_le_bytes());
        page[16] = match self.format {
            Format::Text => 0,
            Format::Json => 1,
        };
        page[17] = self.level;
        page[18] = match self.family {
            Family::V1_2 => 1,
            Family::V2_0 => 2,
        };
        page[19] = match self.interface {
            Interface::Tis => 1,
            Interface::Crb => 2,
        };
        page[20] = match self.psp_drtm {
            PspDrtm::Unknown => 0,
            PspDrtm::NotReady => 1,
            PspDrtm::Ready => 2,
        };
        page[21] = self.sinks.iter().flatten().count() as u8;
        page[24..28].copy_from_slice(&self.banks.to_le_bytes());
        page[32..40].copy_from_slice(&self.tsc_per_ms.to_le_bytes());
        for (slot, uart) in self.sinks.iter().flatten().enumerate() {
            let at = SINKS + slot * SINK_SIZE;
            page[at] = uart.space as u8;
            page[at + 1] = uart.width;
            page[at + 8..at + 16].copy_from_slice(&uart.base.to_le_bytes());
        }
        page[SHA1..SHA256].copy_from_slice(&self.expected_sha1);
        page[SHA256..SIZE].copy_from_slice(&self.expected_sha256);
        page
    }

    pub fn decode(page: &[u8]) -> Result<Self, Invalid> {
        let page: &[u8; SIZE] = page
            .get(..SIZE)
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or(Invalid::Size(page.len() as u32))?;
        let u32_at =
            |at: usize| u32::from_le_bytes([page[at], page[at + 1], page[at + 2], page[at + 3]]);
        let u64_at = |at: usize| u64::from(u32_at(at)) | (u64::from(u32_at(at + 4)) << 32);
        if page[..8] != MAGIC {
            return Err(Invalid::Magic);
        }
        if u32_at(8) != VERSION {
            return Err(Invalid::Version(u32_at(8)));
        }
        if u32_at(12) != SIZE as u32 {
            return Err(Invalid::Size(u32_at(12)));
        }
        let format = match page[16] {
            0 => Format::Text,
            1 => Format::Json,
            _ => return Err(Invalid::Field("format")),
        };
        if page[17] > 5 {
            return Err(Invalid::Field("level"));
        }
        let family = match page[18] {
            1 => Family::V1_2,
            2 => Family::V2_0,
            _ => return Err(Invalid::Field("family")),
        };
        let interface = match page[19] {
            1 => Interface::Tis,
            2 => Interface::Crb,
            _ => return Err(Invalid::Field("interface")),
        };
        let psp_drtm = match page[20] {
            0 => PspDrtm::Unknown,
            1 => PspDrtm::NotReady,
            2 => PspDrtm::Ready,
            _ => return Err(Invalid::Field("psp_drtm")),
        };
        let count = usize::from(page[21]);
        if count > MAX_SINKS {
            return Err(Invalid::Field("sink count"));
        }
        let tsc_per_ms = u64_at(32);
        if tsc_per_ms == 0 {
            return Err(Invalid::Field("tsc_per_ms"));
        }
        let mut sinks = [None; MAX_SINKS];
        for (slot, sink) in sinks.iter_mut().enumerate().take(count) {
            let at = SINKS + slot * SINK_SIZE;
            let space = match page[at] {
                1 => Space::Io,
                2 => Space::Mmio,
                _ => return Err(Invalid::Field("sink space")),
            };
            let width = page[at + 1];
            let base = u64_at(at + 8);
            let fits = match space {
                Space::Io => base <= 0xFFFF,
                Space::Mmio => base.saturating_add(8 * u64::from(width)) <= 1 << 32,
            };
            if !matches!(width, 1 | 4) || !fits {
                return Err(Invalid::Field("sink"));
            }
            *sink = Some(Uart { space, base, width });
        }
        let mut expected_sha1 = [0; 20];
        expected_sha1.copy_from_slice(&page[SHA1..SHA256]);
        let mut expected_sha256 = [0; 32];
        expected_sha256.copy_from_slice(&page[SHA256..SIZE]);
        Ok(Self {
            format,
            level: page[17],
            family,
            interface,
            banks: u32_at(24),
            psp_drtm,
            tsc_per_ms,
            sinks,
            expected_sha1,
            expected_sha256,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uart::COM1;

    fn sample() -> Params {
        Params {
            format: Format::Json,
            level: 4,
            family: Family::V2_0,
            interface: Interface::Tis,
            banks: Bank::Sha1.bit() | Bank::Sha256.bit(),
            psp_drtm: PspDrtm::Ready,
            tsc_per_ms: 2_000_000,
            sinks: [
                Some(COM1),
                Some(Uart {
                    space: Space::Mmio,
                    base: 0xFEDC_9000,
                    width: 4,
                }),
                None,
                None,
            ],
            expected_sha1: [1; 20],
            expected_sha256: [2; 32],
        }
    }

    #[test]
    fn round_trip() {
        let page = sample().encode();
        assert_eq!(Params::decode(&page), Ok(sample()));
        assert_eq!(sample().expected(Bank::Sha256), Some([2; 32].as_slice()));
        assert_eq!(sample().expected(Bank::Sha384), None);
    }

    #[test]
    fn refuses_a_damaged_page() {
        let good = sample().encode();
        let damaged = |at: usize, value: u8| {
            let mut page = good;
            page[at] = value;
            Params::decode(&page)
        };
        assert_eq!(damaged(0, b'X'), Err(Invalid::Magic));
        assert_eq!(damaged(8, 2), Err(Invalid::Version(2)));
        assert_eq!(damaged(12, 0), Err(Invalid::Size(SIZE as u32 & !0xFF)));
        assert_eq!(damaged(16, 7), Err(Invalid::Field("format")));
        assert_eq!(damaged(17, 6), Err(Invalid::Field("level")));
        assert_eq!(damaged(21, 5), Err(Invalid::Field("sink count")));
        assert_eq!(damaged(SINKS, 3), Err(Invalid::Field("sink space")));
        assert_eq!(damaged(SINKS + 1, 2), Err(Invalid::Field("sink")));
        // An MMIO sink moved above 4 GiB.
        assert_eq!(
            damaged(SINKS + SINK_SIZE + 12, 1),
            Err(Invalid::Field("sink"))
        );
        assert_eq!(
            Params::decode(&good[..SIZE - 1]),
            Err(Invalid::Size(SIZE as u32 - 1))
        );
    }
}
