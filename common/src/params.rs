// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The parameter page: what the preamble tells the Secure Loader, written
//! into the SLB behind the measured image at the header's
//! `bootloader_data_offset`, and the DLME the same, behind its own image.
//! drtm's SL adds the [`Handoff`] to the DLME's copy before jumping to it.
//!
//! `SL_DEV` keeps devices away from it but the measurement does not cover
//! it, so the SL validates it before use and trusts it after, which is
//! enough for a loader whose only output is a report. The layout is fixed
//! little-endian fields rather than a Rust struct, so both sides agree on
//! it by construction and the host tests read the same bytes.

use core::fmt;

use crate::record::{Format, Tag};
use crate::tpm::Bank;
use crate::transport::Interface;
use crate::uart::{Space, Uart};

pub const MAGIC: [u8; 8] = *b"DRTMSLPP";
pub const VERSION: u32 = 4;
pub const MAX_SINKS: usize = 4;

const SINK_SIZE: usize = 16;
const SINKS: usize = 40;
const SHA1: usize = SINKS + MAX_SINKS * SINK_SIZE;
const SHA256: usize = SHA1 + 20;
// Version 2, after what version 1 held.
const PATH: usize = SHA256 + 32;
const MAILBOX_SOURCE: usize = PATH + 1;
const PIT_PIN: usize = PATH + 2;
const PIT_FLAGS: usize = PATH + 3;
const IVRS_PRESENT: usize = PATH + 4;
const MAILBOX: usize = PATH + 8;
const CAPABILITY: usize = MAILBOX + 8;
const TMR_BASE: usize = CAPABILITY + 16;
const TMR_SIZE: usize = TMR_BASE + 8;
const IOAPIC: usize = TMR_SIZE + 8;
const IVRS_SHA256: usize = IOAPIC + 8;
const PSP_PCR17: usize = IVRS_SHA256 + 32;
const PSP_PCR18: usize = PSP_PCR17 + 32;
// Version 4.
const DLME: usize = PSP_PCR18 + 32;
const DLME_LEN: usize = DLME + 8;
const DLME_SHA256: usize = DLME + 16;
const DLME_PCR17_SHA1: usize = DLME_SHA256 + 32;
const DLME_PCR17_SHA256: usize = DLME_PCR17_SHA1 + 20;
// The handoff, which the preamble leaves zero.
const FROM: usize = DLME_PCR17_SHA256 + 32;
const HANDOFF_FLAGS: usize = FROM + 1;
const EXTENDED_BANKS: usize = FROM + 4;
const ANTI_ROLLBACK: usize = EXTENDED_BANKS + 4;
const TSME: usize = ANTI_ROLLBACK + 4;
const COUNTS: usize = TSME + 4;

/// Bytes the page takes.
pub const SIZE: usize = COUNTS + 4 * Tag::ALL.len();

/// The DLME's block, which holds its image, its page and its scratch
/// memory, and on the foreign path what AMD's SL leaves for it.
pub const DLME_BLOCK_SIZE: u64 = 1 << 20;

/// `HANDOFF_FLAGS`: `LAUNCH` succeeded, the mailbox stopped answering, and
/// `EXTEND_OSSL_DIGEST` measured the DLME.
const LAUNCHED: u8 = 1 << 0;
const MAILBOX_DEAD: u8 = 1 << 1;
const DLME_EXTENDED: u8 = 1 << 2;

/// `PIT_FLAGS`: an entry is there, and how the pin is wired.
const PIT_PRESENT: u8 = 1 << 0;
const PIT_ACTIVE_LOW: u8 = 1 << 1;
const PIT_LEVEL: u8 = 1 << 2;

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

/// Which launch the preamble prepared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Path {
    /// `SKINIT` alone.
    Classic,
    /// `SKINIT` into the PSP DRTM service, which the SL asks to `LAUNCH`.
    Psp,
}

/// What said where the PSP's mailbox is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailboxSource {
    None,
    Aspt,
    Smn,
}

impl MailboxSource {
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Aspt => "ASPT",
            Self::Smn => "SMN",
        }
    }
}

/// What the preamble set up with the PSP DRTM service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Psp {
    /// `C2PMSG_72`'s address.
    pub mailbox: u64,
    pub source: MailboxSource,
    /// `GET_CAPABILITY`'s three result words.
    pub capability: [u32; 3],
    /// TMR 0, in bytes.
    pub tmr_base: u64,
    pub tmr_size: u64,
    /// PCR 17 in SHA-256 after `LAUNCH`, all but its last extend, which
    /// takes words only `LAUNCH` returns.
    pub pcr17: [u8; 32],
    /// PCR 18 in SHA-256 after `LAUNCH`.
    pub pcr18: [u8; 32],
}

/// The DLME the preamble placed, and what measuring it should leave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dlme {
    /// Its block, [`DLME_BLOCK_SIZE`] bytes on a 64 MiB boundary.
    pub block: u64,
    /// The measured image at the block's start, in whole pages. Its copy of
    /// this page follows it.
    pub len: u32,
    /// The SHA-256 of its measured image.
    pub sha256: [u8; 32],
    /// PCR 17 once the classic SL has extended the image into it.
    pub pcr17_sha1: [u8; 20],
    pub pcr17_sha256: [u8; 32],
}

/// Who jumped to the DLME.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sender {
    /// A foreign SL, which leaves the handoff as the preamble wrote it.
    Foreign,
    Classic,
    Psp,
}

/// What drtm's SL tells the DLME, written into the DLME's page before the
/// jump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handoff {
    pub from: Sender,
    /// `LAUNCH`'s anti-rollback and TSME words, where it succeeded.
    pub launch: Option<(u32, u32)>,
    /// A command got no answer, so no more should be sent.
    pub mailbox_dead: bool,
    /// `EXTEND_OSSL_DIGEST` measured the DLME.
    pub dlme_extended: bool,
    /// The banks the classic SL extended the DLME into, as [`Bank::bit`]
    /// flags.
    pub extended_banks: u32,
    /// The SL's results per tag, indexed by `Tag as usize`.
    pub counts: [u32; Tag::ALL.len()],
}

impl Handoff {
    /// What a foreign SL leaves: the preamble's zeroes.
    pub const FOREIGN: Self = Self {
        from: Sender::Foreign,
        launch: None,
        mailbox_dead: false,
        dlme_extended: false,
        extended_banks: 0,
        counts: [0; Tag::ALL.len()],
    };

    /// Writes the handoff into `page`, a page [`Params::encode`] wrote.
    pub fn write(&self, page: &mut [u8; SIZE]) {
        page[FROM] = match self.from {
            Sender::Foreign => 0,
            Sender::Classic => 1,
            Sender::Psp => 2,
        };
        page[HANDOFF_FLAGS] = if self.launch.is_some() { LAUNCHED } else { 0 }
            | if self.mailbox_dead { MAILBOX_DEAD } else { 0 }
            | if self.dlme_extended { DLME_EXTENDED } else { 0 };
        page[EXTENDED_BANKS..EXTENDED_BANKS + 4]
            .copy_from_slice(&self.extended_banks.to_le_bytes());
        let (anti_rollback, tsme) = self.launch.unwrap_or_default();
        page[ANTI_ROLLBACK..ANTI_ROLLBACK + 4].copy_from_slice(&anti_rollback.to_le_bytes());
        page[TSME..TSME + 4].copy_from_slice(&tsme.to_le_bytes());
        for (i, count) in self.counts.iter().enumerate() {
            let at = COUNTS + 4 * i;
            page[at..at + 4].copy_from_slice(&count.to_le_bytes());
        }
    }

    /// The handoff in a page [`Params::decode`] accepted.
    pub fn read(page: &[u8; SIZE]) -> Result<Self, Invalid> {
        let u32_at = |at: usize| u32::from_le_bytes(page[at..at + 4].try_into().unwrap());
        let from = match page[FROM] {
            0 => Sender::Foreign,
            1 => Sender::Classic,
            2 => Sender::Psp,
            _ => return Err(Invalid::Field("handoff source")),
        };
        let flags = page[HANDOFF_FLAGS];
        Ok(Self {
            from,
            launch: (flags & LAUNCHED != 0).then(|| (u32_at(ANTI_ROLLBACK), u32_at(TSME))),
            mailbox_dead: flags & MAILBOX_DEAD != 0,
            dlme_extended: flags & DLME_EXTENDED != 0,
            extended_banks: u32_at(EXTENDED_BANKS),
            counts: core::array::from_fn(|i| u32_at(COUNTS + 4 * i)),
        })
    }
}

/// Where the PIT's interrupt reaches the IO-APIC, from the MADT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pit {
    /// The IO-APIC's register base.
    pub ioapic: u32,
    pub pin: u8,
    pub active_low: bool,
    pub level: bool,
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
    pub path: Path,
    /// Set up on the PSP path only.
    pub psp: Option<Psp>,
    pub pit: Option<Pit>,
    /// The SHA-256 of the IVRS the checks read from ACPI.
    pub ivrs_sha256: Option<[u8; 32]>,
    pub dlme: Dlme,
}

/// Why a page was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid {
    Magic,
    Version(u32),
    Size(u32),
    Field(&'static str),
}

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Magic => write!(f, "no magic"),
            Self::Version(version) => write!(f, "version {version}, not {VERSION}"),
            Self::Size(size) => write!(f, "size {size}, not {SIZE}"),
            Self::Field(field) => write!(f, "a bad {field}"),
        }
    }
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
            page[at] = uart.space() as u8;
            page[at + 1] = uart.width();
            page[at + 8..at + 16].copy_from_slice(&uart.base().to_le_bytes());
        }
        page[SHA1..SHA256].copy_from_slice(&self.expected_sha1);
        page[SHA256..PATH].copy_from_slice(&self.expected_sha256);
        page[PATH] = match self.path {
            Path::Classic => 0,
            Path::Psp => 1,
        };
        if let Some(psp) = &self.psp {
            page[MAILBOX_SOURCE] = match psp.source {
                MailboxSource::None => 0,
                MailboxSource::Aspt => 1,
                MailboxSource::Smn => 2,
            };
            page[MAILBOX..MAILBOX + 8].copy_from_slice(&psp.mailbox.to_le_bytes());
            for (i, word) in psp.capability.iter().enumerate() {
                let at = CAPABILITY + 4 * i;
                page[at..at + 4].copy_from_slice(&word.to_le_bytes());
            }
            page[TMR_BASE..TMR_BASE + 8].copy_from_slice(&psp.tmr_base.to_le_bytes());
            page[TMR_SIZE..TMR_SIZE + 8].copy_from_slice(&psp.tmr_size.to_le_bytes());
            page[PSP_PCR17..PSP_PCR18].copy_from_slice(&psp.pcr17);
            page[PSP_PCR18..DLME].copy_from_slice(&psp.pcr18);
        }
        if let Some(pit) = &self.pit {
            page[PIT_PIN] = pit.pin;
            page[PIT_FLAGS] = PIT_PRESENT
                | if pit.active_low { PIT_ACTIVE_LOW } else { 0 }
                | if pit.level { PIT_LEVEL } else { 0 };
            page[IOAPIC..IOAPIC + 4].copy_from_slice(&pit.ioapic.to_le_bytes());
        }
        if let Some(ivrs) = &self.ivrs_sha256 {
            page[IVRS_PRESENT] = 1;
            page[IVRS_SHA256..PSP_PCR17].copy_from_slice(ivrs);
        }
        page[DLME..DLME + 8].copy_from_slice(&self.dlme.block.to_le_bytes());
        page[DLME_LEN..DLME_LEN + 4].copy_from_slice(&self.dlme.len.to_le_bytes());
        page[DLME_SHA256..DLME_PCR17_SHA1].copy_from_slice(&self.dlme.sha256);
        page[DLME_PCR17_SHA1..DLME_PCR17_SHA256].copy_from_slice(&self.dlme.pcr17_sha1);
        page[DLME_PCR17_SHA256..FROM].copy_from_slice(&self.dlme.pcr17_sha256);
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
            // Safety: the preamble listed this UART from SPCR or its own
            // probe, and the checks above keep it to what `new` asks.
            *sink = Some(unsafe { Uart::new(space, base, width) });
        }
        let mut expected_sha1 = [0; 20];
        expected_sha1.copy_from_slice(&page[SHA1..SHA256]);
        let digest = |at: usize| -> [u8; 32] { page[at..at + 32].try_into().unwrap() };
        let expected_sha256 = digest(SHA256);
        let path = match page[PATH] {
            0 => Path::Classic,
            1 => Path::Psp,
            _ => return Err(Invalid::Field("path")),
        };
        let psp = match path {
            Path::Classic => None,
            Path::Psp => {
                let source = match page[MAILBOX_SOURCE] {
                    1 => MailboxSource::Aspt,
                    2 => MailboxSource::Smn,
                    _ => return Err(Invalid::Field("mailbox source")),
                };
                let mailbox = u64_at(MAILBOX);
                // The SL maps the low 4 GiB, and C2PMSG_95 lies 0x5C past.
                if mailbox == 0 || mailbox + 0x60 > 1 << 32 {
                    return Err(Invalid::Field("mailbox"));
                }
                Some(Psp {
                    mailbox,
                    source,
                    capability: [
                        u32_at(CAPABILITY),
                        u32_at(CAPABILITY + 4),
                        u32_at(CAPABILITY + 8),
                    ],
                    tmr_base: u64_at(TMR_BASE),
                    tmr_size: u64_at(TMR_SIZE),
                    pcr17: digest(PSP_PCR17),
                    pcr18: digest(PSP_PCR18),
                })
            }
        };
        let flags = page[PIT_FLAGS];
        let pit = if flags & PIT_PRESENT != 0 {
            let ioapic = u32_at(IOAPIC);
            if ioapic == 0 || u64::from(ioapic) + 0x20 > 1 << 32 {
                return Err(Invalid::Field("ioapic"));
            }
            Some(Pit {
                ioapic,
                pin: page[PIT_PIN],
                active_low: flags & PIT_ACTIVE_LOW != 0,
                level: flags & PIT_LEVEL != 0,
            })
        } else {
            None
        };
        let ivrs_sha256 = (page[IVRS_PRESENT] != 0).then(|| digest(IVRS_SHA256));
        // The SLs and the DLME map the low 4 GiB, and the block is aligned
        // as the guide's appendix E.1 asks of an MLE.
        let (block, len) = (u64_at(DLME), u32_at(DLME_LEN));
        if block == 0 || block % (64 << 20) != 0 || block + DLME_BLOCK_SIZE > 1 << 32 {
            return Err(Invalid::Field("dlme"));
        }
        if len == 0 || len % 0x1000 != 0 || u64::from(len) + SIZE as u64 > DLME_BLOCK_SIZE {
            return Err(Invalid::Field("dlme length"));
        }
        let dlme = Dlme {
            block,
            len,
            sha256: digest(DLME_SHA256),
            pcr17_sha1: page[DLME_PCR17_SHA1..DLME_PCR17_SHA256].try_into().unwrap(),
            pcr17_sha256: digest(DLME_PCR17_SHA256),
        };
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
            path,
            psp,
            pit,
            ivrs_sha256,
            dlme,
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
                // Safety: never written to, only encoded.
                Some(unsafe { Uart::new(Space::Mmio, 0xFEDC_9000, 4) }),
                None,
                None,
            ],
            expected_sha1: [1; 20],
            expected_sha256: [2; 32],
            path: Path::Psp,
            psp: Some(Psp {
                mailbox: 0xFEDA_0A20,
                source: MailboxSource::Aspt,
                capability: [5, 1, 0x0100_0000 | 0x10000],
                tmr_base: 0,
                tmr_size: 1 << 32,
                pcr17: [3; 32],
                pcr18: [4; 32],
            }),
            pit: Some(Pit {
                ioapic: 0xFEC0_0000,
                pin: 2,
                active_low: false,
                level: false,
            }),
            ivrs_sha256: Some([5; 32]),
            dlme: Dlme {
                block: 0x0800_0000,
                len: 0x8000,
                sha256: [6; 32],
                pcr17_sha1: [7; 20],
                pcr17_sha256: [8; 32],
            },
        }
    }

    #[test]
    fn round_trip() {
        let page = sample().encode();
        assert_eq!(Params::decode(&page), Ok(sample()));
        let classic = Params {
            path: Path::Classic,
            psp: None,
            pit: None,
            ivrs_sha256: None,
            ..sample()
        };
        assert_eq!(Params::decode(&classic.encode()), Ok(classic));
        assert_eq!(sample().expected(Bank::Sha256), Some([2; 32].as_slice()));
        assert_eq!(sample().expected(Bank::Sha384), None);
    }

    fn damaged_page(at: usize, value: u8) -> [u8; SIZE] {
        let mut page = sample().encode();
        page[at] = value;
        page
    }

    #[test]
    fn carries_the_handoff() {
        let mut page = sample().encode();
        assert_eq!(Handoff::read(&page), Ok(Handoff::FOREIGN));
        let handoff = Handoff {
            from: Sender::Psp,
            launch: Some((0x1234, 0x5678)),
            mailbox_dead: true,
            dlme_extended: true,
            extended_banks: Bank::Sha256.bit(),
            counts: [1, 2, 3, 4, 5],
        };
        handoff.write(&mut page);
        assert_eq!(Handoff::read(&page), Ok(handoff));
        assert_eq!(Params::decode(&page), Ok(sample()));
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
        // A version 3 page, which names no DLME.
        assert_eq!(damaged(8, 3), Err(Invalid::Version(3)));
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
        assert_eq!(damaged(PATH, 2), Err(Invalid::Field("path")));
        assert_eq!(
            damaged(MAILBOX_SOURCE, 0),
            Err(Invalid::Field("mailbox source"))
        );
        // The mailbox moved above 4 GiB.
        assert_eq!(damaged(MAILBOX + 4, 1), Err(Invalid::Field("mailbox")));
        // The DLME moved above 4 GiB, and off its 64 MiB boundary.
        assert_eq!(damaged(DLME + 4, 1), Err(Invalid::Field("dlme")));
        assert_eq!(damaged(DLME + 2, 1), Err(Invalid::Field("dlme")));
        assert_eq!(damaged(DLME_LEN, 1), Err(Invalid::Field("dlme length")));
        assert_eq!(
            Handoff::read(&damaged_page(FROM, 3)),
            Err(Invalid::Field("handoff source"))
        );
        let mut page = good;
        page[IOAPIC..IOAPIC + 4].fill(0);
        assert_eq!(Params::decode(&page), Err(Invalid::Field("ioapic")));
        assert_eq!(
            Params::decode(&good[..SIZE - 1]),
            Err(Invalid::Size(SIZE as u32 - 1))
        );
    }
}
