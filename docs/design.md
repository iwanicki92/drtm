# Design

## What this is

A single-binary tool for testing DRTM (Dynamic Root of Trust for Measurement)
and gathering logs to help debug TrenchBoot issues.

TrenchBoot splits DRTM support across multiple components (GRUB and
Linux/Xen, or GRUB, SKL, and Linux/Xen). This tool is deliberately simpler:
one binary whose only job is to exercise the DRTM launch on real hardware and
report whether it worked, with enough logging to be useful when it didn't.

## Target scope

- Platforms: UEFI first; non-UEFI (legacy BIOS boot path) planned for later.
- Architectures: x86 first, ARM planned for later.
- x86 has two distinct DRTM paths: Intel (TXT) and AMD (SKINIT/SVM).
- First milestone targets AMD only. Intel, ARM, and non-UEFI support come
    after that.

## Architecture principle

Keep DRTM-path-specific code (Intel vs. AMD vs. ARM, UEFI vs. non-UEFI)
cleanly separated from generic, non-DRTM code (UEFI app scaffolding, logging,
CLI/output). Each DRTM path should be easy to read in isolation — a reader
looking at "what does the AMD path do" shouldn't have to wade through Intel
logic or generic plumbing to find it.

DRTM implementation work (the actual AMD SKINIT launch path or otherwise) is
currently on hold pending further design input; see `CLAUDE.md`.

## DRTM launch design decisions (for when that work starts)

These were agreed ahead of time so the surrounding scaffolding (logging,
reporting) can be built without re-litigating them later:

- The tool embeds its own minimal Secure-Loader-like stub rather than driving
  an externally supplied SL/DLME binary (e.g. TrenchBoot's own). It stays a
  single, self-contained binary; it only needs to prove the DRTM launch
  mechanics work, not host a real next-stage payload.
- SKINIT (and equivalent) tears down normal UEFI execution, so the embedded
    stub cannot rely on UEFI console/boot services to report results. It
    writes results directly to the serial UART before halting or resetting.
- On real hardware, the tool is most often run by hand. Output should still
    carry both a human-readable form and a machine-parseable form (e.g. plain
    log lines for people, plus a structured — JSON or similar —
    summary/result line a test rig could parse), so it stays useful if it
    does get scripted later. Prefer a single binary with an argument
    selecting the output mode; fall back to two separate binaries only if
    that turns out impractical. Exact format is still open.
