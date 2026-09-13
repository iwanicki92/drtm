# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Exercises the AMD DRTM precondition checks (see `src/amd/checks.rs`).

QEMU 8.2.2 refuses to set the `skinit` CPUID feature bit under both TCG and
KVM ("TCG doesn't support requested feature" / "host doesn't support
requested feature"), on every CPU model tried, including AMD's own `EPYC`.
So only the "unsupported" branch of this check can be exercised here; see
the design spec for detail.
"""


def test_skinit_check_fails_without_cpu_support(default_boot: str):
    assert "[FAIL] SKINIT CPUID bit not set" in default_boot


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
