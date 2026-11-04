<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# fwscan

fwscan checks whether an AMD firmware image restricts which Secure Loaders (SLs)
its PSP DRTM service will launch. The service runs as a trusted application (the
DRTM TA) on the AMD Secure Processor, and it verifies the SL's signature. Some
TAs also compare a digest of the SL's key token with digests stored in the TA.
An SL signed with a key that has no stored digest, such as `drtm`'s, is expected
to be refused. The [glossary][glossary] explains these terms.

## Usage

```sh
uv run --project tools/fwscan fwscan IMAGE... [--sl FILE] [--json] [--no-hid]
```

For each image fwscan lists the DRTM TAs (PSP directory entry type 0x47), the
token digests each plaintext TA compares against, and the known keys those
digests belong to. `--sl FILE` adds the key of an AMD-signed SL to the keys
fwscan recognises. FILE is anything with a `$AS1` signature header, for example
`amddrtm.dll` from a Windows DRTM driver package. fwscan also prints the `_HID`
of the DRTM device in the ACPI tables, unless `--no-hid` is set.

fwscan gives each image one of the verdicts below. Its exit status is the
highest number among the images, and an error stops it with exit status 1.

| Exit | Verdict        | Meaning                                                  |
| ---- | -------------- | -------------------------------------------------------- |
| 0    | `no-drtm-ta`   | no DRTM TA in the image                                  |
| 2    | `no-allowlist` | the TA only compares digests of non-SL keys              |
| 3    | `encrypted`    | the TA is encrypted, its key checks cannot be read       |
| 4    | `inconclusive` | plaintext TA, but no allowlist code found in it          |
| 5    | `allowlist`    | the TA only takes SL keys whose token digest it lists    |
| 1    |                | an error, e.g. psptool failed on the image               |

No verdict says an SL is accepted: without an allowlist the service can still
refuse an SL for other reasons.

## How it works

fwscan runs [psptool](#psptool), a tool for AMD PSP firmware, as a separate
program. `psptool -E -j` prints the PSP directory entries and their offsets as
JSON, and fwscan reads that output. It then parses each TA's `$PS1` header
itself (encrypted and compressed flags, signing key ID, version).

The allowlist is code, not a table. A plaintext TA (Thumb-2, loaded at 0xD00000
with its header) hashes the 0x440-byte key token with a GlobalPlatform TEE
digest, then compares the result with a stored one:

```text
ldr   rA, =0x50000004           ; TEE_ALG_SHA256, or 0x50000005 SHA-384
mov.w r1, #0x440                ; token length
bl    digest
ldr   rP, =stored digest
cmp   rI, #0x20                 ; compare loop, #0x30 for SHA-384
```

Older TAs load the algorithm and the stored digest's address as `movw` and
`movt` pairs, and some set r1 to 0x440 just before the algorithm load. fwscan
finds the sequence in either form with the Capstone disassembler and reads the
digest the pointer points at, so digests of unknown keys are found as well.

Not every such compare is about the SL. The SHA-384 of AMD key `D2DBD452...`,
found in every TA from 4.11.0.19 on, sits on a separate path that quite likely
checks the SMM supervisor package in the BIOS, whose SMI entry code that key
signs. fwscan names it but does not count it as an allowlist entry, so a TA with
only that digest is `no-allowlist`. A digest of an unknown key still counts.

## psptool

fwscan needs psptool 3.7 or later, the first release that parses client ROMs
with 64 MB flash or direct L2 pointers. `--psptool` runs another copy instead.
psptool is GPL-3.0: fwscan only runs it, it never imports it.

## Tests

```sh
uv run --project tools/fwscan pytest tools/fwscan
```

The tests use synthetic TAs and images. Vendor firmware is never committed.

[glossary]: ../../docs/design.md#glossary
