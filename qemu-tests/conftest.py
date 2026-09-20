# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""Session-wide fixtures for the QEMU tests.

Builds the `drtm` EFI binary once per session, boots it in QEMU once per
distinct QEMU configuration, and writes every run's logs (plus a `results.txt`
summarising test outcomes) to a numbered directory under `logs/`.

Booting takes ~5s, so tests do not boot QEMU themselves. They take a boot
fixture, which yields that configuration's complete serial output as a string,
and assert against it. Collection starts every boot the run needs, several at
a time, so the fixture is usually just waiting on one that is already going.
"""

import functools
import json
import re
import subprocess
import tempfile
from collections.abc import Callable
from pathlib import Path
from types import FunctionType

import pytest
from drtmtest.dasharo import BOOT_PROMPT, FIRMWARE, warmed_firmware
from drtmtest.qemu_vm import QemuVm
from drtmtest.session import BootSession
from machine import (
    MMCONFIG_ABOVE_4GIB,
    MMCONFIG_AS_MAPPED,
    MMCONFIG_BELOW_4GIB,
    inject_corrected_error,
    make_esp,
    options,
    unsupported_binary,
)

REPO_ROOT = Path(__file__).parent.parent
EFI_BINARY = REPO_ROOT / "target" / "x86_64-unknown-uefi" / "debug" / "drtm.efi"

LOGS_DIR = Path(__file__).parent / "logs"
CACHE_DIR = Path(__file__).parent / "dl-cache"

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

# Generous on purpose. Measured boot-to-marker is ~5s under TCG, but this
# only has to be tight enough to catch a genuinely stuck guest. Being slow
# to report a real hang costs one wait; being too tight turns a loaded
# machine into a flaky failure that looks exactly like one.
BOOT_TIMEOUT = 120.0

# The firmware's boot prompt is the first thing on the serial console, so
# the allowance is only against a machine under load.
BOOT_PROMPT_TIMEOUT = 60.0

# A build this long means something is wedged, not slow: a from-scratch
# debug build of this crate is seconds.
BUILD_TIMEOUT = 300.0


_built_binary: Path | None = None


def built_efi_binary() -> Path:
    """Builds `drtm.efi` on first use and returns where it landed.

    A function rather than a fixture: the boots that need it are started
    from a collection hook, and hooks cannot request fixtures.
    """
    global _built_binary
    if _built_binary is None:
        subprocess.run(
            ["cargo", "build"], cwd=REPO_ROOT, check=True, timeout=BUILD_TIMEOUT
        )
        _built_binary = EFI_BINARY
    return _built_binary


def boot_and_capture(
    log_dir: Path,
    firmware: Path,
    tpm: str | None = None,
    config: str | None = FULL_OUTPUT,
    corrected_error_bank: int | None = None,
    save_firmware_to: Path | None = None,
    **machine,
) -> str:
    """Boots `drtm.efi` once and returns everything it printed.

    Shared by the configurations below, each of which returns the keyword
    arguments it wants here. A configuration needing a QEMU argument no
    other one uses adds a parameter to `machine.options`.

    The boot prompt is answered at once, which boots the default entry
    rather than waiting out the boot manager's 3s timeout. Not reaching it
    is an error: the firmware release is pinned, so it either prints this
    or something changed that is worth seeing. Which marker ends the read
    follows from `config`, since the two output forms end differently.
    """
    marker = SUMMARY_RECORD if config and "format=json" in config else CHECKS_COMPLETE
    with tempfile.TemporaryDirectory(prefix="drtm-esp-") as scratch:
        esp = make_esp(Path(scratch), built_efi_binary(), config)
        with QemuVm(
            log_dir,
            options(esp, corrected_error_bank=corrected_error_bank, **machine),
            firmware=firmware,
            tpm=tpm,
            save_firmware_to=save_firmware_to,
        ) as vm:
            if corrected_error_bank is not None:
                inject_corrected_error(vm, corrected_error_bank)
            vm.expect(BOOT_PROMPT, BOOT_PROMPT_TIMEOUT)
            vm.send(b"\r")
            vm.expect(marker, BOOT_TIMEOUT)
            return vm.capture


def _warmed_firmware() -> Path:
    """This release's firmware image after one boot of the binary in text
    mode, made on first use."""

    def boot(scratch: Path) -> None:
        boot_and_capture(
            SESSION.run_log_dir() / "boot-warm-firmware",
            FIRMWARE.fetch(CACHE_DIR),
            config=None,
            save_firmware_to=scratch,
        )

    return warmed_firmware(CACHE_DIR, boot)


_BOOT_CONFIGS: dict[str, Callable[[], dict]] = {}


def boot_configuration(config: FunctionType):
    """Declares one boot configuration, as a session fixture of the same name.

    The decorated function returns the keyword arguments `boot_and_capture`
    takes for it. Registering it by name is what lets collection start the
    boot before any test asks for it, and the fixture waits on that boot.
    """
    name = config.__name__
    _BOOT_CONFIGS[name] = config

    @pytest.fixture(scope="session", name=name)
    def wait_for_boot() -> str:
        return SESSION.booted(name)

    wait_for_boot.__doc__ = config.__doc__
    return wait_for_boot


def _boot(name: str, log_dir: Path) -> str:
    return boot_and_capture(log_dir, _warmed_firmware(), **_BOOT_CONFIGS[name]())


@boot_configuration
def default_boot() -> dict:
    """Serial output of one boot under QEMU's default configuration: no TPM
    device attached, and an AMD CPU model reporting `skinit` and an MMIO
    configuration window below 4 GiB.

    AMD by default so this fixture runs every check rather than skipping the
    vendor-specific ones, which is most of them. Both of those CPU options
    are ones only our QEMU accepts.

    The one boot keeping QEMU's display adapter and network card, so the
    scan tests read a populated bus. Every other boot drops them, since
    initialising them costs 1.3s and no test asserts on either.
    """
    return {"cpu": MMCONFIG_BELOW_4GIB, "vga_and_nic": True}


@boot_configuration
def mmconfig_above_4gib_boot() -> dict:
    """Serial output of one boot whose MMIO configuration window does not fit
    below 4 GiB.

    The only configuration reaching that check's second enabled outcome. Its
    tests only read that one result, so the boot also reports CPUID family
    1Ah, which is the only way the dump's AMD MSR list runs here, and
    carries a Secure Processor whose DRTM service is not ready, which is
    where that mailbox is read in the form naming its address. That Secure
    Processor also publishes the DRTM ACPI device, the only boot that does.
    """
    return {
        "cpu": MMCONFIG_ABOVE_4GIB,
        "psp": 0x156E,
        "psp_ready": False,
        "psp_drtm_device": True,
    }


@boot_configuration
def intel_cpu_boot() -> dict:
    """Serial output of one boot on an Intel CPU model, with no TPM device.

    Exists to exercise the path where the AMD checks are skipped and the
    vendor-independent ones still run.
    """
    return {"cpu": "Skylake-Client"}


@boot_configuration
def machine_check_error_boot() -> dict:
    """Serial output of one boot with a corrected error logged in machine-check
    bank 1 before the firmware runs, as a warm reboot can leave one.

    The only configuration reaching the logged-error outcome of that check.
    Its tests only read results, so the boot also carries an enabled
    `amd-iommu` left out of ACPI, which leaves the guest no IVRS table, and
    a Secure Processor with no ASPT, which leaves nothing saying where its
    registers are.
    """
    return {
        "corrected_error_bank": 1,
        "iommu": "enabled",
        "ivrs": False,
        "psp": 0x156E,
        "psp_aspt": False,
    }


@boot_configuration
def amd_iommu_boot() -> dict:
    """Serial output of one boot with QEMU's `amd-iommu` device attached and
    its capability left disabled, as upstream QEMU always leaves it.

    So `amd.iommu` warns here. This boot also keeps the plain `EPYC` model,
    which is where the `skinit` check's unsupported path is exercised, and
    has a second root bridge, so the IOMMU is one device among two bridges.
    """
    return {"iommu": "disabled", "extra_root_bridge": True}


@boot_configuration
def amd_iommu_enabled_boot() -> dict:
    """Serial output of one boot with an `amd-iommu` whose capability is
    enabled, which is what firmware does on a machine that has one.

    The only configuration where `amd.iommu` can pass, and only our QEMU
    offers it. Its tests only read results, so the boot also carries a
    `drtm.conf` naming a level that does not exist, a Secure Processor
    device with the Turin ASP's device ID whose DRTM service is ready, and
    the one MSR value naming the window the firmware actually mapped.
    """
    return {
        "cpu": MMCONFIG_AS_MAPPED,
        "iommu": "enabled",
        "config": BAD_LEVEL,
        "psp": 0x156E,
    }


@boot_configuration
def amd_drtm_boot() -> dict:
    """Serial output of one boot on our QEMU's `-machine q35,amd-drtm=on`,
    the machine a launch boots on.

    The only configuration with the IOMMU at `0:0.2`, where the Secure
    Kernel Loader looks for it, and with AMD's northbridge at `0:18.0`. The
    plain `EPYC` model, since the machine adds `skinit` itself.
    """
    return {"amd_drtm": True}


@boot_configuration
def unlisted_psp_boot() -> dict:
    """Serial output of one boot with a Secure Processor answering with a
    device ID no loader's table carries.

    What a board newer than the loaders looks like. The only configuration
    reaching the branch that finds a Secure Processor by its PCI class
    rather than by its ID.
    """
    return {"psp": 0x17D8}


@boot_configuration
def tpm_tis_boot() -> dict:
    """Serial output of one boot with an `swtpm`-backed TPM 2.0 on the FIFO
    (TIS) interface, the one layout with a register per locality.
    """
    return {"tpm": "tis"}


@boot_configuration
def tpm_crb_boot() -> dict:
    """Serial output of one boot with an `swtpm`-backed TPM 2.0 on the CRB
    interface.

    Boots with no `drtm.conf`, which the TPM tests do not notice, so this
    boot also covers the defaults.
    """
    return {"tpm": "crb", "config": None}


def _prepare(names: list[str]) -> None:
    """The one build and the one warming boot, ahead of the pool: four
    threads racing for either would only queue up behind it."""
    built_efi_binary()
    _warmed_firmware()


def _check() -> str | None:
    # Upstream QEMU refuses the default boot's options, which would
    # otherwise surface as every test erroring on a boot that never started.
    reason = unsupported_binary()
    if reason is None:
        return None
    return f"This suite needs our QEMU build, see docs/testing.md: {reason}"


SESSION = BootSession(
    LOGS_DIR, _boot, _BOOT_CONFIGS.keys(), prepare=_prepare, check=_check
)


def pytest_configure(config: pytest.Config) -> None:
    config.pluginmanager.register(SESSION)


# Every check reports a tag, an identifier and a detail (see `src/check.rs`),
# as a JSON record under `format=json` and as `[TAG] <identifier>: <detail>`
# in text mode. Tests key on the tag and the identifier, so rewording a detail
# is not a test failure. These two patterns tie the ends of that contract
# together, and `results` reads whichever form a boot is in.
_RESULT_RE = r"\[(PASS|FAIL|WARN|SKIP|INFO)\] {identifier}: (.*)"
_RECORD_PREFIX = "drtm: {"
_ANSI_RE = re.compile(r"\x1b\[[0-9;]*[a-zA-Z]")


@functools.cache
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


def fields(capture: str, identifier: str) -> list[dict]:
    """The named values each of `identifier`'s results carried, in order.

    A check whose result is a value names it here, so a test reads the value
    instead of picking it back out of the detail. Empty for a text-mode boot,
    which does not repeat what the detail already says.
    """
    return [
        record.get("fields", {})
        for record in records(capture)
        if record["rec"] == "check" and record["id"] == identifier
    ]


def dumped(capture: str, kind: str) -> list[dict]:
    """Every platform dump record of one kind, in order.

    The dump has no prose form, so this is empty for a text-mode boot rather
    than falling back to a pattern the way `results` does.
    """
    return [record for record in records(capture) if record["rec"] == kind]


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
