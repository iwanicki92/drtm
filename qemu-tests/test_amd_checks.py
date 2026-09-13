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

from pathlib import Path

from qemu_vm import QemuVm


def test_skinit_check_fails_without_cpu_support(test_log_dir, efi_binary: Path):
    with QemuVm(efi_binary, log_dir=test_log_dir) as vm:
        output = vm.read_serial_until("SKINIT CPUID bit")
    assert "[FAIL]" in output
