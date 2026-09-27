# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""`launch=foreign` with one of AMD's SLs, client or EPYC, which
`DRTM_TEST_FOREIGN_SL` names (see `docs/foreign-sl.md`). Skipped without
it.

The SL reports only through POST codes on port 0x80, so these tests read
the log `isa-post-code` wrote, and the console for the DLME it hands off
to.
"""

import pytest
from conftest import (
    DLME_REACHED,
    FOREIGN_SL,
    REPO_ROOT,
    SESSION,
    launch_state,
    qemu_log,
    qemu_traces,
)
from machine import post_codes
from test_launch import after_launch, checks
from test_psp_launch import (
    SIG_HEADER_LEN,
    SIGNATURE_LEN,
    TOKEN_LEN,
    extend,
    sha256,
    with_dlme,
)

if FOREIGN_SL is None:
    pytest.skip("DRTM_TEST_FOREIGN_SL is not set", allow_module_level=True)

BOOT = "foreign_amd_sl_boot"

# The foreign build's own DLME and the SL it embeds, signed again where
# `DRTM_FOREIGN_SL_RESIGN` asks, in the target directory the fixture names.
FOREIGN_DLME = REPO_ROOT / "target" / "foreign-test" / "dlme.bin"
FOREIGN_IMAGE = REPO_ROOT / "target" / "foreign-test" / "sl-foreign.bin"

# Where the `$AS1` header keeps its SecPatchLevel.
SEC_PATCH_LEVEL = 0x4C

# The SL's own codes carry this prefix, the firmware's before them do not
SL = 0xB000_0000

# Each PSP command shows as 0x18CC before the wait for the mailbox, 0x19CC
# once it is sent and 0x1ACC once answered. 0x1ENN codes are failed steps.
ANSWERED = 0x1A00
FAILED = 0x1E00

# The commands AMD's SLs send, in order, when every one succeeds. The
# client SL also hands the SMM supervisor over, which the EPYC one leaves.
LAUNCH_AND_TMRS = [
    0x4,  # LAUNCH
    0xB,  # EXTEND_OSSL_DIGEST over the DLME
    0x9,  # GET_TMR_DESCRIPTORS
    0x9,
    0x9,
    0x9,
]
SMM = [
    0xA,  # ALLOCATE_SHARED_MEMORY
    0x5,  # PROCESS_SMM
    0x6,  # GET_SMM_POLICY
]
TCG_LOGS = [0x7]
CLIENT = LAUNCH_AND_TMRS + SMM + TCG_LOGS
EPYC = LAUNCH_AND_TMRS + TCG_LOGS


def foreign_pcr18() -> str:
    """SHA-256 PCR 18 after the foreign SL's `LAUNCH`: its key token, then
    its `$AS1` header's SecPatchLevel."""
    sl = FOREIGN_IMAGE.read_bytes()
    block = -(-int.from_bytes(sl[2:4], "little") // 16) * 16
    level = sl[block + SEC_PATCH_LEVEL : block + SEC_PATCH_LEVEL + 4]
    at = block + SIG_HEADER_LEN + SIGNATURE_LEN
    pcr = extend(bytes(32), sha256(sl[at : at + TOKEN_LEN]))
    return extend(pcr, sha256(level)).hex()


def sl_codes() -> list[int]:
    codes = post_codes(SESSION.boot_log_dir(BOOT) / "post-codes.txt")
    return [code & 0xFFFF for code in codes if code & 0xFFFF_0000 == SL]


def test_the_sl_hands_off_to_the_dlme(foreign_amd_sl_boot: str):
    codes = post_codes(SESSION.boot_log_dir(BOOT) / "post-codes.txt")
    assert codes[-1] == DLME_REACHED
    assert 0x1001 in sl_codes()[:2]
    (dlme,) = checks(foreign_amd_sl_boot, "launch.dlme")
    assert dlme["fields"]["from"] == "a foreign SL"


def test_the_dlme_reports_the_launch(foreign_amd_sl_boot: str):
    # Both are info after a foreign SL. PCR 18 follows from the token and
    # SecPatchLevel, and AMD's SLs extend nothing more into it.
    (service,) = checks(foreign_amd_sl_boot, "launch.psp_service")
    assert service["fields"]["expected_pcr18"] == foreign_pcr18()
    (pcr18,) = checks(foreign_amd_sl_boot, "launch.dlme_pcr18")
    assert pcr18["result"] == "info"
    assert pcr18["fields"]["matches"] is True
    assert pcr18["fields"]["pcr18"] == with_dlme(foreign_pcr18(), FOREIGN_DLME)
    (pcr17,) = checks(foreign_amd_sl_boot, "launch.dlme_pcr17")
    assert pcr17["result"] == "info"
    for identifier in ("launch.psp_tmr", "launch.psp_probe"):
        (found,) = checks(foreign_amd_sl_boot, identifier)
        assert found["result"] == "pass", identifier
    found = after_launch(foreign_amd_sl_boot)
    assert [r.get("id", r["rec"]) for r in found[-2:]] == ["summary", "launch.halted"]
    assert found[-2]["verdict"] == "pass"


def test_the_dlme_reads_what_the_sl_left(foreign_amd_sl_boot: str):
    # The header, the TMR the preamble set up, and the service's log,
    # which replays to the PCRs the DLME read.
    (handoff,) = checks(foreign_amd_sl_boot, "launch.amd_handoff")
    assert handoff["fields"]["tmr_descriptors"] == 1
    (service,) = checks(foreign_amd_sl_boot, "launch.psp_service")
    (tmr,) = checks(foreign_amd_sl_boot, "launch.amd_tmr")
    assert (tmr["fields"]["base"], tmr["fields"]["size"]) == (
        service["fields"]["tmr_base"],
        service["fields"]["tmr_size"],
    )
    (log,) = checks(foreign_amd_sl_boot, "launch.amd_log")
    assert log["fields"]["replays"] is True


def test_every_psp_command_is_answered(foreign_amd_sl_boot: str):
    codes = sl_codes()
    assert not [code for code in codes if code & 0xFF00 == FAILED]
    answered = [code & 0xFF for code in codes if code & 0xFF00 == ANSWERED]
    assert answered in (CLIENT, EPYC)


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
