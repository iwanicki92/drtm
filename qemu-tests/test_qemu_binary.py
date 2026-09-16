# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Exercises the check that the suite runs against our QEMU build.

Stand-in binaries play upstream's two ways of turning our options down, so
none of these boot anything.
"""

from pathlib import Path

import pytest
from qemu_vm import unsupported_binary


def _fake_qemu(tmp_path: Path, stderr: str, status: int) -> str:
    (tmp_path / "stderr").write_text(stderr + "\n")
    script = tmp_path / "qemu-system-x86_64"
    script.write_text(
        f"#!/bin/sh\ncat >/dev/null\ncat '{tmp_path}/stderr' >&2\nexit {status}\n"
    )
    script.chmod(0o755)
    return str(script)


def test_the_configured_binary_is_ours():
    assert unsupported_binary() is None


def test_an_unknown_property_is_reported(tmp_path, monkeypatch: pytest.MonkeyPatch):
    error = "can't apply global EPYC-x86_64-cpu.mmio-conf-base=0xe0000021"
    monkeypatch.setenv("DRTM_QEMU_BINARY", _fake_qemu(tmp_path, error, 1))
    assert unsupported_binary() == error


def test_a_dropped_skinit_bit_is_reported(tmp_path, monkeypatch: pytest.MonkeyPatch):
    warning = "warning: TCG doesn't support requested feature: ECX.skinit [bit 12]"
    monkeypatch.setenv("DRTM_QEMU_BINARY", _fake_qemu(tmp_path, warning, 0))
    reason = unsupported_binary()
    assert reason is not None and "skinit" in reason


def test_a_missing_binary_is_reported(tmp_path, monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv("DRTM_QEMU_BINARY", str(tmp_path / "absent"))
    reason = unsupported_binary()
    assert reason is not None and "not found" in reason
