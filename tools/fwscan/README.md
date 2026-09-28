<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# fwscan

Looks for an SL key allowlist in the PSP DRTM TA of a firmware image. The
PSP DRTM service verifies the Secure Loader's signature. When its TA also
compares the SL's key token against digests it carries, an SL signed with
any other key, such as `drtm`'s, is expected to be refused.

## Usage

```sh
uv run fwscan IMAGE... [--sl FILE] [--json] [--no-hid]
```

For each image fwscan lists the DRTM TAs (PSP directory entry type 0x47),
the token digests each plaintext TA compares against, and the known keys
those digests belong to. `--sl` adds the key of an AMD-signed SL (anything
with a `$AS1` signature header, e.g. `amddrtm.dll` from a Windows DRTM
driver package). It also prints the `_HID` of the DRTM device in the ACPI
tables, which `--no-hid` skips.

Verdicts, the exit status being the worst over all images:

| Exit | Verdict        | Meaning                                                  |
| ---- | -------------- | -------------------------------------------------------- |
| 0    | `no-drtm-ta`   | no DRTM TA in the image                                  |
| 2    | `no-allowlist` | the TA only compares digests of non-SL keys              |
| 3    | `encrypted`    | the TA is encrypted, its key checks cannot be read       |
| 4    | `inconclusive` | plaintext TA, but no allowlist code found in it          |
| 5    | `allowlist`    | the TA only takes SL keys whose token digest it lists    |
| 1    |                | an error, e.g. psptool failed on the image               |

No verdict says an SL is accepted: without an allowlist the service can
still refuse an SL for other reasons.

## How it works

`psptool -E -j` lists the PSP directory entries and their offsets in the
image. fwscan runs it as a separate program and reads that JSON, then
parses each TA's `$PS1` header itself (encrypted and compressed flags,
signing key ID, version).

The allowlist is code, not a table. A plaintext TA (Thumb-2, loaded at
0xD00000 with its header) hashes the 0x440-byte key token with a
GlobalPlatform TEE digest, then compares the result with a stored one:

```text
ldr   rA, =0x50000004           ; TEE_ALG_SHA256, or 0x50000005 SHA-384
mov.w r1, #0x440                ; token length
bl    digest
ldr   rP, =stored digest
cmp   rI, #0x20                 ; compare loop, #0x30 for SHA-384
```

Older TAs load both constants as a `movw` and `movt` pair, and some set r1
to 0x440 just before the algorithm load. fwscan finds the sequence in
either form with capstone and reads the digest the pointer points at, so
digests of unknown keys are found as well.

Not every such compare is about the SL. The SHA-384 of AMD key
`D2DBD452...`, found in every TA from 4.11.0.19 on, sits on a separate
path that quite likely checks the SMM supervisor package in the BIOS,
whose SMI entry code that key signs. fwscan names it but does not count it
as an allowlist entry, so a TA with only that digest is `no-allowlist`. A
digest of an unknown key still counts.

## psptool

psptool comes from a fork pinned in `pyproject.toml`, which parses client
ROMs (64 MB flash, direct L2 pointers) that upstream does not yet.
`--psptool` runs another one instead. psptool is GPL-3.0: fwscan only runs
it, it never imports it.

## Tests

```sh
uv run pytest
```

The tests use synthetic TAs and images. Vendor firmware is never committed.
