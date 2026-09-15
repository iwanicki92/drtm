<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: Apache-2.0
-->

# DRTM support in QEMU

What QEMU models of the AMD DRTM path, what it does not, and what would
have to exist before more of that path could be exercised here. How the
test suite drives QEMU is in `testing.md`. This page is about the
emulator's limits.

Observations from our own boots go in [Findings](#findings), one row each,
so the page stays a reference rather than growing a section per discovery.

## What QEMU already models

| Piece                       | State                                                          |
|-----------------------------|----------------------------------------------------------------|
| TPM 2.0, TIS and CRB        | Works through `swtpm`, both interfaces in the suite            |
| AMD IOMMU                   | `amd-iommu` device, in the test matrix, at `0:1.0` not `0:0.2` |
| CPU vendor and CPUID leaves | Selected by `-cpu`, enough to drive the vendor gate            |
| Machine-check architecture  | `IA32_MCG_CAP` reports 10 banks, all readable                  |

## What QEMU does not model

**The `SKINIT` instruction.** The TCG decoder raises `#UD` for it
unconditionally, under the comment "If not intercepted, not implemented"
(QEMU 8.2.2, `target/i386/tcg/translate.c`). Code reaching the instruction
takes an undefined-opcode exception rather than crashing, so no separate
no-DRTM build variant is needed to avoid one.

**The `skinit` CPUID bit.** Refused on every CPU model, AMD's own included:

```text
qemu-system-x86_64: warning: TCG doesn't support requested feature:
CPUID.80000001H:ECX.skinit [bit 12]
```

So only the "unsupported" branch of that check is reachable here.

**`MSR 0xC0010058`, the MMIO configuration base.** Absent from the whole
QEMU tree at 8.2.2 and at master. A read falls into `helper_rdmsr`'s
`default:` case, which returns zero without raising `#GP`
(`target/i386/tcg/sysemu/misc_helper.c`, `system/` on master). Those
helpers switch on the MSR number alone and have no notion of CPU vendor,
so an AMD-only MSR does not fault on an Intel model either.

Under KVM the result is the same for a different reason. The kernel does
know this register, as `MSR_FAM10H_MMIO_CONF_BASE`, and deliberately reads
it as zero rather than exposing the host's value, while rejecting any
nonzero write with `#GP` (`arch/x86/kvm/`). So the window reads disabled on
both accelerators, and no guest firmware could enable it.

**An AMD northbridge.** The x86 host bridges QEMU ships are `i440fx.c` and
`q35.c`, both Intel (`hw/pci-host/`). Q35 carries extended configuration
space through the MCH's `PCIEXBAR` at config offset `0x60`
(`include/hw/pci-host/q35.h`) and advertises it in the ACPI MCFG table, so
`-cpu EPYC` changes CPUID and nothing else. Firmware follows suit: the
`qemu-q35` target in coreboot writes `PCIEXBAR` and is built as an Intel
platform whatever the CPU model.

## What would make more of the path testable

Ranked by what it would buy against what it would cost.

### Useful now, for the precondition checks

**Model `MSR 0xC0010058`.** Small, and the only thing standing between us
and exercising both enabled outcomes of the MMCONFIG check, whose arithmetic
currently runs on no input but zero. A writable register honouring the
documented field layout would be enough. Note the KVM path would need its
own change, since the kernel pins it to zero on purpose.

### Needed before a launch could run at all

1. **The `SKINIT` instruction**, including the hash transaction it drives:
    hashing the Secure Loader Block, extending PCR 17, and entering the SL
    with interrupts and DMA shut off (_APM Vol 2, section 15.27_). This is
    the bulk of the work and nothing past the instruction is reachable
    without it.
2. **The TPM side of that transaction.** `SKINIT` measures through locality
    4 with `_TPM_Hash_Start` / `_TPM_Hash_Data` / `_TPM_Hash_End`, which is
    a hardware interface rather than an ordinary command. Whether `swtpm`
    and libtpms implement it is untested and would need checking before
    assuming the TPM half comes for free.
3. **`SL_DEV` or its family 17h successor**, for anything that wants to
    observe the DMA protection a launch establishes.

Emulating DRTM in QEMU is out of scope for this project. This list exists
so the gaps are written down, and so a future effort has somewhere to
start.

## Findings

Observations from real boots. Newest first.

| Date       | Observation                                                                                                     |
|------------|-----------------------------------------------------------------------------------------------------------------|
| 2026-09-15 | `amd-iommu` attaches to the Dasharo q35 guest without disturbing the boot, and sits at `0:1.0` with class 0806h |
| 2026-09-15 | `MSR 0xC0010058` reads zero on six AMD models and on `Skylake-Client` with the gate bypassed, never faulting    |
| 2026-09-15 | Dasharo's firmware leaves `CR0.NE` clear, so that check warns on every boot                                     |
| 2026-09-15 | `IA32_MCG_CAP` reports 10 banks, all clean and all readable                                                     |
| 2026-09-13 | A line written to the serial device arrives whole, while the UEFI text console wraps at 80 columns              |
