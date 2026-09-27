<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Launching a foreign Secure Loader

A debug build of `drtm` can embed a Secure Loader signed by someone else and
launch it through the PSP DRTM service in place of its own. The point is AMD's
own client SL. Client PSP firmware takes only SLs signed with keys its DRTM TA
lists, which `drtm`'s key never is, so on a client machine that SL is the only
one known to launch. It shows whether the platform's DRTM works at all,
separately from anything `drtm` does.

## Where AMD's SL comes from

Lenovo publishes it inside the Windows driver package "AMD DRTM Boot Driver
for Windows", for the ThinkPad L14 Gen 5 (types 21L5, 21L6) and L16 Gen 1
(types 21L7, 21L8):

- Package: `r2idt05w.exe`, version 1.0.19.4, SHA-256
    `db09603d000e2eb8bbb55c7bba4103cbcccb713e640ea713dd3935cf865f680b`.
- The DLL inside it: `amddrtm.dll`, SHA-256
    `4d12a07f18dbcb13a2a7bc9496c88fc7f20c5c83fa1eef7a5fa4cff63626d903`.
- The SL in the DLL: offset `0x32c0`, `0x9ed0` bytes measured.
- The key ID in its token: `024ca9a5c32f41008aa8dd84c88b8dfc`.

The package is an Inno Setup installer. `innoextract r2idt05w.exe` unpacks
`amddrtm.dll` without running anything. The SL is not part of any BIOS image.
The firmware only carries the DRTM TA with its list of accepted keys.

Other Lenovo AMD client packages carry the same key but not always the same
SL. The 1.0.19.x DLLs for other models have the same layout with a different
body, and 1.0.16.1 has a shorter one (`0x8800` bytes measured).

## Building and running

`DRTM_FOREIGN_SL` names either the DLL or a bare image (the measured bytes
then the `$AS1` signature block). The build finds the image by the GUID in
`AMD_SL_HDR_V1` (AMD publication 58453, appendix B.3), checks the signature
block is where the header says, and prints the offset and key ID it found:

```sh
DRTM_FOREIGN_SL=path/to/amddrtm.dll cargo build
```

Such a build accepts `launch=foreign` in `drtm.conf`. It takes the PSP path
or blocks, since the SL issues `LAUNCH` itself. A build without the variable
reports `launch=foreign` as a `config` warning and keeps the default.

The run's last `drtm:` record is `launch.skinit`, with
`launch.foreign_parameters` before it naming where everything was placed.
From then on the SL owns the machine and reports only through POST codes on
port `0x80`.

## What the SL expects

Appendix B.3 of 58453 is the only public description: `TotalLength` at
header offset `0x18` covers the image and its signature block, and the OS
loader puts the SL's parameters at `SLB + TotalLength`. The layouts below come
from the SL itself, its strings and its code.

`AmdSlInputParameters`, `0x4c` bytes:

| Offset | Size | Field                  | What `drtm` puts there              |
|--------|------|------------------------|-------------------------------------|
| `0x00` | 4    | `Version`              | 1                                   |
| `0x04` | 4    | `Size`                 | `0x4c`                              |
| `0x08` | 8    | `OsSlBlockBase`        | a 1 MiB block on a 64 MiB boundary  |
| `0x10` | 8    | `OsSlBlockSize`        | 1 MiB                               |
| `0x18` | 8    | `OsSlImageOffset`      | 0, within the block                 |
| `0x20` | 8    | `OsSlImageSize`        | `0x1000`                            |
| `0x28` | 8    | `OsSlParametersOffset` | `0x1000`                            |
| `0x30` | 8    | `OsSlParametersSize`   | the rest of the block               |
| `0x38` | 4    | `ApStartupBase`        | 16 KiB below 1 MiB                  |
| `0x3c` | 4    | `ApStartupSize`        | 16 KiB                              |
| `0x40` | 8    | `AmdSlRsvdMemBase`     | 2 MiB for its heap and page tables  |
| `0x48` | 4    | `AmdSlRsvdMemSize`     | 2 MiB                               |

None of these may overlap the SLB, and after `LAUNCH` the SL requires each to
lie inside a TMR. It measures the OS SL image with `EXTEND_OSSL_DIGEST`,
which wants the block 64 MiB aligned. It fills the parameters area with a
header of its own (memory map, TMR descriptors, TCG log, ACPI tables, SMM
configuration) and then jumps to the OS SL.

The OS SL image carries an `OS_SL_DESCRIPTOR_V1` the SL finds by its GUID,
`e5c70468-8342-1a1d-d0cc-d39812881d48` in memory order:

| Offset | Size | Field           |
|--------|------|-----------------|
| `0x00` | 16   | GUID            |
| `0x10` | 4    | `Version`, 1    |
| `0x14` | 8    | `EntryOffset`   |
| `0x1c` | 8    | `ApEntryOffset` |

`drtm`'s OS SL is a stand-in: its entry writes `0xd7d70001` to port `0x80`
and halts, and its AP entry halts.

## Reading the POST codes

The SL's codes are dwords `0xb000NNNN`. The ones that matter:

| Code                       | Meaning                                     |
|----------------------------|---------------------------------------------|
| `0xb0001000`, `0xb0001001` | the SL is running                           |
| `0xb00018CC`               | about to wait for the mailbox, command `CC` |
| `0xb00019CC`               | command `CC` sent                           |
| `0xb0001aCC`               | command `CC` answered                       |
| `0xb000100c`               | `LAUNCH` accepted the signature             |
| `0xb0001eNN`               | step `NN` failed                            |
| `0xb000100a`               | about to jump to the OS SL                  |
| `0xd7d70001`               | `drtm`'s stand-in was reached               |

A fatal error does not write a POST code. The SL stores its `0xdeadNNNN`
code at `0xfed10060` and resets the machine through port `0xcf9`. So a
reboot right after `0xb0001a04` means `LAUNCH` was refused.

The commands it sends, in order: `LAUNCH`, `EXTEND_OSSL_DIGEST`,
`GET_TMR_DESCRIPTORS` four times, `ALLOCATE_SHARED_MEMORY`, `PROCESS_SMM`,
`GET_SMM_POLICY`, `GET_TCG_LOGS`. `PROCESS_SMM` and `GET_SMM_POLICY` are
client only (58453, appendix H.2). Server firmware refuses them and the SL
treats that as fatal, so it cannot complete on a server even where the
firmware accepts its key.

## Under QEMU

`qemu-tests/test_foreign_sl_amd.py` boots a build embedding the SL
`DRTM_TEST_FOREIGN_SL` names, and is skipped without it:

```sh
DRTM_TEST_FOREIGN_SL=path/to/amddrtm.dll \
    uv run --project qemu-tests pytest qemu-tests/test_foreign_sl_amd.py
```

The tests check the stand-in was reached, every command was answered in the
order above, and the PSP model verified the signature. The boot needs four
things from our QEMU beyond a PSP launch:

- `isa-post-code`, which logs every write to ports `0x80` to `0x83` whole,
    since the SL writes dwords.
- `amd-fch-pmio`, the FCH's `AcpiPmTmrBlk` register at `0xfed80364`, which
    the SL reads to find the PM timer it times delays on.
- `mmio-conf-base` on the CPU, since the SL finds PCI configuration space
    through `MSR_AMD64_MMIO_CONF_BASE`, which firmware always sets.
- `client-smm=on` on `amd-psp`, which answers `PROCESS_SMM` and
    `GET_SMM_POLICY` with zero hashes and an empty policy. That is only what
    the SL accepts, not what client firmware returns.
