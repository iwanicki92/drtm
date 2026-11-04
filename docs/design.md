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
- Only AMD on UEFI is implemented. Intel, ARM, and non-UEFI support would come
    after that.

## Architecture principle

Keep DRTM-path-specific code (Intel vs. AMD vs. ARM, UEFI vs. non-UEFI)
cleanly separated from generic, non-DRTM code (UEFI app scaffolding, logging,
CLI/output). Each DRTM path should be easy to read in isolation: a reader
looking at "what does the AMD path do" shouldn't have to wade through Intel
logic or generic plumbing to find it.

## DRTM launch design decisions

- The tool embeds its own Secure Loader and DLME rather than needing externally
    supplied binaries (e.g. TrenchBoot's own), so it stays a single,
    self-contained binary. They only need to prove that the DRTM launch
    mechanics work, so the DLME reports and halts instead of hosting a real
    next-stage payload. A debug build can embed
    [a Secure Loader from elsewhere](foreign-sl.md) instead of `drtm`'s own.
- SKINIT (and equivalent) tears down normal UEFI execution, so the Secure Loader
    and the DLME cannot rely on UEFI console/boot services to report results.
    They write results directly to the serial UART, and the DLME then halts.
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

A run has three stages:

1. `drtm` runs its checks. Each one prints a record with a dotted ID such as
    `amd.skinit`, and a summary follows them. Later names such as `launch.path`
    are record IDs too.
1. [The gate](#the-gate) reads those results and decides whether a launch can go
    ahead. With `launch=off` in `drtm.conf` the run stops after the summary.
1. [The preamble](#the-preamble) prepares the machine and executes `SKINIT`.

The launch proves the launch mechanics on a platform. `drtm`'s own Secure Loader
confirms from inside the launched environment that the TPM measured it. It then
measures `drtm`'s DLME and jumps to it, as a loader would to an OS. The DLME
compares what the launch and its own measurement left with the expected values,
prints the summary of both and halts. Neither the Secure Loader nor the DLME
wakes an AP.

There are two paths. The classic path uses only `SKINIT`. The PSP path, which
Turin firmware offers, also uses AMD's PSP DRTM service. AMD's DRTM Service
Integration Guide (publication 58453, "the guide" below) describes that service:

- Before `SKINIT`, the preamble asks the service to protect memory with a TMR.
- After `SKINIT`, the Secure Loader asks the service to check the image's
    signature and extend the DRTM PCRs.
- The DLME then asks the service to release the TMR.

Together they issue every command the guide lists, except the client-only
`PROCESS_SMM` and `GET_SMM_POLICY`.

### The gate

The gate decides whether the launch goes ahead. It runs after the checks'
summary, reads their results, and blocks the launch only if a launch would be
impossible or unsafe:

- The CPU is not AMD, or `amd.skinit` does not pass.
- `cpu.machine_check`, `cpu.bsp`, `amd.slb` or `tpm.present` fails.
- The CPU has no 1 GiB pages for the Secure Loader's page tables.
- There is no UART the Secure Loader could write to.
- `launch=psp` is set and the PSP path is not available.

After any other failure the launch still goes ahead. For example, a TPM that
offers only locality 0 fails `tpm.locality` and still launches, so the run shows
what a launch measures on that machine. A blocked launch prints one
`launch.gate` record and stops.

The Secure Loader and the DLME write to the UART given in the ACPI SPCR table.
Without a usable one they write to COM1 at `0x3F8`, if UEFI offers a serial
device or a 16550 answers at that address.

### Choosing the path

`launch=auto`, the default, takes the PSP path when all of the conditions below
hold. Otherwise it reports the first one that does not hold as `launch.path` and
launches classically. `launch=psp` blocks the launch in that case, through
`launch.gate`, and `launch=skinit` never tries the PSP path.

1. The TPM is 2.0, since the service extends through TPM 2.0 commands.
1. The address of the mailbox register `C2PMSG_72` is known. It comes from the
    ASPT (the ACPI table that says where the Secure Processor's registers are),
    or else from the PSP's base address read through SMN (AMD's System
    Management Network), as GRUB does.
1. That address is below 4 GiB, where the Secure Loader's page tables reach.
1. The service's ready bit is set.
1. `GET_CAPABILITY` reports DRTM enabled. The service answers nothing until a
    zero has been written to `C2PMSG_72`, so that write comes first.

Condition 5 changes the service's state, so it runs after the checks' summary,
while UEFI output still works.

The preamble sets up the TMR only after `ExitBootServices()`, as GRUB does.
Devices cannot reach the low 4 GiB under the TMR, so a firmware driver that
stops its device during `ExitBootServices()` could hang, and on Turin firmware
one did. If `TMR_SETUP` fails, the preamble reports it and goes on, so the
Secure Loader can still report how `LAUNCH` answers.

A debug build can also launch a Secure Loader signed by someone else, which
hands off to the same DLME. See [foreign-sl.md](foreign-sl.md).

### The preamble

The preamble is the part of `drtm` (`src/launch/`) that prepares the machine for
`SKINIT`. It allocates three regions below 4 GiB:

- the SLB (Secure Loader Block), which `SKINIT` measures and enters
- a 1 MiB block for the DLME
- a page for the trampoline, the code in step 6 below

It copies the Secure Loader into the SLB and the DLME into its block, each with
the same parameter page behind it. The SLB and the DLME's block sit on a 2 MiB
boundary, which their page tables need. A launch through the service puts the
block on a 64 MiB boundary, since the guide's appendix E.1 asks that of the
range `EXTEND_OSSL_DIGEST` measures.

The parameter page, laid out by `common::params`, holds:

- the output format, level and sinks
- the TPM's family, interface and active banks
- the time-stamp counter rate
- whether the PSP DRTM service was ready
- the DLME's address, length and SHA-256, and the PCR 17 value the classic path
    should leave
- on the PSP path, the mailbox, the capability words, the TMR, the IO-APIC pin
    the PIT reaches, the build's PCR predictions and the SHA-256 of the IVRS the
    checks read

The preamble prints the records `launch.skinit` and `launch.dlme_placed` (and
`launch.psp_service` on the PSP path), then runs these steps:

1. It calls `ExitBootServices()`. From here nothing reaches the UEFI logger, so
    progress goes to the sinks as `launch.preamble` records at `debug`.
1. On the PSP path, it sends `TMR_SETUP` of index 0 over `[0, 4 GiB)`, reported
    as `launch.psp_tmr_setup`.
1. It relinquishes locality 0. This comes after the exit because the TCG2 driver
    measures that event.
1. It clears `CR0.CD` and `CR0.NW`, sets `CR0.NE` and turns interrupts off, as
    `grub_skinit_prepare_cpu()` does.
1. It sends every AP an INIT through the all-excluding-self shorthand, by
    `MSR 0x830` or the xAPIC's ICR, and gives them 10 ms to take it. `SKINIT`
    requires them parked (APM Vol 2, section 15.27.8) and does not park them
    itself.
1. It runs the trampoline, which it copies into the low page because `drtm` may
    load above 4 GiB. The trampoline leaves long mode, writes one `S` to the
    UART and executes `SKINIT`.

### The signed image

The service takes an image followed by a signature block, which has three parts:

- an `$AS1` header at the measured length
- an RSA-4096 PSS signature with SHA-384 over the image and that header
- the key token the service checks the signature with

`build.rs` signs the Secure Loader through the `sign` crate. It uses
`keys/test/sl-test-key.pem`, a key published on purpose, or the key given in
`DRTM_SL_KEY`.

`build.rs` also computes the PCR values each path should leave:

- After `SKINIT`, PCR 17 is its reset value (zero) extended with the image's
    digest.
- On the classic path, that value is then extended with the DLME's digest.
- On the PSP path, the service extends that value with the SPLT version. The
    Secure Loader later adds the fuse words it learns from `LAUNCH`.
- On the PSP path, the service also leaves a value in PCR 18, built from the key
    token and the SecPatchLevel in the `$AS1` header.

### The Secure Loader

`sl/` is linked by `sl/sl.ld`. The image opens with AMD's header and keeps
`bootloader_data_offset` at byte 68, where TrenchBoot's layout has it. The
measured length is padded to a multiple of 16. After the measured bytes come the
signature block, the parameter page, `.bss`, a 6 KiB stack and three pages of
page tables, which end at the end of the 64 KiB.

The Secure Loader is built once per launch path, through the `classic` and `psp`
features, and the preamble writes the image of the path it takes into the SLB.
Each image carries only the code of its own path, which leaves the PSP image
more room in the SLB than a single image for both paths would have.

The 32-bit entry loads an empty IDT and clears `VM_CR.DIS_A20M`. `SKINIT` leaves
`SS` as the only usable data segment, so the entry addresses memory through it
until it has loaded its own GDT. It then zeroes `.bss` and enters long mode,
with the low 4 GiB identity-mapped in 1 GiB pages and the SLB's 2 MiB page
mapped at `0xFFFFFFFF80000000`, the address the Secure Loader is linked at. So
it needs no relocations wherever the SLB lands, and the code it runs is the code
the TPM measured. The GDT's accessed bits are preset, since loading a segment
would otherwise set them and change the signed bytes before the service checks
them. Everything from the GDT on is in `common/src/long_mode.s`, which the
DLME's entry shares.

On the classic path, `sl_main` (the Secure Loader's main function) asks for
locality 2 and reads PCR 17 to 22 at that locality. If locality 2 is not
granted, it reads them at locality 0. The PSP DRTM service refuses locality 2
until a PSP launch, so a classic launch takes this fallback on a platform with
the service.

`sl_main` compares PCR 17 with the expected value in every bank the build
computed one for, SHA-1 and SHA-256, and PCR 18 to 22 with zero. Then it hashes
the DLME in SHA-1 and SHA-256 and, holding locality 2, extends PCR 17 with it in
those banks where they are active (`launch.dlme_extend`). Other active banks are
not extended, and the record says so. On this path nothing guards the block
against device writes between the hash and the jump, while on the PSP path TMR 0
covers it.

### The PSP path in the Secure Loader

After `SKINIT` the Secure Loader sends commands to the service through the PSP
mailbox itself. Each command has a 5 s timeout, except `EXTEND_OSSL_DIGEST`,
which has 10 s or the value of `DRTM_EXTEND_TIMEOUT_MS` given at build time. At
`debug` level it logs which command it is waiting for.

On a timeout it reports the value of `C2PMSG_72` and the wait time
(`elapsed_ms`), then skips the remaining mailbox steps. The steps that use only
the TPM still run.

After each command it reports whether it still holds locality 2
(`launch.psp_seize`), because a command can seize the locality, and asks for
locality 2 again if not.

The steps follow the guide's order:

1. `LAUNCH` (`launch.psp_launch`) checks the signature and extends PCR 17 and
    18. After a refusal, `launch.psp_cap` checks that PCR 18 to 20 hold the cap
    the guide says a refusal leaves. Each later command's answer is compared
    with the status that the guide's appendix A lists for that command.
1. The Secure Loader takes locality 2, which `LAUNCH` unlocks, and compares the
    SHA-256 PCRs with the build's predictions.
1. `EXTEND_OSSL_DIGEST` runs over the DLME, which the guide calls the MLE. The
    Secure Loader checks the result against the PCRs it read before the command,
    and reports how long the service took to answer (`elapsed_ms`). It does not
    send a DLME that is not the one the build made.
1. `GET_TMR_DESCRIPTORS`, `ALLOCATE_SHARED_MEMORY`, `GET_TCG_LOGS` and
    `GET_IVRS_TABLE_INFO` follow. The log is parsed and replayed against PCR 17
    and 18, and the IVRS is compared with the one ACPI holds.
1. `TPM_LOCALITY_ACCESS` runs last, and locality 2 must be refused after it.

`TMR_RELEASE` is left to the DLME, as it would be to an OS.

### The handoff

Both paths end in the handoff. If the DLME's SHA-256 is the one the build
computed, the Secure Loader writes its tally and what the DLME needs to know
into the DLME's page. It then leaves long mode as the trampoline does, and jumps
to the DLME's entry in flat 32-bit protected mode with paging off. Otherwise it
reports `launch.handoff`, prints the summary and halts.

### The DLME

`dlme/` is linked by `dlme/dlme.ld` at the Secure Loader's address, and maps its
block's 2 MiB page there the same way. Its image opens with the
`OS_SL_DESCRIPTOR_V1` that AMD's SL looks for, and is padded to whole pages.
After the image come its parameter page, `.bss`, a 16 KiB stack and its page
tables. The rest of the block is left to a foreign SL.

The entry writes the POST code `0xd7d70001` to port `0x80`, then finds its own
address with a call. It needs only a usable stack from its caller, so `drtm`'s
Secure Loaders and AMD's can both jump to it.

The DLME reads the handoff from its page, then compares the PCRs:

- After the classic Secure Loader it compares PCR 17 in each bank with the
    `SKINIT` value, extended with the DLME's digest where the Secure Loader did
    that extend (`launch.dlme_pcr17`).
- After a PSP launch it compares SHA-256 PCR 17 and 18 with what `LAUNCH` and
    the extend of the DLME leave (`launch.dlme_pcr17`, `launch.dlme_pcr18`). It
    skips these comparisons if `LAUNCH` failed, or if the extend got no answer,
    because an unanswered extend may still have measured the DLME.

The DLME then sends `TMR_RELEASE` with a probe around it, which tests whether
the TMR blocks device writes. The IO-APIC delivers the PIT's interrupt as a
device write into the `0xFEE` window. So the interrupt should stay out of the
local APIC's IRR under the TMR, and arrive once the TMR is gone.

An interrupt that arrives under the TMR gives a `warn`, not a `fail`. The TMR
let a device write through, but the guide says nothing about interrupts, and
Turin firmware has been seen both ways. It held back the FCH's interrupts on an
EPYC 9015 and passed the PIT's on another Turin platform. With IF clear the
vector of that interrupt stays in the IRR, so the DLME leaves out the watch
after the release.

A refused `LAUNCH` still leaves the TMR the preamble set up, so the release and
the probe run after a refusal too. The DLME skips both if the mailbox stopped
answering in the Secure Loader.

After `drtm`'s Secure Loaders the summary starts from their tally, so it counts
the whole launch, and `launch.halted` follows it. The machine needs a power
cycle before the next attempt, since a reset would boot `drtm` from the same
media and launch again.

## Glossary

These terms recur across the docs:

- ASP (AMD Secure Processor): the processor in AMD CPUs that runs firmware
    services. This repository and many tools call it the PSP.
- DLME (Dynamically Launched Measured Environment): the code a launch ends in,
    normally an OS kernel. `drtm`'s own DLME reports and halts. AMD's guide
    calls this code the MLE (Measured Launch Environment), and AMD's SLs call it
    the OS SL.
- DRTM TA: the trusted application on the PSP that implements the PSP DRTM
    service. Firmware carries it as PSP directory entry type `0x47`.
- The gate: the step after the checks' summary that decides whether a launch
    goes ahead, see [The gate](#the-gate).
- The guide: AMD's DRTM Service Integration Guide, publication 58453.
- Key token: a public key in AMD's own format, with the key ID, the modulus and
    the exponent. The tokens here are `0x440` bytes. The guide calls it the
    Public Key Token Image.
- The launch machine: `-machine q35,amd-drtm=on` on our QEMU build, see [DRTM
    support in QEMU](qemu-drtm-support.md).
- The preamble: the part of `drtm` that prepares the machine and executes
    `SKINIT`, see [The preamble](#the-preamble).
- PSP DRTM service: the firmware service on the PSP that the PSP path uses. It
    protects memory, checks the SL's signature and extends the DRTM PCRs.
- Record: one result or log line of `drtm`'s output. Results have a dotted ID
    such as `launch.gate`.
- Sink: a device that records are written to, such as the serial port or the
    UEFI text console.
- SL (Secure Loader): the code `SKINIT` measures and jumps to, which then starts
    the DLME.
- SLB (Secure Loader Block): the 64 KiB memory range that holds the SL.
- SPLT (Security Patch Level Table): a firmware table whose version the service
    extends into PCR 17 during `LAUNCH`.
- Tally: the number of results with each tag, which a summary prints. The SL
    passes its tally to the DLME.
- TMR (Trusted Memory Region): a memory range the service keeps devices out of
    until `TMR_RELEASE`.
