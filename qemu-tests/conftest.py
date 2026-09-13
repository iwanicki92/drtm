# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Session-wide fixtures for per-run log directories under `logs/`."""

import re
from datetime import datetime
from pathlib import Path

import pytest

LOGS_DIR = Path(__file__).parent / "logs"
_RUN_DIR_RE = re.compile(r"^(\d{4})-")
_MAX_RUN_NUMBER = 9999


def _next_run_number() -> int:
    if not LOGS_DIR.exists():
        return 1
    numbers = [
        int(match.group(1))
        for path in LOGS_DIR.iterdir()
        if path.is_dir() and (match := _RUN_DIR_RE.match(path.name))
    ]
    return max(numbers, default=0) + 1


@pytest.fixture(scope="session")
def run_log_dir() -> Path:
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
    return run_dir


@pytest.fixture
def test_log_dir(run_log_dir: Path, request: pytest.FixtureRequest) -> Path:
    safe_name = re.sub(r"[^\w.-]", "_", request.node.name)
    test_dir = run_log_dir / safe_name
    test_dir.mkdir()
    return test_dir
