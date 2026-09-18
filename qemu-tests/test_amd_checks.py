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

Every branch here is reachable, but three checks need the QEMU build
`docs/testing.md` describes. No upstream CPU model carries the `skinit`
CPUID bit and upstream refuses it when asked for by name, upstream's
`amd-iommu` leaves its capability's enable bit clear, and `MSR 0xC0010058`
is absent from upstream entirely, so it reads zero and the MMCONFIG check
only ever sees a disabled window.
"""

from conftest import completed, dumped, fields, results, verdicts

# Size and alignment SKINIT requires of the Secure Loader Block, mirroring
# SLB_SIZE in src/amd/memory.rs.
SLB_SIZE = 64 * 1024


def test_skinit_check_passes_on_an_amd_cpu_with_the_feature(default_boot: str):
    assert verdicts(default_boot, "amd.skinit") == ["PASS"]


def test_skinit_check_fails_on_an_amd_cpu_without_the_feature(amd_iommu_boot: str):
    # The plain `EPYC` model, so this is the same check on a CPU that does not
    # report the feature. Both outcomes rest on the one CPUID bit.
    assert verdicts(amd_iommu_boot, "amd.skinit") == ["FAIL"]


def test_slb_region_is_available(default_boot: str):
    # Only the pass path is reachable: forcing the allocation to fail means
    # booting with too little RAM for the firmware itself, which would test
    # QEMU rather than this check.
    assert verdicts(default_boot, "amd.slb") == ["PASS"]


def test_slb_region_address_satisfies_skinit(default_boot: str):
    # UEFI allocates in 4 KiB pages and guarantees nothing coarser, so the
    # 64 KiB alignment comes from our own arithmetic. Check the address it
    # produced, not just that it claimed success.
    ((tag, _),) = results(default_boot, "amd.slb")
    assert tag == "PASS"
    (probed,) = fields(default_boot, "amd.slb")
    address = int(probed["address"], 16)
    assert address % SLB_SIZE == 0, f"{address:#x} is not 64 KiB aligned"
    assert address + SLB_SIZE <= 1 << 32, f"{address:#x} does not fit below 4 GiB"


def test_mmio_config_window_is_reported(default_boot: str):
    # The window MMCONFIG_BELOW_4GIB asks for: 256 buses from 0xe0000000, so
    # it ends at 0xf0000000, below 4 GiB.
    ((tag, detail),) = results(default_boot, "amd.mmconfig")
    assert tag == "INFO"
    assert "0xe0000000..0xf0000000" in detail
    assert "256 buses" in detail
    assert "below 4 GiB" in detail


def test_mmio_config_window_above_4gib_is_reported(mmconfig_above_4gib_boot: str):
    # The same window moved up so its tail crosses 4 GiB. Where it sits
    # decides how a launch reaches it, not whether one can happen, so this is
    # a fact rather than a warning.
    ((tag, detail),) = results(mmconfig_above_4gib_boot, "amd.mmconfig")
    assert tag == "INFO"
    assert "0xf8000000..0x108000000" in detail
    assert "not entirely below 4 GiB" in detail


def test_mmio_config_window_is_skipped_when_disabled(amd_iommu_boot: str):
    # The boot that asks for no window at all, so the MSR reads zero. A
    # platform that does not use this MSR has not failed a check.
    ((tag, detail),) = results(amd_iommu_boot, "amd.mmconfig")
    assert tag == "SKIP"
    assert "enable bit clear" in detail


def test_the_checks_survive_an_attached_iommu(amd_iommu_boot: str):
    # The device changes the PCI topology every check after it walks, so this
    # asserts the rest of the run is unaffected and reaches the end marker.
    assert verdicts(amd_iommu_boot, "amd.slb") == ["PASS"]
    assert verdicts(amd_iommu_boot, "amd.skinit") == ["FAIL"]
    assert completed(amd_iommu_boot)


def test_pci_scan_finds_the_iommu(amd_iommu_boot: str):
    # Class 0806h is the IOMMU subclass. QEMU puts the device at whatever
    # slot is free, so match on what it is rather than where it sits.
    found = [
        device
        for device in dumped(amd_iommu_boot, "device")
        if int(device["class"], 16) == 0x080600
    ]
    assert len(found) == 1, "the scan reported no IOMMU-class function"
    assert int(found[0]["vendor"], 16) == 0x1022


def test_pci_scan_reports_the_iommu_capability(amd_iommu_boot: str):
    # Capability 0Fh with type 3 is what identifies an AMD IOMMU, and it is
    # how the check finds it. Type lives in bits 18:16 of the header.
    assert [
        capability
        for capability in dumped(amd_iommu_boot, "capability")
        if int(capability["id"], 16) == 0x0F and capability["type"] == 3
    ]


def test_pci_scan_reads_each_bridge_once(amd_iommu_boot: str):
    # Each function is recorded once, through the bridge whose bus it sits
    # on, so its class reads back rather than failing as it would elsewhere.
    devices = dumped(amd_iommu_boot, "device")
    functions = [device["fn"] for device in devices]
    assert "[SCAN] PCI root bridges: 2" in amd_iommu_boot
    assert len(functions) == len(set(functions))
    assert {
        (device["fn"], int(device["vendor"], 16), int(device["device"], 16))
        for device in devices
    } >= {("0000:20:00.0", 0x1B36, 0x000C)}
    assert all(int(device["class"], 16) != 0 for device in devices)


def test_no_iommu_without_the_device(default_boot: str):
    # The same scan on a guest with no IOMMU, so a pass above means the scan
    # discriminates rather than always reporting one.
    assert not [
        capability
        for capability in dumped(default_boot, "capability")
        if int(capability["id"], 16) == 0x0F and capability["type"] == 3
    ]
    assert not [
        device
        for device in dumped(default_boot, "device")
        if int(device["class"], 16) == 0x080600
    ]


def test_iommu_is_found_by_its_capability(amd_iommu_boot: str):
    # The address is QEMU's choice, so assert the capability offset came out,
    # which is what anything programming the IOMMU works from.
    found = [
        found for found in fields(amd_iommu_boot, "amd.iommu") if "capability" in found
    ]
    assert len(found) == 1, "expected exactly one IOMMU with the device attached"
    assert int(found[0]["capability"], 16) >= 0x40, (
        "capability cannot precede the header"
    )


def test_iommu_left_disabled_by_firmware_warns(amd_iommu_boot: str):
    # An enable bit left clear means firmware never configured the IOMMU, so
    # it protects nothing yet and is a warning rather than a pass.
    ((tag, detail),) = [
        result
        for result in results(amd_iommu_boot, "amd.iommu")
        if "capability block at" in result[1]
    ]
    assert tag == "WARN"
    assert "enable bit clear" in detail
    assert "PASS" not in verdicts(amd_iommu_boot, "amd.iommu")


def test_iommu_enabled_by_firmware_passes(amd_iommu_enabled_boot: str):
    # The same device with its enable bit set, which is the one configuration
    # a launch would actually program.
    ((tag, _),) = [
        result
        for result in results(amd_iommu_enabled_boot, "amd.iommu")
        if "capability block at" in result[1]
    ]
    assert tag == "PASS"


def test_iommu_away_from_the_launch_address_is_reported(amd_iommu_boot: str):
    # QEMU never puts it at 0:00.2, where SKL expects one, so this branch is
    # exercised on every boot with the device attached.
    notes = [
        detail for tag, detail in results(amd_iommu_boot, "amd.iommu") if tag == "INFO"
    ]
    assert any("0000:00:00.2" in detail for detail in notes), notes


def test_iommu_address_is_never_a_warning(amd_iommu_enabled_boot: str):
    # No specification fixes where the IOMMU sits, so a configured one away
    # from the address one loader expects leaves nothing to warn about.
    assert "WARN" not in verdicts(amd_iommu_enabled_boot, "amd.iommu")


def test_missing_iommu_warns_rather_than_fails(default_boot: str):
    # A missing IOMMU costs DMA protection, which is a weaker launch rather
    # than no launch, so it must not read as a failed precondition.
    assert verdicts(default_boot, "amd.iommu") == ["WARN"]


def test_iommu_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    # Capability 0Fh is AMD's, so the check sits in phase 3 and an Intel boot
    # must not report on it either way.
    assert verdicts(intel_cpu_boot, "amd.iommu") == []


def test_iommu_table_lists_the_iommu(amd_iommu_boot: str, amd_iommu_enabled_boot: str):
    # QEMU lists its one IOMMU under two IVHD types, so a count of one also
    # shows the entries are merged. The enable bit plays no part.
    for capture in (amd_iommu_boot, amd_iommu_enabled_boot):
        ((tag, detail),) = results(capture, "amd.iommu_table")
        assert tag == "INFO"
        assert "listed in IVRS: 1, matching configuration space" in detail


def test_iommu_missing_from_acpi_warns(machine_check_error_boot: str):
    # Our QEMU's `ivrs=off` drops the table, while the IOMMU stays enabled in
    # configuration space, so it is configured and still unusable to an OS.
    assert verdicts(machine_check_error_boot, "amd.iommu") == ["PASS", "INFO"]
    ((tag, detail),) = results(machine_check_error_boot, "amd.iommu_table")
    assert tag == "WARN"
    assert "no IVRS table" in detail


def test_iommu_table_is_skipped_without_an_iommu(default_boot: str):
    # `amd.iommu` already warns about the missing IOMMU, and a table has
    # nothing to describe, so this check does not warn a second time.
    ((tag, detail),) = results(default_boot, "amd.iommu_table")
    assert tag == "SKIP"
    assert "no IVRS table" in detail


def test_iommu_table_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    # IVRS is AMD's table, so an Intel boot must not report on it.
    assert verdicts(intel_cpu_boot, "amd.iommu_table") == []


def test_psp_check_reports_no_secure_processor(default_boot: str):
    # QEMU models no PSP, so the not-found branch is the only one any boot
    # here can reach. The matching branches are unreachable until hardware.
    ((tag, detail),) = results(default_boot, "amd.psp")
    assert tag == "INFO"
    assert "no AMD Secure Processor" in detail


def test_an_amd_device_that_is_not_a_psp_is_not_matched(amd_iommu_boot: str):
    # QEMU's amd-iommu answers 1022:1419, AMD's vendor ID with a device ID
    # that is not in the table. So this exercises the half of the match that
    # discriminates, which the vendor test alone would let through.
    assert [
        device
        for device in dumped(amd_iommu_boot, "device")
        if (int(device["vendor"], 16), int(device["device"], 16)) == (0x1022, 0x1419)
    ]
    ((tag, detail),) = results(amd_iommu_boot, "amd.psp")
    assert tag == "INFO"
    assert "no AMD Secure Processor" in detail


def test_a_secure_processor_is_identified(amd_iommu_enabled_boot: str):
    # Our QEMU's amd-psp answers with the Turin ASP's ID and nothing else, so
    # the ID is all the check can go on.
    ((tag, detail),) = results(amd_iommu_enabled_boot, "amd.psp")
    assert tag == "INFO"
    assert detail.startswith("1022:156e at 0000:00:")
    assert "ASP, the interface a PSP launch would use" in detail


def test_psp_check_never_judges_the_platform(
    default_boot: str, amd_iommu_boot: str, amd_iommu_enabled_boot: str
):
    # Classic SKINIT does not go through the PSP, and which platforms need
    # the PSP path is an open question, so no outcome here is a verdict.
    for capture in (default_boot, amd_iommu_boot, amd_iommu_enabled_boot):
        assert set(verdicts(capture, "amd.psp")) == {"INFO"}


def test_psp_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    # The device table is entirely AMD vendor IDs, so the check sits in
    # phase 3 and an Intel boot must not report on it at all.
    assert verdicts(intel_cpu_boot, "amd.psp") == []
