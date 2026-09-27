# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

import zlib

import pytest
from fwscan import psp
from synthetic import KEY_ID, drtm_directory, fake_psptool, header


def test_plaintext_ta():
    raw = header(version=(0x04, 0x2D, 0x00, 0x3B)) + b"code"
    ta = psp.parse_ta(raw, 0x1000)
    assert ta == psp.Ta(0x1000, len(raw), "4.2D.0.3B", KEY_ID, False, False, raw)


def test_encrypted_ta_has_no_image():
    ta = psp.parse_ta(header(encrypted=True, compressed=True) + b"x" * 16, 0)
    assert ta.encrypted and ta.compressed and ta.image is None


def test_compressed_ta_keeps_its_header():
    h = header(compressed=True)
    raw = h + zlib.compress(b"code" * 100) + b"\xff" * 64
    assert psp.parse_ta(raw, 0).image == h + b"code" * 100


def test_missing_magic_is_an_error():
    with pytest.raises(ValueError, match="no \\$PS1 header at 0x40"):
        psp.parse_ta(bytes(0x200), 0x40)


def test_drtm_entries_skips_other_types_and_repeats():
    directories = drtm_directory((0x3000, 0x10), (0x1000, 0x20)) + drtm_directory(
        (0x1000, 0x20)
    )
    assert psp.drtm_entries(directories) == [(0x1000, 0x20), (0x3000, 0x10)]


def test_run_psptool_parses_its_json(tmp_path):
    directories = drtm_directory((0x1000, 0x20))
    assert psp.run_psptool(fake_psptool(tmp_path, directories), tmp_path) == directories


def test_run_psptool_reports_a_failure(tmp_path):
    with pytest.raises(psp.PsptoolError, match="psptool warning"):
        psp.run_psptool(fake_psptool(tmp_path, [], returncode=1), tmp_path)


def test_run_psptool_reports_a_missing_command(tmp_path):
    with pytest.raises(psp.PsptoolError, match="cannot run"):
        psp.run_psptool(str(tmp_path / "nothing"), tmp_path)
