// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Builds the Secure Loader once per launch path, flattens each build into
//! the image `SKINIT` measures, signs it for the PSP DRTM service, and
//! computes the PCR values its launch should leave.
//!
//! Writes `sl-classic.bin` and `sl-psp.bin`, each the measured image and its
//! signature block, and `sl.rs` into `OUT_DIR` for `src/launch` to embed,
//! and copies both images to `target/` for the QEMU tests. `DRTM_SL_KEY` names a PKCS#8 PEM key to
//! sign with in place of the test key in `keys/test/`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha1::Sha1;
use sha2::{Digest, Sha256};

const SL_TARGET: &str = "x86_64-unknown-none";
const TEST_KEY: &str = "keys/test/sl-test-key.pem";
const SLB_SIZE: usize = 0x10000;

/// The longest measured image the SLB layout allows, which leaves the last
/// 4 KiB for the stack at least.
const MAX_MEASURED: usize = 61440;

/// `bootloader_data_offset`, where TrenchBoot's header keeps it, and the
/// header's length.
const PARAMS_FIELD: usize = 68;
const HEADER_LEN: usize = 70;

const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PT_INTERP: u32 = 3;

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    for path in ["sl", "common", ".cargo/config.toml", "Cargo.toml", TEST_KEY] {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-env-changed=DRTM_SL_KEY");
    let key_path = env::var_os("DRTM_SL_KEY").map_or_else(|| root.join(TEST_KEY), PathBuf::from);
    println!("cargo:rerun-if-changed={}", key_path.display());
    let key = fs::read_to_string(&key_path)
        .map_err(|e| e.to_string())
        .and_then(|pem| sign::Key::from_pem(&pem))
        .unwrap_or_else(|e| panic!("the SL key {} is unusable: {e}", key_path.display()));

    let mut rust = String::new();
    for (feature, name) in [("classic", "CLASSIC"), ("psp", "PSP")] {
        rust += &build_image(&root, &out, &key, feature, name);
    }
    fs::write(out.join("sl.rs"), rust).unwrap();
}

/// Builds, flattens and signs the SL with `feature`, writes it to
/// `sl-{feature}.bin` in `OUT_DIR` and `target/`, and returns the Rust that
/// describes it as the `Image` called `name`.
fn build_image(root: &Path, out: &Path, key: &sign::Key, feature: &str, name: &str) -> String {
    let elf = fs::read(build(root, out, feature)).expect("the SL ELF reads");
    let mut image = flatten(&elf);
    let header = |at: usize| usize::from(u16::from_le_bytes([image[at], image[at + 1]]));
    let (length, params) = (header(2), header(PARAMS_FIELD));
    let params_size = params_size(&elf, params);
    let (sl_start, _) = section(&elf, b".header");
    let (signature_at, signature_size) = section(&elf, b".signature");
    assert!(
        signature_at - sl_start == length && signature_size == sign::BLOCK_LEN,
        "the SL does not reserve the signature block right behind the image"
    );

    let sha1 = extend::<Sha1>(&image);
    let sha256 = extend::<Sha256>(&image);
    let block = sign::sign(&image, key);
    image.extend_from_slice(&block);
    let file = format!("sl-{feature}.bin");
    fs::write(out.join(&file), &image).unwrap();
    let target = env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join(&file), &image).unwrap();

    let bytes = out.join(&file);
    let mut rust = format!(
        "pub const {name}: Image = Image {{
    bytes: include_bytes!({bytes:?}),
    measured: {length:#x},
    params_offset: {params:#x},
    params_size: {params_size:#x},
    expected_sha1: {sha1:?},
    expected_sha256: {sha256:?},
}};
"
    );
    if feature == "psp" {
        let pcr17 = sign::launch_pcr17(sha256.as_slice().try_into().unwrap());
        let pcr18 = sign::launch_pcr18(&key.token());
        rust += &format!(
            "/// SHA-256 PCR 17, all but the last extend, and PCR 18 after the
/// PSP service's `LAUNCH` of the `PSP` image.
pub const PSP_PCR17: [u8; 32] = {pcr17:?};
pub const PSP_PCR18: [u8; 32] = {pcr18:?};
"
        );
    }
    rust
}

/// Builds `sl` in a target directory of its own under `OUT_DIR`, so it
/// never waits on the lock the outer build holds, and returns its ELF.
fn build(root: &Path, out: &Path, feature: &str) -> PathBuf {
    let target_dir = out.join("sl-target");
    let mut cargo = Command::new(env::var("CARGO").unwrap());
    cargo
        .current_dir(root)
        .args([
            "build",
            "--offline",
            "-p",
            "sl",
            "--profile",
            "sl",
            "--target",
            SL_TARGET,
            "--features",
            feature,
        ])
        .arg("--target-dir")
        .arg(&target_dir);
    // What the outer build passes down is for the EFI binary, and the SL
    // takes its flags from `.cargo/config.toml` for its own target.
    for variable in [
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTFLAGS",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_TARGET",
    ] {
        cargo.env_remove(variable);
    }
    let status = cargo.status().expect("cargo runs");
    assert!(status.success(), "building the Secure Loader failed");
    target_dir.join(SL_TARGET).join("sl").join("sl")
}

fn u16_at(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()))
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

fn u64_at(bytes: &[u8], at: usize) -> usize {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()) as usize
}

/// The measured image: every loaded byte from the link address up, padded
/// with zeros to the length the header gives. Fails on anything `SKINIT`
/// or the SLB layout would not take.
fn flatten(elf: &[u8]) -> Vec<u8> {
    assert_eq!(&elf[..4], b"\x7fELF", "the SL is not an ELF file");
    let (phoff, phentsize, phnum) = (u64_at(elf, 0x20), u16_at(elf, 0x36), u16_at(elf, 0x38));
    let segments: Vec<(u32, usize, usize, usize, usize)> = (0..phnum)
        .map(|i| {
            let ph = &elf[phoff + i * phentsize..];
            (
                u32_at(ph, 0),
                u64_at(ph, 8),
                u64_at(ph, 16),
                u64_at(ph, 32),
                u64_at(ph, 40),
            )
        })
        .collect();
    assert!(
        segments
            .iter()
            .all(|s| s.0 != PT_DYNAMIC && s.0 != PT_INTERP),
        "the SL has a dynamic section, and nothing would apply its relocations"
    );
    let loads: Vec<_> = segments.iter().filter(|s| s.0 == PT_LOAD).collect();
    let base = loads
        .iter()
        .map(|s| s.2)
        .min()
        .expect("the SL loads nothing");
    let end = loads.iter().map(|s| s.2 + s.4).max().unwrap();
    assert!(
        end - base <= SLB_SIZE,
        "the SL's memory does not fit in the SLB"
    );

    let mut image = Vec::new();
    for &&(_, offset, vaddr, filesz, _) in loads.iter().filter(|s| s.3 > 0) {
        let at = vaddr - base;
        if image.len() < at + filesz {
            image.resize(at + filesz, 0);
        }
        image[at..at + filesz].copy_from_slice(&elf[offset..offset + filesz]);
    }

    let (entry, length, params) = (
        u16_at(&image, 0),
        u16_at(&image, 2),
        u16_at(&image, PARAMS_FIELD),
    );
    assert!(
        length.is_multiple_of(sign::ALIGN),
        "measured length {length:#x} is not a multiple of {}",
        sign::ALIGN
    );
    assert!(
        length <= MAX_MEASURED,
        "measured length {length:#x} is over {MAX_MEASURED:#x}"
    );
    assert!(
        (HEADER_LEN..length).contains(&entry),
        "entry offset {entry:#x} is not between the header and the measured length {length:#x}"
    );
    assert!(
        image.len() <= length,
        "{:#x} loaded bytes lie past the measured length {length:#x}",
        image.len()
    );
    assert!(
        params >= length + sign::BLOCK_LEN,
        "the parameter page at {params:#x} overlaps the image or its signature"
    );
    image.resize(length, 0);
    image
}

/// The size of the section the parameter page is reserved in, checked to
/// start where the header says and to end inside the SLB. The linker keeps
/// it clear of `.bss`, which the SL's entry zeroes.
fn params_size(elf: &[u8], params: usize) -> usize {
    let (sl_start, _) = section(elf, b".header");
    let (at, size) = section(elf, b".params");
    assert!(size > 0, "the SL reserves no parameter page");
    assert!(
        at - sl_start == params && params + size <= SLB_SIZE,
        "the parameter page at {params:#x} is not where the header says or leaves the SLB"
    );
    size
}

/// The address and size of the section called `name`.
fn section(elf: &[u8], name: &[u8]) -> (usize, usize) {
    let (shoff, shentsize, shnum, shstrndx) = (
        u64_at(elf, 0x28),
        u16_at(elf, 0x3A),
        u16_at(elf, 0x3C),
        u16_at(elf, 0x3E),
    );
    let header = |i: usize| &elf[shoff + i * shentsize..];
    let names = u64_at(header(shstrndx), 0x18);
    (0..shnum)
        .map(header)
        .find(|sh| {
            let at = names + u32_at(sh, 0) as usize;
            elf[at..].starts_with(name) && elf[at + name.len()] == 0
        })
        .map(|sh| (u64_at(sh, 0x10), u64_at(sh, 0x20)))
        .unwrap_or_else(|| panic!("the SL has no {} section", String::from_utf8_lossy(name)))
}

/// What PCR 17 holds after `SKINIT` in the bank `D` hashes: its reset
/// value, zero, extended with the image's digest.
fn extend<D: Digest>(image: &[u8]) -> Vec<u8> {
    let digest = D::digest(image);
    let mut pcr = D::new();
    pcr.update(vec![0; digest.len()]);
    pcr.update(digest);
    pcr.finalize().to_vec()
}
