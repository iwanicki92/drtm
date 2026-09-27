# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

import hashlib

import pytest
from fwscan import keys

TOKEN = bytes(i % 251 for i in range(keys.TOKEN_LEN))


def signed_sl(prefix: bytes = b"MZ" + bytes(0x200)) -> bytes:
    """An SL behind `prefix`: its header, signature and token."""
    sig_header = bytearray(0x100)
    sig_header[0x10:0x14] = keys.SL_TAG
    return prefix + bytes(sig_header) + bytes(0x200) + TOKEN


def test_sl_token_is_found_behind_other_data():
    assert keys.sl_token(signed_sl()) == TOKEN


def test_missing_sl_header_is_an_error():
    with pytest.raises(ValueError, match="no \\$AS1"):
        keys.sl_token(bytes(0x1000))


def test_short_token_is_an_error():
    with pytest.raises(ValueError, match="cut short"):
        keys.sl_token(signed_sl()[:-1])


def test_key_matches_both_digests_of_its_token():
    key = keys.from_token("test", TOKEN)
    assert key.matches(hashlib.sha256(TOKEN).digest())
    assert key.matches(hashlib.sha384(TOKEN).digest())
    assert not key.matches(bytes(32))
