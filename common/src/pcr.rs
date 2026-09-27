// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The DRTM PCRs through [`crate::transport`]: reading one bank, and
//! extending PCR 17, for the SL and the DLME.

use core::fmt;

use crate::params::Family;
use crate::tpm::{self, Bank, DrtmPcrs, FIRST_DRTM_PCR, ReadError};
use crate::transport::{self, Tpm};

/// Header, update counter, selection and six SHA-512 digests fit in 512.
/// An extend answers in 30 bytes at most, a TPM 1.2 with the new value.
const RESPONSE_MAX: usize = 512;
const EXTEND_RESPONSE_MAX: usize = 32;

/// Why a command did not do what was asked.
pub enum Failure {
    Transport(transport::Error),
    Read(ReadError),
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(error) => write!(f, "the TPM did not answer: {}", error.name()),
            Self::Read(ReadError::Response(rc)) => write!(f, "response code {rc:#x}"),
            Self::Read(ReadError::Malformed) => write!(f, "a malformed response"),
        }
    }
}

pub fn read(tpm: &Tpm, family: Family, bank: Bank) -> Result<DrtmPcrs, Failure> {
    let mut response = [0; RESPONSE_MAX];
    match family {
        Family::V2_0 => {
            let len = tpm
                .transmit(&tpm::pcr_read_command(bank), &mut response)
                .map_err(Failure::Transport)?;
            tpm::parse_pcr_read(&response[..len], bank).map_err(Failure::Read)
        }
        Family::V1_2 => {
            let mut pcrs = [tpm::Digest::new(&[]).ok_or(Failure::Read(ReadError::Malformed))?;
                tpm::DRTM_PCR_COUNT];
            for (i, pcr) in pcrs.iter_mut().enumerate() {
                let len = tpm
                    .transmit(
                        &tpm::pcr_read_command_1_2(FIRST_DRTM_PCR + i),
                        &mut response,
                    )
                    .map_err(Failure::Transport)?;
                *pcr = tpm::parse_pcr_read_1_2(&response[..len]).map_err(Failure::Read)?;
            }
            Ok(pcrs)
        }
    }
}

/// Extends PCR 17 with `digests`, one per bank. A TPM 1.2 takes the SHA-1
/// digest alone.
pub fn extend17(tpm: &Tpm, family: Family, digests: &[(Bank, &[u8])]) -> Result<(), Failure> {
    let mut response = [0; EXTEND_RESPONSE_MAX];
    let pcr = FIRST_DRTM_PCR as u32;
    let len = match family {
        Family::V2_0 => {
            let mut command = [0; tpm::EXTEND_MAX];
            let len = tpm::pcr_extend_command(pcr, digests, &mut command);
            tpm.transmit(&command[..len], &mut response)
        }
        Family::V1_2 => {
            let sha1 = digests
                .iter()
                .find_map(|&(bank, digest)| (bank == Bank::Sha1).then_some(digest))
                .and_then(|digest| digest.try_into().ok())
                .ok_or(Failure::Read(ReadError::Malformed))?;
            tpm.transmit(&tpm::pcr_extend_command_1_2(pcr, sha1), &mut response)
        }
    }
    .map_err(Failure::Transport)?;
    tpm::parse_response_code(&response[..len]).map_err(Failure::Read)
}
