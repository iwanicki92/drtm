# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Session-wide fixtures for the QEMU tests.

Builds the `drtm` EFI binary once per session, boots it in QEMU once per
distinct QEMU configuration, and writes every run's logs (plus a `results.txt`
summarising test outcomes) to a numbered directory under `logs/`.

Booting takes ~25s, so tests do not boot QEMU themselves. They take a boot
fixture, which yields that configuration's complete serial output as a string,
and assert against it.
"""

import json
import re
import subprocess
import time
from datetime import datetime
from pathlib import Path

import pytest
from qemu_vm import (
    DEFAULT_CPU,
    MMCONFIG_ABOVE_4GIB,
    MMCONFIG_BELOW_4GIB,
    QemuVm,
    unsupported_binary,
)

REPO_ROOT = Path(__file__).parent.parent
EFI_BINARY = REPO_ROOT / "target" / "x86_64-unknown-uefi" / "debug" / "drtm.efi"

LOGS_DIR = Path(__file__).parent / "logs"
_RUN_DIR_RE = re.compile(r"^(\d{4})-")
_MAX_RUN_NUMBER = 9999

# The last thing src/main.rs prints, in each of the two output forms: the
# summary record under `format=json`, the prose marker in text mode. Reading
# until the one a boot asks for means the capture holds every check's result
# rather than a partially booted guest.
SUMMARY_RECORD = 'drtm: {"rec":"summary"'
CHECKS_COMPLETE = "drtm: checks complete"

# The `drtm.conf` most boots get: records rather than prose, so tests key on
# fields instead of wording, and `trace`, since the `[SCAN]` lines some of
# them asserts on sit below the default `info` level.
FULL_OUTPUT = "level=trace\nformat=json\n"

# A value `level=` does not accept, on line 2 behind a comment so the reported
# line number is not trivially 1.
BAD_LEVEL = "# drtm.conf\nlevel=verbose\n"

# Generous on purpose. Measured boot-to-marker is ~11s under TCG and ~23s
# under KVM on a nested-virtualisation host, but this only has to be tight
# enough to catch a genuinely stuck guest. Being slow to report a real hang
# costs one wait; being too tight turns a loaded machine into a flaky
# failure that looks exactly like one.
BOOT_TIMEOUT = 120.0

# A build this long means something is wedged, not slow: a from-scratch
# debug build of this crate is seconds.
BUILD_TIMEOUT = 300.0


@pytest.fixture(scope="session")
def efi_binary() -> Path:
    subprocess.run(["cargo", "build"], cwd=REPO_ROOT, check=True, timeout=BUILD_TIMEOUT)
    return EFI_BINARY


def _next_run_number() -> int:
    if not LOGS_DIR.exists():
        return 1
    numbers = [
        int(match.group(1))
        for path in LOGS_DIR.iterdir()
        if path.is_dir() and (match := _RUN_DIR_RE.match(path.name))
    ]
    return max(numbers, default=0) + 1


_run_log_dir: Path | None = None


def _get_run_log_dir() -> Path:
    """Creates this run's log directory on first use, then reuses it.

    Both the `run_log_dir` fixture and the result-reporting hooks need this
    path, and hooks cannot request fixtures, so it is cached here rather than
    held by the fixture.
    """
    global _run_log_dir
    if _run_log_dir is not None:
        return _run_log_dir
    number = _next_run_number()
    if number > _MAX_RUN_NUMBER:
        pytest.exit(
            f"qemu-tests/logs/ has reached run {_MAX_RUN_NUMBER}, the "
            "4-digit limit. Clean out old runs before testing again.",
            returncode=1,
        )
    now = datetime.now().astimezone()
    run_dir = LOGS_DIR / f"{number:04d}-{now.date().isoformat()}"
    run_dir.mkdir(parents=True)
    (run_dir / "timestamp.txt").write_text(now.isoformat() + "\n")
    _run_log_dir = run_dir
    return run_dir


@pytest.fixture(scope="session")
def run_log_dir() -> Path:
    return _get_run_log_dir()


def boot_and_capture(
    efi_binary: Path,
    log_dir: Path,
    tpm: str | None = None,
    cpu: str = DEFAULT_CPU,
    iommu: str | None = None,
    ivrs: bool = True,
    config: str | None = FULL_OUTPUT,
    extra_root_bridge: bool = False,
    corrected_error_bank: int | None = None,
    psp: int | None = None,
) -> str:
    """Boots `efi_binary` once and returns everything it printed.

    Shared by the per-configuration boot fixtures below. A configuration
    needing different QEMU arguments gets its own fixture calling this, and a
    matching parameter here and on `QemuVm`.

    Which marker ends the read follows from `config`, since the two output
    forms end differently.
    """
    log_dir.mkdir(parents=True, exist_ok=True)
    with QemuVm(
        efi_binary,
        log_dir=log_dir,
        tpm=tpm,
        cpu=cpu,
        iommu=iommu,
        ivrs=ivrs,
        config=config,
        extra_root_bridge=extra_root_bridge,
        corrected_error_bank=corrected_error_bank,
        psp=psp,
    ) as vm:
        marker = (
            SUMMARY_RECORD if config and "format=json" in config else CHECKS_COMPLETE
        )
        return vm.read_serial_until(marker, timeout=BOOT_TIMEOUT)


@pytest.fixture(scope="session")
def default_boot(run_log_dir: Path, efi_binary: Path) -> str:
    """Serial output of one boot under QEMU's default configuration: no TPM
    device attached, and an AMD CPU model reporting `skinit` and an MMIO
    configuration window below 4 GiB.

    AMD by default so this fixture runs every check rather than skipping the
    vendor-specific ones, which is most of them. Both of those CPU options
    are ones only our QEMU accepts.
    """
    return boot_and_capture(
        efi_binary, run_log_dir / "boot-default", cpu=MMCONFIG_BELOW_4GIB
    )


@pytest.fixture(scope="session")
def mmconfig_above_4gib_boot(run_log_dir: Path, efi_binary: Path) -> str:
    """Serial output of one boot whose MMIO configuration window does not fit
    below 4 GiB.

    The only configuration reaching that check's second enabled outcome.
    Nothing else here differs from `default_boot`, down to the bus count.
    """
    return boot_and_capture(
        efi_binary, run_log_dir / "boot-mmconfig-above-4gib", cpu=MMCONFIG_ABOVE_4GIB
    )


@pytest.fixture(scope="session")
def intel_cpu_boot(run_log_dir: Path, efi_binary: Path) -> str:
    """Serial output of one boot on an Intel CPU model, with no TPM device.

    Exists to exercise the path where the AMD checks are skipped and the
    vendor-independent ones still run.
    """
    return boot_and_capture(
        efi_binary, run_log_dir / "boot-intel-cpu", cpu="Skylake-Client"
    )


@pytest.fixture(scope="session")
def machine_check_error_boot(run_log_dir: Path, efi_binary: Path) -> str:
    """Serial output of one boot with a corrected error logged in machine-check
    bank 1 before the firmware runs, as a warm reboot can leave one.

    The only configuration reaching the logged-error outcome of that check.
    Its tests only read results, so the boot also carries an enabled
    `amd-iommu` left out of ACPI, which leaves the guest no IVRS table.
    """
    return boot_and_capture(
        efi_binary,
        run_log_dir / "boot-machine-check-error",
        corrected_error_bank=1,
        iommu="enabled",
        ivrs=False,
    )


@pytest.fixture(scope="session")
def amd_iommu_boot(run_log_dir: Path, efi_binary: Path) -> str:
    """Serial output of one boot with QEMU's `amd-iommu` device attached and
    its capability left disabled, as upstream QEMU always leaves it.

    So `amd.iommu` warns here. This boot also keeps the plain `EPYC` model,
    which is where the `skinit` check's unsupported path is exercised, and
    has a second root bridge, so the IOMMU is one device among two bridges.
    """
    return boot_and_capture(
        efi_binary,
        run_log_dir / "boot-amd-iommu",
        iommu="disabled",
        extra_root_bridge=True,
    )


@pytest.fixture(scope="session")
def amd_iommu_enabled_boot(run_log_dir: Path, efi_binary: Path) -> str:
    """Serial output of one boot with an `amd-iommu` whose capability is
    enabled, which is what firmware does on a machine that has one.

    The only configuration where `amd.iommu` can pass, and only our QEMU
    offers it. Its tests only read results, so the boot also carries a
    `drtm.conf` naming a level that does not exist, and a Secure Processor
    device with the Turin ASP's device ID.
    """
    return boot_and_capture(
        efi_binary,
        run_log_dir / "boot-amd-iommu-enabled",
        iommu="enabled",
        config=BAD_LEVEL,
        psp=0x156E,
    )


@pytest.fixture(scope="session")
def tpm_tis_boot(run_log_dir: Path, efi_binary: Path) -> str:
    """Serial output of one boot with an `swtpm`-backed TPM 2.0 on the FIFO
    (TIS) interface, the one layout with a register per locality.
    """
    return boot_and_capture(efi_binary, run_log_dir / "boot-tpm-tis", tpm="tis")


@pytest.fixture(scope="session")
def tpm_crb_boot(run_log_dir: Path, efi_binary: Path) -> str:
    """Serial output of one boot with an `swtpm`-backed TPM 2.0 on the CRB
    interface.

    Boots with no `drtm.conf`, which the TPM tests do not notice, so this
    boot also covers the defaults.
    """
    return boot_and_capture(
        efi_binary, run_log_dir / "boot-tpm-crb", tpm="crb", config=None
    )


# Every check reports a tag, an identifier and a detail (see `src/check.rs`),
# as a JSON record under `format=json` and as `[TAG] <identifier>: <detail>`
# in text mode. Tests key on the tag and the identifier, so rewording a detail
# is not a test failure. These two patterns tie the ends of that contract
# together, and `results` reads whichever form a boot is in.
_RESULT_RE = r"\[(PASS|FAIL|WARN|SKIP|INFO)\] {identifier}: (.*)"
_RECORD_PREFIX = "drtm: {"
_ANSI_RE = re.compile(r"\x1b\[[0-9;]*[a-zA-Z]")


def records(capture: str) -> list[dict]:
    """Every record the guest emitted, in order. Empty for a text-mode boot.

    A line carrying the prefix but no usable record is an error rather than
    something to skip: firmware shares this UART but does not write that
    prefix, so the line is ours and malformed.
    """
    found = []
    for line in _ANSI_RE.sub("", capture).splitlines():
        at = line.find(_RECORD_PREFIX)
        if at < 0:
            continue
        text = line[at + len("drtm: ") :]
        try:
            found.append(json.loads(text))
        except json.JSONDecodeError as error:
            raise ValueError(f"unparsable record: {text!r}") from error
    return found


def results(capture: str, identifier: str) -> list[tuple[str, str]]:
    """Every result `identifier` reported, as (tag, detail) pairs in order.

    Empty when the check did not report at all, which is what a vendor gate
    or a phase that never ran looks like.
    """
    checks = [
        (record["result"].upper(), record["detail"])
        for record in records(capture)
        if record["rec"] == "check" and record["id"] == identifier
    ]
    if checks:
        return checks
    pattern = _RESULT_RE.format(identifier=re.escape(identifier))
    return [
        (match.group(1), match.group(2).strip())
        for match in re.finditer(pattern, capture)
    ]


def messages(capture: str) -> list[str]:
    """Every freeform log message, without the prefix or the source location.

    The `[SCAN]` lines and the few plain `info!` calls, which carry no
    identifier of their own.
    """
    from_records = [
        record["msg"] for record in records(capture) if record["rec"] == "log"
    ]
    if from_records:
        return from_records
    return [
        line.split("drtm: ", 1)[1]
        for line in _ANSI_RE.sub("", capture).splitlines()
        if "drtm: " in line
    ]


def summary(capture: str) -> dict:
    """The record a JSON-mode run ends with: a count per tag and a verdict."""
    (found,) = [record for record in records(capture) if record["rec"] == "summary"]
    return found


def completed(capture: str) -> bool:
    """Whether the run reached its end, in whichever form it printed."""
    return SUMMARY_RECORD in capture or CHECKS_COMPLETE in capture


def verdicts(capture: str, identifier: str) -> list[str]:
    """Just the tags from `results`, for the common case of not caring what
    the detail said.
    """
    return [tag for tag, _ in results(capture, identifier)]


_reports: list[pytest.TestReport] = []
_session_start = 0.0


def pytest_sessionstart(session: pytest.Session) -> None:
    global _session_start
    _session_start = time.monotonic()
    # Upstream QEMU refuses the default boot's options, which would otherwise
    # surface as every test erroring on a boot that never started.
    reason = unsupported_binary()
    if reason is not None:
        pytest.exit(
            f"This suite needs our QEMU build, see docs/testing.md: {reason}",
            returncode=1,
        )


def pytest_runtest_logreport(report: pytest.TestReport) -> None:
    _reports.append(report)


def _outcome_per_test() -> dict[str, tuple[str, float]]:
    """Collapses every phase report into one outcome and duration per test.

    All three phases are summed, because a boot fixture's cost lands in setup
    and its teardown in the last test's teardown. Attributing only the `call`
    phase would report every test as taking 0.0s. A non-passing phase decides
    the outcome, so a failed fixture shows up as that test's error rather than
    being lost.
    """
    collapsed: dict[str, tuple[str, float]] = {}
    for report in _reports:
        outcome, duration = collapsed.get(report.nodeid, ("passed", 0.0))
        if report.outcome != "passed":
            outcome = report.outcome
        collapsed[report.nodeid] = (outcome, duration + report.duration)
    return collapsed


def pytest_sessionfinish(session: pytest.Session, exitstatus: int) -> None:
    """Writes `results.txt` next to the run's QEMU logs, so a log directory
    records what was being tested and how it went, not just what the guest
    printed.
    """
    if not _reports:
        return

    collapsed = _outcome_per_test()
    counts: dict[str, int] = {}
    for outcome, _ in collapsed.values():
        counts[outcome] = counts.get(outcome, 0) + 1
    summary = ", ".join(
        f"{count} {outcome}" for outcome, count in sorted(counts.items())
    )

    lines = [
        f"finished:    {datetime.now().astimezone().isoformat()}",
        f"exit status: {exitstatus}",
        f"summary:     {summary} in {time.monotonic() - _session_start:.1f}s",
        "",
    ]
    lines += [
        f"{outcome.upper():<7} {duration:7.1f}s  {nodeid}"
        for nodeid, (outcome, duration) in collapsed.items()
    ]

    failures = [report for report in _reports if report.failed]
    if failures:
        lines.append("")
        for report in failures:
            lines += [
                f"=== {report.nodeid} ({report.when}) ===",
                report.longreprtext,
                "",
            ]

    (_get_run_log_dir() / "results.txt").write_text("\n".join(lines) + "\n")
