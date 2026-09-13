<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: Apache-2.0
-->

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Comments

Two or three lines, maximum. State what is true now, not how it was found
out, what was tried, or a spec quoted verbatim. Anything longer belongs in
`docs/`, not inline.

## Project status

Base scaffolding only: a `no_std`/`no_main` UEFI binary that logs a line and
returns, plus a QEMU smoke test proving it boots. No DRTM logic yet.

**DRTM implementation work is on hold.** Do not design or write the actual
DRTM launch logic (AMD SKINIT path or otherwise) until the user provides
further documents describing it. Current work is limited to non-DRTM
scaffolding.

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
- Run the QEMU smoke test: `uv run --project qemu-tests pytest qemu-tests`
    (builds the binary itself first, see `docs/testing.md`).
- Lint everything: `pre-commit run --all-files` (config in
    `.pre-commit-config.yaml`, needs `pre-commit install` once, plus
    `rustup component add rustfmt clippy`).
- `task build`/`task lint`/`task test` (Taskfile.yml) wrap the three commands
    above.
