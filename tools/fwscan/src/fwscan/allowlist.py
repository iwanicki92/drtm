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

Older TAs (4.11.0.2F and before) build both constants with a `mov` or
`movw` of the low half and a `movt` of the high half instead of `ldr`, and
some set r1 to 0x440 a few instructions before the algorithm load.
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


def immediate(
    insn: capstone.CsInsn, mnemonics: tuple[str, ...]
) -> tuple[int, int] | None:
    """`(register, value)` for `insn rX, #imm` with one of `mnemonics`."""
    if insn.mnemonic not in mnemonics or len(insn.operands) != 2:
        return None
    dst, src = insn.operands
    if dst.type != arm.ARM_OP_REG or src.type != arm.ARM_OP_IMM:
        return None
    return dst.reg, src.imm


class Constants:
    """Words loaded into registers, by `ldr` or by a low half plus `movt`."""

    def __init__(self, ta: bytes) -> None:
        self.ta = ta
        self.low: dict[int, int] = {}

    def feed(self, insn: capstone.CsInsn) -> int | None:
        """The word `insn` completes loading, if any."""
        value = literal(insn, self.ta)
        if value is not None:
            return value
        low = immediate(insn, ("mov", "movs", "mov.w", "movw"))
        if low is not None:
            self.low[low[0]] = low[1] & 0xFFFF
            return None
        high = immediate(insn, ("movt",))
        if high is not None and high[0] in self.low:
            return high[1] << 16 | self.low.pop(high[0])
        return None


def compares_length(insn: capstone.CsInsn, length: int) -> bool:
    return insn.mnemonic in ("cmp", "cmp.w") and insn.op_str.endswith(f", #{length:#x}")


def sets_token_length(insn: capstone.CsInsn) -> bool:
    return insn.mnemonic in ("mov.w", "movw") and insn.op_str == "r1, #0x440"


# How many bytes before the algorithm load r1 may be set to 0x440.
BACK = 0x40


def token_length_before(md: capstone.Cs, ta: bytes, start: int) -> bool:
    for at in range(max(0, start - BACK), start, 2):
        insn = next(md.disasm(ta[at : at + 4], LOAD_ADDRESS + at, 1), None)
        if insn is not None and sets_token_length(insn) and at + insn.size <= start:
            return True
    return False


def pattern_at(
    md: capstone.Cs, ta: bytes, start: int, length: int, hashed: bool = False
) -> int | None:
    """Offset of the stored digest if the pattern follows `start`.

    `hashed` says r1 was already set to the token length before `start`.
    """
    code = ta[start : start + 4 * WINDOW]
    stored = None
    constants = Constants(ta)
    for insn in md.disasm(code, LOAD_ADDRESS + start, WINDOW):
        if not hashed:
            if sets_token_length(insn):
                hashed = True
        elif stored is None:
            value = constants.feed(insn)
            if value is not None:
                offset = value - LOAD_ADDRESS
                if not 0 <= offset <= len(ta) - length:
                    return None
                stored = offset
        elif compares_length(insn, length):
            return stored
    return None


# How far past the low half the `movt` of an algorithm may lie.
HIGH_WINDOW = 4


def algorithm_at(md: capstone.Cs, ta: bytes, start: int) -> tuple[int, int] | None:
    """`(algorithm, offset past its load)` if a TEE digest ID loads at `start`."""
    code = ta[start : start + 4 * (HIGH_WINDOW + 1)]
    constants = Constants(ta)
    for n, insn in enumerate(md.disasm(code, LOAD_ADDRESS + start, HIGH_WINDOW + 1)):
        value = constants.feed(insn)
        if value is not None:
            end = insn.address + insn.size - LOAD_ADDRESS
            return (value, end) if value in ALGORITHMS else None
        # Only the first instruction may start the load.
        if n == 0 and not constants.low:
            return None
    return None


def find(ta: bytes) -> list[Digest]:
    """The digests a plaintext TA (header included) checks tokens against."""
    md = disassembler()
    found: dict[tuple[str, bytes], Digest] = {}
    for start in range(0, len(ta) - 1, 2):
        loaded = algorithm_at(md, ta, start)
        if loaded is None:
            continue
        name, length = ALGORITHMS[loaded[0]]
        before = token_length_before(md, ta, start)
        offset = pattern_at(md, ta, loaded[1], length, before)
        if offset is not None:
            value = ta[offset : offset + length]
            found.setdefault((name, value), Digest(name, value, offset))
    return list(found.values())
