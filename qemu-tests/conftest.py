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

import re
import subprocess
import time
from datetime import datetime
from pathlib import Path

import pytest
from qemu_vm import QemuVm

REPO_ROOT = Path(__file__).parent.parent
EFI_BINARY = REPO_ROOT / "target" / "x86_64-unknown-uefi" / "debug" / "drtm.efi"

LOGS_DIR = Path(__file__).parent / "logs"
_RUN_DIR_RE = re.compile(r"^(\d{4})-")
_MAX_RUN_NUMBER = 9999

# The last line src/main.rs prints. Reading until this marker means the
# capture holds every check's result rather than a partially booted guest.
CHECKS_COMPLETE = "drtm: checks complete"

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


def boot_and_capture(efi_binary: Path, log_dir: Path) -> str:
    """Boots `efi_binary` once and returns everything it printed.

    Shared by the per-configuration boot fixtures below. A configuration
    needing different QEMU arguments gets its own fixture calling this, and a
    matching parameter here and on `QemuVm`.
    """
    log_dir.mkdir(parents=True, exist_ok=True)
    with QemuVm(efi_binary, log_dir=log_dir) as vm:
        return vm.read_serial_until(CHECKS_COMPLETE, timeout=BOOT_TIMEOUT)


@pytest.fixture(scope="session")
def default_boot(run_log_dir: Path, efi_binary: Path) -> str:
    """Serial output of one boot under QEMU's default configuration: no TPM
    device attached, and a CPU model that reports neither AMD nor `SKINIT`.
    """
    return boot_and_capture(efi_binary, run_log_dir / "boot-default")


_reports: list[pytest.TestReport] = []
_session_start = 0.0


def pytest_sessionstart(session: pytest.Session) -> None:
    global _session_start
    _session_start = time.monotonic()


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
