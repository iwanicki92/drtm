<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Launching a foreign Secure Loader

A debug build of `drtm` can embed a [Secure Loader][glossary] (SL) signed by
someone else and launch it through the [PSP DRTM service][glossary] in place of
its own. The main use is running AMD's own SLs. AMD has one for client platforms
and one for server platforms, and it names the server one for EPYC. Both are
signed with the same AMD key. Each SL runs only on CPUs it knows, and those
differ between SL versions. Newer client firmware and some server firmware
accept only SLs signed with certain keys, and `drtm`'s key is never one of them.
On that firmware AMD's SL is the only one known to launch. On any firmware,
running AMD's SL shows whether the platform's DRTM works at all, separately from
anything `drtm` does.

The idea of using the Windows SL as a shim comes from the talk [A Practical
Security Assessment of DRTM on AMD-based Systems][talk] by Christian Werling and
Julian Prein ([slides][talk-slides]). The talk proposes a shim and reuse of the
SL that Windows launches as ways to live with the key check.

## Where AMD's SLs come from

Neither is part of any BIOS image. The firmware only carries the [DRTM
TA][glossary], which holds any list of accepted keys. Both SLs ship inside
Windows drivers, as `amddrtm.dll`.

### Client

Lenovo publishes it inside the Windows driver package "AMD DRTM Boot Driver for
Windows", for the ThinkPad L14 Gen 5 (types 21L5, 21L6) and L16 Gen 1 (types
21L7, 21L8). The driver binds `ACPI\DRTM0001`.

- Package: `r2idt05w.exe`, version 1.0.19.4, SHA-256
    `db09603d000e2eb8bbb55c7bba4103cbcccb713e640ea713dd3935cf865f680b`.
- The DLL inside it: `amddrtm.dll`, SHA-256
    `4d12a07f18dbcb13a2a7bc9496c88fc7f20c5c83fa1eef7a5fa4cff63626d903`.
- The SL in the DLL: offset `0x32c0`, `0x9ed0` bytes measured.
- The key ID in its token: `024ca9a5c32f41008aa8dd84c88b8dfc`.

The package is an Inno Setup installer. `innoextract r2idt05w.exe` unpacks
`amddrtm.dll` without running anything.

Other Lenovo AMD client packages carry the same key but not always the same SL.
The 1.0.19.x DLLs for other models have the same layout with a different body,
and 1.0.16.1 has a shorter one (`0x8800` bytes measured).

### Server

The "AMD EPYC DRTM Boot Driver" binds `ACPI\DRTM0002`, the server DRTM device.
Lenovo publishes it for ThinkSystem servers as plain zips in its Windows Server
driver repository (`windows-server.lenovo.com`), versions 1.10.1.0 to 1.10.4.9.
The newest:

- Package: `sysboard_chipset.sp6t-amddrtm-1.10.4.9-016.zip`, SHA-256
    `b7b26857090b418df715ae3f9baaf306a5dd9d94601a8ad8428bb27f5a231da1`.
- The DLL inside it: `amddrtm.dll`, SHA-256
    `e4e468994b7e9e3bd0775004b99767cfccccdb6d578455d60a478120b74e60ee`.
- The SL in the DLL: offset `0x34c0`, `0x9270` bytes measured.
- The key ID in its token: the client SL's, `024ca9a5c32f41008aa8dd84c88b8dfc`.

From 1.10.4.1 on, the SL has no SMM step, as [Reading the POST
codes](#reading-the-post-codes) explains. In the `$AS1` signature header of
these SLs, `SizeFWSigned` gives the size of the image only. The signature still
covers both the image and the header, as it does in every other SL.

## Building and running

`DRTM_FOREIGN_SL` gives the path to either the DLL or a bare image (the measured
bytes then the `$AS1` signature block). In a DLL, the build finds the SL by
searching for the fixed GUID in its header, a format that AMD publication 58453
calls `AMD_SL_HDR_V1` (appendix B.3).

The SL's header gives the length of its measured bytes, and the signature block
must come right after them, so the build fails if it isn't there. A successful
build prints where the SL starts in the file and the ID of the key that signed
it. To build with the DLL, run:

```sh
DRTM_FOREIGN_SL=path/to/amddrtm.dll cargo build
```

`DRTM_FOREIGN_SL_RESIGN=1` replaces the signature block with one made with
`drtm`'s key (`DRTM_SL_KEY`, or the test key), keeping the header's
`SecPatchLevel`. The code is AMD's and only the key differs, so a `LAUNCH`
refused there but accepted with AMD's signature points at a key allowlist.

A build with `DRTM_FOREIGN_SL` set accepts `launch=foreign` in `drtm.conf`. That
setting always uses the PSP path, because the SL issues `LAUNCH` itself. If the
PSP path is not available, the gate blocks the launch. A build without the
variable reports `launch=foreign` as a `config` warning and keeps the default.

Before `SKINIT` the preamble prints `launch.foreign_parameters`, naming where
everything was placed. From then on the SL owns the machine and reports only
through POST codes on port `0x80`, until it jumps to `drtm`'s [DLME][glossary],
which reports on the sinks like the rest of `drtm`.

## What the SL expects

AMD's SL measures a second image, which AMD calls the OS SL, and jumps to it
when it finishes. In `drtm`, the OS SL is the DLME, the same one its own Secure
Loaders hand off to.

Appendix B.3 of 58453 is the only public description: `TotalLength` at header
offset `0x18` covers the image and its signature block, and the OS loader puts
the SL's parameters at `SLB + TotalLength`. The layouts below come from the SL
itself, its strings and its code.

The SL also has to know the CPU. The SL's header has a pointer at offset `0x1c`
to a list of the CPUs it supports. EPYC 1.10.4.9 looks up the PSP mailbox's
offset by CPUID, using the same values as that list. On any other CPU it polls a
wrong address until it gives up, writes the POST code `0xb0001e08` (see [Reading
the POST codes](#reading-the-post-codes)) and resets.

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
| `0x38` | 4    | `ApStartupBase`        | a 16 KiB area anywhere below 1 MiB  |
| `0x3c` | 4    | `ApStartupSize`        | 16 KiB                              |
| `0x40` | 8    | `AmdSlRsvdMemBase`     | its heap, page tables and stacks    |
| `0x48` | 4    | `AmdSlRsvdMemSize`     | 2 MiB                               |

None of the memory ranges in this table may overlap the SLB. After `LAUNCH`, the
SL requires each range to lie inside a [TMR][glossary]. It measures the OS SL
image with `EXTEND_OSSL_DIGEST`, which wants the block 64 MiB aligned. It fills
the parameters area with a header of its own and the tables that header points
to, as described below. Then it jumps to the OS SL.

The OS SL image carries an `OS_SL_DESCRIPTOR_V1` the SL finds by its GUID,
`e5c70468-8342-1a1d-d0cc-d39812881d48` in memory order:

| Offset | Size | Field           |
|--------|------|-----------------|
| `0x00` | 16   | GUID            |
| `0x10` | 4    | `Version`, 1    |
| `0x14` | 8    | `EntryOffset`   |
| `0x1c` | 8    | `ApEntryOffset` |

The block holds the DLME's image, its parameter page and scratch memory, and the
rest is the parameters area. The SL jumps to the entry with `jmp edx`, using its
own flat 32-bit segments with paging off. At that point `EDX` holds the entry
address and `EAX` holds the SL's SLB address. The DLME writes `0xd7d70001` to
port `0x80` first, then finds its own address, and uses neither register. Its AP
entry halts.

## What the SL leaves the OS SL

The header at the start of the parameters area, read from the code that prints
it. Offsets of the tables are from the header, and `Size` covers the header and
its tables. The client SL writes version 4, a `0x7c`-byte header. The EPYC SL
writes version 2, whose tables start at `0x54` under QEMU. The fields up to
`0x40` read the same in both.

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

Memory and TMR descriptors are `0x20` bytes each: a version of 1, a type (2 for
a TMR), a size of `0x20`, one more word (a TMR's index, 1 in the memory
descriptors seen so far), then a 64-bit base and a 64-bit length in bytes. The
log is a copy of the one `GET_TCG_LOGS` returns.

The DLME reports the header (`launch.amd_handoff`), each TMR (`launch.amd_tmr`),
each memory descriptor at `debug`, and whether the log replays to PCR 17 and 18
(`launch.amd_log`), all as `info`, since the layout is not published.

PCR 18 is `info` too, because another SL may extend it further. The record says
whether PCR 18 matches the value expected from the [key token][glossary] and the
`$AS1` header's `SecPatchLevel`, extended with the DLME the way
`EXTEND_OSSL_DIGEST` does. PCR 17 is only reported, because its last extend uses
words that `LAUNCH` returned to the SL.

The DLME then sends `TMR_RELEASE` and runs the same interrupt probe as after
`drtm`'s own PSP launch (see [The DLME](design.md#the-dlme)). The summary counts
only the DLME's records, since AMD's SL passes on no [tally][glossary].

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

A fatal error does not write a POST code. The SL stores its `0xdeadNNNN` code at
`0xfed10060` and resets the machine through port `0xcf9`. So a reboot right
after `0xb0001a04` (command `04` answered) means `LAUNCH`, which is command
`04`, was refused.

The commands the client SL sends, in order: `LAUNCH`, `EXTEND_OSSL_DIGEST`,
`GET_TMR_DESCRIPTORS` four times, `ALLOCATE_SHARED_MEMORY`, `PROCESS_SMM`,
`GET_SMM_POLICY`, `GET_TCG_LOGS`. `PROCESS_SMM` and `GET_SMM_POLICY` are client
only (58453, appendix H.2). Server firmware refuses them and the SL treats that
as fatal, so it cannot complete on a server even where the firmware accepts its
key.

The EPYC SLs send the other commands in the same order. Whether they also send
`ALLOCATE_SHARED_MEMORY`, `PROCESS_SMM` and `GET_SMM_POLICY` depends on the
version:

- From 1.10.4.1 on, nothing calls these three commands.
- 1.10.1.0 and 1.10.2.0 send all three unless the CPU is one they skip, as the
    table below shows.

The CPUs in the table are family and model. The SL compares the family and the
model's high digit, so 19h/10h covers models 10h to 1Fh.

| EPYC SL  | Skips the three commands on        | Other CPUs on its list             |
|----------|------------------------------------|------------------------------------|
| 1.10.1.0 | 19h/00h                            | 17h/60h, 19h/50h                   |
| 1.10.2.0 | 19h/00h, 19h/10h, 19h/80h, 19h/A0h | 17h/60h, 19h/40h, 19h/50h, 19h/70h |

The client SL's list has every CPU in the last column as well. The table comes
from reading the code, because no boot has reached this step on those CPUs.

## Under QEMU

`qemu-tests/test_foreign_sl_amd.py` boots a build embedding the SL given in
`DRTM_TEST_FOREIGN_SL`, and is skipped without it:

```sh
DRTM_TEST_FOREIGN_SL=path/to/amddrtm.dll \
    uv run --project qemu-tests pytest qemu-tests/test_foreign_sl_amd.py
```

The tests check that the DLME was reached, every command was answered in the
order listed above for that SL, and the PSP model verified the signature. They
also check the DLME's report: PCR 18 against a value the tests compute from the
SL, `TMR_RELEASE`, the probe, the TMR and log the SL left, and a passing
summary. A boot that never reaches the DLME is still judged by its POST codes.
Under QEMU, on the `EPYC-Genoa` CPU these tests use, the client SL 1.0.19.4 and
the EPYC SLs 1.10.4.1 to 1.10.4.9 all pass. The EPYC 1.10.1.0 doesn't list that
CPU, and on the CPUs it lists it stops at `0xb0001a09`, short of the DLME,
because our QEMU doesn't model the Data Fabric registers it reads. The EPYC
1.10.2.0 usually stops at `0xb0001002`, also short of the DLME: it starts the
second CPU, then gives up waiting for it. Why that CPU doesn't finish is not
known. The boot needs four things from our QEMU beyond a PSP launch:

- `isa-post-code`, which logs every write to ports `0x80` to `0x83` whole, since
    the SL writes dwords.
- `amd-fch-pmio`, the FCH's `AcpiPmTmrBlk` register at `0xfed80364`, which the
    SL reads to find the PM timer it times delays on.
- `mmio-conf-base` on the CPU, since the SL finds PCI configuration space
    through `MSR_AMD64_MMIO_CONF_BASE`, which firmware always sets.
- `client-smm=on` on `amd-psp`, which answers `PROCESS_SMM` and `GET_SMM_POLICY`
    with zero hashes and an empty policy. That is only what the client SL
    accepts, not what client firmware returns. The EPYC SLs don't send them on
    `EPYC-Genoa`.

[glossary]: design.md#glossary
[talk]: https://cfp.3mdeb.com/bsmconf/talk/ZJWHCU/
[talk-slides]: https://cfp.3mdeb.com/media/bsmconf/submissions/ZJWHCU/resources/20260924-drtm-on-amd-slides_5B8J8mF.pdf
