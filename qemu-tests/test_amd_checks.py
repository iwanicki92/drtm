# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Exercises the AMD-gated checks (`src/amd/`).

These run only on an AMD CPU, so they all take `default_boot`, which QEMU
gives an `EPYC` model. That the skip path really skips them is asserted in
`test_platform_checks.py`.

QEMU 8.2.2 refuses to set the `skinit` CPUID feature bit under both TCG and
KVM ("TCG doesn't support requested feature" / "host doesn't support
requested feature"), on every CPU model tried, including AMD's own `EPYC`.
So only the "unsupported" branch of that check can be exercised here, see
the design spec for detail.
"""

import re

# Size and alignment SKINIT requires of the Secure Loader Block, mirroring
# SLB_SIZE in src/amd/memory.rs.
SLB_SIZE = 64 * 1024


def test_skinit_check_fails_on_an_amd_cpu_without_the_feature(default_boot: str):
    assert "[FAIL] SKINIT CPUID bit not set" in default_boot
    assert "SKINIT unsupported on this CPU" in default_boot


def test_slb_region_is_available(default_boot: str):
    # Only the pass path is reachable: forcing the allocation to fail means
    # booting with too little RAM for the firmware itself, which would test
    # QEMU rather than this check.
    assert "[PASS] 64 KiB-aligned SLB region available below 4 GiB" in default_boot


def test_slb_region_address_satisfies_skinit(default_boot: str):
    # UEFI allocates in 4 KiB pages and guarantees nothing coarser, so the
    # 64 KiB alignment comes from our own arithmetic. Check the address it
    # produced, not just that it claimed success.
    match = re.search(
        r"SLB region available below 4 GiB \(probed at (0x[0-9a-f]+)\)", default_boot
    )
    assert match is not None, "check reported no SLB address"
    address = int(match.group(1), 16)
    assert address % SLB_SIZE == 0, f"{address:#x} is not 64 KiB aligned"
    assert address + SLB_SIZE <= 1 << 32, f"{address:#x} does not fit below 4 GiB"
