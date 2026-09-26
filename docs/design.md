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

## The classic `SKINIT` launch

A run that the checks allow ends in a launch, unless `drtm.conf` says
`launch=off`. It proves the launch mechanics on a platform: drtm prepares and
issues `SKINIT`, and its own Secure Loader confirms from inside the launched
environment that the TPM measured it. There is no DLME. The Secure Loader is
the last code that runs, extends nothing, wakes no AP and halts, so there is
no event log to write either.

### The gate

After the checks' summary, the gate reads back what they reported and blocks
only where a launch is impossible or unsafe: a CPU that is not AMD or
`amd.skinit` not passing, `cpu.machine_check`, `cpu.bsp`, `amd.slb` or
`tpm.present` failing, no 1 GiB pages for the Secure Loader's page tables,
and no UART it could write to. Any other failure is launched through. A TPM
with locality 0 alone fails `tpm.locality` and still launches, so the run
shows on the machine at hand what a launch there measures. A blocked launch
prints one `launch.gate` record and stops.

The Secure Loader writes to the UART the ACPI SPCR table names, or else to
COM1 at `0x3F8`, where UEFI offers a serial device or a 16550 answers there.

### The preamble

`src/launch/` allocates the SLB and a page for the trampoline below 4 GiB,
copies the Secure Loader in and writes the parameter page behind the
measured image. That page carries the output format, level and sinks, the
TPM's family, interface and active banks, the time-stamp counter rate and
whether the PSP DRTM service was ready, as `common::params` lays it out.
Then, after `launch.skinit`:

1. `ExitBootServices()`. From here nothing reaches the UEFI logger, and
    progress goes out as `launch.preamble` records at `debug` through the
    sink list.
1. Locality 0 is relinquished, after the exit, since the TCG2 driver
    measures that event.
1. `CR0.CD` and `CR0.NW` cleared, `CR0.NE` set, interrupts off, as
    `grub_skinit_prepare_cpu()` does.
1. Every AP gets INIT through the all-excluding-self shorthand, by
    `MSR 0x830` or the xAPIC's ICR, and 10 ms to take it. `SKINIT` requires
    them parked (APM Vol 2, section 15.27.8) and does not park them itself.
1. The trampoline, copied into the low page since drtm may load above
    4 GiB, leaves long mode, writes one `S` to the UART and executes
    `SKINIT`.

### The Secure Loader

`sl/` is linked by `sl/sl.ld`. The image opens with AMD's header and keeps
`bootloader_data_offset` at byte 68, where TrenchBoot's layout has it, and
the measured length is padded to a multiple of 4. `.bss`, five pages of
page tables and the stack follow the parameter page inside the 64 KiB.

The 32-bit entry loads an empty IDT, clears `VM_CR.DIS_A20M`, loads its own
GDT through `SS`, zeroes `.bss` and enters long mode with the low 4 GiB
identity-mapped in 1 GiB pages and the SLB mapped at `0xFFFFFFFF80000000`,
the address the Secure Loader is linked at. So it needs no relocations
wherever the SLB lands, and the code it runs is the code the TPM measured.

`sl_main` asks for locality 2 and reads PCR 17 to 22 there, or at locality
0 where 2 is not granted, which is what the PSP DRTM service leaves before
a PSP launch. It judges PCR 17 in every bank the build computed a value for,
SHA-1 and SHA-256, and PCR 18 to 22 against zero, prints its own summary and
`launch.halted`, and halts. The machine needs a power cycle before the next
attempt, since a reset would boot drtm from the same media and launch again.

`build.rs` computes the expected PCR 17 values as `SKINIT` leaves them: the
reset value, zero, extended with the image's digest.
