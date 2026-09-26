// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The TCG2 event log the PSP DRTM service hands out (58453, appendix C):
//! a `TCG_PCClientPCREvent` carrying the `Spec ID Event03` structure, then
//! `TCG_PCR_EVENT2` entries (TCG PC Client Platform Firmware Profile,
//! sections 10.2.1 and 10.2.2).

use core::fmt;

use crate::sha256;

/// `TPM_ALG_SHA256`.
const ALG_SHA256: u16 = 0x000B;
const SPEC_ID: &[u8; 16] = b"Spec ID Event03\0";
/// The most algorithms a header may declare here.
const MAX_ALGORITHMS: usize = 8;

/// Why a log did not parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Malformed {
    /// The first event is not a `Spec ID Event03` header.
    Header,
    /// The header declares no SHA-256 digest.
    NoSha256,
    /// An event runs past the log's end, or names an algorithm the header
    /// does not declare, at this offset.
    Event(usize),
}

impl fmt::Display for Malformed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header => write!(f, "no Spec ID Event03 header"),
            Self::NoSha256 => write!(f, "no SHA-256 digests"),
            Self::Event(at) => write!(f, "a broken event at {at:#x}"),
        }
    }
}

/// One event, with its SHA-256 digest where it carries one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event<'a> {
    pub offset: usize,
    pub pcr: u32,
    pub kind: u32,
    pub sha256: Option<[u8; sha256::LEN]>,
    pub data: &'a [u8],
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

/// A parsed log's header, and an iterator over its events.
pub struct Log<'a> {
    bytes: &'a [u8],
    algorithms: [(u16, u16); MAX_ALGORITHMS],
    count: usize,
    at: usize,
    failed: bool,
}

impl<'a> Log<'a> {
    /// Reads the header. The events are read as the iterator reaches them.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Malformed> {
        // pcrIndex, eventType, a SHA-1 digest, eventDataSize, then the data.
        let size = u32_at(bytes, 28).ok_or(Malformed::Header)? as usize;
        let spec = bytes.get(32..32 + size).ok_or(Malformed::Header)?;
        if spec.get(..16) != Some(SPEC_ID.as_slice()) {
            return Err(Malformed::Header);
        }
        // platformClass, the version and uintnSize, then the algorithms.
        let count = u32_at(spec, 24).ok_or(Malformed::Header)? as usize;
        if count > MAX_ALGORITHMS {
            return Err(Malformed::Header);
        }
        let mut algorithms = [(0, 0); MAX_ALGORITHMS];
        for (i, slot) in algorithms.iter_mut().enumerate().take(count) {
            let at = 28 + 4 * i;
            *slot = (
                u16_at(spec, at).ok_or(Malformed::Header)?,
                u16_at(spec, at + 2).ok_or(Malformed::Header)?,
            );
        }
        if !algorithms[..count]
            .iter()
            .any(|&(alg, len)| alg == ALG_SHA256 && usize::from(len) == sha256::LEN)
        {
            return Err(Malformed::NoSha256);
        }
        Ok(Self {
            bytes,
            algorithms,
            count,
            at: 32 + size,
            failed: false,
        })
    }

    fn digest_len(&self, alg: u16) -> Option<usize> {
        self.algorithms[..self.count]
            .iter()
            .find(|&&(id, _)| id == alg)
            .map(|&(_, len)| usize::from(len))
    }

    fn event(&self, offset: usize) -> Option<(Event<'a>, usize)> {
        let bytes = self.bytes;
        let pcr = u32_at(bytes, offset)?;
        let kind = u32_at(bytes, offset + 4)?;
        let digests = u32_at(bytes, offset + 8)?;
        let mut at = offset + 12;
        let mut sha = None;
        for _ in 0..digests.min(MAX_ALGORITHMS as u32 + 1) {
            let alg = u16_at(bytes, at)?;
            let len = self.digest_len(alg)?;
            let digest = bytes.get(at + 2..at + 2 + len)?;
            if alg == ALG_SHA256 {
                sha = Some(digest.try_into().ok()?);
            }
            at += 2 + len;
        }
        if digests as usize > MAX_ALGORITHMS {
            return None;
        }
        let size = u32_at(bytes, at)? as usize;
        let data = bytes.get(at + 4..(at + 4).checked_add(size)?)?;
        Some((
            Event {
                offset,
                pcr,
                kind,
                sha256: sha,
                data,
            },
            at + 4 + size,
        ))
    }
}

impl<'a> Iterator for Log<'a> {
    type Item = Result<Event<'a>, Malformed>;

    /// Stops at the end, and at zeros to the end, which is padding.
    fn next(&mut self) -> Option<Self::Item> {
        let rest = self.bytes.get(self.at..)?;
        if self.failed || rest.iter().all(|&byte| byte == 0) {
            return None;
        }
        match self.event(self.at) {
            Some((event, next)) => {
                self.at = next;
                Some(Ok(event))
            }
            None => {
                self.failed = true;
                Some(Err(Malformed::Event(self.at)))
            }
        }
    }
}

/// PCR 17 and PCR 18 as the log's SHA-256 digests leave them, from zero.
pub fn replay(log: &[u8]) -> Result<[[u8; sha256::LEN]; 2], Malformed> {
    let mut pcrs = [[0; sha256::LEN]; 2];
    for event in Log::parse(log)? {
        let event = event?;
        if let (17 | 18, Some(digest)) = (event.pcr, event.sha256) {
            let pcr = &mut pcrs[event.pcr as usize - 17];
            *pcr = sha256::extend(pcr, &digest);
        }
    }
    Ok(pcrs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    /// The log Turin firmware's service handed out after a launch and an
    /// extend: the header, the launch's five events and the extend's four.
    const TURIN: &str = "\
        000000000300000000000000000000000000000000000000000000002100000053706563\
        204944204576656e743033000000000000020002010000000b0020000011000000018000\
        00010000000b000376f71cf0763f187c2ccc908005aa675bd833b4d61d40b136901848e1\
        d48c99000000001100000002800000010000000b00df3f619804a92fdb4057192dc43dd7\
        48ea778adc52bc498ce80524c014b8111904000000000000001100000003800000010000\
        000b0001acecb507abfe1a354aa8064f4af5d3f1acd019e37db3c11c97523b71c76e9d08\
        00000000000000010000001200000004800000010000000b00da9bcd2a719952e18c8638\
        4f42c10d3232a86e627b4fbe74c67f007331d69b5f000000001200000005800000010000\
        000b0067abdd721024f0ff4e0b3f4c2fc13bc5bad42d0b7851d456d88d203d15aaa45004\
        000000010000001100000006800000010000000b004f1fbf673e6a589d9e687d43949148\
        980a5916d4df3a31dffc72ddd36f148e02000000001200000006800000010000000b004f\
        1fbf673e6a589d9e687d43949148980a5916d4df3a31dffc72ddd36f148e020000000011\
        00000007800000010000000b00d26b8af5496bcf17f82b80b52145875666602eef3123c3\
        a1af4909708e99c97705000000414d44534c1200000007800000010000000b00d26b8af5\
        496bcf17f82b80b52145875666602eef3123c3a1af4909708e99c97705000000414d4453\
        4c";

    fn unhex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }

    fn hex(bytes: &[u8]) -> std::string::String {
        bytes.iter().map(|b| std::format!("{b:02x}")).collect()
    }

    /// A header declaring SHA-256 alone, as the service's does.
    fn header() -> Vec<u8> {
        let mut log = Vec::new();
        log.extend_from_slice(&[0; 4]);
        log.extend_from_slice(&3u32.to_le_bytes());
        log.extend_from_slice(&[0; 20]);
        log.extend_from_slice(&33u32.to_le_bytes());
        log.extend_from_slice(SPEC_ID);
        log.extend_from_slice(&[0, 0, 0, 0, 0, 2, 0, 2, 1, 0, 0, 0, 0x0b, 0, 0x20, 0, 0]);
        log
    }

    fn event(log: &mut Vec<u8>, pcr: u32, kind: u32, digest: &[u8; 32], data: &[u8]) {
        log.extend_from_slice(&pcr.to_le_bytes());
        log.extend_from_slice(&kind.to_le_bytes());
        log.extend_from_slice(&1u32.to_le_bytes());
        log.extend_from_slice(&ALG_SHA256.to_le_bytes());
        log.extend_from_slice(digest);
        log.extend_from_slice(&(data.len() as u32).to_le_bytes());
        log.extend_from_slice(data);
    }

    #[test]
    fn replays_a_log_built_by_hand() {
        let mut log = header();
        let a = sha256::digest(b"a");
        let b = sha256::digest(b"b");
        event(&mut log, 17, 0x8001, &a, &[]);
        event(&mut log, 18, 0x8004, &b, b"xyz");
        event(&mut log, 17, 0x8007, &b, &[]);
        let events: Vec<_> = Log::parse(&log).unwrap().collect();
        assert_eq!(events.len(), 3);
        let second = events[1].unwrap();
        assert_eq!(
            (second.pcr, second.kind, second.data),
            (18, 0x8004, b"xyz".as_slice())
        );
        let zero = [0; 32];
        assert_eq!(
            replay(&log),
            Ok([
                sha256::extend(&sha256::extend(&zero, &a), &b),
                sha256::extend(&zero, &b)
            ])
        );
        // Padding after the last event is not one.
        log.extend_from_slice(&[0; 64]);
        assert_eq!(Log::parse(&log).unwrap().count(), 3);
    }

    #[test]
    fn replays_the_turin_log() {
        let log = unhex(TURIN);
        assert_eq!(log.len(), 0x21d);
        let events: Vec<_> = Log::parse(&log).unwrap().map(Result::unwrap).collect();
        let kinds: Vec<_> = events.iter().map(|e| (e.pcr, e.kind)).collect();
        assert_eq!(
            kinds,
            [
                (17, 0x8001),
                (17, 0x8002),
                (17, 0x8003),
                (18, 0x8004),
                (18, 0x8005),
                (17, 0x8006),
                (18, 0x8006),
                (17, 0x8007),
                (18, 0x8007)
            ]
        );
        assert_eq!(events[7].data, b"AMDSL");
        assert_eq!(events[7].sha256, Some(sha256::digest(b"AMDSL")));
        let [pcr17, pcr18] = replay(&log).unwrap();
        assert!(hex(&pcr17).starts_with("e55d9fee") && hex(&pcr17).ends_with("f3722f17"));
        assert!(hex(&pcr18).starts_with("7a9c55b6") && hex(&pcr18).ends_with("b89792d4"));
    }

    #[test]
    fn refuses_malformed_logs() {
        let log = unhex(TURIN);
        assert!(matches!(Log::parse(&log[..40]), Err(Malformed::Header)));
        let mut other = log.clone();
        other[32] = b'X';
        assert!(matches!(Log::parse(&other), Err(Malformed::Header)));
        // The one algorithm declared as SHA-1.
        let mut sha1 = log.clone();
        sha1[60] = 0x04;
        assert!(matches!(Log::parse(&sha1), Err(Malformed::NoSha256)));
        // Cut inside the last event.
        let cut = &log[..log.len() - 3];
        let last = Log::parse(cut).unwrap().last().unwrap();
        assert!(matches!(last, Err(Malformed::Event(_))));
        assert!(replay(cut).is_err());
        // An event naming an algorithm the header does not declare.
        let mut alg = log.clone();
        alg[0x41 + 12] = 0x04;
        assert_eq!(replay(&alg), Err(Malformed::Event(0x41)));
    }
}
