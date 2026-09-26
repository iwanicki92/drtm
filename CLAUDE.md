<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Comments and docs

Two or three lines, maximum. State what is true now, not how it was found
out, what was tried, or a spec quoted verbatim. Anything longer belongs in
`docs/`, not inline.

`docs/` carries the longer explanation, but it is edited in place, not
grown: rework the paragraph that is now wrong instead of appending a new
one. A page that gains a section per change stops being read.

Every `unsafe` block carries its own `// Safety:` comment immediately above,
saying what makes it sound. Inside an `unsafe fn` that reason is usually the
function's own contract, so say so rather than leaving the block bare.
`clippy::undocumented_unsafe_blocks` enforces this, down to one comment not
covering the block after it.

## Project status

A `no_std`/`no_main` UEFI binary, exercised by booting it under QEMU. Which
checks it currently performs is whatever `src/main.rs` reports and
`qemu-tests/` asserts on, so read those rather than a summary here.

The AMD launch, classic `SKINIT` and through the PSP DRTM service, is
built to agreed designs: the preamble in `drtm`, the Secure Loader in `sl`,
what both share in `common`, and the SL's signer in `sign`. Intel TXT and
ARM stay out of scope until the user asks for them.

Committed files cite upstream sources only: APM section numbers, or paths
into EDK2/QEMU/Linux. Never the private notes repo the reference material
arrived in, which a reader of this repo does not have.

The line between a check and the launch: a check only observes the platform
and leaves it as it found it. Anything that changes machine state to make a
launch happen is the launch, and runs only after the launch gate.

For the "why" behind the project and its testing setup, see `docs/design.md`
and `docs/testing.md`. This file only covers agent-operational specifics.

## Commands

Requires `cargo`/`rustc` (via rustup, with the `x86_64-unknown-uefi` and
`x86_64-unknown-none` targets added) on `PATH`. If this machine's Rust
toolchain lives somewhere other than the default `~/.rustup`/`~/.cargo`, set
`RUSTUP_HOME`/`CARGO_HOME` accordingly first. Check for a machine-specific
note before assuming defaults.

- Build the EFI binary: `cargo build` (target is `x86_64-unknown-uefi`, set
    via `.cargo/config.toml`, output at
    `target/x86_64-unknown-uefi/debug/drtm.efi`).
- Run the QEMU tests: `uv run --project qemu-tests pytest qemu-tests`
    (builds the binary itself first, see `docs/testing.md`).
- Lint everything: `pre-commit run --all-files` (config in
    `.pre-commit-config.yaml`, needs `pre-commit install` once, plus
    `rustup component add rustfmt clippy`). Clippy by hand is plain
    `cargo clippy -- -D warnings`: `--all-targets` fails on a duplicate
    `panic_impl`, since the test target links `std` against the `uefi`
    crate's panic handler. `sl` and `common` have hooks of their own, for
    their own targets, in the same config.
- Run the host unit tests: `cargo test-host`, an alias for `cargo test`
    on the host target of the crates that can run there (`common`, `sign`).
- `task build`/`task lint`/`task test`/`task test-host` (Taskfile.yml) wrap
    the commands above.
