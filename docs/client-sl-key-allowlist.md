<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Client SL key allowlist

On client AMD platforms the PSP DRTM service checks which key signed the
Secure Loader. Firmware whose DRTM TA is version 4.11.0.33 or later most
likely accepts only SLs signed with AMD's own key, so `drtm`'s SL is expected
to be refused there. This document explains the check, lists the TA versions
and says how to check a firmware image. Every release checked is listed in
[releases][releases].

## What the allowlist is

The PSP runs its DRTM service as a trusted application, the DRTM TA (PSP
directory entry type `0x47`). When the host launches an SL, the TA hashes
the `0x440`-byte key token of the key that signed it. Newer TAs then compare
that digest with one they carry in their code: the SHA-256 of the token of
the key that signs AMD's Windows SL (key ID
`024ca9a5c32f41008aa8dd84c88b8dfc`, see [foreign SL][foreign-sl]). A key
with any other token, such as `drtm`'s, would fail the compare. Server
firmware has no such check.

TAs also carry a second digest, the SHA-384 of AMD key `D2DBD452...`. It is
compared on a separate code path and is not an SL key: that key signs the
SMI entry code of the SMM supervisor package that many client BIOS images
carry. So it is quite likely the TA's check of that package. It does not
restrict the SL.

## TA versions

| DRTM TA                | SL key token digest                | `D2DBD452` digest | fwscan verdict |
| ---------------------- | ---------------------------------- | ----------------- | -------------- |
| 0.7.0.1, 4.11.0.14     | not compared                       | no                | `inconclusive` |
| 4.11.0.19 to 4.11.0.30 | not compared                       | yes               | `no-allowlist` |
| 4.11.0.33 to 4.11.0.39 | compared with the Windows SL key's | yes               | `allowlist`    |
| 4.2D.0.33 to 4.2D.0.41 | compared with the Windows SL key's | yes               | `allowlist`    |

"Not compared" comes from reading the code of 0.7.0.1, 4.11.0.14 and
4.11.0.2F: the TA hashes the token, keeps the digest and goes on with the
launch. In the other versions up to 4.11.0.30, fwscan finds the `D2DBD452`
compare but none for the SL key token. No verdict has been tried on
hardware.

The Windows SL key is the only entry in every allowlist found. No encrypted
DRTM TA turned up.

## What it means for `drtm`

- `allowlist`: the TA most likely has an SL key allowlist, so `drtm`'s own
    SL is expected to be refused. A debug build can still launch AMD's
    client SL to check that the platform's DRTM works at all, see
    [foreign SL][foreign-sl].
- `no-allowlist` and `inconclusive`: nothing found in the TA restricts the
    SL key, so `drtm`'s own SL is quite likely accepted. Most of these are
    older releases of models whose later releases carry the allowlist, and
    anti-rollback may block a downgrade to them (not checked).
- `no-drtm-ta`: the firmware quite likely offers no DRTM at all.

## Checking a firmware image

[fwscan] reads a BIOS image or an SPI flash dump, finds the DRTM TAs in it
and prints the digests each one compares and the verdict. For a release in
the survey, [releases][releases] already has the answer.

## Coverage

The survey took public BIOS releases for client AMD platforms from the
Lenovo, HP and Dell download catalogs, from LVFS, from ASUS and for a few
MSI boards, and ran fwscan over them. That is 820 distinct releases under
about 190 model names, as of 2026-09-29. Most are business laptops, with
some business desktops. Retail desktop boards are covered by two ASUS and
three MSI boards, and consumer laptops only where their vendor publishes on
LVFS. The DRTM TA quite likely comes with AMD's AGESA, so boards of other
vendors on the same AGESA release should carry the same TA. The ASUS and MSI
B550 boards agree: both gain the allowlist in mid-2023.

[releases]: client-sl-key-allowlist/releases.md
[foreign-sl]: foreign-sl.md
[fwscan]: ../tools/fwscan/README.md
