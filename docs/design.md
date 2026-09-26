<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Design

## What this is

A single-binary tool for testing DRTM (Dynamic Root of Trust for Measurement)
and gathering logs to help debug TrenchBoot issues.

TrenchBoot splits DRTM support across multiple components (GRUB and
Linux/Xen, or GRUB, SKL, and Linux/Xen). This tool is deliberately simpler:
one binary whose only job is to exercise the DRTM launch on real hardware and
report whether it worked, with enough logging to be useful when it didn't.

## Target scope

- Platforms: UEFI first, non-UEFI (legacy BIOS boot path) planned for later.
- Architectures: x86 first, ARM planned for later.
- x86 has two distinct DRTM paths: Intel (TXT) and AMD (SKINIT/SVM).
- First milestone targets AMD only. Intel, ARM, and non-UEFI support come
    after that.

## Architecture principle

Keep DRTM-path-specific code (Intel vs. AMD vs. ARM, UEFI vs. non-UEFI)
cleanly separated from generic, non-DRTM code (UEFI app scaffolding, logging,
CLI/output). Each DRTM path should be easy to read in isolation: a reader
looking at "what does the AMD path do" shouldn't have to wade through Intel
logic or generic plumbing to find it.

## DRTM launch design decisions

These were agreed before the launch was built, so the surrounding
scaffolding (logging, reporting) could be built without re-litigating them:

- The tool embeds its own minimal Secure-Loader-like stub rather than driving
  an externally supplied SL/DLME binary (e.g. TrenchBoot's own). It stays a
  single, self-contained binary. It only needs to prove the DRTM launch
  mechanics work, not host a real next-stage payload.
- SKINIT (and equivalent) tears down normal UEFI execution, so the embedded
    stub cannot rely on UEFI console/boot services to report results. It
    writes results directly to the serial UART before halting or resetting.
- On real hardware, the tool is most often run by hand, over serial, with
    no second console. Output should carry a human-readable form and a
    machine-parseable form, but as two exclusive modes of one binary rather
    than both at once, since they share the single stream the operator is
    reading. The mode is chosen at runtime from `drtm.conf` beside the
    binary on the ESP, because the removable-media boot path this tool uses
    passes it no arguments. That file carries `format=`, which picks the
    mode, and `level=`, which sets how much is printed. The
    machine-parseable mode is one JSON object per line, each behind the same
    `drtm:` prefix, so one line lost to noise costs one record.
