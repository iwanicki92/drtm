# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""Checks the drtm binary boots and runs to completion without crashing.

Uses Dasharo's coreboot+UEFI firmware for QEMU Q35 rather than stock OVMF.
See CLAUDE.md and firmware.py for why.
"""

import re
from collections import Counter

from conftest import (
    CHECKS_COMPLETE,
    completed,
    messages,
    records,
    summary,
)


def test_binary_boots_and_prints_hello(default_boot: str):
    assert "hello from UEFI" in messages(default_boot)


def test_binary_runs_every_check_to_completion(default_boot: str):
    # The boot fixture reads until the marker its form ends with, so reaching
    # it is what proves the binary got past every check rather than hanging or
    # faulting midway.
    assert completed(default_boot)


def test_the_summary_counts_every_result(default_boot: str):
    # `records` refuses a malformed record, so this also holds every line of
    # this boot to being parseable JSON.
    counted = Counter(
        record["result"] for record in records(default_boot) if record["rec"] == "check"
    )
    tally = summary(default_boot)
    assert {tag: tally[tag] for tag in counted} == dict(counted)
    # No TPM answers on this boot, so one check fails and that decides it.
    assert tally["verdict"] == "fail"


def test_text_mode_prints_prose_instead(tpm_crb_boot: str):
    # The one boot with no `drtm.conf`, so it runs in the default form and
    # keeps the human-readable one under test.
    assert records(tpm_crb_boot) == []
    assert "drtm: [PASS] tpm.present: " in tpm_crb_boot
    assert CHECKS_COMPLETE in tpm_crb_boot


def test_text_mode_ends_with_the_same_tally(tpm_crb_boot: str):
    # Prose carries the counts too, so nobody has to add up the lines above.
    match = re.search(r"checks complete, verdict (\w+): (.*)", tpm_crb_boot)
    assert match is not None
    tally = {
        tag: int(count)
        for count, tag in (pair.split() for pair in match.group(2).strip().split(", "))
    }
    counted = Counter(
        tag.lower()
        for tag in re.findall(r"drtm: \[(PASS|FAIL|WARN|SKIP|INFO)\] ", tpm_crb_boot)
    )
    assert tally == {tag: counted.get(tag, 0) for tag in tally}
    expected = "fail" if tally["fail"] else "warn" if tally["warn"] else "pass"
    assert match.group(1) == expected
