# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Exercises the checks that run whatever the CPU vendor is.

Phase 1 identifies the CPU (`src/platform.rs`) and phase 2 runs the checks
that hold on every DRTM path (`src/cpu_state.rs`, `src/tpm.rs`, `src/pci.rs`).
Only phase 3 is vendor-gated, and its tests live in `test_amd_checks.py`.

Assertions key on a check's identifier and verdict via `results`/`verdicts`,
not on its wording. Where a test needs a value out of the detail text it says
so by scraping it.

Both TPM interfaces are covered because the check reports them differently,
and an earlier bug got CRB wrong while passing on TIS.
"""

import re

from conftest import results, verdicts


def test_cpu_vendor_is_reported(default_boot: str):
    assert "drtm: CPU vendor AuthenticAMD" in default_boot


def test_cpu_vendor_is_reported_on_an_intel_cpu(intel_cpu_boot: str):
    # The vendor string is read, not inferred, so a second model proves the
    # CPUID word order is right rather than accidentally symmetric.
    assert "drtm: CPU vendor GenuineIntel" in intel_cpu_boot


def test_amd_checks_are_skipped_on_an_intel_cpu(intel_cpu_boot: str):
    assert [
        (tag, "GenuineIntel" in detail)
        for tag, detail in results(intel_cpu_boot, "amd.checks")
    ] == [("SKIP", True)]


def test_skipping_means_the_amd_checks_did_not_run(intel_cpu_boot: str):
    # A skip that still ran the checks would be worse than no skip at all, so
    # assert every AMD identifier is absent rather than trusting the skip line.
    # On hardware the gate is also what keeps this Intel boot from taking a #GP
    # on the AMD-only MSR 0xC0010058 behind amd.mmconfig.
    for identifier in ("amd.skinit", "amd.slb", "amd.mmconfig", "amd.iommu"):
        assert verdicts(intel_cpu_boot, identifier) == [], identifier


def test_running_on_the_bootstrap_processor(default_boot: str):
    # A UEFI application is entered on the BSP, so this passes anywhere the
    # tool can run at all. It is here to state the requirement, and to catch
    # the day something runs us somewhere else.
    assert verdicts(default_boot, "cpu.bsp") == ["PASS"]


def test_bsp_check_runs_on_a_non_amd_cpu(intel_cpu_boot: str):
    # IA32_APIC_BASE is architectural, so this check belongs to phase 2 even
    # though the spec numbers it among the AMD ones.
    assert verdicts(intel_cpu_boot, "cpu.bsp") == ["PASS"]


def test_control_register_deviation_is_reported(default_boot: str):
    # Dasharo's firmware leaves CR0.NE clear, so this is the live path rather
    # than a hypothetical one. It is the reason the check exists at all. Which
    # bit deviated is the check's output, so that much of the detail is worth
    # holding it to.
    ((tag, detail),) = results(default_boot, "cpu.launch_state")
    assert tag == "WARN"
    assert detail.startswith("CR0.NE clear")


def test_control_register_deviations_are_not_failures(default_boot: str):
    # A launch sets CR0 and EFLAGS itself, unconditionally, so a deviation is
    # information about the firmware and not a reason to refuse.
    assert "FAIL" not in verdicts(default_boot, "cpu.launch_state")


def test_machine_check_state_permits_a_launch(default_boot: str):
    assert verdicts(default_boot, "cpu.machine_check") == ["PASS"]


def test_machine_check_banks_are_actually_read(default_boot: str):
    # The pass line is reachable with zero banks, which would mean IA32_MCG_CAP
    # was misread and no bank was ever looked at. Assert the count is real.
    ((tag, detail),) = results(default_boot, "cpu.machine_check")
    assert tag == "PASS"
    match = re.search(r"all (\d+) status banks clear", detail)
    assert match is not None
    assert int(match.group(1)) > 0


def test_cpu_state_checks_run_on_a_non_amd_cpu(intel_cpu_boot: str):
    # CR0, EFLAGS and the machine-check MSRs are architectural, and Intel TXT
    # carries the same requirements, so these are phase 2 like the BSP check.
    assert verdicts(intel_cpu_boot, "cpu.launch_state") == ["WARN"]
    assert verdicts(intel_cpu_boot, "cpu.machine_check") == ["PASS"]


def test_tpm_check_runs_on_a_non_amd_cpu(intel_cpu_boot: str):
    # Phase 2 is not vendor-gated. If the gate ever swallowed it, this is
    # what would catch it.
    assert verdicts(intel_cpu_boot, "tpm.present") == ["FAIL"]


def test_tpm_check_fails_without_a_tpm_device(default_boot: str):
    assert verdicts(default_boot, "tpm.present") == ["FAIL"]


def test_tpm_check_detects_a_tis_tpm(tpm_tis_boot: str):
    # The family and interface are the check's output, so they are asserted
    # from the detail rather than through the verdict alone.
    assert results(tpm_tis_boot, "tpm.present") == [
        ("PASS", "locality 0 answered, V2_0 Tis")
    ]


def test_tpm_check_detects_a_crb_tpm(tpm_crb_boot: str):
    # Guards the register-ordering fix: reading TPM_INTF_CAPABILITY before
    # establishing the active interface reported a CRB TPM as V1_2 Tis,
    # because offset 0x14 holds TPM_DATA_CSUM under the CRB layout.
    assert results(tpm_crb_boot, "tpm.present") == [
        ("PASS", "locality 0 answered, V2_0 Crb")
    ]


def test_tpm_warns_that_discreteness_is_unverifiable(tpm_tis_boot: str):
    # A pass must never read as "SKINIT will work here": software cannot
    # tell a discrete TPM from a firmware one, and SKINIT needs a discrete.
    assert verdicts(tpm_tis_boot, "tpm.discrete") == ["WARN"]


def test_locality_check_passes_where_the_tpm_offers_five(tpm_tis_boot: str):
    # A launch measures through locality 4 and its DCE runs at locality 2, so
    # the TPM has to implement more than locality 0 for any of it to work.
    assert verdicts(tpm_tis_boot, "tpm.locality") == ["PASS"]


def test_locality_check_fails_where_the_tpm_offers_only_one(tpm_crb_boot: str):
    # QEMU's CRB device reports CapLocality clear, the same thing a firmware
    # TPM reports, and it is the one way software can prove no launch is
    # possible rather than warning that it cannot tell.
    ((tag, detail),) = results(tpm_crb_boot, "tpm.locality")
    assert tag == "FAIL"
    assert "locality 0" in detail


def test_a_tpm_without_localities_still_asks_for_a_manual_check(tpm_crb_boot: str):
    # Locality 0 only is what a firmware TPM looks like, and a platform
    # offering both chooses in firmware setup, so this is exactly where the
    # manual check has something to act on.
    assert verdicts(tpm_crb_boot, "tpm.discrete") == ["WARN"]


def test_locality_check_is_skipped_without_a_tpm(default_boot: str):
    # Nothing to ask, and tpm.present has already failed, so this must not
    # report a second failure for the same missing device.
    assert verdicts(default_boot, "tpm.locality") == ["SKIP"]


def test_pci_root_bridge_protocol_is_available(default_boot: str):
    # Everything read out of configuration space depends on the firmware
    # publishing this. QEMU's q35 has one root bridge, hence one handle, and
    # pci.root_bridge reports only when something went wrong.
    assert "[SCAN] PCI root bridges: 1" in default_boot
    assert verdicts(default_boot, "pci.root_bridge") == []


def test_pci_scan_finds_the_host_bridge(default_boot: str):
    # 8086:29c0 is the Q35 MCH, at 0:00.0 on every boot, so it is the one
    # device the scan can be held to by name.
    assert "[SCAN] 0000:00:00.0 id 8086:29c0 class 060000" in default_boot


def test_pci_scan_walks_capability_lists(default_boot: str):
    # A device with no capabilities would still print its identity line, so
    # assert some list was actually walked rather than silently skipped.
    assert re.search(
        r"\[SCAN\]   capability [0-9a-f]{2} type \d at [0-9a-f]{2}", default_boot
    )


def test_pci_scan_runs_on_a_non_amd_cpu(intel_cpu_boot: str):
    # Configuration space is not a vendor-specific interface, so this runs in
    # phase 2 rather than behind the AMD gate.
    assert "[SCAN] PCI root bridges: 1" in intel_cpu_boot
    assert verdicts(intel_cpu_boot, "pci.root_bridge") == []
