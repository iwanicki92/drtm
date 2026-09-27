# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""Finds the DRTM TAs in a firmware image through the psptool CLI."""

import json
import subprocess
import zlib
from dataclasses import dataclass
from pathlib import Path

# PSP directory entry type 0x47, which psptool names DRTM_TA.
DRTM_TA_TYPE = "DRTM_TA"

# $PS1 header fields.
HEADER_LEN = 0x100
MAGIC = b"$PS1"
MAGIC_OFFSET = 0x10
ENCRYPTED_OFFSET = 0x18
KEY_ID_OFFSET = 0x38
COMPRESSED_OFFSET = 0x48
VERSION_OFFSET = 0x60


class PsptoolError(Exception):
    pass


@dataclass(frozen=True)
class Ta:
    address: int
    size: int
    version: str
    key_id: bytes
    encrypted: bool
    compressed: bool
    # The TA as loaded, header included, or None when encrypted.
    image: bytes | None


def run_psptool(psptool: str, path: Path) -> list[dict]:
    """The directories `psptool -E -j` lists for the image at `path`."""
    try:
        done = subprocess.run(
            [psptool, "-E", "-j", str(path)],
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError as e:
        raise PsptoolError(f"cannot run {psptool}: {e}") from e
    if done.returncode != 0:
        raise PsptoolError(f"psptool failed on {path}:\n{done.stderr.strip()}")
    try:
        return json.loads(done.stdout)
    except json.JSONDecodeError as e:
        raise PsptoolError(f"psptool printed no JSON for {path}: {e}") from e


def drtm_entries(directories: list[dict]) -> list[tuple[int, int]]:
    """(address, size) of every DRTM TA entry, each address once."""
    found: dict[int, int] = {}
    for directory in directories:
        for entry in directory.get("entries", []):
            if entry.get("sectionType", "").startswith(DRTM_TA_TYPE):
                found.setdefault(entry["address"], entry["size"])
    return sorted(found.items())


def u32(data: bytes, offset: int) -> int:
    return int.from_bytes(data[offset : offset + 4], "little")


def parse_ta(raw: bytes, address: int) -> Ta:
    if len(raw) < HEADER_LEN or raw[MAGIC_OFFSET : MAGIC_OFFSET + 4] != MAGIC:
        raise ValueError(f"no $PS1 header at {address:#x}")
    encrypted = u32(raw, ENCRYPTED_OFFSET) != 0
    compressed = u32(raw, COMPRESSED_OFFSET) != 0
    version = ".".join(
        f"{b:X}" for b in reversed(raw[VERSION_OFFSET : VERSION_OFFSET + 4])
    )
    image = None
    if not encrypted:
        image = raw
        if compressed:
            # zlib after the header. Keeping the header in front assumes the
            # PSP loads it with the body, as it does for uncompressed TAs.
            image = raw[:HEADER_LEN] + zlib.decompressobj().decompress(raw[HEADER_LEN:])
    return Ta(
        address=address,
        size=len(raw),
        version=version,
        key_id=raw[KEY_ID_OFFSET : KEY_ID_OFFSET + 16],
        encrypted=encrypted,
        compressed=compressed,
        image=image,
    )


def drtm_tas(firmware: bytes, directories: list[dict]) -> list[Ta]:
    return [
        parse_ta(firmware[address : address + size], address)
        for address, size in drtm_entries(directories)
    ]
