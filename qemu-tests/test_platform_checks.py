# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Exercises the checks that run whatever the CPU vendor is.

Phase 1 identifies the CPU (`src/platform.rs`) and phase 2 runs the checks
that hold on every DRTM path (`src/cpu_state.rs`, `src/tpm.rs`). Only phase 3
is vendor-gated, and its tests live in `test_amd_checks.py`.

Both TPM interfaces are covered because the check reports them differently,
and an earlier bug got CRB wrong while passing on TIS.
"""


def test_cpu_vendor_is_reported(default_boot: str):
    assert "drtm: CPU vendor AuthenticAMD" in default_boot


def test_cpu_vendor_is_reported_on_an_intel_cpu(intel_cpu_boot: str):
    # The vendor string is read, not inferred, so a second model proves the
    # CPUID word order is right rather than accidentally symmetric.
    assert "drtm: CPU vendor GenuineIntel" in intel_cpu_boot


def test_amd_checks_are_skipped_on_an_intel_cpu(intel_cpu_boot: str):
    assert "[SKIP] AMD checks" in intel_cpu_boot
    assert "GenuineIntel" in intel_cpu_boot


def test_skipping_means_the_amd_checks_did_not_run(intel_cpu_boot: str):
    # A skip that still ran the checks would be worse than no skip at all,
    # so assert their output is absent rather than trusting the skip line.
    assert "SKINIT CPUID bit" not in intel_cpu_boot
    assert "SLB region" not in intel_cpu_boot


def test_running_on_the_bootstrap_processor(default_boot: str):
    # A UEFI application is entered on the BSP, so this passes anywhere the
    # tool can run at all. It is here to state the requirement, and to catch
    # the day something runs us somewhere else.
    assert "[PASS] running on the bootstrap processor" in default_boot


def test_bsp_check_runs_on_a_non_amd_cpu(intel_cpu_boot: str):
    # IA32_APIC_BASE is architectural, so this check belongs to phase 2 even
    # though the spec numbers it among the AMD ones.
    assert "[PASS] running on the bootstrap processor" in intel_cpu_boot


def test_tpm_check_runs_on_a_non_amd_cpu(intel_cpu_boot: str):
    # Phase 2 is not vendor-gated. If the gate ever swallowed it, this is
    # what would catch it.
    assert "[FAIL] no TPM answered" in intel_cpu_boot


def test_tpm_check_fails_without_a_tpm_device(default_boot: str):
    assert "[FAIL] no TPM answered" in default_boot


def test_tpm_check_detects_a_tis_tpm(tpm_tis_boot: str):
    assert "[PASS] TPM present at locality 0: V2_0 Tis" in tpm_tis_boot


def test_tpm_check_detects_a_crb_tpm(tpm_crb_boot: str):
    # Guards the register-ordering fix: reading TPM_INTF_CAPABILITY before
    # establishing the active interface reported a CRB TPM as V1_2 Tis,
    # because offset 0x14 holds TPM_DATA_CSUM under the CRB layout.
    assert "[PASS] TPM present at locality 0: V2_0 Crb" in tpm_crb_boot


def test_tpm_warns_that_discreteness_is_unverifiable(tpm_tis_boot: str):
    # A pass must never read as "SKINIT will work here": software cannot
    # tell a discrete TPM from a firmware one, and SKINIT needs a discrete.
    assert "[WARN] software cannot tell a discrete TPM" in tpm_tis_boot
