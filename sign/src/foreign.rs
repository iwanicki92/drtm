// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! A signed Secure Loader built elsewhere, for a debug build that launches
//! it in place of drtm's own. Either the bare image, measured bytes then
//! the signature block, or a PE file that carries one.

use crate::{
    ALIGN, BLOCK_LEN, HDR_VERSION, HEADER_LEN, SIGNATURE_LEN, TAG, TOKEN_KEY_ID, TOKEN_LEN,
};

/// The GUID at offset 4 of `AMD_SL_HDR_V1` (AMD publication 58453, appendix
/// B.3), which finds the image inside a PE file.
const SL_GUID: [u8; 16] = [
    0xda, 0x75, 0xc4, 0x0e, 0xd0, 0xd3, 0x5d, 0x48, 0xb6, 0xae, 0x3c, 0xb2, 0x0f, 0xb7, 0xad, 0x3a,
];

/// What `SKINIT` and the SL may use: the SLB.
const SLB_SIZE: usize = 0x10000;

/// `TotalLength` in `AMD_SL_HDR_V1`: the image and its signature block,
/// with the OS loader's parameters for the SL right behind.
const TOTAL_LENGTH: usize = 0x18;

/// The size of those parameters, `AmdSlInputParameters`.
pub const INPUT_PARAMETERS_LEN: usize = 0x4c;

/// A signed SL, cut out of whatever carried it.
#[derive(Debug)]
pub struct Foreign {
    /// The measured image, padded to [`ALIGN`], then the signature block.
    pub image: Vec<u8>,
    /// Bytes `SKINIT` measures, as the header gives them.
    pub measured: usize,
    /// Where the image started in the input.
    pub offset: usize,
    /// The key the token names.
    pub key_id: [u8; 16],
    /// Where in the SLB its `AmdSlInputParameters` go, for a header with
    /// the GUID.
    pub parameters: Option<usize>,
}

impl Foreign {
    pub fn token(&self) -> &[u8; TOKEN_LEN] {
        let at = self.image.len() - TOKEN_LEN;
        self.image[at..].try_into().unwrap()
    }

    /// The `$AS1` header's `SecPatchLevel`, which `LAUNCH` extends into
    /// PCR 18.
    pub fn sec_patch_level(&self) -> u32 {
        let at = self.image.len() - crate::BLOCK_LEN + crate::HDR_SEC_PATCH_LEVEL;
        u32::from_le_bytes(self.image[at..at + 4].try_into().unwrap())
    }
}

/// Finds the SL in `input` and checks it has the layout the SLB needs.
pub fn extract(input: &[u8]) -> Result<Foreign, String> {
    let offset = if input.starts_with(b"MZ") {
        let mut found = input
            .windows(SL_GUID.len())
            .enumerate()
            .filter(|(_, window)| *window == SL_GUID)
            .map(|(at, _)| at);
        match (found.next(), found.next()) {
            (Some(at), None) if at >= 4 => at - 4,
            (None, _) => return Err("the PE file carries no SL header GUID".into()),
            (Some(_), None) => return Err("the SL header GUID starts the file".into()),
            (Some(_), Some(_)) => {
                return Err("the PE file carries the SL header GUID more than once".into());
            }
        }
    } else {
        0
    };
    let sl = &input[offset..];
    if sl.len() < 4 {
        return Err("the input ends inside the SL header".into());
    }
    let entry = usize::from(u16::from_le_bytes([sl[0], sl[1]]));
    let measured = usize::from(u16::from_le_bytes([sl[2], sl[3]]));
    if measured == 0 || entry >= measured {
        return Err(format!(
            "entry {entry:#x} is not inside the measured length {measured:#x}"
        ));
    }
    let block = measured.next_multiple_of(ALIGN);
    let end = block + BLOCK_LEN;
    if end >= SLB_SIZE {
        return Err(format!(
            "the image and its signature end at {end:#x}, leaving nothing of the SLB"
        ));
    }
    if sl.len() < end {
        return Err(format!(
            "the input ends at {:#x}, inside the signature block ending at {end:#x}",
            sl.len()
        ));
    }
    if sl[block + HDR_VERSION..block + HDR_VERSION + 4] != *TAG {
        return Err(format!("no $AS1 signature header at {block:#x}"));
    }
    let parameters = if sl[4..20] == SL_GUID {
        let total = u32::from_le_bytes(sl[TOTAL_LENGTH..TOTAL_LENGTH + 4].try_into().unwrap());
        let total = total as usize;
        if total < end || total + INPUT_PARAMETERS_LEN > SLB_SIZE {
            return Err(format!(
                "TotalLength {total:#x} is not between the signature block's end \
                 {end:#x} and room for the parameters in the SLB"
            ));
        }
        Some(total)
    } else {
        None
    };
    let token = block + HEADER_LEN + SIGNATURE_LEN;
    Ok(Foreign {
        parameters,
        image: sl[..end].to_vec(),
        measured,
        offset,
        key_id: sl[token + TOKEN_KEY_ID..token + TOKEN_KEY_ID + 16]
            .try_into()
            .unwrap(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Key, sign};

    const PEM: &str = include_str!("../../keys/test/sl-test-key.pem");

    /// A signed image with the GUID in its header, `len` bytes measured.
    fn signed(len: usize) -> Vec<u8> {
        let mut image: Vec<u8> = (0..len.next_multiple_of(ALIGN))
            .map(|i| (i * 7 + 3) as u8)
            .collect();
        image[..2].copy_from_slice(&0x60u16.to_le_bytes());
        image[2..4].copy_from_slice(&(len as u16).to_le_bytes());
        image[4..20].copy_from_slice(&SL_GUID);
        let total = len.next_multiple_of(ALIGN) + BLOCK_LEN;
        image[TOTAL_LENGTH..TOTAL_LENGTH + 4].copy_from_slice(&(total as u32).to_le_bytes());
        let block = sign(&image, &Key::from_pem(PEM).unwrap());
        image.extend(block);
        image
    }

    fn pe(sl: &[u8]) -> Vec<u8> {
        let mut file = b"MZ".to_vec();
        file.resize(0x32c0, 0);
        file.extend_from_slice(sl);
        file.extend_from_slice(&[0xcc; 0x800]);
        file
    }

    #[test]
    fn takes_a_bare_image() {
        let image = signed(0x1230);
        let found = extract(&image).unwrap();
        assert_eq!(found.image, image);
        assert_eq!((found.measured, found.offset), (0x1230, 0));
        assert_eq!(found.parameters, Some(image.len()));
        let key = Key::from_pem(PEM).unwrap();
        assert_eq!(found.key_id, crate::key_id(&key.public()));
        assert_eq!(found.token(), &key.token());
        assert_eq!(found.sec_patch_level(), crate::SEC_PATCH_LEVEL);
    }

    #[test]
    fn reads_the_sec_patch_level() {
        let mut image = signed(0x1230);
        image[0x1230 + crate::HDR_SEC_PATCH_LEVEL] = 7;
        assert_eq!(extract(&image).unwrap().sec_patch_level(), 7);
    }

    #[test]
    fn cuts_the_image_out_of_a_pe_file() {
        let image = signed(0x1230);
        let found = extract(&pe(&image)).unwrap();
        assert_eq!(found.image, image);
        assert_eq!(found.offset, 0x32c0);
    }

    #[test]
    fn pads_an_unaligned_measured_length() {
        let image = signed(0x1238);
        let found = extract(&image).unwrap();
        assert_eq!(found.measured, 0x1238);
        assert_eq!(found.image.len(), 0x1240 + BLOCK_LEN);
    }

    #[test]
    fn refuses_what_does_not_fit() {
        let image = signed(0x1230);
        let refused = |input: &[u8]| extract(input).unwrap_err();

        assert!(refused(b"MZ no header").contains("no SL header GUID"));
        let mut twice = pe(&image);
        twice.extend_from_slice(&image);
        assert!(refused(&twice).contains("more than once"));
        assert!(refused(&image[..image.len() - 1]).contains("inside the signature block"));
        let mut no_tag = image.clone();
        no_tag[0x1230 + HDR_VERSION] ^= 1;
        assert!(refused(&no_tag).contains("no $AS1"));
        let mut entry_out = image.clone();
        entry_out[..2].copy_from_slice(&0x1230u16.to_le_bytes());
        assert!(refused(&entry_out).contains("not inside"));
        let mut too_long = image.clone();
        too_long[2..4].copy_from_slice(&0xfa00u16.to_le_bytes());
        assert!(refused(&too_long).contains("leaving nothing"));
        let mut short_total = image.clone();
        short_total[TOTAL_LENGTH] = 0;
        assert!(refused(&short_total).contains("TotalLength"));
    }

    #[test]
    fn a_header_without_the_guid_takes_no_parameters() {
        let mut image = signed(0x1230);
        image[4] ^= 1;
        assert_eq!(extract(&image).unwrap().parameters, None);
    }
}
