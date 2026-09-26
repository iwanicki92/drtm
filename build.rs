// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Builds the Secure Loader, flattens it into the image `SKINIT` measures,
//! and computes the PCR 17 values that measurement should leave.
//!
//! Writes `sl.bin` and `sl.rs` into `OUT_DIR` for `src/launch` to embed,
//! and copies `sl.bin` to `target/` for the QEMU tests.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha1::Sha1;
use sha2::{Digest, Sha256};

const SL_TARGET: &str = "x86_64-unknown-none";
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
    for path in ["sl", "common", ".cargo/config.toml", "Cargo.toml"] {
        println!("cargo:rerun-if-changed={path}");
    }

    let elf = fs::read(build(&root, &out)).expect("the SL ELF reads");
    let image = flatten(&elf);
    let header = |at: usize| usize::from(u16::from_le_bytes([image[at], image[at + 1]]));
    let (length, params) = (header(2), header(PARAMS_FIELD));
    let params_size = params_size(&elf, params);

    let sha1 = extend::<Sha1>(&image);
    let sha256 = extend::<Sha256>(&image);
    fs::write(out.join("sl.bin"), &image).unwrap();
    fs::write(
        out.join("sl.rs"),
        format!(
            "/// Bytes `SKINIT` measures.\n\
             pub const MEASURED: usize = {length:#x};\n\
             /// Where the parameter page goes in the SLB.\n\
             pub const PARAMS_OFFSET: usize = {params:#x};\n\
             /// Bytes the SL reserves there.\n\
             pub const PARAMS_SIZE: usize = {params_size:#x};\n\
             /// PCR 17 after the launch, per bank.\n\
             pub const EXPECTED_SHA1: [u8; 20] = {sha1:?};\n\
             pub const EXPECTED_SHA256: [u8; 32] = {sha256:?};\n"
        ),
    )
    .unwrap();

    let target = env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("sl.bin"), &image).unwrap();
}

/// Builds `sl` in a target directory of its own under `OUT_DIR`, so it
/// never waits on the lock the outer build holds, and returns its ELF.
fn build(root: &Path, out: &Path) -> PathBuf {
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
        length % 4 == 0,
        "measured length {length:#x} is not a multiple of 4"
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
        params >= length,
        "the parameter page at {params:#x} overlaps the image"
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
