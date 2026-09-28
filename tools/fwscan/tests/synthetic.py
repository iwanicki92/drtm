# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""Synthetic DRTM TAs, SLs and psptool runs for the tests."""

import json
import sys
from pathlib import Path

from fwscan import allowlist, psp

KEY_ID = bytes(range(16))
SHA256 = 0x50000004
SHA384 = 0x50000005

MOV_R1_0x440 = bytes.fromhex("4ff48861")
BL = bytes.fromhex("00f000f8")
CMP_R0_0 = bytes.fromhex("0028")
MOVS_R0_0 = bytes.fromhex("0020")
MOVS_R2_0 = bytes.fromhex("0022")
LOOP = bytes.fromhex("fee7")  # b .
NOP = bytes.fromhex("00bf")


def header(encrypted=False, compressed=False, version=(0x04, 0x11, 0x00, 0x39)):
    h = bytearray(psp.HEADER_LEN)
    h[psp.MAGIC_OFFSET : psp.MAGIC_OFFSET + 4] = psp.MAGIC
    h[psp.ENCRYPTED_OFFSET] = int(encrypted)
    h[psp.COMPRESSED_OFFSET] = int(compressed)
    h[psp.KEY_ID_OFFSET : psp.KEY_ID_OFFSET + 16] = KEY_ID
    h[psp.VERSION_OFFSET : psp.VERSION_OFFSET + 4] = bytes(reversed(version))
    return bytes(h)


def ldr_literal(rt: int, at: int, target: int) -> bytes:
    """Thumb `ldr rt, [pc, #imm]` at offset `at` loading the word at `target`."""
    imm = target - ((at + 4) & ~3)
    assert imm >= 0 and imm % 4 == 0 and target % 4 == 0
    return (0x4800 | rt << 8 | imm // 4).to_bytes(2, "little")


def mov16(rd: int, value: int, top: bool = False) -> bytes:
    """Thumb-2 `movw rd, #value`, or `movt` with `top`."""
    hw1 = 0xF240 | top << 7 | (value >> 11 & 1) << 10 | value >> 12
    hw2 = (value >> 8 & 7) << 12 | rd << 8 | value & 0xFF
    return hw1.to_bytes(2, "little") + hw2.to_bytes(2, "little")


def load(rt: int, at: int, target: int, value: int, halves: bool) -> bytes:
    """Load `value` into `rt`, from the pool word at `target` or in halves."""
    if halves:
        return mov16(rt, value & 0xFFFF) + NOP + mov16(rt, value >> 16, top=True)
    return ldr_literal(rt, at, target)


def cmp_r0(value: int) -> bytes:
    return bytes([value, 0x28])


def ta_code(
    algorithm: int,
    digest: bytes,
    length: int | None = None,
    pointer: int | None = None,
    token_len: bytes = MOV_R1_0x440,
    halves: bool = False,
    length_first: bool = False,
) -> bytes:
    """A plaintext TA whose code compares a token digest with `digest`.

    `length` is the compare loop's bound, `pointer` the stored digest's
    address, both taken from `digest` unless given. With `halves` both
    constants load as `movw` plus `movt`, as older TAs do. With
    `length_first` r1 is set to the token length before the algorithm load.
    """
    code_at = psp.HEADER_LEN
    pool = 0x140
    digest_at = 0x150
    if pointer is None:
        pointer = allowlist.LOAD_ADDRESS + digest_at
    if length_first:
        code = token_len + load(0, code_at + len(token_len), pool, algorithm, halves)
        code += BL + CMP_R0_0
    else:
        code = load(0, code_at, pool, algorithm, halves)
        code += token_len + BL + CMP_R0_0
    code += load(1, code_at + len(code), pool + 4, pointer, halves)
    code += MOVS_R0_0 + MOVS_R2_0 + cmp_r0(length or len(digest)) + LOOP
    body = code + NOP * ((pool - code_at - len(code)) // 2)
    body += algorithm.to_bytes(4, "little") + pointer.to_bytes(4, "little")
    body = body.ljust(digest_at - code_at, b"\0") + digest
    return header() + body.ljust(0x100, b"\0")


def fake_psptool(tmp_path: Path, directories: list[dict], returncode: int = 0) -> str:
    """A stand-in for the psptool CLI that prints `directories` as JSON."""
    script = tmp_path / "psptool"
    script.write_text(
        f"#!{sys.executable}\n"
        "import sys\n"
        f"sys.stdout.write({json.dumps(json.dumps(directories))})\n"
        "sys.stderr.write('psptool warning\\n')\n"
        f"sys.exit({returncode})\n"
    )
    script.chmod(0o755)
    return str(script)


def drtm_directory(*entries: tuple[int, int]) -> list[dict]:
    return [
        {
            "directory": 0,
            "entries": [{"sectionType": "AMD_PUBLIC_KEY~0x0", "address": 0, "size": 0}]
            + [
                {"sectionType": "DRTM_TA~0x47", "address": address, "size": size}
                for address, size in entries
            ],
        }
    ]
