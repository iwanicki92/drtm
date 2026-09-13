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
- On real hardware, the tool runs unattended/scripted: something external
  (a test rig) captures and parses the serial output rather than a human
  watching it interactively. Output should therefore carry both a
  human-readable form and a machine-parseable form (e.g. plain log lines for
  people, plus a structured — JSON or similar — summary/result line for the
  rig). Exact format is still open.

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
- Robot Framework is the test runner, driving QEMU over its serial console
  (exposed via `-serial telnet:...`, as Dasharo does) and controlling the VM
  over its QMP socket for things like reset. Reference implementation:
  Dasharo/open-source-firmware-validation — see `scripts/ci/qemu-run.sh` for
  the QEMU invocation, `lib/QemuMonitor.py` for the QMP-over-Robot-Framework
  library pattern, and `trenchboot/*.robot` for how they structure DRTM-aware
  test suites (their tests run against a fully booted Linux and read PCRs
  from `/sys/class/tpm/tpm0/pcr-*` and the event log from
  `/sys/kernel/security/slaunch/eventlog` — not applicable verbatim to us
  since we don't boot a full OS, but useful as a model).
- A separate future project may add simple DRTM emulation to QEMU itself;
  that is out of scope here.
