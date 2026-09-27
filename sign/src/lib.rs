// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Signs the Secure Loader for the PSP DRTM service, and predicts what its
//! `LAUNCH` extends, for `build.rs`.
//!
//! Behind the measured image, padded to 16 bytes, comes the block AMD's
//! signing tool produces (AMD publication 58453, appendices B.1, B.2 and
//! G.1): a `$AS1` `FW_SIG_HEADER`, an RSA-4096 PSS signature over the image
//! and that header, and the key token. The signature and token are
//! little-endian, as the service reads them.

pub mod foreign;

use rand_core::{CryptoRng, RngCore};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::pss::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier};
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256, Sha384};

pub const HEADER_LEN: usize = 0x100;
pub const SIGNATURE_LEN: usize = 0x200;
pub const TOKEN_LEN: usize = 0x440;
/// Everything behind the measured image.
pub const BLOCK_LEN: usize = HEADER_LEN + SIGNATURE_LEN + TOKEN_LEN;

/// The measured image is padded to this.
pub const ALIGN: usize = 16;

/// `SecPatchLevel`, which `LAUNCH` extends into PCR 18.
pub const SEC_PATCH_LEVEL: u32 = 0;

const TAG: &[u8; 4] = b"$AS1";
const KEY_BITS: usize = 8 * SIGNATURE_LEN;

// FW_SIG_HEADER fields (appendix B.1).
const HDR_VERSION: usize = 0x10;
const HDR_SIZE_FW_SIGNED: usize = 0x14;
const HDR_SIG_OPTION: usize = 0x30;
const HDR_SIG_PARAMETERS: usize = 0x38;
pub(crate) const HDR_SEC_PATCH_LEVEL: usize = 0x4C;
const HDR_SIZE_IMAGE: usize = 0x6C;

// KEY_IMAGE_4096 fields (appendix B.2). Version 2 and the exponent's size
// as 4096 bits are what the service accepted from SKL's signing tool.
const TOKEN_VERSION: u32 = 2;
const TOKEN_KEY_ID: usize = 4;
const TOKEN_CERT_KEY_ID: usize = 20;
const TOKEN_EXP_BITS: usize = 56;
const TOKEN_MOD_BITS: usize = 60;
const TOKEN_EXPONENT: usize = 64;
const TOKEN_MODULUS: usize = 64 + SIGNATURE_LEN;

/// A 4096-bit RSA key.
pub struct Key(RsaPrivateKey);

impl Key {
    /// A PKCS#8 PEM private key.
    pub fn from_pem(pem: &str) -> Result<Self, String> {
        let key = RsaPrivateKey::from_pkcs8_pem(pem).map_err(|e| e.to_string())?;
        if key.size() * 8 != KEY_BITS {
            return Err(format!("the key has {} bits, not 4096", key.size() * 8));
        }
        Ok(Self(key))
    }

    pub fn public(&self) -> RsaPublicKey {
        self.0.to_public_key()
    }

    /// The token: its header, then the exponent and modulus little-endian.
    pub fn token(&self) -> [u8; TOKEN_LEN] {
        token(&self.public())
    }
}

fn le(value: &rsa::BigUint) -> Vec<u8> {
    let mut bytes = value.to_bytes_le();
    bytes.resize(SIGNATURE_LEN, 0);
    bytes
}

/// The key ID: the first 16 bytes of the SHA-256 of the token's modulus.
pub fn key_id(public: &RsaPublicKey) -> [u8; 16] {
    Sha256::digest(le(public.n()))[..16].try_into().unwrap()
}

pub fn token(public: &RsaPublicKey) -> [u8; TOKEN_LEN] {
    let mut token = [0; TOKEN_LEN];
    let id = key_id(public);
    token[..4].copy_from_slice(&TOKEN_VERSION.to_le_bytes());
    token[TOKEN_KEY_ID..TOKEN_KEY_ID + 16].copy_from_slice(&id);
    token[TOKEN_CERT_KEY_ID..TOKEN_CERT_KEY_ID + 16].copy_from_slice(&id);
    token[TOKEN_EXP_BITS..TOKEN_EXP_BITS + 4].copy_from_slice(&(KEY_BITS as u32).to_le_bytes());
    token[TOKEN_MOD_BITS..TOKEN_MOD_BITS + 4].copy_from_slice(&(KEY_BITS as u32).to_le_bytes());
    token[TOKEN_EXPONENT..TOKEN_MODULUS].copy_from_slice(&le(public.e()));
    token[TOKEN_MODULUS..].copy_from_slice(&le(public.n()));
    token
}

/// The signature header for a measured image `padded_len` bytes long.
fn header(padded_len: usize, id: &[u8; 16]) -> [u8; HEADER_LEN] {
    let signed = (padded_len + HEADER_LEN) as u32;
    let mut header = [0; HEADER_LEN];
    header[HDR_VERSION..HDR_VERSION + 4].copy_from_slice(TAG);
    header[HDR_SIZE_FW_SIGNED..HDR_SIZE_FW_SIGNED + 4].copy_from_slice(&signed.to_le_bytes());
    header[HDR_SIG_OPTION..HDR_SIG_OPTION + 4].copy_from_slice(&1u32.to_le_bytes());
    header[HDR_SIG_PARAMETERS..HDR_SIG_PARAMETERS + 16].copy_from_slice(id);
    header[HDR_SEC_PATCH_LEVEL..HDR_SEC_PATCH_LEVEL + 4]
        .copy_from_slice(&SEC_PATCH_LEVEL.to_le_bytes());
    header[HDR_SIZE_IMAGE..HDR_SIZE_IMAGE + 4]
        .copy_from_slice(&(signed + SIGNATURE_LEN as u32).to_le_bytes());
    header
}

/// The salt and blinding come from the image, so a build is reproducible.
/// Any source serves: the signature's strength does not rest on them.
struct ImageRng {
    seed: [u8; 48],
    counter: u64,
}

impl RngCore for ImageRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(48) {
            let block = Sha384::new()
                .chain_update(self.seed)
                .chain_update(self.counter.to_le_bytes())
                .finalize();
            self.counter += 1;
            chunk.copy_from_slice(&block[..chunk.len()]);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for ImageRng {}

/// The block behind `measured`, which must already be padded to [`ALIGN`].
pub fn sign(measured: &[u8], key: &Key) -> Vec<u8> {
    assert!(
        measured.len().is_multiple_of(ALIGN),
        "the measured image is {:#x} bytes, not a multiple of {ALIGN}",
        measured.len()
    );
    let token = key.token();
    let header = header(measured.len(), &key_id(&key.public()));
    let mut signed = measured.to_vec();
    signed.extend_from_slice(&header);
    let mut rng = ImageRng {
        seed: Sha384::digest(&signed).into(),
        counter: 0,
    };
    let signature = SigningKey::<Sha384>::new(key.0.clone()).sign_with_rng(&mut rng, &signed);
    let mut block = header.to_vec();
    block.extend(signature.to_bytes().iter().rev());
    block.extend_from_slice(&token);
    assert_eq!(block.len(), BLOCK_LEN);
    block
}

/// Checks a signed image the way the service does: the header right
/// behind the padded measured length, the signature at `SizeFWSigned`
/// over everything before it, against `public`.
pub fn verify(image: &[u8], public: &RsaPublicKey) -> Result<(), String> {
    let word = |at: usize| u32::from_le_bytes(image[at..at + 4].try_into().unwrap()) as usize;
    let measured = usize::from(u16::from_le_bytes([image[2], image[3]]));
    let at = measured.next_multiple_of(ALIGN);
    if image.get(at + HDR_VERSION..at + HDR_VERSION + 4) != Some(TAG.as_slice()) {
        return Err(format!("no {} header at {at:#x}", "$AS1"));
    }
    let signed = word(at + HDR_SIZE_FW_SIGNED);
    if signed != at + HEADER_LEN || image.len() < signed + SIGNATURE_LEN + TOKEN_LEN {
        return Err(format!("SizeFWSigned {signed:#x} does not fit"));
    }
    let signature: Vec<u8> = image[signed..signed + SIGNATURE_LEN]
        .iter()
        .rev()
        .copied()
        .collect();
    if image[signed + SIGNATURE_LEN..signed + SIGNATURE_LEN + TOKEN_LEN] != token(public) {
        return Err("the token is not this key's".into());
    }
    let signature = Signature::try_from(signature.as_slice()).map_err(|e| e.to_string())?;
    VerifyingKey::<Sha384>::new(public.clone())
        .verify(&image[..signed], &signature)
        .map_err(|e| e.to_string())
}

pub fn public_from_pem(pem: &str) -> Result<RsaPublicKey, String> {
    RsaPublicKey::from_public_key_pem(pem).map_err(|e| e.to_string())
}

fn extend(pcr: &[u8], measurement: &[u8]) -> [u8; 32] {
    Sha256::new()
        .chain_update(pcr)
        .chain_update(measurement)
        .finalize()
        .into()
}

/// PCR 18 in SHA-256 after `LAUNCH`: zero extended with the token's digest,
/// then with the digest of the header's `SecPatchLevel` as a 32-bit word.
pub fn launch_pcr18(token: &[u8; TOKEN_LEN], sec_patch_level: u32) -> [u8; 32] {
    let pcr = extend(&[0; 32], &Sha256::digest(token));
    extend(&pcr, &Sha256::digest(sec_patch_level.to_le_bytes()))
}

/// PCR 17 in SHA-256 after `LAUNCH` but its last extend: the `SKINIT` value
/// extended with the digest of an SPLT version of 0, as a 32-bit word.
pub fn launch_pcr17(skinit: &[u8; 32]) -> [u8; 32] {
    extend(skinit, &Sha256::digest(0u32.to_le_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PEM: &str = include_str!("../../keys/test/sl-test-key.pem");
    const PUB: &str = include_str!("../../keys/test/sl-test-key.pub");

    /// A measured image the way `build.rs` makes one: the header's length
    /// field at offset 2, padded to 16.
    fn measured(len: usize) -> Vec<u8> {
        let mut image: Vec<u8> = (0..len).map(|i| (i * 13 + 5) as u8).collect();
        image[2..4].copy_from_slice(&(len as u16).to_le_bytes());
        image
    }

    #[test]
    fn signs_what_the_service_checks() {
        let key = Key::from_pem(PEM).unwrap();
        let public = public_from_pem(PUB).unwrap();
        let mut image = measured(0x1230);
        let block = sign(&image, &key);
        assert_eq!(&block[HDR_VERSION..HDR_VERSION + 4], b"$AS1");
        image.extend_from_slice(&block);
        verify(&image, &public).unwrap();
        // The same bytes every build.
        assert_eq!(sign(&measured(0x1230), &key), block);

        // The token names the key the header does.
        let token = &image[0x1230 + HEADER_LEN + SIGNATURE_LEN..];
        assert_eq!(&token[TOKEN_KEY_ID..TOKEN_KEY_ID + 16], &key_id(&public));
        assert_eq!(
            &block[HDR_SIG_PARAMETERS..HDR_SIG_PARAMETERS + 16],
            &key_id(&public)
        );
        assert_eq!(&token[TOKEN_EXPONENT..TOKEN_EXPONENT + 4], &[1, 0, 1, 0]);
    }

    #[test]
    fn a_changed_byte_fails_the_check() {
        let key = Key::from_pem(PEM).unwrap();
        let public = key.public();
        let mut image = measured(0x800);
        image.extend(sign(&image, &key));
        for at in [0x100, 0x800 + 0x60, 0x800 + HEADER_LEN + 7] {
            let mut bad = image.clone();
            bad[at] ^= 1;
            assert!(verify(&bad, &public).is_err(), "byte {at:#x}");
        }
        let mut bad = image.clone();
        bad[0x800 + HDR_SIZE_FW_SIGNED] ^= 0x10;
        assert!(verify(&bad, &public).is_err());
    }

    #[test]
    #[should_panic(expected = "not a multiple of 16")]
    fn refuses_an_unpadded_image() {
        sign(&measured(0x804), &Key::from_pem(PEM).unwrap());
    }

    #[test]
    fn predicts_the_launch() {
        // What the service extends, recomputed step by step.
        let token = Key::from_pem(PEM).unwrap().token();
        let zero = [0u8; 32];
        let mut pcr = Sha256::new();
        pcr.update(zero);
        pcr.update(Sha256::digest(token));
        let pcr: [u8; 32] = pcr.finalize().into();
        let mut next = Sha256::new();
        next.update(pcr);
        next.update(Sha256::digest([0u8; 4]));
        assert_eq!(
            launch_pcr18(&token, SEC_PATCH_LEVEL),
            <[u8; 32]>::from(next.finalize())
        );
        assert_ne!(launch_pcr17(&zero), zero);
    }
}
