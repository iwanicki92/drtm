# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Exercises the AMD DRTM precondition checks (see `src/amd/checks.rs`).

QEMU 8.2.2 refuses to set the `skinit` CPUID feature bit under both TCG and
KVM ("TCG doesn't support requested feature" / "host doesn't support
requested feature"), on every CPU model tried, including AMD's own `EPYC`.
So only the "unsupported" branch of this check can be exercised here; see
the design spec for detail. Which *reason* it reports is still testable,
since the vendor string follows the CPU model.
"""

import re

# Size and alignment SKINIT requires of the Secure Loader Block, mirroring
# SLB_SIZE in src/amd/memory.rs.
SLB_SIZE = 64 * 1024


def test_skinit_check_fails_and_blames_the_vendor_on_a_non_amd_cpu(
    default_boot: str,
):
    assert "[FAIL] SKINIT CPUID bit not set" in default_boot
    assert "does not report itself as AMD" in default_boot


def test_skinit_check_fails_without_blaming_the_vendor_on_an_amd_cpu(
    amd_cpu_boot: str,
):
    # An AMD CPU lacking SKINIT is a different situation than a non-AMD one,
    # and only an AMD CPU model reaches this branch.
    assert "SKINIT unsupported on this CPU" in amd_cpu_boot
    assert "does not report itself as AMD" not in amd_cpu_boot


def test_tpm_check_fails_without_a_tpm_device(default_boot: str):
    assert "[FAIL] no TPM answered" in default_boot


def test_tpm_check_detects_a_tis_tpm(tpm_tis_boot: str):
    assert "[PASS] TPM present at locality 0: V2_0 Tis" in tpm_tis_boot


def test_tpm_check_detects_a_crb_tpm(tpm_crb_boot: str):
    # Guards the register-ordering fix: reading TPM_INTF_CAPABILITY before
    # establishing the active interface reported a CRB TPM as V1_2 Tis,
    # because offset 0x14 holds TPM_DATA_CSUM under the CRB layout.
    assert "[PASS] TPM present at locality 0: V2_0 Crb" in tpm_crb_boot


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


def test_tpm_warns_that_discreteness_is_unverifiable(tpm_tis_boot: str):
    # A pass must never read as "SKINIT will work here": software cannot
    # tell a discrete TPM from a firmware one, and SKINIT needs a discrete.
    assert "[WARN] software cannot tell a discrete TPM" in tpm_tis_boot
