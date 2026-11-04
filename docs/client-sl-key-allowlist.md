<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Client SL key allowlist

On client AMD platforms the PSP DRTM service checks which key signed the
Secure Loader. Firmware whose DRTM TA is version 4.11.0.33 or later most
likely accepts only SLs signed with the key of AMD's Windows SL, so `drtm`'s SL
is expected to be refused there. This document explains the check, lists the TA
versions and says how to check a firmware image. Every release checked is
listed in [releases][releases].

The idea of checking which firmware versions make this key check comes from the
talk [A Practical Security Assessment of DRTM on AMD-based Systems][talk] by
Christian Werling and Julian Prein ([slides][talk-slides]). The talk reports the
check on client firmware.

## What the allowlist is

The PSP runs its DRTM service as a trusted application, the DRTM TA (PSP
directory entry type `0x47`). When the host launches an SL, the TA hashes
the `0x440`-byte key token of the key that signed it. Newer TAs then compare
that digest with one they carry in their code: the SHA-256 of the token of
the key that signs AMD's Windows SL (key ID
`024ca9a5c32f41008aa8dd84c88b8dfc`, see [foreign SL][foreign-sl]). A key
with any other token, such as `drtm`'s, would fail the compare.

Server firmware's TA is encrypted, so this compare can't be read there, and
firmware differs. Some Turin firmware launches an SL signed with `drtm`'s key.
Other Turin firmware checks the key: it refuses `drtm`'s SL but launches AMD's.

TAs also carry a second digest, the SHA-384 of the token of another key,
`D2DBD452...`. It is compared on a separate code path and is not an SL key: that
key signs the SMI entry code of the SMM supervisor package that many client BIOS
images carry ([IOActive][ioactive] identifies the file that holds this code). So
it is quite likely the TA's check of that package. It does not restrict the SL.

## The two keys

A key token holds a public key in AMD's own format: `0x440` bytes with the key
ID, the modulus and the exponent. Both digests are taken over the whole token,
and [fwscan] recognises both keys by them. The TA carries one digest of each key,
marked below, and the other digests are there to identify a key from either hash.

| Field         | Windows SL key                                                                                     | SMM supervisor key                                                                                                |
|---------------|----------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------|
| Key ID        | `024ca9a5c32f41008aa8dd84c88b8dfc`                                                                 | `d2dbd452602d43638a7d4f5f87d8c124`                                                                                |
| Key           | RSA-4096, exponent `0x10001`                                                                       | RSA-4096, exponent `0x10001`                                                                                      |
| Token SHA-256 | `8de6f196fe89b6b5dedede3822168ae42b4dadbff28128351784f2b6ac39784e` (in the TA)                     | `054012d15f91c3dc2aa0534a2723bf8eef27ac63cd0a0e78082948819024d407`                                                |
| Token SHA-384 | `9a0fe23fd62242bf65bd45d486ad60de3e9a192d300acbd5668346d9de0ab2d9753a9ecc98e4d7f7cccb5e5909bf2359` | `4c8a4b8c19902428353d303106b3b7507da1d4f4a13845f85ac171600d1d0201608d86d739a4e503b2cc3974d1ee9e48` (in the TA)    |
| Found in      | the `$AS1` signature block of AMD's SLs in `amddrtm.dll`                                           | entry `0xC3` of a `$PSP` directory in FFS file `83E1F409-21A3-491D-A415-B163A153776D` (`SmmSupervisorBinRelease`) |

No certificate for either key has been found. A token has no subject, issuer or
validity fields, its certifying key ID is all zero in both, and nothing in those
packages certifies either key. So calling them AMD's rests on where they appear:
AMD's driver package and the digests in the TA. The Authenticode signature on
`amddrtm.dll` comes from Microsoft and uses a separate key.

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

In the [client firmware survey](#coverage), the Windows SL key is the only entry
in every allowlist found, and no DRTM TA is encrypted.

## What it means for `drtm`

- `allowlist`: the TA most likely has an SL key allowlist, so `drtm`'s own
    SL is expected to be refused. A debug build can still launch AMD's
    client SL to check that the platform's DRTM works at all, see
    [foreign SL][foreign-sl].
- `no-allowlist` and `inconclusive`: nothing found in the TA restricts the
    SL key, so `drtm`'s own SL is quite likely accepted. Most of these are
    older releases of models whose later releases carry the allowlist, and
    anti-rollback may block a downgrade to them (not checked).
- `encrypted`: the TA can't be read, so only a launch shows whether the firmware
    checks the key.
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
[ioactive]: https://www.ioactive.com/tales-from-the-call-gate-an-smm-supervisor-vulnerability/
[talk]: https://cfp.3mdeb.com/bsmconf/talk/ZJWHCU/
[talk-slides]: https://cfp.3mdeb.com/media/bsmconf/submissions/ZJWHCU/resources/20260924-drtm-on-amd-slides_5B8J8mF.pdf
