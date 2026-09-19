<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: Apache-2.0
-->

# DRTM support in QEMU

What upstream QEMU models of the AMD DRTM path, what our own build adds on
top, and what would have to exist before more of that path could be
exercised here. Everything said about upstream was checked against
`v11.1.1`, which is what our build branches from. How the test suite drives
QEMU, and how to build it, is in `testing.md`.

Observations from our own boots go in [Findings](#findings), one row each,
so the page stays a reference rather than growing a section per discovery.

## What upstream models

| Piece                       | State                                                                              |
|-----------------------------|------------------------------------------------------------------------------------|
| TPM 2.0, TIS and CRB        | Works through `swtpm`. TIS models five localities, CRB reports locality 0 only     |
| AMD IOMMU                   | `amd-iommu` device, in the test matrix, at whatever slot is free, enable bit clear |
| PCI configuration space     | Read through the root bridge protocol, extended space included                     |
| CPU vendor and CPUID leaves | Selected by `-cpu`, with `family=`/`model=` overriding the version field           |
| Machine-check architecture  | `IA32_MCG_CAP` reports 10 banks, all readable                                      |

## What upstream does not model

**The `SKINIT` instruction.** The TCG decoder raises `#UD` for it
unconditionally, under the comment "If not intercepted, not implemented"
(`target/i386/tcg/translate.c`). Code reaching the instruction takes an
undefined-opcode exception rather than crashing, so no separate no-DRTM
build variant is needed to avoid one.

**The `skinit` CPUID bit.** No CPU model carries it, AMD's own included:
`CPUID_EXT3_SKINIT` is defined in `target/i386/cpu.h` and used nowhere else
in the tree. Asking for it by name is refused too, since it is absent from
`TCG_EXT3_FEATURES` (`target/i386/cpu.c`):

```text
qemu-system-x86_64: warning: TCG doesn't support requested feature:
CPUID[eax=80000001h].ECX.skinit [bit 12]
```

So on upstream only the "unsupported" branch of that check is reachable.

**`MSR 0xC0010058`, the MMIO configuration base.** Absent from the whole
QEMU tree. A read falls into `helper_rdmsr`'s `default:` case, which returns
zero without raising `#GP` (`target/i386/tcg/system/misc_helper.c`). Those
helpers switch on the MSR number alone and have no notion of CPU vendor, so
an AMD-only MSR does not fault on an Intel model either, and that stays true
of our build.

Under KVM the result is the same for a different reason. The kernel does
know this register, as `MSR_FAM10H_MMIO_CONF_BASE`, and deliberately reads
it as zero rather than exposing the host's value, while rejecting any
nonzero write with `#GP` (`arch/x86/kvm/`). So the window reads disabled on
both accelerators, and no guest firmware could enable it.

**The AMD Secure Processor.** No device in the QEMU tree carries any of the
Secure Processor IDs the loaders match on, and `hw/misc/` has no PSP model
at all. Nothing builds an ASPT either, which is the table naming where the
Secure Processor's registers are. So on upstream the PSP check only reaches
its not-found branch, and the mailbox that would say whether the DRTM
service is enabled has nothing to answer it.

The `amd-iommu` device is the nearest thing to a false positive, answering
`1022:1419`: AMD's vendor ID with a device ID no loader's table lists. That
makes it worth keeping in the matrix for this check too, since it is a
system peripheral by class rather than an encryption controller, so neither
half of the match takes it.

**An AMD northbridge.** The x86 host bridges QEMU ships are `i440fx.c` and
`q35.c`, both Intel (`hw/pci-host/`). Q35 carries extended configuration
space through the MCH's `PCIEXBAR` at config offset `0x60`
(`include/hw/pci-host/q35.h`) and advertises it in the ACPI MCFG table, so
`-cpu EPYC` changes CPUID and nothing else. Firmware follows suit: the
`qemu-q35` target in coreboot writes `PCIEXBAR` and is built as an Intel
platform whatever the CPU model.

## What our build adds

Patches carried on the `drtm` branch of our QEMU fork, described by what
they make observable rather than by their diff. All are opt-in, so a guest
that does not ask for them behaves as it would upstream.

**The `skinit` CPUID bit can be asked for.** `-cpu EPYC,+skinit` then
reports the feature instead of being refused. The instruction still raises
`#UD`, so this advertises a capability the CPU does not have: enough for
code that checks for `SKINIT` before attempting a launch, and nothing past
that.

**`amd-iommu` can report its capability as enabled.**
`-device amd-iommu,firmware-enabled=on` sets bit 0 of the capability's base
address low register, which firmware sets once it has programmed the MMIO
base. Nothing in a QEMU guest does that, and the Dasharo firmware we boot
is an Intel platform build that would not touch an AMD IOMMU, so without
this the bit can only ever read clear.

**`MSR 0xC0010058` holds a value.** `-cpu EPYC,mmio-conf-base=0xe0000021`
gives the MMIO configuration base register a reset value, since no firmware
here programs it, and writes to it stick. Nothing is mapped: the register
reports a window rather than backing one, which is what software reading it
to locate configuration space needs and nothing more.

**A Secure Processor with mailbox registers.** `-device amd-psp` adds an AMD
function with the Turin ASP's device ID, `156e`, and `device-id=` picks any
other. Its class stays `1080`, an encryption controller, whatever the ID,
which is what a board newer than the loaders' table looks like. Its
`C2PMSG` mailboxes answer on BAR 2 and again through the window an ASPT
declares, `drtm-ready=` sets the bit the DRTM service sets when it will
take a command, and `aspt=off` leaves the table out. Writes are
dropped: issuing a command means implementing the service behind it, which
is the launch.

**A DRTM ACPI device.** `-device amd-psp,drtm-device=on` adds an SSDT
holding `\_SB.DRTM` with `_HID "DRTM0002"`, which is what AMD firmware
publishes for it. Nothing implements what the device describes, and on
hardware it is firmware's rather than the Secure Processor's, so it rides
on that device here only because it is the AMD one such a guest already has.

**`amd-iommu` can stay out of ACPI.** `-device amd-iommu,ivrs=off` leaves
the IVRS table out, which upstream writes whenever the device is present
(`hw/i386/acpi-build.c`). The IOMMU still answers in configuration space,
so this is a configured IOMMU no OS would find.

## What would still be needed

Every precondition check reaches its outcomes, except the two that take a
deliberately wrong ACPI table: `amd.iommu_table` never meets an IVRS that is
malformed or disagrees with configuration space, and `amd.psp_drtm` never
meets an ASPT whose checksum or register structures are wrong. Those are
left to hardware. What is left is the launch itself.

1. **The `SKINIT` instruction**, including the hash transaction it drives:
    hashing the Secure Loader Block, extending PCR 17, and entering the SL
    with interrupts and DMA shut off (_APM Vol 2, section 15.27_). This is
    the bulk of the work and nothing past the instruction is reachable
    without it.
2. **The TPM side of that transaction.** `SKINIT` measures through locality
    4 with `_TPM_Hash_Start` / `_TPM_Hash_Data` / `_TPM_Hash_End`, which is
    a hardware interface rather than an ordinary command. The `swtpm`
    control channel already carries all three as `CMD_HASH_START`,
    `CMD_HASH_DATA` and `CMD_HASH_END` (`backends/tpm/tpm_ioctl.h`), but
    QEMU sends none of them, and the TIS device ignores every write to the
    locality 4 window (`hw/tpm/tpm_tis_common.c`).
3. **A writable IOMMU enable bit.** Our property sets it at realize and
    leaves it read-only, which is enough to observe but not to program. The
    Secure Kernel Loader sets up the IOMMU itself and expects the bit to
    take a write (RW1S), along with the capability-block lock that implies.
4. **`SL_DEV` or its family 17h successor**, for anything that wants to
    observe the DMA protection a launch establishes.

Emulating DRTM in QEMU is out of scope for this project. This list exists
so the gaps are written down, and so a future effort has somewhere to
start.

## Findings

Observations from real boots. Newest first. `QEMU` says which build the
observation belongs to, since the older rows were taken on Debian's 8.2.2
and some of them no longer hold.

| Date       | QEMU   | Observation                                                                                                               |
|------------|--------|---------------------------------------------------------------------------------------------------------------------------|
| 2026-09-18 | ours   | `-device amd-psp,drtm-device=on` publishes the SSDT AMD firmware does, so `amd.drtm_device` finds it                      |
| 2026-09-18 | ours   | `-device amd-psp` answers C2PMSG_72 through the window its ASPT declares, so `amd.psp_drtm` reads it both ways            |
| 2026-09-18 | ours   | `-device amd-psp` publishes an ASPT of the same 112 bytes and revision Turin firmware does, differing in the base address |
| 2026-09-18 | ours   | Firmware here enables the Secure Processor's memory decode, which Turin firmware does not, so the BAR answers too         |
| 2026-09-18 | 11.1.1 | `family=` and `model=` are upstream CPU properties, so `-cpu EPYC,family=26,model=2` reaches the family 1Ah MSR list      |
| 2026-09-17 | ours   | `-device amd-iommu,ivrs=off` leaves no IVRS table, so `amd.iommu_table` warns about an enabled IOMMU                      |
| 2026-09-17 | ours   | Dasharo passes QEMU's IVRS through, and it lists the one IOMMU under two IVHD types                                       |
| 2026-09-17 | ours   | `-device amd-psp` makes `amd.psp` identify an ASP, the first boot to reach that branch                                    |
| 2026-09-17 | ours   | A corrected error injected with `mce` before the firmware runs is still logged when `drtm` reads the banks                |
| 2026-09-17 | ours   | Dasharo gives a `pxb-pcie` its own root bridge handle, and each handle refuses buses it does not own                      |
| 2026-09-16 | ours   | `-cpu EPYC,mmio-conf-base=...` reaches both enabled outcomes of `amd.mmconfig`, which had only ever been skipped          |
| 2026-09-16 | ours   | `-device amd-iommu,firmware-enabled=on` makes `amd.iommu` pass, the first boot to reach that branch                       |
| 2026-09-16 | ours   | `-cpu EPYC,+skinit` makes `amd.skinit` pass, the first boot to reach that branch                                          |
| 2026-09-16 | 11.1.1 | `amd-iommu` answers `1022:1419`, so an AMD vendor ID alone would take it for a Secure Processor                           |
| 2026-09-16 | 8.2.2  | `tpm-crb` reports `CapLocality` clear and `tpm-tis` set, so both locality outcomes are reachable                          |
| 2026-09-16 | 8.2.2  | `amd-iommu` reads its capability's enable bit clear, so a launch would treat it as disabled by firmware                   |
| 2026-09-16 | 8.2.2  | No guest configuration answers with any Secure Processor device ID, so the PSP check only reaches its not-found branch    |
| 2026-09-15 | 8.2.2  | Dasharo publishes `EFI_PCI_ROOT_BRIDGE_IO_PROTOCOL`, one handle on segment 0, and config reads through it work            |
| 2026-09-15 | 8.2.2  | That protocol reaches extended config space too: the e1000 answers `14020001` at offset 0x100                             |
| 2026-09-15 | 8.2.2  | `amd-iommu` lands at `0:03.0` in the full test configuration and `0:01.0` with `-nodefaults`, so its address is not fixed |
| 2026-09-15 | 8.2.2  | `amd-iommu` attaches to the Dasharo q35 guest without disturbing the boot, with class 0806h and capability 0Fh type 3     |
| 2026-09-15 | 8.2.2  | `MSR 0xC0010058` reads zero on six AMD models and on `Skylake-Client` with the gate bypassed, never faulting              |
| 2026-09-15 | 8.2.2  | Dasharo's firmware leaves `CR0.NE` clear, so that check warns on every boot                                               |
| 2026-09-15 | 8.2.2  | `IA32_MCG_CAP` reports 10 banks, all clean and all readable                                                               |
| 2026-09-13 | 8.2.2  | A line written to the serial device arrives whole, while the UEFI text console wraps at 80 columns                        |
