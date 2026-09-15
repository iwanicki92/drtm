# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Exercises the AMD-gated checks (`src/amd/`).

These run only on an AMD CPU, so they take a boot fixture QEMU gives an
`EPYC` model: `default_boot` for the checks themselves, `amd_iommu_boot`
where an IOMMU has to be present. That the skip path really skips them is
asserted in `test_platform_checks.py`.

Assertions key on a check's identifier and verdict via `results`/`verdicts`,
not on its wording, except where a test needs a value out of the detail text.

Three branches here are unreachable under QEMU rather than merely untaken.
The MMCONFIG check only ever sees an unimplemented MSR reading zero, so
neither enabled outcome can be produced. The IOMMU check only ever sees the
enable bit clear, so it never passes. The `SKINIT` check only ever sees the
feature absent.

QEMU 8.2.2 refuses to set the `skinit` CPUID feature bit under both TCG and
KVM ("TCG doesn't support requested feature" / "host doesn't support
requested feature"), on every CPU model tried, including AMD's own `EPYC`.
So only the "unsupported" branch of that check can be exercised here, see
the design spec for detail.
"""

import re

from conftest import CHECKS_COMPLETE, results, verdicts

# Size and alignment SKINIT requires of the Secure Loader Block, mirroring
# SLB_SIZE in src/amd/memory.rs.
SLB_SIZE = 64 * 1024


def test_skinit_check_fails_on_an_amd_cpu_without_the_feature(default_boot: str):
    assert verdicts(default_boot, "amd.skinit") == ["FAIL"]


def test_slb_region_is_available(default_boot: str):
    # Only the pass path is reachable: forcing the allocation to fail means
    # booting with too little RAM for the firmware itself, which would test
    # QEMU rather than this check.
    assert verdicts(default_boot, "amd.slb") == ["PASS"]


def test_slb_region_address_satisfies_skinit(default_boot: str):
    # UEFI allocates in 4 KiB pages and guarantees nothing coarser, so the
    # 64 KiB alignment comes from our own arithmetic. Check the address it
    # produced, not just that it claimed success.
    ((tag, detail),) = results(default_boot, "amd.slb")
    assert tag == "PASS"
    match = re.search(r"probed at (0x[0-9a-f]+)", detail)
    assert match is not None, "check reported no SLB address"
    address = int(match.group(1), 16)
    assert address % SLB_SIZE == 0, f"{address:#x} is not 64 KiB aligned"
    assert address + SLB_SIZE <= 1 << 32, f"{address:#x} does not fit below 4 GiB"


def test_mmio_config_window_is_reported(default_boot: str):
    # QEMU 8.2.2 does not model MSR 0xC0010058: it reads as zero on every AMD
    # model tried, and on an Intel one too, where hardware would raise #GP.
    # So this asserts the enable-bit-clear branch is reached and worded as a
    # skip, not that any platform really leaves the window disabled.
    ((tag, detail),) = results(default_boot, "amd.mmconfig")
    assert tag == "SKIP"
    assert "enable bit clear" in detail


def test_mmio_config_window_is_not_reported_as_a_failure(default_boot: str):
    # The whole point of the third outcome. A platform that does not use this
    # MSR is not a platform that failed a check.
    assert "FAIL" not in verdicts(default_boot, "amd.mmconfig")


def test_the_checks_survive_an_attached_iommu(amd_iommu_boot: str):
    # The device changes the PCI topology every check after it walks, so this
    # asserts the rest of the run is unaffected and reaches the end marker.
    assert verdicts(amd_iommu_boot, "amd.slb") == ["PASS"]
    assert verdicts(amd_iommu_boot, "amd.skinit") == ["FAIL"]
    assert CHECKS_COMPLETE in amd_iommu_boot


def test_pci_scan_finds_the_iommu(amd_iommu_boot: str):
    # Class 0806h is the IOMMU subclass. QEMU puts the device at whatever
    # slot is free, so match on what it is rather than where it sits.
    match = re.search(r"\[SCAN\] \S+ id 1022:[0-9a-f]{4} class 080600", amd_iommu_boot)
    assert match is not None, "the scan reported no IOMMU-class AMD function"


def test_pci_scan_reports_the_iommu_capability(amd_iommu_boot: str):
    # Capability 0Fh with type 3 is what identifies an AMD IOMMU, and it is
    # how the check finds it. Type lives in bits 18:16 of the header.
    assert "[SCAN]   capability 0f type 3 at" in amd_iommu_boot


def test_no_iommu_without_the_device(default_boot: str):
    # The same scan on a guest with no IOMMU, so a pass above means the scan
    # discriminates rather than always reporting one.
    assert "capability 0f type 3" not in default_boot
    assert "class 080600" not in default_boot


def test_iommu_is_found_by_its_capability(amd_iommu_boot: str):
    # The address is QEMU's choice, so assert the capability offset came out,
    # which is what anything programming the IOMMU works from.
    found = [
        detail
        for _, detail in results(amd_iommu_boot, "amd.iommu")
        if "capability block at" in detail
    ]
    assert len(found) == 1, "expected exactly one IOMMU with the device attached"
    match = re.search(r"capability block at (0x[0-9a-f]+)", found[0])
    assert match is not None, "no capability offset reported"
    assert int(match.group(1), 16) >= 0x40, "capability cannot precede the header"


def test_iommu_left_disabled_by_firmware_warns(amd_iommu_boot: str):
    # QEMU leaves the capability's enable bit clear and read-only, which SKL
    # treats as no IOMMU at all. So the found IOMMU is a warning, never a pass,
    # and the pass path is unreachable under QEMU.
    ((tag, detail),) = [
        result
        for result in results(amd_iommu_boot, "amd.iommu")
        if "capability block at" in result[1]
    ]
    assert tag == "WARN"
    assert "enable bit clear" in detail
    assert "PASS" not in verdicts(amd_iommu_boot, "amd.iommu")


def test_iommu_away_from_the_launch_address_is_reported(amd_iommu_boot: str):
    # QEMU never puts it at 0:00.2, the one address SKL sets an IOMMU up at,
    # so this branch is exercised on every boot with the device attached.
    warnings = [
        detail for tag, detail in results(amd_iommu_boot, "amd.iommu") if tag == "WARN"
    ]
    assert any("0000:00:00.2" in detail for detail in warnings), warnings


def test_missing_iommu_warns_rather_than_fails(default_boot: str):
    # A missing IOMMU costs DMA protection, which is a weaker launch rather
    # than no launch, so it must not read as a failed precondition.
    assert verdicts(default_boot, "amd.iommu") == ["WARN"]


def test_iommu_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    # Capability 0Fh is AMD's, so the check sits in phase 3 and an Intel boot
    # must not report on it either way.
    assert verdicts(intel_cpu_boot, "amd.iommu") == []


def test_psp_check_reports_no_secure_processor(default_boot: str):
    # QEMU models no PSP, so the not-found branch is the only one any boot
    # here can reach. The matching branches are unreachable until hardware.
    ((tag, detail),) = results(default_boot, "amd.psp")
    assert tag == "INFO"
    assert "no AMD Secure Processor" in detail


def test_an_amd_device_that_is_not_a_psp_is_not_matched(amd_iommu_boot: str):
    # QEMU's amd-iommu answers 1022:0000, AMD's vendor ID with a device ID
    # that is not in the table. So this exercises the half of the match that
    # discriminates, which the vendor test alone would let through.
    assert "id 1022:0000" in amd_iommu_boot
    ((tag, detail),) = results(amd_iommu_boot, "amd.psp")
    assert tag == "INFO"
    assert "no AMD Secure Processor" in detail


def test_psp_check_never_judges_the_platform(default_boot: str, amd_iommu_boot: str):
    # Classic SKINIT does not go through the PSP, and which platforms need
    # the PSP path is an open question, so no outcome here is a verdict.
    for capture in (default_boot, amd_iommu_boot):
        assert set(verdicts(capture, "amd.psp")) == {"INFO"}


def test_psp_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    # The device table is entirely AMD vendor IDs, so the check sits in
    # phase 3 and an Intel boot must not report on it at all.
    assert verdicts(intel_cpu_boot, "amd.psp") == []
