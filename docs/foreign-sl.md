<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Launching a foreign Secure Loader

A debug build of `drtm` can embed a Secure Loader signed by someone else and
launch it through the PSP DRTM service in place of its own. The point is AMD's
own SLs, one for clients and one for EPYC, both signed with the same AMD key.
Client PSP firmware takes only SLs signed with keys its DRTM TA lists, which
`drtm`'s key never is, so on a client machine AMD's SL is the only one known
to launch. On either kind it shows whether the platform's DRTM works at all,
separately from anything `drtm` does.

The idea of using the Windows SL as a shim comes from the talk
[A Practical Security Assessment of DRTM on AMD-based Systems][talk] by
Christian Werling and Julian Prein ([slides][talk-slides]). The talk proposes a
shim and reuse of the SL that Windows launches as ways to live with the key
check.

## Where AMD's SLs come from

Neither is part of any BIOS image. The firmware only carries the DRTM TA with
its list of accepted keys. Both SLs ship inside Windows drivers, as
`amddrtm.dll`.

### Client

Lenovo publishes it inside the Windows driver package "AMD DRTM Boot Driver
for Windows", for the ThinkPad L14 Gen 5 (types 21L5, 21L6) and L16 Gen 1
(types 21L7, 21L8). The driver binds `ACPI\DRTM0001`.

- Package: `r2idt05w.exe`, version 1.0.19.4, SHA-256
    `db09603d000e2eb8bbb55c7bba4103cbcccb713e640ea713dd3935cf865f680b`.
- The DLL inside it: `amddrtm.dll`, SHA-256
    `4d12a07f18dbcb13a2a7bc9496c88fc7f20c5c83fa1eef7a5fa4cff63626d903`.
- The SL in the DLL: offset `0x32c0`, `0x9ed0` bytes measured.
- The key ID in its token: `024ca9a5c32f41008aa8dd84c88b8dfc`.

The package is an Inno Setup installer. `innoextract r2idt05w.exe` unpacks
`amddrtm.dll` without running anything.

Other Lenovo AMD client packages carry the same key but not always the same
SL. The 1.0.19.x DLLs for other models have the same layout with a different
body, and 1.0.16.1 has a shorter one (`0x8800` bytes measured).

### Server

The "AMD EPYC DRTM Boot Driver" binds `ACPI\DRTM0002`, the server DRTM
device. Lenovo publishes it for ThinkSystem servers as plain zips in its
Windows Server driver repository (`windows-server.lenovo.com`), versions
1.10.1.0 to 1.10.4.9. The newest:

- Package: `sysboard_chipset.sp6t-amddrtm-1.10.4.9-016.zip`, SHA-256
    `b7b26857090b418df715ae3f9baaf306a5dd9d94601a8ad8428bb27f5a231da1`.
- The DLL inside it: `amddrtm.dll`, SHA-256
    `e4e468994b7e9e3bd0775004b99767cfccccdb6d578455d60a478120b74e60ee`.
- The SL in the DLL: offset `0x34c0`, `0x9270` bytes measured.
- The key ID in its token: the client SL's, `024ca9a5c32f41008aa8dd84c88b8dfc`.

From 1.10.4.1 on, the SL leaves SMM alone. Its `$AS1` header also gives
`SizeFWSigned` as the image alone, where its signature covers the image and
the header, as every other SL's does.

## Building and running

`DRTM_FOREIGN_SL` names either the DLL or a bare image (the measured bytes
then the `$AS1` signature block). The build finds the image by the GUID in
`AMD_SL_HDR_V1` (AMD publication 58453, appendix B.3), checks the signature
block is where the header says, and prints the offset and key ID it found:

```sh
DRTM_FOREIGN_SL=path/to/amddrtm.dll cargo build
```

`DRTM_FOREIGN_SL_RESIGN=1` replaces the signature block with one made with
`drtm`'s key (`DRTM_SL_KEY`, or the test key), keeping the header's
`SecPatchLevel`. The code is AMD's and only the key differs, so a `LAUNCH`
refused there but accepted with AMD's signature points at a key allowlist.

Such a build accepts `launch=foreign` in `drtm.conf`. It takes the PSP path
or blocks, since the SL issues `LAUNCH` itself. A build without the variable
reports `launch=foreign` as a `config` warning and keeps the default.

Before `SKINIT` the preamble prints `launch.foreign_parameters`, naming
where everything was placed. From then on the SL owns the machine and
reports only through POST codes on port `0x80`, until it jumps to `drtm`'s
DLME, which reports on the sinks like the rest of `drtm`.

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
| `0x20` | 8    | `OsSlImageSize`        | the DLME's measured length          |
| `0x28` | 8    | `OsSlParametersOffset` | the end of the DLME's scratch       |
| `0x30` | 8    | `OsSlParametersSize`   | the rest of the block               |
| `0x38` | 4    | `ApStartupBase`        | 16 KiB below 1 MiB                  |
| `0x3c` | 4    | `ApStartupSize`        | 16 KiB                              |
| `0x40` | 8    | `AmdSlRsvdMemBase`     | 2 MiB for its heap and page tables  |
| `0x48` | 4    | `AmdSlRsvdMemSize`     | 2 MiB                               |

None of these may overlap the SLB, and after `LAUNCH` the SL requires each to
lie inside a TMR. It measures the OS SL image with `EXTEND_OSSL_DIGEST`,
which wants the block 64 MiB aligned. It fills the parameters area with a
header of its own and the tables it names, described below, and then jumps
to the OS SL.

The OS SL image carries an `OS_SL_DESCRIPTOR_V1` the SL finds by its GUID,
`e5c70468-8342-1a1d-d0cc-d39812881d48` in memory order:

| Offset | Size | Field           |
|--------|------|-----------------|
| `0x00` | 16   | GUID            |
| `0x10` | 4    | `Version`, 1    |
| `0x14` | 8    | `EntryOffset`   |
| `0x1c` | 8    | `ApEntryOffset` |

`drtm`'s OS SL is its DLME, the one its own Secure Loaders hand off to. The
block holds its image, its parameter page and scratch memory, and the rest
is the parameters area. The SL jumps to the entry with `jmp edx`, in its
own flat 32-bit segments with paging off, `EDX` the entry and `EAX` its
SLB. The DLME writes `0xd7d70001` to port `0x80` first, then finds its own
address and needs nothing else from those registers. Its AP entry halts.

## What the SL leaves the OS SL

The header at the start of the parameters area, read from the code that
prints it. Offsets of the tables are from the header, and `Size` covers
the header and its tables. The client SL writes version 4, a `0x7c`-byte
header. The EPYC SL writes version 2, whose tables start at `0x54` under
QEMU. The fields up to `0x40` read the same in both.

| Offset | Size  | Field                                      |
|--------|-------|--------------------------------------------|
| `0x00` | 4     | `Version`                                  |
| `0x04` | 4     | `Size`                                     |
| `0x08` | 8     | `EnabledFeatures`                          |
| `0x10` | 4     | `MemoryDescriptorsOffset`                  |
| `0x14` | 4     | `MemoryDescriptorsCount`                   |
| `0x18` | 4     | `TmrDescriptorsOffset`                     |
| `0x1c` | 4     | `TmrDescriptorsCount`                      |
| `0x20` | 4     | `TcgLogOffset`                             |
| `0x24` | 4     | `TcgLogSize`                               |
| `0x28` | 4     | `AcpiTablesOffset`                         |
| `0x2c` | 4     | `AcpiTablesSize`                           |
| `0x30` | 4     | `SmmConfigurationOffset`                   |
| `0x34` | 4     | `SmmConfigurationSize`                     |
| `0x38` | 8     | `DrtmTaCommandRegister`, the mailbox       |
| `0x40` | 4     | `ReleaseTmrCmd`, `0x30000`                 |
| `0x44` | 4     | `LocalityAccessCmd`, `0x80000`             |
| `0x48` | 4 × 2 | `ErrorLog` port, data port, index and size |
| `0x50` | 4     | `EnableSmmCmd`                             |

Memory and TMR descriptors are `0x20` bytes each: a version of 1, a type
(2 for a TMR), a size of `0x20`, one more word (a TMR's index, 1 in the
memory descriptors seen so far), then a 64-bit base and a 64-bit length in
bytes. The log is a copy of the one `GET_TCG_LOGS` names.

The DLME reports the header (`launch.amd_handoff`), each TMR
(`launch.amd_tmr`), each memory descriptor at `debug`, and whether the log
replays to PCR 17 and 18 (`launch.amd_log`), all as `info`, since the layout
is not published. PCR 18 is `info` too, with whether it matches the token
and the `$AS1` header's `SecPatchLevel`, extended with the DLME as
`EXTEND_OSSL_DIGEST` does: another SL may extend more. PCR 17 is only
reported, since its last extend takes the words `LAUNCH` returned to the
SL. Then it sends `TMR_RELEASE` and runs the probe as after `drtm`'s own
PSP launch. The summary counts the DLME's records alone, since AMD's SL
passes on no tally.

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
| `0xd7d70001`               | `drtm`'s DLME was reached                   |

A fatal error does not write a POST code. The SL stores its `0xdeadNNNN`
code at `0xfed10060` and resets the machine through port `0xcf9`. So a
reboot right after `0xb0001a04` means `LAUNCH` was refused.

The commands the client SL sends, in order: `LAUNCH`, `EXTEND_OSSL_DIGEST`,
`GET_TMR_DESCRIPTORS` four times, `ALLOCATE_SHARED_MEMORY`, `PROCESS_SMM`,
`GET_SMM_POLICY`, `GET_TCG_LOGS`. `PROCESS_SMM` and `GET_SMM_POLICY` are
client only (58453, appendix H.2). Server firmware refuses them and the SL
treats that as fatal, so it cannot complete on a server even where the
firmware accepts its key. The EPYC SL sends the same commands without
`ALLOCATE_SHARED_MEMORY`, `PROCESS_SMM` and `GET_SMM_POLICY`.

## Under QEMU

`qemu-tests/test_foreign_sl_amd.py` boots a build embedding the SL
`DRTM_TEST_FOREIGN_SL` names, and is skipped without it:

```sh
DRTM_TEST_FOREIGN_SL=path/to/amddrtm.dll \
    uv run --project qemu-tests pytest qemu-tests/test_foreign_sl_amd.py
```

The tests check the DLME was reached, every command was answered in either
SL's order above, the PSP model verified the signature, and the DLME's
report: PCR 18 against a value the tests compute from the SL, `TMR_RELEASE`,
the probe, the TMR and log the SL left, and a passing summary. A boot that
never reaches the DLME is still judged by its POST codes. Under QEMU the
client SL 1.0.19.4 and the EPYC SLs 1.10.4.1 to 1.10.4.9 all pass. The EPYC
1.10.1.0 and 1.10.2.0 do not reach the DLME, and why is not known.
The boot needs four things from our QEMU beyond a PSP launch:

- `isa-post-code`, which logs every write to ports `0x80` to `0x83` whole,
    since the SL writes dwords.
- `amd-fch-pmio`, the FCH's `AcpiPmTmrBlk` register at `0xfed80364`, which
    the SL reads to find the PM timer it times delays on.
- `mmio-conf-base` on the CPU, since the SL finds PCI configuration space
    through `MSR_AMD64_MMIO_CONF_BASE`, which firmware always sets.
- `client-smm=on` on `amd-psp`, which answers `PROCESS_SMM` and
    `GET_SMM_POLICY` with zero hashes and an empty policy. That is only what
    the client SL accepts, not what client firmware returns. The EPYC SL
    never sends them.

[talk]: https://cfp.3mdeb.com/bsmconf/talk/ZJWHCU/
[talk-slides]: https://cfp.3mdeb.com/media/bsmconf/submissions/ZJWHCU/resources/20260924-drtm-on-amd-slides_5B8J8mF.pdf
