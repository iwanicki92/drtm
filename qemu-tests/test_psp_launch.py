# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""The PSP DRTM launch: `SKINIT`, then the service's `LAUNCH` and every
command after it, as the Secure Loader and the DLME report them.

The PCRs are predicted here from `target/sl-psp.bin`, apart from what the
build predicted, so a wrong prediction in either shows.
"""

import hashlib
from pathlib import Path

from conftest import (
    DLME,
    LONG_EXTEND_MS,
    SL_PSP,
    launch_state,
    qemu_log,
    qemu_traces,
    records,
)
from test_launch import STACK_LIMIT, after_launch, checks, measured_image, stack_used

# The signature block behind the measured image: the `$AS1` header, then
# the signature, then the key token.
SIG_HEADER_LEN = 0x100
SIGNATURE_LEN = 0x200
TOKEN_LEN = 0x440

# The mailbox steps after `LAUNCH`, in the guide's order. `TMR_RELEASE` is
# the DLME's.
AFTER_LAUNCH = [
    "launch.psp_extend",
    "launch.psp_tmrs",
    "launch.psp_shared",
    "launch.psp_log",
    "launch.psp_ivrs",
    "launch.psp_teardown",
    "launch.psp_tmr",
]


def sha256(data: bytes) -> bytes:
    return hashlib.sha256(data).digest()


def extend(pcr: bytes, digest: bytes) -> bytes:
    return sha256(pcr + digest)


def token() -> bytes:
    image = SL_PSP.read_bytes()
    at = len(measured_image(SL_PSP)) + SIG_HEADER_LEN + SIGNATURE_LEN
    return image[at : at + TOKEN_LEN]


def expected_pcr17(anti_rollback: int, tsme: int) -> str:
    """SHA-256 PCR 17 after `LAUNCH`: the `SKINIT` value, then an SPLT
    version of 0, then the TSME and anti-rollback words."""
    pcr = extend(bytes(32), sha256(measured_image(SL_PSP)))
    pcr = extend(pcr, sha256((0).to_bytes(4, "little")))
    words = tsme.to_bytes(4, "little") + anti_rollback.to_bytes(4, "little")
    return extend(pcr, sha256(words)).hex()


def expected_pcr18() -> str:
    """SHA-256 PCR 18 after `LAUNCH`: the key token, then a SecPatchLevel
    of 0."""
    pcr = extend(bytes(32), sha256(token()))
    return extend(pcr, sha256((0).to_bytes(4, "little"))).hex()


def with_dlme(pcr: str, dlme: Path = DLME) -> str:
    """SHA-256 `pcr` after `EXTEND_OSSL_DIGEST` of the DLME in `dlme`: its
    digest, then the marker's."""
    extended = extend(bytes.fromhex(pcr), sha256(dlme.read_bytes()))
    return extend(extended, sha256(b"AMDSL")).hex()


def result(capture: str, identifier: str) -> str:
    """The one result `identifier` reported after the launch."""
    (found,) = [
        record["result"]
        for record in after_launch(capture)
        if record.get("id") == identifier
    ]
    return found


def sha256_pcr17(capture: str) -> dict:
    (found,) = [
        r for r in checks(capture, "launch.pcr17") if r["fields"]["bank"] == "sha256"
    ]
    return found


def sl_summary(capture: str) -> dict:
    """The summary the DLME prints for itself and the SL, the last of the
    two."""
    return [r for r in records(capture) if r["rec"] == "summary"][-1]


def test_auto_takes_the_psp_path(psp_launch_boot: str):
    (start,) = checks(psp_launch_boot, "launch.start")
    assert start["fields"]["path"] == "psp"
    assert not checks(psp_launch_boot, "launch.path")
    (service,) = checks(psp_launch_boot, "launch.psp_service")
    assert service["fields"]["source"] == "ASPT"
    assert service["fields"]["ivrs"] is True
    assert service["fields"]["expected_pcr18"] == expected_pcr18()


def test_every_psp_step_passes(psp_launch_boot: str):
    for identifier in [
        "launch.psp_launch",
        "launch.locality",
        "launch.pcr18",
        "launch.pcr19_22",
        *[step for step in AFTER_LAUNCH if step != "launch.psp_shared"],
        "launch.psp_probe",
        "launch.dlme_pcr17",
        "launch.dlme_pcr18",
    ]:
        assert result(psp_launch_boot, identifier) == "pass", identifier
    assert sha256_pcr17(psp_launch_boot)["result"] == "pass"
    # The guide says nothing of the buffer's use, so it is only reported.
    (shared,) = checks(psp_launch_boot, "launch.psp_shared")
    assert shared["result"] == "info"
    assert (shared["fields"]["over_slb"], shared["fields"]["written"]) == (False, True)
    summary = sl_summary(psp_launch_boot)
    assert (summary["fail"], summary["warn"], summary["verdict"]) == (0, 0, "pass")


def test_the_extend_measures_the_dlme(psp_launch_boot: str):
    (placed,) = checks(psp_launch_boot, "launch.dlme_placed")
    (extend,) = checks(psp_launch_boot, "launch.psp_extend")
    block = int(placed["fields"]["block"], 16)
    assert block % (64 << 20) == 0
    assert int(extend["fields"]["base"], 16) == block
    assert int(extend["fields"]["size"], 16) == len(DLME.read_bytes())
    assert extend["fields"]["digest"] == sha256(DLME.read_bytes()).hex()
    assert (extend["fields"]["aligned"], extend["fields"]["matches_build"]) == (
        True,
        True,
    )
    assert 0 <= extend["fields"]["elapsed_ms"] < 10000
    assert f"answered in {extend['fields']['elapsed_ms']} ms" in extend["detail"]


def test_the_dlme_judges_the_final_pcrs(psp_launch_boot: str):
    (dlme,) = checks(psp_launch_boot, "launch.dlme")
    assert dlme["fields"]["from"] == "drtm's PSP SL"
    (launch,) = checks(psp_launch_boot, "launch.psp_launch")
    words = {k: int(launch["fields"][k], 16) for k in ("anti_rollback", "tsme")}
    (pcr17,) = checks(psp_launch_boot, "launch.dlme_pcr17")
    assert pcr17["fields"]["pcr17"] == with_dlme(expected_pcr17(**words))
    (pcr18,) = checks(psp_launch_boot, "launch.dlme_pcr18")
    assert pcr18["fields"]["pcr18"] == with_dlme(expected_pcr18())


def test_the_launch_measures_as_predicted(psp_launch_boot: str):
    (launch,) = checks(psp_launch_boot, "launch.psp_launch")
    words = {k: int(launch["fields"][k], 16) for k in ("anti_rollback", "tsme")}
    assert sha256_pcr17(psp_launch_boot)["fields"]["pcr17"] == expected_pcr17(**words)
    (pcr18,) = checks(psp_launch_boot, "launch.pcr18")
    assert pcr18["fields"]["pcr18"] == expected_pcr18()


def test_the_psp_steps_run_in_order(psp_launch_boot: str):
    ids = [
        record["id"]
        for record in after_launch(psp_launch_boot)
        if record.get("id", "").startswith("launch.psp_")
        and record["id"] not in ("launch.psp_seize", "launch.psp_service")
    ]
    assert ids == [
        "launch.psp_tmr_setup",
        "launch.psp_launch",
        *AFTER_LAUNCH,
        "launch.psp_probe",
    ]


def test_the_tmr_blocks_the_probe_until_released(psp_launch_boot: str):
    (probe,) = checks(psp_launch_boot, "launch.psp_probe")
    assert (probe["fields"]["before"], probe["fields"]["after"]) == (False, True)
    state = launch_state("psp_launch_boot")
    assert state["psp"]["launch"] == "ok"
    assert state["psp"]["signature"] == "verified"
    assert not [b for b in state["dma-blocks"] if b["name"].startswith("tmr")]
    assert qemu_log("psp_launch_boot") == ""


def test_qemu_traces_the_launch(psp_launch_boot: str):
    events = [line.split(" ", 1)[0] for line in qemu_traces("psp_launch_boot")]
    for event in (
        "amd_psp_drtm_tmr_setup",
        "x86_skinit",
        "amd_psp_drtm_launch_ok",
        "amd_psp_drtm_tmr_release",
    ):
        assert event in events, event
    assert events.index("amd_psp_drtm_tmr_setup") < events.index("x86_skinit")
    assert events.index("x86_skinit") < events.index("amd_psp_drtm_launch_ok")


def test_the_tmr_follows_exit_boot_services(psp_launch_boot: str):
    # Under the TMR devices cannot reach the low 4 GiB, and firmware
    # drivers stopping theirs in ExitBootServices() would hang.
    found = after_launch(psp_launch_boot)
    details = [record.get("detail") for record in found]
    exited = details.index("boot services exited")
    (setup,) = [i for i, r in enumerate(found) if r.get("id") == "launch.psp_tmr_setup"]
    assert exited < setup
    assert found[setup]["result"] == "pass"
    assert found[setup]["fields"]["status"] == "DRTM_NO_ERROR"


def test_a_failed_tmr_setup_still_launches(psp_tmr_timeout_boot: str):
    (setup,) = checks(psp_tmr_timeout_boot, "launch.psp_tmr_setup")
    assert setup["result"] == "fail"
    assert setup["fields"]["status"] == "no answer"
    assert checks(psp_tmr_timeout_boot, "launch.psp_launch")
    assert records(psp_tmr_timeout_boot)[-1]["id"] == "launch.halted"


def test_the_sl_stack_has_headroom(psp_launch_boot: str):
    assert stack_used(psp_launch_boot) <= STACK_LIMIT


def test_another_splt_version_misses_pcr17(psp_splt_boot: str):
    # The log still describes the TPM, whatever the build assumed.
    assert sha256_pcr17(psp_splt_boot)["result"] == "fail"
    assert result(psp_splt_boot, "launch.pcr18") == "pass"
    assert result(psp_splt_boot, "launch.psp_log") == "pass"


def test_auto_falls_back_when_the_service_is_not_ready(psp_not_ready_boot: str):
    (path,) = checks(psp_not_ready_boot, "launch.path")
    assert "not ready" in path["detail"]
    (start,) = checks(psp_not_ready_boot, "launch.start")
    assert start["fields"]["path"] == "classic"
    assert not checks(psp_not_ready_boot, "launch.psp_launch")
    assert [r["result"] for r in checks(psp_not_ready_boot, "launch.pcr17")] == [
        "pass",
        "pass",
    ]


def blocked(capture: str, reason: str) -> None:
    """The gate refused `launch=psp` for `reason`, and nothing launched."""
    (gate,) = checks(capture, "launch.gate")
    assert gate["result"] == "fail"
    assert reason in gate["detail"]
    assert not checks(capture, "launch.start")


# Each boot is a fixture argument of its own test, so collection starts it
# with the others rather than when the test runs.
def test_launch_psp_blocks_without_a_service(psp_forced_no_service_boot: str):
    blocked(psp_forced_no_service_boot, "no Secure Processor")


def test_launch_psp_blocks_with_a_tpm_1_2(psp_forced_tpm_1_2_boot: str):
    blocked(psp_forced_tpm_1_2_boot, "TPM is 1.2")


def test_a_launch_timeout_skips_the_mailbox(psp_timeout_boot: str):
    (launch,) = checks(psp_timeout_boot, "launch.psp_launch")
    assert launch["result"] == "fail"
    # The command still waits in C2PMSG_72, the ready bit clear.
    assert int(launch["fields"]["c2pmsg_72"], 16) == 0x0004_0000
    for step in AFTER_LAUNCH:
        assert result(psp_timeout_boot, step) == "skip", step
    # The TPM steps still run, and the DLME expects nothing of LAUNCH.
    assert checks(psp_timeout_boot, "launch.locality")
    assert checks(psp_timeout_boot, "launch.pcr17")
    assert result(psp_timeout_boot, "launch.dlme_pcr17") == "skip"
    assert result(psp_timeout_boot, "launch.dlme_pcr18") == "skip"


def test_an_extend_timeout_leaves_the_pcrs_unjudged(psp_extend_timeout_boot: str):
    # The late extend may still have measured the DLME, so the DLME judges
    # neither PCR, and sends nothing to the dead mailbox.
    assert result(psp_extend_timeout_boot, "launch.psp_launch") == "pass"
    (extend,) = checks(psp_extend_timeout_boot, "launch.psp_extend")
    assert extend["result"] == "fail"
    assert extend["fields"]["elapsed_ms"] >= 10000
    assert "got no answer in 10000 ms" in extend["detail"]
    for step in AFTER_LAUNCH[1:]:
        assert result(psp_extend_timeout_boot, step) == "skip", step
    for pcr in ("launch.dlme_pcr17", "launch.dlme_pcr18"):
        (found,) = checks(psp_extend_timeout_boot, pcr)
        assert found["result"] == "skip", pcr
        assert "EXTEND_OSSL_DIGEST got no answer" in found["detail"], pcr


def test_a_built_in_extend_timeout_waits_out_a_slow_extend(
    psp_long_extend_boot: str,
):
    # The same 11 s extend as above, which a build waiting longer takes.
    (extend,) = checks(psp_long_extend_boot, "launch.psp_extend")
    assert extend["result"] == "pass"
    assert 10000 <= extend["fields"]["elapsed_ms"] < LONG_EXTEND_MS
    for step in AFTER_LAUNCH[1:]:
        assert result(psp_long_extend_boot, step) != "skip", step
    assert result(psp_long_extend_boot, "launch.dlme_pcr17") == "pass"
    assert result(psp_long_extend_boot, "launch.dlme_pcr18") == "pass"


def test_a_changed_dlme_is_neither_extended_nor_run(psp_bad_dlme_boot: str):
    (extend,) = checks(psp_bad_dlme_boot, "launch.psp_extend")
    assert extend["result"] == "skip"
    (handoff,) = checks(psp_bad_dlme_boot, "launch.handoff")
    assert handoff["result"] == "fail"
    assert not checks(psp_bad_dlme_boot, "launch.dlme")
    assert records(psp_bad_dlme_boot)[-1]["id"] == "launch.halted"
    assert sl_summary(psp_bad_dlme_boot)["verdict"] == "fail"


def refused(capture: str, boot: str, reason: str) -> None:
    """The service refused `LAUNCH` for `reason`, and the SL went on."""
    assert reason in qemu_log(boot)
    assert result(capture, "launch.psp_launch") == "fail"
    assert result(capture, "launch.psp_cap") == "pass"
    # Every later command answers as the guide's appendix A says.
    for step in AFTER_LAUNCH[:-1]:
        (declined,) = checks(capture, step)
        assert declined["result"] == "info", step
        fields = declined["fields"]
        assert fields["status"] == fields["expected"], step
    # The DLME expects nothing of a refused LAUNCH, and has no TMR to free.
    assert result(capture, "launch.dlme_pcr17") == "skip"
    assert result(capture, "launch.dlme_pcr18") == "skip"
    assert result(capture, "launch.psp_tmr") == "skip"
    assert not checks(capture, "launch.psp_probe")


def test_a_flipped_signature_byte_is_refused(psp_bad_signature_boot: str):
    refused(
        psp_bad_signature_boot,
        "psp_bad_signature_boot",
        "the signature does not verify",
    )


def test_a_flipped_modulus_byte_is_refused(psp_bad_modulus_boot: str):
    refused(
        psp_bad_modulus_boot, "psp_bad_modulus_boot", "the signature does not verify"
    )


def test_an_oversized_signed_length_is_refused(psp_bad_size_boot: str):
    refused(psp_bad_size_boot, "psp_bad_size_boot", "SizeFWSigned 0x10000 does not fit")


def test_a_short_measured_length_is_refused(psp_bad_length_boot: str):
    refused(psp_bad_length_boot, "psp_bad_length_boot", "no $AS1 signature header")


def test_a_patch_level_below_the_platform_is_refused(psp_min_spl_boot: str):
    refused(
        psp_min_spl_boot,
        "psp_min_spl_boot",
        "SecPatchLevel 0 is below the platform's 1",
    )


def test_a_refused_signature_is_recorded(psp_bad_signature_boot: str):
    state = launch_state("psp_bad_signature_boot")
    assert state["psp"]["launch"] == "failed"
    assert state["psp"]["signature"] == "failed"
    # Only the unmeasured signature changed, so SKINIT's value stands.
    assert sha256_pcr17(psp_bad_signature_boot)["result"] == "pass"


def test_no_ivrs_is_reported(psp_no_ivrs_boot: str):
    (ivrs,) = checks(psp_no_ivrs_boot, "launch.psp_ivrs")
    assert ivrs["result"] == "info"
    assert "no IVRS" in ivrs["detail"]
    assert sl_summary(psp_no_ivrs_boot)["fail"] == 0
