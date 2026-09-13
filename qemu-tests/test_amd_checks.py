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
