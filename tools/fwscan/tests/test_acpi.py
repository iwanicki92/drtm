# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

import lzma
import random

from fwscan import acpi

# A 1 MiB dictionary, as firmware volumes use: the default 8 MiB one gives
# a header the scan does not look for.
FILTERS = [{"id": lzma.FILTER_LZMA1, "dict_size": 1 << 20}]


def noise(n: int) -> bytes:
    return random.Random(n).randbytes(n)


def test_hid_in_a_compressed_volume():
    volume = lzma.compress(
        noise(0x100) + b"SSDT....DRTM0002" + noise(0x80),
        format=lzma.FORMAT_ALONE,
        filters=FILTERS,
    )
    assert acpi.drtm_hids(noise(0x1000) + volume + noise(0x1000)) == ["DRTM0002"]


def test_uncompressed_hids_are_found_too():
    assert acpi.drtm_hids(b"DRTM0001" + noise(0x100) + b"DRTM0002") == [
        "DRTM0001",
        "DRTM0002",
    ]


def test_no_hid():
    assert acpi.drtm_hids(noise(0x1000)) == []
