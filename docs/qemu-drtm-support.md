<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# DRTM support in QEMU

What upstream QEMU models of the AMD DRTM path, what our own build adds on
top, up to a launch that boots TrenchBoot's Xen, and what it still models
loosely. Everything said about upstream was checked against `v11.1.1`, and our
build is on a later development version, `11.1.50`. How the test suite drives
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

Patches carried on the `drtm-txt` branch of
[our QEMU fork](https://github.com/iwanicki92/qemu), described by what
they make observable rather than by their diff. All are opt-in, so a guest
that does not ask for them behaves as it would upstream.

**The `skinit` CPUID bit can be asked for.** `-cpu EPYC,+skinit` then
reports the feature instead of being refused. On its own this advertises a
capability the CPU does not have, which is enough for code that checks for
`SKINIT` before attempting a launch.

**`SKINIT` executes.** With `-device amd-drtm-platform` on `q35` the
instruction does what _APM Vol 2, section 15.27_ describes: it reads the
two-word header at the base in `EAX`, hashes that many bytes through the
TPM's locality 4 sequence, resets the core to the flat entry state with
`GIF` clear and `DPD`, `R_INIT` and `DIS_A20M` ORed into `VM_CR`, and jumps
to the entry offset. Without the device it raises `#UD` as upstream does.
The other CPUs must be parked in wait-for-SIPI first, and each one started
after the launch arrives with the same `VM_CR` bits and `GIF` clear. Under
TCG a halted CPU never woke for INIT alone, only for the SIPI after it,
which the branch fixes since parking the APs depends on it.

**`VM_CR` exists.** MSR `0xC0010114` carries `DPD`, `R_INIT`, `DIS_A20M`,
`LOCK` and `SVMDIS`. Reserved bits raise `#GP`, `LOCK` freezes `SVMDIS`, and
INIT clears `SVMDIS` unless locked, so the Secure Kernel Loader and Xen
find the bits they clear after a launch. `-cpu EPYC,vm-cr=...` gives the
register a reset value, since no firmware here locks SVM, and a reserved
bit in it is refused at startup.

**The locality 4 hash reaches `swtpm`.** `SKINIT` sends `CMD_HASH_START`,
`CMD_HASH_DATA` and `CMD_HASH_END` over the control channel
(`backends/tpm/tpm_ioctl.h`), which resets PCRs 17 to 22 and extends PCR 17
with the block's digest in every bank. The TIS frontends carry it, and
`-device tpm-tis,locality4=on` lets a guest drive the same sequence through
the locality 4 registers, which no hardware allows, so software standing in
for the instruction can measure into PCR 17 too. `query-amd-drtm` reports
whether the hash went through, and `unsupported` for a backend without the
capability.

**An AMD northbridge function.** `-device amd-nb` adds `00:18.0` with AMD's
vendor ID, the host bridge class and `MEMPROT_CR` at `0x384`. The Secure
Kernel Loader writes that register back with `MEMPROT_EN` clear to lift
`SL_DEV` protection of the block, and the device records the release. The
loader reaches the register through MMCONFIG when MSR `0xC0010058` says
the window is enabled, and otherwise through `0xCF8` with AMD's extended
bits, which upstream masks off and
`-global q35-pcihost.cf8-extended-config=on` decodes.
`-global q35-pcihost.mch-multifunction=on` lets the IOMMU sit at `00:00.2`,
where the loader looks for it, as function 2 of the host bridge.

**`SL_DEV` keeps DMA out.** With an `amd-iommu` present, the block's range
answers every device access with a decode error until the northbridge
releases it, and the IOMMU's own command, event log and table fetches are
held back the same way. The Secure Kernel Loader's first IOMMU pass, which
puts its command buffer inside the block on purpose, stops at the event
log interrupt and its second pass, after the release, completes, which is
the sequence its code expects on hardware. Without an IOMMU the block is
only recorded, and `query-amd-drtm` says which with `enforced`.

**INIT waits for `STGI`.** A core with `GIF` clear holds an INIT instead of
resetting, and `STGI` delivers it. With `R_INIT` set in `VM_CR` the INIT
arrives as `#SX` with error code 1, as _APM Vol 2, table 15-12_ says, so
the loader's window between `SKINIT` and `STGI` is closed the way it is on
hardware.

**The launch is observable.** `query-amd-drtm` returns whether a launch
happened, the block's base and length, whether `SL_DEV` is still armed,
the hash outcome and the ranges kept from DMA. `x-amd-drtm-launch` records
and measures a launch without executing the instruction, for tests. Trace
events cover the instruction, `VM_CR` writes, APs started after the
launch, the northbridge write and the platform object. A block shorter
than its header, an entry offset past its end, an AP not parked, a
release with no launch and a missing TPM are guest errors under
`-d guest_errors`. `-device amd-drtm-platform,strict=on` stops the VM on
any of them instead, as a guest panic that names the rule, so a test cannot
miss one.

**`amd-iommu` can report its capability as enabled.**
`-device amd-iommu,firmware-enabled=on` sets bit 0 of the capability's base
address low register, which firmware sets once it has programmed the MMIO
base. Nothing in a QEMU guest does that, and the Dasharo firmware we boot
is an Intel platform build that would not touch an AMD IOMMU, so without
this the bit can only ever read clear.

**`MSR 0xC0010058` maps MMCONFIG.** A write by the BSP maps or unmaps the
window on `q35` the way PCIEXBAR does, with the bus range setting its size,
so software that programs the register the way AMD firmware does finds
configuration space at the base it named. `-cpu EPYC,mmio-conf-base=...`
gives the register a reset value, since no firmware here programs it. The
reset value is reported, not mapped, so it has to name a window the
firmware did map for software that then uses it.

**One machine option.** `-machine q35,amd-drtm=on` adds the `skinit`
feature to every CPU, the platform object, `amd-nb` at `00:18.0`, an
`amd-iommu` at `00:00.2` with its capability enabled, and turns on the two
host bridge properties above. It is the configuration the TrenchBoot boot
test uses. Each device still works on its own, and the MCH stays Intel
underneath.

**A Secure Processor with mailbox registers.** `-device amd-psp` adds an AMD
function with the Turin ASP's device ID, `156e`, and `device-id=` picks any
other. Its class stays `1080`, an encryption controller, whatever the ID,
which is what a board newer than the loaders' table looks like. Its
`C2PMSG` mailboxes answer on BAR 2 and again through the window an ASPT
declares, `drtm-ready=` sets the bit the DRTM service sets when it will
take a command, and `aspt=off` leaves the table out.

**The PSP DRTM service.** `-device amd-psp,drtm-service=on` answers every
command AMD's integration guide (publication 58453) lists. `TMR_SETUP`
blocks device DMA to a range in the IOMMU's view, the interrupt window
included, so the IO-APIC's messages are dropped under a TMR over it until
`TMR_RELEASE`. `LAUNCH` checks the Secure Loader's signature against the
key token it carries, extends PCR 17 and 18 as Turin firmware does, and
unlocks localities 1 and 2, which the service keeps locked from reset.
`EXTEND_OSSL_DIGEST` extends a range and logs it in the TCG log that
`GET_TCG_LOGS` hands out, and `TPM_LOCALITY_ACCESS` locks locality 2 again.
`GET_TMR_DESCRIPTORS`, `ALLOCATE_SHARED_MEMORY` and `GET_IVRS_TABLE_INFO`
answer from regions of the service's own, the last with a copy of the IVRS
ACPI holds. Before a successful launch the commands answer as the guide's
appendix A says. `min-spl=`, `splt-version=`, `command-delay=` and
`delayed-command=` give the tests a platform patch level, another SPLT
version and a service slow to answer.

**A DRTM ACPI device.** `-device amd-psp,drtm-device=on` adds an SSDT
holding `\_SB.DRTM` with `_HID "DRTM0002"`, which is what AMD firmware
publishes for it. Nothing implements what the device describes, and on
hardware it is firmware's rather than the Secure Processor's, so it rides
on that device here only because it is the AMD one such a guest already has.

**`amd-iommu` can stay out of ACPI.** `-device amd-iommu,ivrs=off` leaves
the IVRS table out, which upstream writes whenever the device is present
(`hw/i386/acpi-build.c`). The IOMMU still answers in configuration space,
so this is a configured IOMMU no OS would find.

**AMD's SLs can run.** Three opt-in pieces carry them to their hand-off,
see `docs/foreign-sl.md`. `-device isa-post-code,chardev=` logs each write
to ports `0x80` to `0x83` whole, which the machine otherwise discards.
`-device amd-fch-pmio` answers the FCH's `AcpiPmTmrBlk` at `0xfed80364`
with the ICH9 PM timer's port. `amd-psp,client-smm=on` answers the client
only `PROCESS_SMM` and `GET_SMM_POLICY` with zero hashes and an empty
policy, which the client SL accepts and no firmware is known to return.

## What would still be needed

Every precondition check reaches its outcomes, with three exceptions. Two
take a deliberately wrong ACPI table: `amd.iommu_table` never meets an IVRS
that is malformed or disagrees with configuration space, and
`amd.mmconfig_table` and `amd.psp_drtm` never meet a table whose checksum
or structures are wrong. One takes state only firmware leaves: `amd-nb`
resets `MEMPROT_CR` to zero, so `amd.northbridge` never sees `MEMPROT_EN`
set. These are left to hardware.

The launch itself runs: the meta-trenchboot v0.5.2 image boots Xen and
dom0 through GRUB's EFI TrenchBoot entry with PCR 17 extended, which a
functional test in the QEMU tree checks, since it exercises the emulator
rather than this binary. What the build still models loosely, in the order
a stricter guest would notice:

1. **The Secure Processor's DRTM service is the guide's.** Which commands
    seize a locality, where the regions lie and what the three commands no
    loader issues return are our reading of it, and hardware may differ.
2. **A writable IOMMU enable bit.** The property sets it at realize and
    leaves it read-only. The Secure Kernel Loader programs the IOMMU's MMIO
    registers, which upstream models, but a loader that expects the
    capability bit to take a write (RW1S) would not see it stick.
3. **`SL_DEV` covers devices only.** CPU accesses to the block pass, as on
    hardware, and the range is one block at the SLB rather than the
    64 KiB granule and exclusion range the real register set describes.
4. **The MMCONFIG window is per machine.** Every core carries the MSR, but
    only the BSP's write moves the window, and PCIEXBAR and the MSR share
    it, so whichever was written last wins.

Both TrenchBoot's loader and `drtm`'s own launch exercise the emulation:
`drtm`'s launch boots in `test_launch.py` go through `SKINIT` on the strict
platform, and those in `test_psp_launch.py` through the PSP DRTM service as
well. A CRB TPM still has no locality 4 path, so its launch boot runs
lenient and the hash never reaches it, as do the PSP boots whose `LAUNCH`
is refused on purpose.

## Findings

Observations from real boots. Newest first. `QEMU` says which build the
observation belongs to, since the older rows were taken on Debian's 8.2.2
and some of them no longer hold.

| Date       | QEMU   | Observation                                                                                                               |
|------------|--------|---------------------------------------------------------------------------------------------------------------------------|
| 2026-09-26 | ours   | A Secure Loader's GDT loaded with the accessed bits clear changes its signed bytes, so `LAUNCH` fails the signature       |
| 2026-09-26 | ours   | The PIT's interrupt through the IO-APIC stays out of the IRR under a TMR over `[0, 4 GiB)`, arriving after `TMR_RELEASE`  |
| 2026-09-20 | ours   | `vm-cr=0x8` reads back as family 1Ah firmware leaves it, and `vm-cr=0x7` reaches the `R_INIT` warning of `amd.vm_cr`      |
| 2026-09-20 | ours   | A `drtm` boot of the launch machine under strict mode with `-d guest_errors` leaves the log empty and no launch recorded  |
| 2026-09-20 | ours   | `amd-nb` answers offset `0x384` through the root bridge protocol, reading zero, as function 0 of a family 1Ah node does   |
| 2026-09-20 | ours   | `mmio-conf-base=0xb0000021` names the window Dasharo's MCFG publishes, so `amd.mmconfig_table` agrees there alone         |
| 2026-09-20 | ours   | `VM_CR` reads zero on every boot, so `amd.vm_cr` reaches only its all-clear outcome                                       |
| 2026-09-20 | ours   | With `SL_DEV` enforced, SKL's first IOMMU pass stops at the event log interrupt, the second completes after the release   |
| 2026-09-20 | ours   | SKL's `IOMMU event log not empty` with an all-zero dump is that first pass, not a fault: the entry was never written      |
| 2026-09-20 | ours   | `-machine q35,amd-drtm=on` boots the same image with SKL taking the `0xCF8` route, MSR `0xC0010058` left at zero          |
| 2026-09-20 | ours   | `amd-iommu` passes DMA through unless `dma-remap=on`, so `amd-drtm=on` sets it and a Linux booted directly finds its disk |
| 2026-09-20 | ours   | meta-trenchboot v0.5.2 boots Xen and dom0 through `SKINIT`, PCR 17 extended in every bank, PCR 18 in SHA-1 and SHA-256    |
| 2026-09-20 | ours   | SKL writes `MEMPROT_CR` over `0xCF8` with AMD's extended bits unless MSR `0xC0010058` is enabled, bits QEMU masks off     |
| 2026-09-20 | ours   | Dasharo maps MMCONFIG at `0xb0000000`, so `mmio-conf-base=0xb0000001` names the window it did map                         |
| 2026-09-20 | ours   | SKL's serial lines sit between Xen's EFI stub banner and its first `(XEN)` line, so they read as part of Xen's boot       |
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
