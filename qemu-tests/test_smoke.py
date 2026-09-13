"""Boots the drtm binary in QEMU and checks it runs without crashing.

Uses Dasharo's coreboot+UEFI firmware for QEMU Q35 rather than stock OVMF —
see CLAUDE.md and firmware.py for why.
"""

import subprocess
from pathlib import Path

from qemu_vm import QemuVm

REPO_ROOT = Path(__file__).parent.parent
EFI_BINARY = REPO_ROOT / "target" / "x86_64-unknown-uefi" / "debug" / "drtm.efi"


def build_efi_binary():
    subprocess.run(["cargo", "build"], cwd=REPO_ROOT, check=True)


def test_binary_boots_and_prints_hello():
    build_efi_binary()
    with QemuVm(EFI_BINARY) as vm:
        output = vm.read_serial_until("drtm: hello from UEFI")
    assert "drtm: hello from UEFI" in output
