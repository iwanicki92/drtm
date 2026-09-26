# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""The classic SKINIT launch, from the gate to the Secure Loader's report.

The SL reports with the same records the checks do, after a second summary
of its own, so the helpers in `conftest` read both parts of a launch boot.
"""

import hashlib
import re

from conftest import SL_IMAGE, launch_state, qemu_log, records, results


def checks(capture: str, identifier: str) -> list[dict]:
    """Every check record `identifier` reported, in order."""
    return [
        record
        for record in records(capture)
        if record["rec"] == "check" and record["id"] == identifier
    ]


def measured_image() -> bytes:
    """What SKINIT measures of `target/sl.bin`: the length its header's
    second word gives. The signature block behind it is not measured."""
    image = SL_IMAGE.read_bytes()
    return image[: int.from_bytes(image[2:4], "little")]


def expected_pcr17(algorithm: str) -> str:
    """PCR 17 after SKINIT measures `target/sl.bin`: zero extended with the
    image's digest. Computed here from the file rather than taken from what
    drtm printed, so a wrong length or padding in the build shows."""
    digest = hashlib.new(algorithm, measured_image()).digest()
    return hashlib.new(algorithm, bytes(len(digest)) + digest).hexdigest()


def after_launch(capture: str) -> list[dict]:
    """The records from `launch.start` on."""
    found = records(capture)
    start = next(
        i for i, record in enumerate(found) if record.get("id") == "launch.start"
    )
    return found[start:]


def test_the_launch_measures_the_image(launch_boot: str):
    for bank in ("sha1", "sha256"):
        (pcr17,) = [
            r
            for r in checks(launch_boot, "launch.pcr17")
            if r["fields"]["bank"] == bank
        ]
        assert pcr17["result"] == "pass"
        assert pcr17["fields"]["pcr17"] == expected_pcr17(bank)
        assert pcr17["fields"]["locality"] == 2
    assert [r["result"] for r in checks(launch_boot, "launch.pcr18_22")] == [
        "pass",
        "pass",
    ]


def test_the_build_predicts_pcr17(launch_boot: str):
    (skinit,) = checks(launch_boot, "launch.skinit")
    assert skinit["fields"]["expected_sha1"] == expected_pcr17("sha1")
    assert skinit["fields"]["expected_sha256"] == expected_pcr17("sha256")
    assert skinit["fields"]["measured"] == len(measured_image())


def test_the_sl_takes_locality_2(launch_boot: str):
    assert [r["result"] for r in checks(launch_boot, "launch.locality")] == ["pass"]


def test_the_launch_runs_in_order(launch_boot: str):
    # From the gate to the halt, with every preamble step and the SL's
    # tally in between, and nothing of the SL's before the trampoline.
    found = after_launch(launch_boot)
    ids = [record.get("id", record["rec"]) for record in found]
    assert ids[:3] == ["launch.start", "launch.skinit", "launch.preamble"]
    assert ids[-2:] == ["summary", "launch.halted"]
    steps = [r["detail"] for r in found if r.get("id") == "launch.preamble"]
    assert steps[-1] == "entering the trampoline, SKINIT follows"
    assert found[-2]["verdict"] == "pass"
    assert found[-2]["fail"] == 0
    # The trampoline's marker byte, just before the SL's first line.
    assert "\nSdrtm: " in launch_boot.replace("\r", "")


def test_the_platform_saw_a_clean_launch(launch_boot: str):
    # Our QEMU checks what hardware assumes: every AP parked in INIT, the
    # header's entry inside the measured length. Strict, so a broken rule
    # would have stopped the VM before the SL printed anything.
    state = launch_state("launch_boot")
    assert state["launched"] is True
    assert state["hash"] == "ok"
    assert qemu_log("launch_boot") == ""


def test_auto_launches_classically_without_a_service(launch_boot: str):
    (path,) = checks(launch_boot, "launch.path")
    assert path["result"] == "info"
    (start,) = checks(launch_boot, "launch.start")
    assert start["fields"]["path"] == "classic"


def test_the_psp_service_holds_locality_2(launch_psp_boot: str):
    (locality,) = checks(launch_psp_boot, "launch.locality")
    assert locality["result"] == "warn"
    assert "PSP DRTM service" in locality["detail"]
    # PCR reads are allowed at every locality, so the launch still checks.
    for pcr17 in checks(launch_psp_boot, "launch.pcr17"):
        assert pcr17["result"] == "pass"
        assert pcr17["fields"]["locality"] == 0


def test_no_config_still_launches(launch_defaults_boot: str):
    # Prose at `info`, the defaults: the launch is opt-out.
    assert "drtm: [INFO] launch.halted: " in launch_defaults_boot
    tags = [tag for tag, _ in results(launch_defaults_boot, "launch.pcr17")]
    assert tags.count("PASS") == 2


def test_a_bank_without_a_computed_value_is_reported(launch_defaults_boot: str):
    (tag,) = [
        tag
        for tag, detail in results(launch_defaults_boot, "launch.pcr17")
        if detail.startswith("sha384")
    ]
    assert tag == "INFO"
    assert [tag for tag, _ in results(launch_defaults_boot, "launch.pcr18_22")] == [
        "PASS"
    ] * 3


def test_a_fail_outside_the_gate_still_launches(launch_crb_boot: str):
    # A CRB TPM here has locality 0 alone, which fails the check, and no
    # locality 4 path, so the hash never reaches it.
    assert [r["result"] for r in checks(launch_crb_boot, "tpm.locality")] == ["fail"]
    assert checks(launch_crb_boot, "launch.start")
    assert launch_state("launch_crb_boot")["launched"] is True
    assert "no locality 4 path" in qemu_log("launch_crb_boot")


def test_the_sl_reads_through_crb(launch_crb_boot: str):
    # This firmware has no TCG2 driver for a CRB TPM, so nothing sent it
    # TPM2_Startup and it answers every command with TPM_RC_INITIALIZE.
    # Getting that answer back is the CRB transport working.
    assert [r["result"] for r in checks(launch_crb_boot, "launch.locality")] == ["warn"]
    reads = checks(launch_crb_boot, "launch.pcr_read")
    assert [r["fields"]["bank"] for r in reads] == ["sha1", "sha256"]
    for read in reads:
        assert read["result"] == "fail"
        assert read["fields"]["locality"] == 0
        assert "response code 0x100" in read["detail"]


def burst_waits(name: str) -> tuple[set[int], list[str]]:
    """The localities that moved a TIS FIFO byte past the burst count in
    boot `name`, and every other line of its `qemu.log`."""
    localities = set()
    others = []
    for line in qemu_log(name).splitlines():
        found = re.search(r"tpm_tis_burst_wait locality (\d)", line)
        if found:
            localities.add(int(found.group(1)))
        else:
            others.append(line)
    return localities, others


def test_a_small_tis_burst_still_measures(launch_burst_boot: str):
    # The SL, at locality 2, waits out every burst. Firmware's TIS driver
    # at locality 0 may move a byte past one, which a TPM allows.
    for bank in ("sha1", "sha256"):
        (pcr17,) = [
            r
            for r in checks(launch_burst_boot, "launch.pcr17")
            if r["fields"]["bank"] == bank
        ]
        assert pcr17["result"] == "pass"
        assert pcr17["fields"]["pcr17"] == expected_pcr17(bank)
    assert [r["result"] for r in checks(launch_burst_boot, "launch.pcr18_22")] == [
        "pass",
        "pass",
    ]
    assert launch_state("launch_burst_boot")["hash"] == "ok"
    assert burst_waits("launch_burst_boot") in ((set(), []), ({0}, []))


def test_a_small_tpm_1_2_burst_still_measures(launch_tpm_1_2_burst_boot: str):
    (pcr17,) = checks(launch_tpm_1_2_burst_boot, "launch.pcr17")
    assert pcr17["result"] == "pass"
    assert pcr17["fields"]["pcr17"] == expected_pcr17("sha1")
    assert launch_state("launch_tpm_1_2_burst_boot")["hash"] == "ok"
    assert burst_waits("launch_tpm_1_2_burst_boot") in ((set(), []), ({0}, []))


def test_the_gate_blocks_without_a_tpm(launch_blocked_boot: str):
    (gate,) = checks(launch_blocked_boot, "launch.gate")
    assert gate["result"] == "fail"
    assert "tpm.present" in gate["detail"]
    assert not checks(launch_blocked_boot, "launch.start")
    assert launch_state("launch_blocked_boot")["launched"] is False


def test_a_tpm_1_2_measures_the_image(launch_tpm_1_2_boot: str):
    # One SHA-1 bank, read a PCR at a time. The hash transaction is the
    # same locality 4 one a TPM 2.0 gets.
    (pcr17,) = checks(launch_tpm_1_2_boot, "launch.pcr17")
    assert pcr17["result"] == "pass"
    assert pcr17["fields"]["bank"] == "sha1"
    assert pcr17["fields"]["pcr17"] == expected_pcr17("sha1")
    assert [r["result"] for r in checks(launch_tpm_1_2_boot, "launch.pcr18_22")] == [
        "pass"
    ]
    assert launch_state("launch_tpm_1_2_boot")["hash"] == "ok"
    assert qemu_log("launch_tpm_1_2_boot") == ""
