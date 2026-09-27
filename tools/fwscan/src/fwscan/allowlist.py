# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""Finds the SL key token digests a plaintext DRTM TA compares against.

The client TA (Thumb-2) hashes the 0x440-byte token with a GlobalPlatform
TEE digest, then compares the result byte by byte with a stored digest:

    ldr   rA, =TEE_ALG_SHA256 or TEE_ALG_SHA384
    ...
    mov.w r1, #0x440
    bl    digest
    ...
    ldr   rP, =stored digest
    ...
    cmp   rI, #0x20 or #0x30

The digests follow from the code, not from a list of known hashes.
"""

from dataclasses import dataclass

import capstone
from capstone import arm

# The TA is loaded here, header included.
LOAD_ADDRESS = 0xD00000

ALGORITHMS = {
    0x50000004: ("sha256", 32),  # TEE_ALG_SHA256
    0x50000005: ("sha384", 48),  # TEE_ALG_SHA384
}

# How far past the algorithm load the rest of the pattern may lie.
WINDOW = 24


@dataclass(frozen=True)
class Digest:
    algorithm: str
    value: bytes
    # Offset of the stored digest in the TA, header included.
    offset: int


def disassembler() -> capstone.Cs:
    md = capstone.Cs(capstone.CS_ARCH_ARM, capstone.CS_MODE_THUMB)
    md.detail = True
    return md


def literal(insn: capstone.CsInsn, ta: bytes) -> int | None:
    """The word a PC-relative `ldr` loads, or None for any other insn."""
    if not insn.mnemonic.startswith("ldr") or insn.mnemonic.startswith("ldrb"):
        return None
    if len(insn.operands) != 2 or insn.operands[1].type != arm.ARM_OP_MEM:
        return None
    mem = insn.operands[1].mem
    if mem.base != arm.ARM_REG_PC:
        return None
    offset = ((insn.address + 4) & ~3) + mem.disp - LOAD_ADDRESS
    if not 0 <= offset <= len(ta) - 4:
        return None
    return int.from_bytes(ta[offset : offset + 4], "little")


def compares_length(insn: capstone.CsInsn, length: int) -> bool:
    return insn.mnemonic in ("cmp", "cmp.w") and insn.op_str.endswith(f", #{length:#x}")


def pattern_at(md: capstone.Cs, ta: bytes, start: int, length: int) -> int | None:
    """Offset of the stored digest if the pattern follows `start`."""
    code = ta[start : start + 4 * WINDOW]
    hashed = False
    stored = None
    for insn in md.disasm(code, LOAD_ADDRESS + start, WINDOW):
        if not hashed:
            if insn.mnemonic in ("mov.w", "movw") and insn.op_str == "r1, #0x440":
                hashed = True
        elif stored is None:
            value = literal(insn, ta)
            if value is not None:
                offset = value - LOAD_ADDRESS
                if not 0 <= offset <= len(ta) - length:
                    return None
                stored = offset
        elif compares_length(insn, length):
            return stored
    return None


def find(ta: bytes) -> list[Digest]:
    """The digests a plaintext TA (header included) checks tokens against."""
    md = disassembler()
    found: dict[tuple[str, bytes], Digest] = {}
    for start in range(0, len(ta) - 1, 2):
        insn = next(md.disasm(ta[start : start + 4], LOAD_ADDRESS + start, 1), None)
        if insn is None:
            continue
        algorithm = ALGORITHMS.get(literal(insn, ta) or 0)
        if algorithm is None:
            continue
        name, length = algorithm
        offset = pattern_at(md, ta, start + insn.size, length)
        if offset is not None:
            value = ta[offset : offset + length]
            found.setdefault((name, value), Digest(name, value, offset))
    return list(found.values())
