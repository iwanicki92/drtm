# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project status

Base scaffolding only: a `no_std`/`no_main` UEFI binary that logs a line and
returns, plus a QEMU smoke test proving it boots. No DRTM logic yet.

## Commands

The Rust toolchain and Python venv are not on `/`, since this machine is a VM
with a small rootfs (see the machine-wide instructions) — put them on PATH
before running anything:

```sh
export RUSTUP_HOME=/home/iwans/workspace/rust/.rustup
export CARGO_HOME=/home/iwans/workspace/rust/.cargo
export PATH="$CARGO_HOME/bin:$PATH"
```

- Build the EFI binary: `cargo build` (target is `x86_64-unknown-uefi`, set
  via `.cargo/config.toml`; output at
  `target/x86_64-unknown-uefi/debug/drtm.efi`).
- Run the QEMU smoke test: `.venv-vm/bin/pytest qemu-tests/` (builds the
  binary itself first). `.venv-vm` is a Python venv local to this repo, kept
  out of `.venv` on purpose — see its `.gitignore` comment.
- First run of the test suite downloads Dasharo's QEMU Q35 firmware image
  into `qemu-tests/dl-cache/` (a few MB, gitignored, cached after that).

## What this is

A single-binary tool for testing DRTM (Dynamic Root of Trust for Measurement)
and gathering logs to help debug TrenchBoot issues.

TrenchBoot splits DRTM support across multiple components (GRUB and
Linux/Xen, or GRUB, SKL, and Linux/Xen). This tool is deliberately simpler: one
binary whose only job is to exercise the DRTM launch on real hardware and
report whether it worked, with enough logging to be useful when it didn't.

## Target scope

- Platforms: UEFI first; non-UEFI (legacy BIOS boot path) planned for later.
- Architectures: x86 first, ARM planned for later.
- x86 has two distinct DRTM paths: Intel (TXT) and AMD (SKINIT/SVM).
- **First milestone targets AMD only.** Intel, ARM, and non-UEFI support come
  after that.

## Architecture principle

Keep DRTM-path-specific code (Intel vs. AMD vs. ARM, UEFI vs. non-UEFI)
cleanly separated from generic, non-DRTM code (UEFI app scaffolding, logging,
CLI/output). Each DRTM path should be easy to read in isolation — a reader
looking at "what does the AMD path do" shouldn't have to wade through Intel
logic or generic plumbing to find it.

**DRTM implementation work is on hold.** Do not design or write the actual
DRTM launch logic (AMD SKINIT path or otherwise) until the user provides
further documents describing it. Current work is limited to non-DRTM
scaffolding: the base UEFI binary and the QEMU test harness.

## DRTM launch design decisions (for when that work starts)

These were agreed ahead of time so the surrounding scaffolding (logging,
reporting) can be built without re-litigating them later:

- The tool embeds its own minimal Secure-Loader-like stub rather than driving
  an externally supplied SL/DLME binary (e.g. TrenchBoot's own). It stays a
  single, self-contained binary; it only needs to prove the DRTM launch
  mechanics work, not host a real next-stage payload.
- SKINIT (and equivalent) tears down normal UEFI execution, so the embedded
  stub cannot rely on UEFI console/boot services to report results. It writes
  results directly to the serial UART before halting or resetting.
- On real hardware, the tool is most often run by hand. Output should still
  carry both a human-readable form and a machine-parseable form (e.g. plain
  log lines for people, plus a structured — JSON or similar — summary/result
  line a test rig could parse), so it stays useful if it does get scripted
  later. Prefer a single binary with an argument selecting the output mode;
  fall back to two separate binaries only if that turns out impractical.
  Exact format is still open.

## Testing strategy

A QEMU-based test suite exists from the start, even before real DRTM launches
are supported, to prove the UEFI application itself boots and runs correctly:

- QEMU does not support real DRTM launches. Real DRTM-path code either needs
  to be stubbed out for QEMU builds (a no-DRTM test build variant) or is
  expected to fault/crash under QEMU — which of these applies depends on how
  the DRTM instructions actually behave when executed in QEMU, and should be
  verified rather than assumed. Dasharo's own TrenchBoot QEMU tests suggest
  the failure is graceful and happens before the DRTM instruction executes
  (GRUB reports "secure launch not enabled" rather than the VM crashing), but
  this should be confirmed for our own launch path rather than assumed.
- Firmware for QEMU testing is Dasharo's coreboot+UEFI build for QEMU Q35
  (from the `Dasharo/coreboot` releases, `qemu_q35_all_menus.rom`), not stock
  OVMF. It is a more feature-complete, open-source UEFI implementation
  (already used by Dasharo to test TrenchBoot itself), which leaves room to
  grow into TPM/DRTM-adjacent QEMU testing later without switching firmware.
- Tests are plain `pytest`, not Robot Framework: RF's test-case syntax is
  `.robot` files, which is more ceremony than this project wants. QEMU is
  driven over its serial console, exposed as a raw TCP socket
  (`-serial tcp:...`, not `telnet:...`, to avoid depending on Python's
  removed/deprecated `telnetlib`), plus its QMP socket for VM control (reset,
  quit). See `qemu-tests/qemu_vm.py`.
- `qemu-tests/qmp_client.py` is adapted from
  Dasharo/open-source-firmware-validation's `lib/QemuMonitor.py` (Apache-2.0;
  see its SPDX header and the repo's `NOTICE` file) with the Robot-Framework
  specific bits stripped. Dasharo's `trenchboot/*.robot` tests are a useful
  model for DRTM-aware test structure later (they run against a fully booted
  Linux and read PCRs from `/sys/class/tpm/tpm0/pcr-*` and the event log from
  `/sys/kernel/security/slaunch/eventlog`), though not applicable verbatim
  since we don't boot a full OS.
- A separate future project may add simple DRTM emulation to QEMU itself;
  that is out of scope here.
