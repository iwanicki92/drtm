<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: Apache-2.0
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

**The DRTM launch itself is on hold.** Do not design or write launch logic
(building the Secure Loader Block, parking the APs, executing `SKINIT`, or
the equivalent for another vendor) until the user asks for it. The reference
material for it has been provided, so what it waits on is an agreed design
and hardware to validate it on, not missing documentation.

Committed files cite upstream sources only: APM section numbers, or paths
into EDK2/QEMU/Linux. Never the private notes repo the reference material
arrived in, which a reader of this repo does not have.

Precondition checks are in scope and land one at a time. The line between
the two: a check only observes the platform and leaves it as it found it.
Anything that changes machine state to make a launch happen is the launch.

For the "why" behind the project and its testing setup, see `docs/design.md`
and `docs/testing.md`. This file only covers agent-operational specifics.

## Commands

Requires `cargo`/`rustc` (via rustup, with the `x86_64-unknown-uefi` target
added) on `PATH`. If this machine's Rust toolchain lives somewhere other than
the default `~/.rustup`/`~/.cargo`, set `RUSTUP_HOME`/`CARGO_HOME`
accordingly first. Check for a machine-specific note before assuming
defaults.

- Build the EFI binary: `cargo build` (target is `x86_64-unknown-uefi`, set
    via `.cargo/config.toml`, output at
    `target/x86_64-unknown-uefi/debug/drtm.efi`).
- Run the QEMU tests: `uv run --project qemu-tests pytest qemu-tests`
    (builds the binary itself first, see `docs/testing.md`).
- Lint everything: `pre-commit run --all-files` (config in
    `.pre-commit-config.yaml`, needs `pre-commit install` once, plus
    `rustup component add rustfmt clippy`).
- `task build`/`task lint`/`task test` (Taskfile.yml) wrap the three commands
    above.
