# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""`launch=foreign` with AMD's client SL, which `DRTM_TEST_FOREIGN_SL`
names (see `docs/foreign-sl.md`). Skipped without it.

The SL reports only through POST codes on port 0x80, so these tests read
the log `isa-post-code` wrote rather than the console.
"""

import pytest
from conftest import (
    FOREIGN_SL,
    SESSION,
    STANDIN_REACHED,
    launch_state,
    qemu_log,
    qemu_traces,
)
from machine import post_codes

if FOREIGN_SL is None:
    pytest.skip("DRTM_TEST_FOREIGN_SL is not set", allow_module_level=True)

BOOT = "foreign_amd_sl_boot"

# The SL's own codes carry this prefix, the firmware's before them do not
SL = 0xB000_0000

# Each PSP command shows as 0x18CC before the wait for the mailbox, 0x19CC
# once it is sent and 0x1ACC once answered. 0x1ENN codes are failed steps.
ANSWERED = 0x1A00
FAILED = 0x1E00

# The commands AMD's SL sends, in order, when every one succeeds
COMMANDS = [
    0x4,  # LAUNCH
    0xB,  # EXTEND_OSSL_DIGEST over the stand-in
    0x9,  # GET_TMR_DESCRIPTORS
    0x9,
    0x9,
    0x9,
    0xA,  # ALLOCATE_SHARED_MEMORY
    0x5,  # PROCESS_SMM
    0x6,  # GET_SMM_POLICY
    0x7,  # GET_TCG_LOGS
]


def sl_codes() -> list[int]:
    codes = post_codes(SESSION.boot_log_dir(BOOT) / "post-codes.txt")
    return [code & 0xFFFF for code in codes if code & 0xFFFF_0000 == SL]


def test_the_sl_hands_off_to_the_stand_in(foreign_amd_sl_boot: str):
    codes = post_codes(SESSION.boot_log_dir(BOOT) / "post-codes.txt")
    assert codes[-1] == STANDIN_REACHED
    assert sl_codes()[:2] == [0x1000, 0x1001]


def test_every_psp_command_is_answered(foreign_amd_sl_boot: str):
    codes = sl_codes()
    assert not [code for code in codes if code & 0xFF00 == FAILED]
    answered = [code & 0xFF for code in codes if code & 0xFF00 == ANSWERED]
    assert answered == COMMANDS


def test_the_psp_verifies_its_signature(foreign_amd_sl_boot: str):
    state = launch_state(BOOT)
    assert state["psp"]["launch"] == "ok"
    assert state["psp"]["signature"] == "verified"
    statuses = [
        line for line in qemu_traces(BOOT) if line.startswith("amd_psp_drtm_status")
    ]
    assert statuses
    assert all(line.endswith(" status 0x0") for line in statuses)
    assert qemu_log(BOOT) == ""
