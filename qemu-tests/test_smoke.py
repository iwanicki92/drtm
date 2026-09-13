# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Checks the drtm binary boots and runs to completion without crashing.

Uses Dasharo's coreboot+UEFI firmware for QEMU Q35 rather than stock OVMF.
See CLAUDE.md and firmware.py for why.
"""

from conftest import CHECKS_COMPLETE


def test_binary_boots_and_prints_hello(default_boot: str):
    assert "drtm: hello from UEFI" in default_boot


def test_binary_runs_every_check_to_completion(default_boot: str):
    # The boot fixture reads until this marker, so reaching it is what proves
    # the binary got past every check rather than hanging or faulting midway.
    assert CHECKS_COMPLETE in default_boot
