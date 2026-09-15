# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Exercises the AMD-gated checks (`src/amd/`).

These run only on an AMD CPU, so they take a boot fixture QEMU gives an
`EPYC` model: `default_boot` for the checks themselves, `amd_iommu_boot`
where an IOMMU has to be present. That the skip path really skips them is
asserted in `test_platform_checks.py`.

Two branches here are unreachable under QEMU rather than merely untaken.
The MMCONFIG check only ever sees an unimplemented MSR reading zero, so
neither enabled outcome can be produced, and the `SKINIT` check only ever
sees the feature absent.

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


def test_mmio_config_window_is_reported(default_boot: str):
    # QEMU 8.2.2 does not model MSR 0xC0010058: it reads as zero on every AMD
    # model tried, and on an Intel one too, where hardware would raise #GP.
    # So this asserts the enable-bit-clear branch is reached and worded as a
    # skip, not that any platform really leaves the window disabled.
    assert "[SKIP] MMIO configuration space window" in default_boot
    assert "enable bit clear" in default_boot


def test_mmio_config_window_is_not_reported_as_a_failure(default_boot: str):
    # The whole point of the third outcome. A platform that does not use this
    # MSR is not a platform that failed a check.
    assert "[FAIL] MMIO configuration space" not in default_boot


def test_the_checks_survive_an_attached_iommu(amd_iommu_boot: str):
    # No IOMMU check exists yet, so this only proves the device attaches and
    # the guest still boots to the end, which is what the IOMMU check will
    # be built on top of.
    assert "[PASS] 64 KiB-aligned SLB region available below 4 GiB" in amd_iommu_boot
    assert "[FAIL] SKINIT CPUID bit not set" in amd_iommu_boot
    assert "checks complete" in amd_iommu_boot


def test_pci_scan_finds_the_iommu(amd_iommu_boot: str):
    # Class 0806h is the IOMMU subclass. QEMU puts the device at whatever
    # slot is free, so match on what it is rather than where it sits.
    match = re.search(r"\[SCAN\] \S+ id 1022:[0-9a-f]{4} class 080600", amd_iommu_boot)
    assert match is not None, "the scan reported no IOMMU-class AMD function"


def test_pci_scan_reports_the_iommu_capability(amd_iommu_boot: str):
    # Capability 0Fh with type 3 is what identifies an AMD IOMMU, and it is
    # how the check will find it. Type lives in bits 18:16 of the header.
    assert "[SCAN]   capability 0f type 3 at" in amd_iommu_boot


def test_no_iommu_without_the_device(default_boot: str):
    # The same scan on a guest with no IOMMU, so a pass above means the scan
    # discriminates rather than always reporting one.
    assert "capability 0f type 3" not in default_boot
    assert "class 080600" not in default_boot
