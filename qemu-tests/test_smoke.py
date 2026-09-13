# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Boots the drtm binary in QEMU and checks it runs without crashing.

Uses Dasharo's coreboot+UEFI firmware for QEMU Q35 rather than stock OVMF.
See CLAUDE.md and firmware.py for why.
"""

from pathlib import Path

from qemu_vm import QemuVm


def test_binary_boots_and_prints_hello(test_log_dir, efi_binary: Path):
    with QemuVm(efi_binary, log_dir=test_log_dir) as vm:
        output = vm.read_serial_until("drtm: hello from UEFI")
    assert "drtm: hello from UEFI" in output
