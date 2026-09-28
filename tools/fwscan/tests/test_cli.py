# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

import hashlib
import json

from fwscan import keys
from fwscan.cli import Verdict, main
from synthetic import SHA256, SHA384, drtm_directory, fake_psptool, header, ta_code
from test_keys import TOKEN, signed_sl

TA_AT = 0x2000
WINDOWS_SHA256 = keys.KNOWN[0].sha256
D2DBD452_SHA384 = keys.KNOWN[1].sha384


def firmware(tmp_path, ta: bytes | None):
    image = tmp_path / "image.bin"
    data = bytearray(0x4000)
    entries = []
    if ta is not None:
        data[TA_AT : TA_AT + len(ta)] = ta
        entries.append((TA_AT, len(ta)))
    image.write_bytes(bytes(data))
    return image, fake_psptool(tmp_path, drtm_directory(*entries))


def run(capsys, *args):
    status = main([str(a) for a in args])
    return status, capsys.readouterr().out


def test_allowlist_names_the_listed_keys(tmp_path, capsys):
    image, psptool = firmware(tmp_path, ta_code(SHA256, WINDOWS_SHA256))
    status, out = run(capsys, image, "--psptool", psptool, "--no-hid")
    assert status == Verdict.ALLOWLIST
    assert f"{image}: allowlist" in out
    assert "sha256 8de6f196fe89b6b5... Windows AMD SL key" in out
    assert "Windows AMD SL key: listed" in out
    assert "SKL test key: not listed" in out


def test_smm_supervisor_digest_alone_is_no_allowlist(tmp_path, capsys):
    image, psptool = firmware(tmp_path, ta_code(SHA384, D2DBD452_SHA384))
    status, out = run(capsys, image, "--psptool", psptool, "--no-hid")
    assert status == Verdict.NO_ALLOWLIST
    assert f"{image}: no-allowlist" in out
    assert "sha384 4c8a4b8c19902428... D2DBD452 AMD key" in out


def test_unknown_digest_counts_as_allowlist(tmp_path, capsys):
    image, psptool = firmware(tmp_path, ta_code(SHA384, bytes(range(0x30))))
    status, out = run(capsys, image, "--psptool", psptool, "--no-hid")
    assert status == Verdict.ALLOWLIST
    assert "unknown key" in out


def test_sl_file_adds_a_key(tmp_path, capsys):
    image, psptool = firmware(tmp_path, ta_code(SHA256, hashlib.sha256(TOKEN).digest()))
    sl = tmp_path / "amddrtm.dll"
    sl.write_bytes(signed_sl())
    status, out = run(capsys, image, "--psptool", psptool, "--no-hid", "--sl", sl)
    assert status == Verdict.ALLOWLIST
    assert "amddrtm.dll: listed" in out
    assert "Windows AMD SL key: not listed" in out


def test_plaintext_ta_without_the_pattern_is_inconclusive(tmp_path, capsys):
    image, psptool = firmware(tmp_path, header() + bytes(0x200))
    status, out = run(capsys, image, "--psptool", psptool, "--no-hid")
    assert status == Verdict.INCONCLUSIVE
    assert "no key allowlist code found" in out


def test_encrypted_ta(tmp_path, capsys):
    image, psptool = firmware(tmp_path, header(encrypted=True) + bytes(0x200))
    status, out = run(capsys, image, "--psptool", psptool)
    assert status == Verdict.ENCRYPTED
    assert "DRTM TA 4.11.0.39 at 0x2000: encrypted" in out
    assert "DRTM device _HID: none found" in out


def test_no_drtm_ta(tmp_path, capsys):
    image, psptool = firmware(tmp_path, None)
    status, out = run(capsys, image, "--psptool", psptool, "--no-hid")
    assert status == Verdict.NO_DRTM_TA
    assert f"{image}: no-drtm-ta" in out


def test_json_and_worst_verdict_over_images(tmp_path, capsys):
    image, psptool = firmware(tmp_path, ta_code(SHA256, WINDOWS_SHA256))
    status, out = run(capsys, image, image, "--psptool", psptool, "--json")
    assert status == Verdict.ALLOWLIST
    [report, _] = json.loads(out)
    assert report["verdict"] == "allowlist"
    [ta] = report["tas"]
    assert ta["verdict"] == "allowlist"
    assert ta["digests"] == [
        {
            "algorithm": "sha256",
            "digest": WINDOWS_SHA256.hex(),
            "offset": 0x150,
            "keys": ["Windows AMD SL key"],
        }
    ]
    assert report["keys"] == {
        "Windows AMD SL key": True,
        "D2DBD452 AMD key": False,
        "SKL test key": False,
    }


def test_psptool_failure_is_an_error(tmp_path, capsys):
    image = tmp_path / "image.bin"
    image.write_bytes(bytes(16))
    status = main([str(image), "--psptool", fake_psptool(tmp_path, [], returncode=1)])
    assert status == 1
    assert "psptool failed" in capsys.readouterr().err
