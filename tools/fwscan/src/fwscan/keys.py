# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""SL key tokens that fwscan names when an allowlist holds their digest."""

import hashlib
from dataclasses import dataclass

# KEY_IMAGE_4096 (AMD publication 58453, appendix B.2).
TOKEN_LEN = 0x440

# An AMD-signed SL: the $AS1 tag sits 0x10 into its FW_SIG_HEADER, the token
# follows the 0x100 header and the 0x200 signature.
SL_TAG = b"$AS1"
SL_TAG_OFFSET = 0x10
SL_TOKEN_OFFSET = 0x300


@dataclass(frozen=True)
class Key:
    name: str
    sha256: bytes
    sha384: bytes
    # False for a key the TA checks on another path than the SL key token's.
    sl: bool = True

    def matches(self, digest: bytes) -> bool:
        return digest in (self.sha256, self.sha384)


def from_token(name: str, token: bytes) -> Key:
    return Key(name, hashlib.sha256(token).digest(), hashlib.sha384(token).digest())


def sl_token(data: bytes) -> bytes:
    """The key token of the first $AS1-signed SL in `data`, e.g. amddrtm.dll."""
    tag = data.find(SL_TAG)
    if tag < SL_TAG_OFFSET:
        raise ValueError("no $AS1 signature header found")
    start = tag - SL_TAG_OFFSET + SL_TOKEN_OFFSET
    token = data[start : start + TOKEN_LEN]
    if len(token) != TOKEN_LEN:
        raise ValueError("the $AS1 key token is cut short")
    return token


KNOWN = (
    # The SL in AMD's Windows DRTM driver (amddrtm.dll), key ID
    # 024CA9A5C32F41008AA8DD84C88B8DFC.
    Key(
        "Windows AMD SL key",
        bytes.fromhex(
            "8de6f196fe89b6b5dedede3822168ae42b4dadbff28128351784f2b6ac39784e"
        ),
        bytes.fromhex(
            "9a0fe23fd62242bf65bd45d486ad60de3e9a192d300acbd5668346d9de0ab2d9"
            "753a9ecc98e4d7f7cccb5e5909bf2359"
        ),
    ),
    # Signs the SMI entry code in the SMM supervisor package of HP, Lenovo
    # and Dell BIOS images. The TA quite likely checks it on the SMM
    # supervisor path, not the SL key's. Key ID
    # D2DBD452602D43638A7D4F5F87D8C124.
    Key(
        "D2DBD452 AMD key",
        bytes.fromhex(
            "054012d15f91c3dc2aa0534a2723bf8eef27ac63cd0a0e78082948819024d407"
        ),
        bytes.fromhex(
            "4c8a4b8c19902428353d303106b3b7507da1d4f4a13845f85ac171600d1d0201"
            "608d86d739a4e503b2cc3974d1ee9e48"
        ),
        sl=False,
    ),
    # testPublicKey.bin in TrenchBoot's secure-kernel-loader, AMD's
    # development key, key ID 93E543343B4146FF968AA24DF512029A.
    Key(
        "SKL test key",
        bytes.fromhex(
            "da9bcd2a719952e18c86384f42c10d3232a86e627b4fbe74c67f007331d69b5f"
        ),
        bytes.fromhex(
            "ede35081e31a05b24dcdb8093281c5f7f4adc9ba90c2ca9d1731d3bb393f705b"
            "f6445fc8597ee9291ff00b96fa59184d"
        ),
    ),
)
