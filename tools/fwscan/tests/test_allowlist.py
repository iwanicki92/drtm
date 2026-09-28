# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

from fwscan import allowlist
from synthetic import NOP, SHA256, SHA384, ta_code

DIGEST_256 = bytes(range(0x20))
DIGEST_384 = bytes(range(0x40, 0x70))


def test_sha256_digest_is_found():
    [digest] = allowlist.find(ta_code(SHA256, DIGEST_256))
    assert digest == allowlist.Digest("sha256", DIGEST_256, 0x150)


def test_sha384_digest_is_found():
    [digest] = allowlist.find(ta_code(SHA384, DIGEST_384))
    assert digest == allowlist.Digest("sha384", DIGEST_384, 0x150)


def test_constants_built_from_halves_are_followed():
    [digest] = allowlist.find(ta_code(SHA384, DIGEST_384, halves=True))
    assert digest == allowlist.Digest("sha384", DIGEST_384, 0x150)


def test_halves_pointer_outside_the_ta_is_ignored():
    assert (
        allowlist.find(ta_code(SHA256, DIGEST_256, pointer=0x1000, halves=True)) == []
    )


def test_token_length_set_before_the_algorithm_is_found():
    [digest] = allowlist.find(ta_code(SHA384, DIGEST_384, length_first=True))
    assert digest == allowlist.Digest("sha384", DIGEST_384, 0x150)


def test_loop_bound_must_match_the_algorithm():
    assert allowlist.find(ta_code(SHA256, DIGEST_256, length=0x30)) == []


def test_token_length_must_be_0x440():
    assert allowlist.find(ta_code(SHA256, DIGEST_256, token_len=NOP + NOP)) == []


def test_pointer_outside_the_ta_is_ignored():
    assert allowlist.find(ta_code(SHA256, DIGEST_256, pointer=0x1000)) == []


def test_other_algorithms_are_ignored():
    assert allowlist.find(ta_code(0x50000006, DIGEST_256)) == []


def test_one_digest_checked_twice_is_listed_once():
    ta = ta_code(SHA256, DIGEST_256)
    assert len(allowlist.find(ta + ta[0x100:])) == 1
