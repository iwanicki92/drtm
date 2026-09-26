# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""Exercises the AMD-gated checks (`src/amd/`).

These run only on an AMD CPU, so they take a boot fixture QEMU gives an
`EPYC` model: `default_boot` for the checks themselves, `amd_iommu_boot`
where an IOMMU has to be present, `amd_drtm_boot` for the machine a launch
runs on. That the skip path really skips them is asserted in
`test_platform_checks.py`.

Assertions key on a check's identifier and verdict via `results`/`verdicts`,
not on its wording, except where a test needs a value out of the detail text.

Every branch here is reachable, but four checks need the QEMU build
`docs/testing.md` describes. No upstream CPU model carries the `skinit`
CPUID bit and upstream refuses it when asked for by name, upstream's
`amd-iommu` leaves its capability's enable bit clear, and `MSR 0xC0010058`
and `VM_CR` are absent from upstream entirely, so they read zero, the
MMCONFIG check only ever sees a disabled window and `VM_CR` never shows a
firmware lock or a launch's bits.
"""

from conftest import (
    completed,
    dumped,
    fields,
    launch_state,
    messages,
    qemu_log,
    records,
    results,
    verdicts,
)

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
    # Attached on its own, the device lands at whatever slot is free rather
    # than 0:00.2, where SKL expects one, so this branch is exercised on
    # every boot with `-device amd-iommu`.
    notes = [
        detail for tag, detail in results(amd_iommu_boot, "amd.iommu") if tag == "INFO"
    ]
    assert any("0000:00:00.2" in detail for detail in notes), notes


def test_iommu_address_is_never_a_warning(amd_iommu_enabled_boot: str):
    # No specification fixes where the IOMMU sits, so a configured one away
    # from the address one loader expects leaves nothing to warn about.
    assert "WARN" not in verdicts(amd_iommu_enabled_boot, "amd.iommu")


def test_iommu_at_the_launch_address_has_nothing_to_note(amd_drtm_boot: str):
    # The launch machine puts the IOMMU at 0:00.2 as function 2 of the host
    # bridge, so the check finds it where SKL looks and the note stays out.
    assert verdicts(amd_drtm_boot, "amd.iommu") == ["PASS"]
    (found,) = [
        found for found in fields(amd_drtm_boot, "amd.iommu") if "capability" in found
    ]
    assert found["function"] == "0000:00:00.2"


def test_pci_scan_finds_the_northbridge_beside_the_mch(amd_drtm_boot: str):
    # AMD's northbridge function at 0:18.0, which SKL's node walk reads, next
    # to the Intel MCH the machine keeps at 0:00.0. Both host bridges, one
    # per vendor, so the scan tells them apart by address rather than class.
    devices = {device["fn"]: device for device in dumped(amd_drtm_boot, "device")}
    northbridge = devices["0000:00:18.0"]
    assert int(northbridge["vendor"], 16) == 0x1022
    assert int(northbridge["device"], 16) == 0x12C0
    assert int(northbridge["class"], 16) == 0x060000
    assert int(devices["0000:00:00.0"]["vendor"], 16) == 0x8086
    assert int(devices["0000:00:00.0"]["class"], 16) == 0x060000


def test_every_launch_precondition_passes_on_the_launch_machine(amd_drtm_boot: str):
    # The one configuration where the AMD checks a launch depends on pass
    # together: skinit from the machine rather than the CPU model, the IOMMU
    # enabled and listed in IVRS, and the block allocated.
    assert verdicts(amd_drtm_boot, "amd.skinit") == ["PASS"]
    assert verdicts(amd_drtm_boot, "amd.slb") == ["PASS"]
    assert verdicts(amd_drtm_boot, "amd.iommu") == ["PASS"]
    ((tag, detail),) = results(amd_drtm_boot, "amd.iommu_table")
    assert tag == "INFO"
    assert "listed in IVRS: 1, matching configuration space" in detail
    assert completed(amd_drtm_boot)


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


def test_a_secure_processor_the_loaders_do_not_list_is_still_found(
    unlisted_psp_boot: str,
):
    # A board newer than the loaders answers with an ID no table carries.
    # The PCI class finds it anyway, so the report names the device rather
    # than saying the platform has no Secure Processor.
    ((tag, detail),) = results(unlisted_psp_boot, "amd.psp")
    assert tag == "INFO"
    assert detail.startswith("1022:17d8 at 0000:00:")
    assert "no loader's device table lists" in detail


def test_an_unlisted_secure_processor_has_its_mailbox_read(
    unlisted_psp_boot: str,
):
    # drtm's own PSP launch finds the device by its class, so the check
    # reads its mailbox too and says the loaders' tables miss it.
    ((tag, detail),) = results(unlisted_psp_boot, "amd.psp_drtm")
    assert tag == "INFO"
    assert "no loader's device table lists" in detail
    (found,) = fields(unlisted_psp_boot, "amd.psp_drtm")
    assert (found["ready"], found["listed"]) == (True, False)


def test_psp_check_never_judges_the_platform(
    default_boot: str,
    amd_iommu_boot: str,
    amd_iommu_enabled_boot: str,
    unlisted_psp_boot: str,
):
    # Classic SKINIT does not go through the PSP, and which platforms need
    # the PSP path is an open question, so no outcome here is a verdict.
    for capture in (
        default_boot,
        amd_iommu_boot,
        amd_iommu_enabled_boot,
        unlisted_psp_boot,
    ):
        assert set(verdicts(capture, "amd.psp")) == {"INFO"}


def test_psp_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    # The device table is entirely AMD vendor IDs, so the check sits in
    # phase 3 and an Intel boot must not report on it at all.
    assert verdicts(intel_cpu_boot, "amd.psp") == []


def test_a_ready_drtm_service_is_reported(amd_iommu_enabled_boot: str):
    # The boot whose Secure Processor answers the way one waiting for a
    # command does. This form prints no fields, so the address the check
    # decoded has to come out of the sentence.
    ((tag, detail),) = results(amd_iommu_enabled_boot, "amd.psp_drtm")
    assert tag == "INFO"
    assert "DRTM service ready" in detail
    assert "0xfeda0a20 reads 0x80000000" in detail


def test_the_drtm_mailbox_is_decoded_from_the_table(mmconfig_above_4gib_boot: str):
    # Where C2PMSG_72 is comes from the ASPT twice over: the register block's
    # address, and how far into it the feature register C2PMSG_63 sits, nine
    # mailboxes below. Both are the platform's to choose, so the test reads
    # them back out of the dumped table rather than assuming this guest's.
    ((aspt,)) = [
        table
        for table in dumped(mmconfig_above_4gib_boot, "acpi")
        if table.get("sig") == "ASPT"
    ]
    table = bytes.fromhex(aspt["hex"])
    assert table[52:54] == b"\x00\x00", "first register structure is not the global one"
    base = int.from_bytes(table[36:44], "little")
    feature = int.from_bytes(table[60:64], "little")
    (values,) = fields(mmconfig_above_4gib_boot, "amd.psp_drtm")
    assert int(values["mailbox"], 16) == base + feature + 9 * 4


def test_the_mailbox_address_is_printed_before_it_is_read(
    mmconfig_above_4gib_boot: str,
):
    # Reading an address firmware named is where a hardware run can hang, so
    # the address goes out first and its value after it.
    (message,) = [
        line for line in messages(mmconfig_above_4gib_boot) if "[PSP]" in line
    ]
    assert "0xfeda0a20" in message
    order = [
        record.get("msg", record.get("id"))
        for record in records(mmconfig_above_4gib_boot)
        if record["rec"] in ("log", "check")
    ]
    assert order.index(message) < order.index("amd.psp_drtm")


def test_a_drtm_service_that_is_not_ready_is_reported(mmconfig_above_4gib_boot: str):
    # The same register with its top bit clear, which is what a Secure
    # Processor whose DRTM service is not up reads as.
    (values,) = fields(mmconfig_above_4gib_boot, "amd.psp_drtm")
    assert values["ready"] is False
    assert int(values["value"], 16) & (1 << 31) == 0


def test_no_table_leaves_the_registers_unlocatable(machine_check_error_boot: str):
    # The Secure Processor is there and its registers answer, but the only
    # other route to their address is an SMN access, which writes to the host
    # bridge. So a platform publishing no ASPT is reported, not worked around.
    ((tag, detail),) = results(machine_check_error_boot, "amd.psp_drtm")
    assert tag == "INFO"
    assert "no ASPT" in detail


def test_the_drtm_mailbox_is_not_read_without_a_secure_processor(default_boot: str):
    # Nothing answers with a Secure Processor ID here, so there is no mailbox
    # to read whatever ACPI says.
    ((tag, detail),) = results(default_boot, "amd.psp_drtm")
    assert tag == "SKIP"
    assert "no Secure Processor" in detail


def test_the_drtm_mailbox_check_never_judges_the_platform(
    amd_iommu_enabled_boot: str,
    mmconfig_above_4gib_boot: str,
    machine_check_error_boot: str,
):
    # A ready service is a fact about the PSP path, and classic SKINIT does
    # not take that path, so this reports like the check above it.
    for capture in (
        amd_iommu_enabled_boot,
        mmconfig_above_4gib_boot,
        machine_check_error_boot,
    ):
        assert set(verdicts(capture, "amd.psp_drtm")) == {"INFO"}


def test_the_drtm_mailbox_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    # It reads an AMD table through an AMD device, so an Intel boot must not
    # report on it at all.
    assert verdicts(intel_cpu_boot, "amd.psp_drtm") == []


def test_the_drtm_acpi_device_is_reported(mmconfig_above_4gib_boot: str):
    # The only boot publishing it, since firmware puts the device there only
    # when its own DRTM option is on and this is the check's found branch.
    ((tag, detail),) = results(mmconfig_above_4gib_boot, "amd.drtm_device")
    assert tag == "INFO"
    (values,) = fields(mmconfig_above_4gib_boot, "amd.drtm_device")
    assert values["found"] is True
    assert values["table"] == "SSDT"
    assert "DRTM0002" in detail


def test_the_named_table_is_the_one_holding_the_device(mmconfig_above_4gib_boot: str):
    # The check names a table rather than just answering yes, so the name has
    # to lead a reader to the bytes. Which table a board uses is firmware's,
    # so this reads the dump back instead of assuming this guest's.
    (values,) = fields(mmconfig_above_4gib_boot, "amd.drtm_device")
    holding = [
        table
        for table in dumped(mmconfig_above_4gib_boot, "acpi")
        if "hex" in table and b"DRTM0002".hex() in table["hex"]
    ]
    assert len(holding) == 1, "more than one table describes the device"
    assert holding[0]["sig"] == values["table"]


def test_a_platform_describing_no_device_is_reported(default_boot: str):
    # A board with the option off publishes no such device at all, which is
    # the state this has to be able to tell apart from the one above.
    ((tag, detail),) = results(default_boot, "amd.drtm_device")
    assert tag == "INFO"
    (values,) = fields(default_boot, "amd.drtm_device")
    assert values["found"] is False
    assert "no ACPI table describes" in detail


def test_the_drtm_device_check_never_judges_the_platform(
    default_boot: str,
    mmconfig_above_4gib_boot: str,
    amd_iommu_enabled_boot: str,
):
    # Nothing in the reference set reads the device, and neither its presence
    # nor its absence is known to stop a launch, so it is reported either way.
    for capture in (default_boot, mmconfig_above_4gib_boot, amd_iommu_enabled_boot):
        assert set(verdicts(capture, "amd.drtm_device")) == {"INFO"}


def test_the_drtm_device_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    # An AMD device described by AMD firmware, so an Intel boot must not
    # report on it at all.
    assert verdicts(intel_cpu_boot, "amd.drtm_device") == []


def test_the_northbridge_is_read_on_the_launch_machine(amd_drtm_boot: str):
    # AMD's northbridge function 0 at 0:18.0, which the Secure Kernel Loader
    # walks to release SL_DEV through MEMPROT_CR. Before any launch the
    # register reads zero, as it does on family 1Ah silicon.
    ((tag, detail),) = results(amd_drtm_boot, "amd.northbridge")
    assert tag == "INFO"
    assert "1022:12c0 at 0000:00:18.0" in detail
    (found,) = fields(amd_drtm_boot, "amd.northbridge")
    assert found["function"] == "0000:00:18.0"
    assert int(found["memprot"], 16) == 0
    assert found["memprot_en"] is False


def test_no_northbridge_is_a_fact_not_a_verdict(default_boot: str):
    # Q35 has an Intel MCH and nothing at 0:18.0. What a loader does about
    # that is the loader's, so this only says where nothing answered.
    ((tag, detail),) = results(default_boot, "amd.northbridge")
    assert tag == "INFO"
    assert "no AMD function at 0000:00:18.0" in detail


def test_northbridge_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    assert verdicts(intel_cpu_boot, "amd.northbridge") == []


def test_mcfg_disagreeing_with_the_msr_warns(default_boot: str):
    # MMCONFIG_BELOW_4GIB puts the MSR's window at 0xe0000000 while the
    # firmware's MCFG says 0xb0000000, the window it actually mapped. A
    # loader reading the MSR and an OS reading the table then reach
    # different windows, so this is more than a fact.
    ((tag, detail),) = results(default_boot, "amd.mmconfig_table")
    assert tag == "WARN"
    (found,) = fields(default_boot, "amd.mmconfig_table")
    assert int(found["mcfg_base"], 16) == 0xB0000000
    assert int(found["msr_base"], 16) == 0xE0000000
    assert "0xb0000000" in detail and "0xe0000000" in detail


def test_mcfg_agreeing_with_the_msr_is_reported(amd_iommu_enabled_boot: str):
    # The one boot whose MSR names the window Dasharo mapped, 256 buses from
    # 0xb0000000, so the table and the register agree.
    ((tag, detail),) = results(amd_iommu_enabled_boot, "amd.mmconfig_table")
    assert tag == "INFO"
    assert "0xb0000000" in detail
    assert "matching" in detail


def test_mcfg_check_is_skipped_when_the_msr_places_no_window(amd_iommu_boot: str):
    ((tag, _),) = results(amd_iommu_boot, "amd.mmconfig_table")
    assert tag == "SKIP"


def test_mcfg_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    assert verdicts(intel_cpu_boot, "amd.mmconfig_table") == []


def test_vm_cr_is_decoded(default_boot: str):
    # This boot leaves VM_CR at our QEMU's reset value of zero, so every
    # bit reads clear: SVM neither disabled nor locked, and no launch state
    # left behind.
    ((tag, detail),) = results(default_boot, "amd.vm_cr")
    assert tag == "INFO"
    (found,) = fields(default_boot, "amd.vm_cr")
    assert int(found["value"], 16) == 0
    assert found["svmdis"] is False
    assert found["lock"] is False
    assert found["r_init"] is False
    assert "SVM not disabled" in detail


def test_vm_cr_reports_a_firmware_lock(mmconfig_above_4gib_boot: str):
    # The value Turin firmware leaves, LOCK without SVMDIS: SVM usable, the
    # disable frozen. Still a fact, since a lock changes no launch.
    ((tag, detail),) = results(mmconfig_above_4gib_boot, "amd.vm_cr")
    assert tag == "INFO"
    (found,) = fields(mmconfig_above_4gib_boot, "amd.vm_cr")
    assert int(found["value"], 16) == 0x8
    assert found["svmdis"] is False
    assert found["lock"] is True
    assert "locked by firmware" in detail


def test_vm_cr_warns_when_a_launch_left_r_init_set(unlisted_psp_boot: str):
    # The three bits SKINIT sets, still in place. R_INIT is the one that
    # matters: an INIT arrives as #SX, so it would not park a processor.
    ((tag, detail),) = results(unlisted_psp_boot, "amd.vm_cr")
    assert tag == "WARN"
    (found,) = fields(unlisted_psp_boot, "amd.vm_cr")
    assert int(found["value"], 16) == 0x7
    assert found["r_init"] is True
    assert found["dis_a20m"] is True
    assert found["dpd"] is True
    assert "#SX" in detail


def test_vm_cr_check_runs_after_the_vendor_gate(intel_cpu_boot: str):
    assert verdicts(intel_cpu_boot, "amd.vm_cr") == []


def test_the_checks_launch_nothing(amd_drtm_boot: str):
    # A check observes the platform and leaves it as it found it. The
    # platform device is the witness: no launch recorded, SL_DEV never
    # armed, no range kept from DMA, and no rule broken, which under strict
    # mode would have stopped the VM before the run completed. The guest
    # error log is the other: with a TPM mapped, nothing the checks read
    # falls outside a modelled register, so it stays empty.
    assert completed(amd_drtm_boot)
    state = launch_state("amd_drtm_boot")
    assert state["launched"] is False
    assert state["sl-dev"] is False
    assert state["dma-blocks"] == []
    assert state["hash"] == "none"
    assert qemu_log("amd_drtm_boot") == ""


def test_the_launch_machine_has_an_ap(amd_drtm_boot: str):
    # Two processors, one of them the BSP we run on, so the BSP check and
    # the processor list are read on a machine with something to park.
    (counts,) = dumped(amd_drtm_boot, "processors")
    assert counts["total"] == 2
    bsps = [p for p in dumped(amd_drtm_boot, "processor") if p["bsp"]]
    assert len(bsps) == 1
    assert verdicts(amd_drtm_boot, "cpu.bsp") == ["PASS"]
