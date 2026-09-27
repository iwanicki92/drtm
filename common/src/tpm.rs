// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Reading and extending the DRTM PCRs: the commands and their responses,
//! for either TPM family. How the bytes reach the TPM is the caller's, through the UEFI
//! TCG2 protocol before the launch or [`crate::transport`] after it.

/// The DRTM PCRs, 17 to 22. A TPM 2.0 starts them at all ones and only a
/// locality 4 start, which is what a dynamic launch does, resets them to
/// zero (TCG PC Client Platform Firmware Profile, section 2.3.4).
pub const FIRST_DRTM_PCR: usize = 17;
pub const DRTM_PCR_COUNT: usize = 6;

/// The largest digest any bank here carries, SHA-512's.
pub const MAX_DIGEST: usize = 64;

const TPM_RC_SUCCESS: u32 = 0;

/// A PCR bank, by the hash algorithm it uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bank {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
    Sm3,
}

impl Bank {
    pub const ALL: [Self; 5] = [
        Self::Sha1,
        Self::Sha256,
        Self::Sha384,
        Self::Sha512,
        Self::Sm3,
    ];

    /// `TPM_ALG_ID` (TPM 2.0 Library Part 2, table 9).
    pub const fn algorithm(self) -> u16 {
        match self {
            Self::Sha1 => 0x0004,
            Self::Sha256 => 0x000B,
            Self::Sha384 => 0x000C,
            Self::Sha512 => 0x000D,
            Self::Sm3 => 0x0012,
        }
    }

    pub const fn digest_len(self) -> usize {
        match self {
            Self::Sha1 => 20,
            Self::Sha256 | Self::Sm3 => 32,
            Self::Sha384 => 48,
            Self::Sha512 => 64,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Sha384 => "sha384",
            Self::Sha512 => "sha512",
            Self::Sm3 => "sm3_256",
        }
    }

    /// Its bit in `EFI_TCG2_EVENT_ALGORITHM_BITMAP`, which is also how the
    /// parameter page lists the active banks (TCG EFI Protocol, section 6.4).
    pub const fn bit(self) -> u32 {
        match self {
            Self::Sha1 => 1 << 0,
            Self::Sha256 => 1 << 1,
            Self::Sha384 => 1 << 2,
            Self::Sha512 => 1 << 3,
            Self::Sm3 => 1 << 4,
        }
    }
}

/// One PCR value, as long as its bank's digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Digest {
    bytes: [u8; MAX_DIGEST],
    len: usize,
}

impl Digest {
    pub fn new(value: &[u8]) -> Option<Self> {
        let mut bytes = [0; MAX_DIGEST];
        bytes.get_mut(..value.len())?.copy_from_slice(value);
        Some(Self {
            bytes,
            len: value.len(),
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn all(&self, byte: u8) -> bool {
        self.as_bytes().iter().all(|&b| b == byte)
    }
}

/// PCR 17 to 22 of one bank, in PCR order.
pub type DrtmPcrs = [Digest; DRTM_PCR_COUNT];

/// Why the PCRs could not be read, once the TPM has answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    /// The TPM answered with a response code other than success.
    Response(u32),
    /// The response did not parse, or did not carry all six digests.
    Malformed,
}

/// `TPM2_PCR_Read` of PCR 17 to 22 in `bank`, as one command with no
/// sessions (TPM 2.0 Library Part 3, section 22.4). The selection is a
/// bitmap of three bytes, PCR `n` at byte `n / 8` bit `n % 8`, so 17 to 22
/// is `0x7E` in the third byte.
pub fn pcr_read_command(bank: Bank) -> [u8; 20] {
    let [alg_high, alg_low] = bank.algorithm().to_be_bytes();
    [
        0x80, 0x01, // TPM_ST_NO_SESSIONS
        0x00, 0x00, 0x00, 0x14, // commandSize
        0x00, 0x00, 0x01, 0x7E, // TPM_CC_PCR_Read
        0x00, 0x00, 0x00, 0x01, // pcrSelectionIn.count
        alg_high, alg_low, // hash
        0x03,    // sizeofSelect
        0x00, 0x00, 0x7E, // pcrSelect
    ]
}

/// `TPM2_PCR_Extend` of `pcr` with one digest per bank, authorised by the
/// empty password session `TPM_RS_PW` (TPM 2.0 Library Part 3, section
/// 22.2). Writes the command into `command` and returns its length.
pub fn pcr_extend_command(
    pcr: u32,
    digests: &[(Bank, &[u8])],
    command: &mut [u8; EXTEND_MAX],
) -> usize {
    let mut len = 0;
    let mut put = |bytes: &[u8]| {
        command[len..len + bytes.len()].copy_from_slice(bytes);
        len += bytes.len();
    };
    put(&[0x80, 0x02]); // TPM_ST_SESSIONS
    put(&[0; 4]); // commandSize, below
    put(&[0x00, 0x00, 0x01, 0x82]); // TPM_CC_PCR_Extend
    put(&pcr.to_be_bytes());
    put(&9u32.to_be_bytes()); // authorizationSize
    put(&[0x40, 0x00, 0x00, 0x09]); // TPM_RS_PW
    put(&[0, 0, 0, 0, 0]); // no nonce, no attributes, no HMAC
    put(&(digests.len() as u32).to_be_bytes());
    for (bank, digest) in digests {
        put(&bank.algorithm().to_be_bytes());
        put(digest);
    }
    command[2..6].copy_from_slice(&(len as u32).to_be_bytes());
    len
}

/// The longest `TPM2_PCR_Extend` [`pcr_extend_command`] writes, one digest
/// in every bank.
pub const EXTEND_MAX: usize = 31 + 5 * (2 + MAX_DIGEST);

/// `TPM_Extend` of `pcr` on a TPM 1.2 (TPM Main Part 3, section 16.1).
pub fn pcr_extend_command_1_2(pcr: u32, digest: &[u8; 20]) -> [u8; 34] {
    let mut command = [0; 34];
    command[..10].copy_from_slice(&[0x00, 0xC1, 0, 0, 0, 34, 0, 0, 0, 0x14]);
    command[10..14].copy_from_slice(&pcr.to_be_bytes());
    command[14..].copy_from_slice(digest);
    command
}

/// Whether a response of either family reports success.
pub fn parse_response_code(response: &[u8]) -> Result<(), ReadError> {
    Reader {
        data: response,
        at: 0,
    }
    .header()
}

/// Big-endian reads over a TPM response, each `None` once past its end.
struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        let bytes = self.data.get(self.at..self.at.checked_add(len)?)?;
        self.at += len;
        Some(bytes)
    }

    fn u8(&mut self) -> Option<u8> {
        self.bytes(1).map(|b| b[0])
    }

    fn u16(&mut self) -> Option<u16> {
        self.bytes(2).map(|b| u16::from_be_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Option<u32> {
        self.bytes(4)
            .map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// The header every response opens with, leaving the reader past it.
    fn header(&mut self) -> Result<(), ReadError> {
        let (Some(_tag), Some(_size), Some(rc)) = (self.u16(), self.u32(), self.u32()) else {
            return Err(ReadError::Malformed);
        };
        if rc == TPM_RC_SUCCESS {
            Ok(())
        } else {
            Err(ReadError::Response(rc))
        }
    }
}

/// The size a response's header says it is, for a transport reading it in
/// pieces. `None` until the ten header bytes are there.
pub fn response_size(header: &[u8]) -> Option<usize> {
    let bytes = header.get(2..6)?;
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize)
}

/// Parses a `TPM2_PCR_Read` response for `bank` into the six digests.
///
/// The TPM reports which PCRs it returned in `pcrSelectionOut`, then the
/// digests in that order, so the selection is walked bit by bit and each
/// digest taken as its PCR comes up. A TPM may return fewer than asked
/// for, which counts as malformed here since all six were requested.
pub fn parse_pcr_read(response: &[u8], bank: Bank) -> Result<DrtmPcrs, ReadError> {
    let mut reader = Reader {
        data: response,
        at: 0,
    };
    reader.header()?;
    let _update_counter = reader.u32().ok_or(ReadError::Malformed)?;

    // pcrSelectionOut: the one selection asked for, echoed back with the
    // PCRs the TPM is about to return. A bank the TPM does not have comes
    // back with an empty selection, and so no digests.
    let selections = reader.u32().ok_or(ReadError::Malformed)?;
    if selections != 1 {
        return Err(ReadError::Malformed);
    }
    let algorithm = reader.u16().ok_or(ReadError::Malformed)?;
    if algorithm != bank.algorithm() {
        return Err(ReadError::Malformed);
    }
    let select_len = usize::from(reader.u8().ok_or(ReadError::Malformed)?);
    let select = reader.bytes(select_len).ok_or(ReadError::Malformed)?;

    // pcrValues: one TPM2B_DIGEST per selected PCR, in ascending order.
    let digests = reader.u32().ok_or(ReadError::Malformed)?;
    let mut pcrs = [Digest::new(&[]).ok_or(ReadError::Malformed)?; DRTM_PCR_COUNT];
    let mut filled = 0;
    let mut taken = 0;
    for (byte, bits) in select.iter().enumerate() {
        for bit in 0..8 {
            if bits & (1 << bit) == 0 {
                continue;
            }
            if taken == digests {
                return Err(ReadError::Malformed);
            }
            taken += 1;
            let len = usize::from(reader.u16().ok_or(ReadError::Malformed)?);
            let digest = reader.bytes(len).ok_or(ReadError::Malformed)?;
            let slot = (byte * 8 + bit).wrapping_sub(FIRST_DRTM_PCR);
            if slot < DRTM_PCR_COUNT && len == bank.digest_len() {
                pcrs[slot] = Digest::new(digest).ok_or(ReadError::Malformed)?;
                filled += 1;
            }
        }
    }
    if filled == DRTM_PCR_COUNT {
        Ok(pcrs)
    } else {
        Err(ReadError::Malformed)
    }
}

/// `TPM_PcrRead` of one PCR on a TPM 1.2, which reads its single SHA-1 bank
/// a PCR at a time (TPM Main Part 3, section 16.2).
pub fn pcr_read_command_1_2(pcr: usize) -> [u8; 14] {
    let [a, b, c, d] = (pcr as u32).to_be_bytes();
    [
        0x00, 0xC1, // TPM_TAG_RQU_COMMAND
        0x00, 0x00, 0x00, 0x0E, // paramSize
        0x00, 0x00, 0x00, 0x15, // TPM_ORD_PcrRead
        a, b, c, d, // pcrIndex
    ]
}

/// Parses a `TPM_PcrRead` response into its SHA-1 digest.
pub fn parse_pcr_read_1_2(response: &[u8]) -> Result<Digest, ReadError> {
    let mut reader = Reader {
        data: response,
        at: 0,
    };
    reader.header()?;
    let digest = reader
        .bytes(Bank::Sha1.digest_len())
        .ok_or(ReadError::Malformed)?;
    Digest::new(digest).ok_or(ReadError::Malformed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;
    use std::vec::Vec;

    /// A `TPM2_PCR_Read` response for `bank` holding `pcrs`, built the way
    /// the TPM lays it out.
    fn response(bank: Bank, select: [u8; 3], pcrs: &[&[u8]]) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(&7u32.to_be_bytes());
        body.extend_from_slice(&1u32.to_be_bytes());
        body.extend_from_slice(&bank.algorithm().to_be_bytes());
        body.push(3);
        body.extend_from_slice(&select);
        body.extend_from_slice(&(pcrs.len() as u32).to_be_bytes());
        for pcr in pcrs {
            body.extend_from_slice(&(pcr.len() as u16).to_be_bytes());
            body.extend_from_slice(pcr);
        }
        let mut out = Vec::from([0x80, 0x01]);
        out.extend_from_slice(&((10 + body.len()) as u32).to_be_bytes());
        out.extend_from_slice(&0u32.to_be_bytes());
        out.extend_from_slice(&body);
        out
    }

    #[test]
    fn command_names_the_bank() {
        let command = pcr_read_command(Bank::Sha384);
        assert_eq!(&command[14..16], &[0x00, 0x0C]);
        assert_eq!(response_size(&command), Some(20));
    }

    #[test]
    fn parses_every_bank() {
        for bank in Bank::ALL {
            let values: Vec<Vec<u8>> = (0..6u8).map(|i| vec![i; bank.digest_len()]).collect();
            let refs: Vec<&[u8]> = values.iter().map(Vec::as_slice).collect();
            let pcrs = parse_pcr_read(&response(bank, [0, 0, 0x7E], &refs), bank).unwrap();
            for (i, pcr) in pcrs.iter().enumerate() {
                assert_eq!(pcr.as_bytes(), values[i].as_slice());
            }
        }
    }

    #[test]
    fn refuses_what_was_not_asked_for() {
        let six = [[0u8; 32].as_slice(); 6];
        // Another bank echoed back.
        let other = response(Bank::Sha1, [0, 0, 0x7E], &six);
        assert_eq!(
            parse_pcr_read(&other, Bank::Sha256),
            Err(ReadError::Malformed)
        );
        // A bank the TPM lacks: empty selection, no digests.
        let empty = response(Bank::Sha256, [0, 0, 0], &[]);
        assert_eq!(
            parse_pcr_read(&empty, Bank::Sha256),
            Err(ReadError::Malformed)
        );
        // Fewer digests than the selection names.
        let short = response(Bank::Sha256, [0, 0, 0x7E], &six[..5]);
        assert_eq!(
            parse_pcr_read(&short, Bank::Sha256),
            Err(ReadError::Malformed)
        );
        // Truncated.
        let whole = response(Bank::Sha256, [0, 0, 0x7E], &six);
        assert_eq!(
            parse_pcr_read(&whole[..whole.len() - 1], Bank::Sha256),
            Err(ReadError::Malformed)
        );
        assert_eq!(parse_pcr_read(&[], Bank::Sha256), Err(ReadError::Malformed));
    }

    #[test]
    fn passes_on_a_response_code() {
        let failed = [0x80, 0x01, 0, 0, 0, 10, 0, 0, 0x01, 0x84];
        assert_eq!(
            parse_pcr_read(&failed, Bank::Sha256),
            Err(ReadError::Response(0x184))
        );
    }

    #[test]
    fn extend_command() {
        let mut command = [0; EXTEND_MAX];
        let (sha1, sha256) = ([0x11; 20], [0x22; 32]);
        let len = pcr_extend_command(
            17,
            &[(Bank::Sha1, &sha1), (Bank::Sha256, &sha256)],
            &mut command,
        );
        assert_eq!(len, 31 + 22 + 34);
        assert_eq!(response_size(&command), Some(len));
        assert_eq!(
            &command[..14],
            &[0x80, 0x02, 0, 0, 0, len as u8, 0, 0, 1, 0x82, 0, 0, 0, 17]
        );
        assert_eq!(&command[27..33], &[0, 0, 0, 2, 0, 0x04]);
        assert_eq!(&command[len - 34..len - 32], &[0, 0x0B]);
        let every: Vec<(Bank, &[u8])> = Bank::ALL
            .iter()
            .map(|&b| (b, &[0u8; MAX_DIGEST][..b.digest_len()]))
            .collect();
        assert!(pcr_extend_command(17, &every, &mut command) <= EXTEND_MAX);
        assert_eq!(
            parse_response_code(&[0x80, 0x02, 0, 0, 0, 19, 0, 0, 0, 0]),
            Ok(())
        );
        assert_eq!(
            parse_response_code(&[0x80, 0x01, 0, 0, 0, 10, 0, 0, 0x09, 0x22]),
            Err(ReadError::Response(0x922))
        );
    }

    #[test]
    fn tpm_1_2_extend() {
        let command = pcr_extend_command_1_2(17, &[0xAB; 20]);
        assert_eq!(
            &command[..14],
            &[0, 0xC1, 0, 0, 0, 34, 0, 0, 0, 0x14, 0, 0, 0, 17]
        );
        assert_eq!(&command[14..], &[0xAB; 20]);
    }

    #[test]
    fn tpm_1_2_read() {
        assert_eq!(
            pcr_read_command_1_2(17),
            [0, 0xC1, 0, 0, 0, 14, 0, 0, 0, 0x15, 0, 0, 0, 17]
        );
        let mut answer = Vec::from([0x00, 0xC4, 0, 0, 0, 30, 0, 0, 0, 0]);
        answer.extend_from_slice(&[0xAB; 20]);
        assert_eq!(parse_pcr_read_1_2(&answer).unwrap().as_bytes(), &[0xAB; 20]);
        assert_eq!(parse_pcr_read_1_2(&answer[..29]), Err(ReadError::Malformed));
        let refused = [0x00, 0xC4, 0, 0, 0, 10, 0, 0, 0, 0x26];
        assert_eq!(parse_pcr_read_1_2(&refused), Err(ReadError::Response(0x26)));
    }
}
