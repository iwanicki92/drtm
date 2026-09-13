# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project status

No code yet — only `README.md` and this file. There is no build system
(`Cargo.toml`, etc.) in place, so there are no build/lint/test commands to
document until that scaffolding exists.

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

## Testing strategy

A QEMU-based test suite exists from the start, even before real DRTM launches
are supported, to prove the UEFI application itself boots and runs correctly:

- QEMU does not support real DRTM launches. Real DRTM-path code either needs
  to be stubbed out for QEMU builds (a no-DRTM test build variant) or is
  expected to fault/crash under QEMU — which of these applies depends on how
  the DRTM instructions actually behave when executed in QEMU, and should be
  verified rather than assumed.
- Robot Framework is the intended test runner, driving QEMU over its serial
  console. Look at Dasharo/open-source-firmware-validation for a reference
  implementation of connecting Robot Framework tests to QEMU's serial port.
- A separate future project may add simple DRTM emulation to QEMU itself;
  that is out of scope here.
