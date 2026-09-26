// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! What the DRTM PCRs say about a launch, once the Secure Loader has read
//! them.

use crate::tpm::DrtmPcrs;

/// PCR 17 of one bank against what the build computed for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pcr17 {
    /// The TPM measured exactly this image.
    Expected,
    /// Still the value a TPM starts it at: no hash transaction reached it.
    Reset,
    /// Something else: the TPM hashed something other than this image.
    Other,
    /// The build computed no value for this bank, so there is nothing to
    /// judge against.
    Unjudged,
}

pub fn pcr17(pcrs: &DrtmPcrs, expected: Option<&[u8]>) -> Pcr17 {
    let read = &pcrs[0];
    match expected {
        _ if read.all(0xFF) => Pcr17::Reset,
        None => Pcr17::Unjudged,
        Some(expected) if read.as_bytes() == expected => Pcr17::Expected,
        Some(_) => Pcr17::Other,
    }
}

/// Whether PCR 18 to 22 hold zero, which the launch reset them to and
/// nothing extends when the SL extends nothing.
pub fn pcr18_22_reset(pcrs: &DrtmPcrs) -> bool {
    pcrs[1..].iter().all(|pcr| pcr.all(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tpm::Digest;

    fn pcrs(pcr17: &[u8], rest: u8) -> DrtmPcrs {
        let other = Digest::new(&[rest; 32]).unwrap();
        [
            Digest::new(pcr17).unwrap(),
            other,
            other,
            other,
            other,
            other,
        ]
    }

    #[test]
    fn judges_pcr17() {
        let good = [7; 32];
        assert_eq!(pcr17(&pcrs(&good, 0), Some(&good)), Pcr17::Expected);
        assert_eq!(pcr17(&pcrs(&[0xFF; 32], 0), Some(&good)), Pcr17::Reset);
        assert_eq!(pcr17(&pcrs(&[8; 32], 0), Some(&good)), Pcr17::Other);
        assert_eq!(pcr17(&pcrs(&[8; 48], 0), None), Pcr17::Unjudged);
        // All ones is judged even where nothing was computed.
        assert_eq!(pcr17(&pcrs(&[0xFF; 48], 0), None), Pcr17::Reset);
    }

    #[test]
    fn judges_pcr18_22() {
        assert!(pcr18_22_reset(&pcrs(&[7; 32], 0)));
        assert!(!pcr18_22_reset(&pcrs(&[7; 32], 0xFF)));
    }
}
