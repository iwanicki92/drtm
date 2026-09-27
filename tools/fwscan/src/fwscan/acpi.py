# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""Finds the _HID of the DRTM device in a firmware image's ACPI tables."""

import lzma
import re

# The tables sit in LZMA-compressed firmware volumes. This matches the
# LZMA-alone header's properties byte and the low bytes of its dictionary
# size, which is loose on purpose: a stricter pattern misses volumes.
LZMA_HEADER = re.compile(rb"\x5d\x00\x00[\x00-\x10][\x00-\x10]")
HID = re.compile(rb"DRTM000\d")

# Upper bounds that keep a false LZMA match from running away.
MAX_INPUT = 0x1000000
MAX_OUTPUT = 0x8000000


def drtm_hids(firmware: bytes) -> list[str]:
    found = set(HID.findall(firmware))
    for match in LZMA_HEADER.finditer(firmware):
        start = match.start()
        try:
            data = lzma.LZMADecompressor(lzma.FORMAT_ALONE).decompress(
                firmware[start : start + MAX_INPUT], max_length=MAX_OUTPUT
            )
        except lzma.LZMAError:
            continue
        found.update(HID.findall(data))
    return sorted(h.decode() for h in found)
