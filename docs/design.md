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

## The launch

A run that the checks allow ends in a launch, unless `drtm.conf` says
`launch=off`. It proves the launch mechanics on a platform: drtm prepares and
issues `SKINIT`, and its own Secure Loader confirms from inside the launched
environment that the TPM measured it. There is no DLME. The Secure Loader is
the last code that runs, wakes no AP and halts.

There are two paths. The classic one is `SKINIT` alone. The PSP one, which
Turin firmware offers, adds AMD's PSP DRTM service (publication 58453): the
preamble has it protect memory with a TMR before `SKINIT`, and the Secure
Loader has it check the image's signature, extend the DRTM PCRs and tear the
launch down again, issuing every command the guide lists.

### Choosing the path

`launch=auto`, the default, takes the PSP path when all of these hold, and
otherwise reports the first that does not as `launch.path` and launches
classically. `launch=psp` makes that failure a `launch.gate` blocker, and
`launch=skinit` never tries.

1. The TPM is 2.0, since the service extends through TPM 2.0 commands.
1. A mailbox address is known: `C2PMSG_72` from the ASPT, or else from the
    PSP's base address read through SMN, as GRUB does.
1. That address is below 4 GiB, where the Secure Loader's page tables reach.
1. The service's ready bit is set.
1. After the zero write the service needs first, `GET_CAPABILITY` reports
    DRTM enabled.

The last one changes the service's state, so it runs after the checks'
summary, while UEFI output still works. The TMR waits for the preamble,
after `ExitBootServices()`, as in GRUB. Under it devices cannot reach the
low 4 GiB, and a firmware driver stopping its device in
`ExitBootServices()` can then hang, as Turin firmware did. A `TMR_SETUP`
that fails there is reported and the launch goes on, so the Secure Loader
reports how `LAUNCH` answers.

### The gate

After the checks' summary, the gate reads back what they reported and blocks
only where a launch is impossible or unsafe: a CPU that is not AMD or
`amd.skinit` not passing, `cpu.machine_check`, `cpu.bsp`, `amd.slb` or
`tpm.present` failing, no 1 GiB pages for the Secure Loader's page tables,
and no UART it could write to. Any other failure is launched through. A TPM
with locality 0 alone fails `tpm.locality` and still launches, so the run
shows on the machine at hand what a launch there measures. A blocked launch
prints one `launch.gate` record and stops. With `launch=psp` it also blocks
where the PSP path is not available.

The Secure Loader writes to the UART the ACPI SPCR table names, or else to
COM1 at `0x3F8`, where UEFI offers a serial device or a 16550 answers there.

### The preamble

`src/launch/` allocates the SLB and a page for the trampoline below 4 GiB,
copies the Secure Loader in and writes the parameter page behind it. The
SLB sits on a 2 MiB boundary, which the Secure Loader's page tables need,
and on the PSP path on a 64 MiB one if one is free, since the guide's
appendix E.1 asks that of an extended range. The page carries the output
format, level and sinks, the TPM's family, interface and active banks, the
time-stamp counter rate and whether the PSP DRTM service was ready, as
`common::params` lays it out. On the PSP path it also carries the mailbox,
the capability words, the TMR, the IO-APIC pin the PIT reaches, the build's
PCR predictions and the SHA-256 of the IVRS the checks read. Then, after
`launch.skinit` (and `launch.psp_service` on the PSP path):

1. `ExitBootServices()`. From here nothing reaches the UEFI logger, and
    progress goes out as `launch.preamble` records at `debug` through the
    sink list.
1. On the PSP path, `TMR_SETUP` of index 0 over `[0, 4 GiB)`, reported as
    `launch.psp_tmr_setup`.
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
the measured length is padded to a multiple of 16. The signature block
follows it, then the parameter page, then `.bss`, a 6 KiB stack and three
pages of page tables at the end of the 64 KiB. One image serves both paths
and fits with little room to spare. Should it outgrow the SLB, it can be
split in two, one image per path.

The 32-bit entry loads an empty IDT, clears `VM_CR.DIS_A20M`, loads its own
GDT through `SS`, zeroes `.bss` and enters long mode with the low 4 GiB
identity-mapped in 1 GiB pages and the SLB's 2 MiB page mapped at
`0xFFFFFFFF80000000`, the address the Secure Loader is linked at. So it
needs no relocations wherever the SLB lands, and the code it runs is the
code the TPM measured. The GDT's accessed bits are preset, since loading a
segment would otherwise set them and change the signed bytes before the
service checks them.

On the classic path `sl_main` asks for locality 2 and reads PCR 17 to 22
there, or at locality 0 where 2 is not granted, which is what the PSP DRTM
service leaves before a PSP launch. It judges PCR 17 in every bank the build
computed a value for, SHA-1 and SHA-256, and PCR 18 to 22 against zero,
prints its own summary and `launch.halted`, and halts. The machine needs a
power cycle before the next attempt, since a reset would boot drtm from the
same media and launch again.

`build.rs` computes the expected PCR 17 values as `SKINIT` leaves them: the
reset value, zero, extended with the image's digest.

### The PSP path in the Secure Loader

After `SKINIT` the Secure Loader drives the mailbox itself, with a 5 s
timeout per command and 10 s for the extend, and says at `debug` which
command it waits for. A timeout skips the mailbox steps after it, and the
TPM steps still run. After each command it reports whether it still holds
locality 2 (`launch.psp_seize`) and asks for it again if not. In the guide's
order:

1. `LAUNCH` (`launch.psp_launch`), which checks the signature and extends
    PCR 17 and 18. After a refusal, `launch.psp_cap` checks the cap the
    guide says it leaves on PCR 18 to 20, and every later command is
    reported against the status the guide's appendix A gives it.
1. Locality 2, which the launch unlocks, and the SHA-256 PCRs against the
    build's predictions.
1. `EXTEND_OSSL_DIGEST` over the SLB from its base to the end of the
    parameter page, checked against the PCRs read before it.
1. `GET_TMR_DESCRIPTORS`, `ALLOCATE_SHARED_MEMORY`, `GET_TCG_LOGS` and
    `GET_IVRS_TABLE_INFO`. The log is parsed and replayed against PCR 17
    and 18, and the IVRS compared with the one ACPI holds.
1. `TPM_LOCALITY_ACCESS`, after which locality 2 must be refused.
1. `TMR_RELEASE`, with a probe around it: the PIT's interrupt, which the
    IO-APIC delivers as a device write into the `0xFEE` window, must stay
    out of the local APIC's IRR under the TMR and arrive once it is gone.

### The signed image

The service takes an image followed by a signature block: an `$AS1`
header at the measured length, an RSA-4096 PSS signature with SHA-384 over
the image and that header, and the key token the service checks it with.
`build.rs` signs the Secure Loader through the `sign` crate with
`keys/test/sl-test-key.pem`, a key published on purpose, or with the one
`DRTM_SL_KEY` names. It also predicts what the service leaves in PCR 17,
the `SKINIT` value extended with the SPLT version (the Secure Loader adds
the fuse words it learns from `LAUNCH`), and in PCR 18, the key token and
the SecPatchLevel.
