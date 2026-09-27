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

import dataclasses
import fnmatch
import functools
import json
import os
import re
import subprocess
import tempfile
import time
from collections.abc import Callable
from pathlib import Path
from types import FunctionType

import pytest
from drtmtest.dasharo import BOOT_PROMPT, FIRMWARE, warmed_firmware
from drtmtest.qemu_vm import QemuVm, Settings
from drtmtest.session import BootSession
from machine import (
    MMCONFIG_ABOVE_4GIB,
    MMCONFIG_AS_MAPPED,
    MMCONFIG_BELOW_4GIB,
    TRACES,
    VM_CR_AFTER_LAUNCH,
    inject_corrected_error,
    make_esp,
    options,
    patch_dlme,
    patch_sl,
    post_codes,
    unsupported_binary,
)

REPO_ROOT = Path(__file__).parent.parent
# A signed SL built elsewhere, or a PE file carrying one, for the boots of
# `test_foreign_sl_amd.py`, which are skipped without it.
FOREIGN_SL = os.environ.get("DRTM_TEST_FOREIGN_SL")
EFI_BINARY = REPO_ROOT / "target" / "x86_64-unknown-uefi" / "debug" / "drtm.efi"
# The signed SLs the build embeds in the binary, one per launch path.
SL_CLASSIC = REPO_ROOT / "target" / "sl-classic.bin"
SL_PSP = REPO_ROOT / "target" / "sl-psp.bin"
DLME = REPO_ROOT / "target" / "dlme.bin"

LOGS_DIR = Path(__file__).parent / "logs"
CACHE_DIR = Path(__file__).parent / "dl-cache"

# The last thing src/main.rs prints, in each of the two output forms: the
# summary record under `format=json`, the prose marker in text mode. Reading
# until the one a boot asks for means the capture holds every check's result
# rather than a partially booted guest.
SUMMARY_RECORD = 'drtm: {"rec":"summary"'
CHECKS_COMPLETE = "drtm: checks complete"

# What a launch boot prints last, from the DLME or the SL, and what ends one
# early: a blocked gate, a failed preamble, or the SL or the DLME refusing
# its parameters or panicking. Both forms carry the identifier as is.
LAUNCH_HALTED = "launch.halted"
LAUNCH_GATE = "launch.gate"
LAUNCH_STOPPED = [
    "launch.gate",
    "launch.params",
    "launch.panic",
    "[FAIL] launch.preamble",
    '"id":"launch.preamble","result":"fail"',
]

# The `drtm.conf` most boots get: records rather than prose, so tests key on
# fields instead of wording, and `trace`, since the `[SCAN]` lines some of
# them asserts on sit below the default `info` level.
FULL_OUTPUT = "level=trace\nformat=json\n"

# A value `level=` does not accept, on line 2 behind a comment so the reported
# line number is not trivially 1.
BAD_CONFIG = "# drtm.conf\nlevel=verbose\nlaunch=foreign\n"

# Generous on purpose. Measured boot-to-marker is ~5s under TCG, but this
# only has to be tight enough to catch a genuinely stuck guest. Being slow
# to report a real hang costs one wait; being too tight turns a loaded
# machine into a flaky failure that looks exactly like one.
BOOT_TIMEOUT = 120.0

# The firmware's boot prompt is the first thing on the serial console, so
# the allowance is only against a machine under load.
BOOT_PROMPT_TIMEOUT = 60.0

# The rest of a line whose start has already arrived.
LINE_TIMEOUT = 10.0

# How long a foreign SL gets to reach a code that ends its run. It reports
# only on port 0x80, so nothing on the console marks the end.
POST_CODE_TIMEOUT = 60.0

# A build this long means something is wedged, not slow: a from-scratch
# debug build of this crate is seconds.
BUILD_TIMEOUT = 300.0


_built_binary: Path | None = None
_foreign_binaries: dict[Path, Path] = {}

# What `query-amd-drtm` answered once each launch-machine boot had printed
# its last record, by log directory. What the platform device saw of the
# guest, which for a checker that only observes must be nothing.
_LAUNCH_STATE: dict[Path, dict] = {}


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


def foreign_efi_binary(sl: Path, name: str) -> Path:
    """Builds `drtm.efi` with `sl` as its foreign SL, in a target directory
    of its own called `name`, so no other build is redone, on first use."""
    if sl not in _foreign_binaries:
        target = REPO_ROOT / "target" / name
        subprocess.run(
            ["cargo", "build"],
            cwd=REPO_ROOT,
            check=True,
            timeout=BUILD_TIMEOUT,
            env={
                **os.environ,
                "DRTM_FOREIGN_SL": str(sl),
                "CARGO_TARGET_DIR": str(target),
            },
        )
        _foreign_binaries[sl] = target / "x86_64-unknown-uefi" / "debug" / "drtm.efi"
    return _foreign_binaries[sl]


def _wait_for_post_codes(
    vm: QemuVm, log: Path, done: Callable[[list[int]], bool]
) -> None:
    """Waits until the codes in `log` satisfy `done`, QEMU exits, or
    `POST_CODE_TIMEOUT` passes. Tests judge what arrived."""
    deadline = time.monotonic() + POST_CODE_TIMEOUT
    while time.monotonic() < deadline and vm.running:
        if done(post_codes(log)):
            return
        time.sleep(0.2)


def boot_and_capture(
    log_dir: Path,
    firmware: Path,
    tpm: str | None = None,
    config: str | None = FULL_OUTPUT,
    corrected_error_bank: int | None = None,
    save_firmware_to: Path | None = None,
    query_launch: bool = False,
    launch: str | None = None,
    pcr_banks: tuple[str, ...] | None = None,
    sl_patch: str | None = None,
    dlme_patch: bool = False,
    build: Callable[[], Path] | None = None,
    post_codes_done: Callable[[list[int]], bool] | None = None,
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

    `query_launch` asks the platform device what it saw once the guest is
    done, for `launch_state` to read.

    `launch` leaves the launch on and reads on past the summary to the
    line it names: `LAUNCH_HALTED`, the last line of the DLME or of an SL
    that cannot hand off, or `LAUNCH_GATE` for a launch the gate blocks. `pcr_banks` gives the TPM
    other banks than the session's. `sl_patch` names a field of the signed
    PSP SL that `machine.patch_sl` changes in the ESP's copy of the binary,
    and `dlme_patch` has `machine.patch_dlme` change the DLME there.
    `build` builds and names another binary to boot.

    `post_codes_done` attaches `isa-post-code`, logging to `post-codes.txt`
    in `log_dir`, and waits before `launch` until the codes logged so far
    satisfy it. Where they never do, `launch` is not read.
    """
    marker = SUMMARY_RECORD if config and "format=json" in config else CHECKS_COMPLETE
    with tempfile.TemporaryDirectory(prefix="drtm-esp-") as scratch:
        binary = built_efi_binary() if build is None else build()
        if sl_patch is not None:
            patched = Path(scratch) / "patched.efi"
            patched.write_bytes(
                patch_sl(binary.read_bytes(), SL_PSP.read_bytes(), sl_patch)
            )
            binary = patched
        if dlme_patch:
            patched = Path(scratch) / "patched-dlme.efi"
            patched.write_bytes(patch_dlme(binary.read_bytes(), DLME.read_bytes()))
            binary = patched
        esp = make_esp(Path(scratch), binary, config, launch is not None)
        settings = Settings.from_environment()
        if pcr_banks is not None:
            settings = dataclasses.replace(settings, banks=pcr_banks)
        codes = log_dir / "post-codes.txt" if post_codes_done else None
        with QemuVm(
            log_dir,
            options(
                esp,
                corrected_error_bank=corrected_error_bank,
                post_codes=codes,
                **machine,
            ),
            firmware=firmware,
            tpm=tpm,
            save_firmware_to=save_firmware_to,
            settings=settings,
        ) as vm:
            if corrected_error_bank is not None:
                inject_corrected_error(vm, corrected_error_bank)
            vm.expect(BOOT_PROMPT, BOOT_PROMPT_TIMEOUT)
            vm.send(b"\r")
            vm.expect(marker, BOOT_TIMEOUT)
            # The marker is the start of the last line, and the rest of it
            # is still in flight. Read to its end, or a loaded host leaves
            # the summary record cut short and unparsable.
            vm.expect("\n", LINE_TIMEOUT)
            # Codes first: a launch that never reaches the code that prints
            # `launch` is still judged by what it logged.
            reached = True
            if codes is not None and post_codes_done is not None:
                _wait_for_post_codes(vm, codes, post_codes_done)
                reached = post_codes_done(post_codes(codes))
            if launch is not None and reached:
                stops = [stop for stop in LAUNCH_STOPPED if stop != launch]
                vm.expect(launch, BOOT_TIMEOUT, failures=stops)
                vm.expect("\n", LINE_TIMEOUT)
            # A guest that reset has taken QEMU down with it.
            if query_launch and vm.running:
                _LAUNCH_STATE[log_dir] = vm.wait_for_qmp().execute("query-amd-drtm")
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
    1Ah, which is the only way the dump's AMD MSR list runs here, with
    `VM_CR` locked the way that family's firmware leaves it, and carries a
    Secure Processor whose DRTM service is not ready, which is where that
    mailbox is read in the form naming its address. That Secure Processor
    also publishes the DRTM ACPI device, the only boot that does.
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
    `drtm.conf` naming a level that does not exist and a launch this build
    cannot make, a Secure Processor device with the Turin ASP's device ID
    whose DRTM service is ready, and the one MSR value naming the window
    the firmware actually mapped.
    """
    return {
        "cpu": MMCONFIG_AS_MAPPED,
        "iommu": "enabled",
        "config": BAD_CONFIG,
        "psp": 0x156E,
    }


@boot_configuration
def amd_drtm_boot() -> dict:
    """Serial output of one boot on our QEMU's `-machine q35,amd-drtm=on`,
    the machine a launch boots on.

    The only configuration with the IOMMU at `0:0.2`, where the Secure
    Kernel Loader looks for it, with AMD's northbridge at `0:18.0`, with a
    second processor and with the platform device running strict. The plain
    `EPYC` model, since the machine adds `skinit` itself, and a TIS TPM,
    the interface a launch measures through.
    """
    return {"amd_drtm": True, "tpm": "tis", "query_launch": True}


@boot_configuration
def unlisted_psp_boot() -> dict:
    """Serial output of one boot with a Secure Processor answering with a
    device ID no loader's table carries.

    What a board newer than the loaders looks like. The only configuration
    reaching the branch that finds a Secure Processor by its PCI class
    rather than by its ID. It also starts with the `VM_CR` bits a launch
    sets still in place, the one boot where that check warns.
    """
    return {"psp": 0x17D8, "cpu": VM_CR_AFTER_LAUNCH}


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


@boot_configuration
def launch_boot() -> dict:
    """Serial output of one boot that launches: the launch machine with a
    TIS TPM, as `amd_drtm_boot`, but with the launch left on, so the run
    goes on past the checks through `SKINIT` to the Secure Loader's report.
    """
    return {
        "amd_drtm": True,
        "tpm": "tis",
        "launch": LAUNCH_HALTED,
        "query_launch": True,
    }


@boot_configuration
def launch_psp_boot() -> dict:
    """Serial output of one classic launch, asked for with `launch=skinit`,
    with the PSP DRTM service ready. The service keeps TPM localities 1 to 4
    locked, as Turin firmware does until a PSP launch the classic path never
    issues, so the SL can only read at 0.
    """
    return {
        **_psp_machine(),
        "config": FULL_OUTPUT + "launch=skinit\n",
        "launch": LAUNCH_HALTED,
    }


def _psp_machine(**properties) -> dict:
    """The launch machine with a TIS TPM and the PSP DRTM service ready,
    `properties` set on its `amd-psp` by name."""
    return {
        "amd_drtm": True,
        "tpm": "tis",
        "psp": 0x156E,
        "psp_drtm_service": True,
        "psp_properties": properties,
    }


@boot_configuration
def psp_launch_boot() -> dict:
    """Serial output of one launch on the PSP path, which `auto` takes when
    the service is ready: `SKINIT`, then the SL's `LAUNCH` and the commands
    after it, ending with the TMR's release."""
    return {**_psp_machine(), "launch": LAUNCH_HALTED, "query_launch": True}


@boot_configuration
def psp_splt_boot() -> dict:
    """Serial output of one PSP launch where the service reports SPLT
    version 1, which the build's PCR 17 prediction does not assume."""
    return {**_psp_machine(**{"splt-version": 1}), "launch": LAUNCH_HALTED}


@boot_configuration
def psp_not_ready_boot() -> dict:
    """Serial output of one boot whose service never sets the ready bit, so
    `auto` falls back to the classic launch."""
    return {**_psp_machine(), "psp_ready": False, "launch": LAUNCH_HALTED}


@boot_configuration
def psp_forced_no_service_boot() -> dict:
    """Serial output of one boot asking for `launch=psp` on the launch
    machine with no Secure Processor, which the gate refuses."""
    return {
        "amd_drtm": True,
        "tpm": "tis",
        "config": FULL_OUTPUT + "launch=psp\n",
        "launch": LAUNCH_GATE,
    }


@boot_configuration
def psp_forced_tpm_1_2_boot() -> dict:
    """Serial output of one boot asking for `launch=psp` with a TPM 1.2,
    which the service cannot extend, so the gate refuses."""
    return {
        **_psp_machine(),
        "tpm": "tis-1.2",
        "config": FULL_OUTPUT + "launch=psp\n",
        "launch": LAUNCH_GATE,
    }


@boot_configuration
def psp_timeout_boot() -> dict:
    """Serial output of one PSP launch whose `LAUNCH` answers after 6 s, past
    the SL's 5 s, while every other command answers at once."""
    return {
        **_psp_machine(**{"command-delay": 6000, "delayed-command": 4}),
        "launch": LAUNCH_HALTED,
    }


@boot_configuration
def psp_extend_timeout_boot() -> dict:
    """Serial output of one PSP launch whose `EXTEND_OSSL_DIGEST` answers
    after 11 s, past the SL's 10 s, and still measures the DLME."""
    return {
        **_psp_machine(**{"command-delay": 11000, "delayed-command": 0xB}),
        "launch": LAUNCH_HALTED,
    }


@boot_configuration
def launch_bad_dlme_boot() -> dict:
    """Serial output of one classic launch whose DLME differs from the
    build's by a byte, so the SL does not jump to it."""
    return {
        "amd_drtm": True,
        "tpm": "tis",
        "launch": LAUNCH_HALTED,
        "dlme_patch": True,
    }


@boot_configuration
def psp_bad_dlme_boot() -> dict:
    """Serial output of one PSP launch whose DLME differs from the
    build's by a byte."""
    return {**_psp_machine(), "launch": LAUNCH_HALTED, "dlme_patch": True}


@boot_configuration
def psp_tmr_timeout_boot() -> dict:
    """Serial output of one PSP launch whose `TMR_SETUP` answers after 6 s,
    past the preamble's 5 s, which still launches. The late command sets
    the TMR before the SL's `LAUNCH` gets the mailbox, so `LAUNCH` passes."""
    return {
        **_psp_machine(**{"command-delay": 6000, "delayed-command": 2}),
        "launch": LAUNCH_HALTED,
    }


def _psp_rejected(**properties) -> dict:
    """A PSP launch the service's `LAUNCH` refuses, which breaks a platform
    rule on purpose, so the platform runs lenient."""
    return {**_psp_machine(**properties), "strict": False, "launch": LAUNCH_HALTED}


@boot_configuration
def psp_bad_signature_boot() -> dict:
    """Serial output of one PSP launch with a signature byte flipped."""
    return {**_psp_rejected(), "sl_patch": "signature", "query_launch": True}


@boot_configuration
def psp_bad_modulus_boot() -> dict:
    """Serial output of one PSP launch with a byte of the key token's
    modulus flipped."""
    return {**_psp_rejected(), "sl_patch": "modulus"}


@boot_configuration
def psp_bad_size_boot() -> dict:
    """Serial output of one PSP launch whose `SizeFWSigned` runs past the
    SLB."""
    return {**_psp_rejected(), "sl_patch": "size_fw_signed"}


@boot_configuration
def psp_bad_length_boot() -> dict:
    """Serial output of one PSP launch whose header gives a measured length
    16 bytes short, so the signature header is not at the padded length."""
    return {**_psp_rejected(), "sl_patch": "length"}


@boot_configuration
def psp_min_spl_boot() -> dict:
    """Serial output of one PSP launch on a platform whose SecPatchLevel is
    1, above the image's 0."""
    return _psp_rejected(**{"min-spl": 1})


@boot_configuration
def psp_no_ivrs_boot() -> dict:
    """Serial output of one PSP launch on a machine without an IVRS, so the
    service has no table to hand out."""
    return {**_psp_machine(), "ivrs": False, "launch": LAUNCH_HALTED}


@boot_configuration
def launch_defaults_boot() -> dict:
    """Serial output of one launch with no `drtm.conf`, so it runs on the
    defaults, prose at `info`, and launches because that is the default.
    Its TPM also has a SHA-384 bank, which the build computes no value for.
    """
    return {
        "amd_drtm": True,
        "tpm": "tis",
        "config": None,
        "launch": LAUNCH_HALTED,
        "pcr_banks": ("sha1", "sha256", "sha384"),
    }


@boot_configuration
def launch_crb_boot() -> dict:
    """Serial output of one launch through a CRB TPM, which our QEMU gives
    locality 0 alone and no locality 4 path to hash through.

    So `tpm.locality` fails and the launch goes ahead regardless, as a FAIL
    outside the gate's list should. The platform runs lenient, since the
    missing hash breaks one of its rules on purpose.
    """
    return {
        "amd_drtm": True,
        "strict": False,
        "tpm": "crb",
        "launch": LAUNCH_HALTED,
        "query_launch": True,
    }


@boot_configuration
def launch_tpm_1_2_boot() -> dict:
    """Serial output of one launch through a TPM 1.2 behind TIS, whose one
    SHA-1 bank the SL reads a PCR at a time with `TPM_PcrRead`."""
    return {
        "amd_drtm": True,
        "tpm": "tis-1.2",
        "launch": LAUNCH_HALTED,
        "query_launch": True,
    }


@boot_configuration
def launch_burst_boot() -> dict:
    """Serial output of one launch through a TIS FIFO that moves 8 bytes a
    burst and reads a burst count of 0 between bursts, so every command and
    response before and after SKINIT spans several bursts."""
    return {
        "amd_drtm": True,
        "tpm": "tis",
        "tpm_burst_limit": 8,
        "launch": LAUNCH_HALTED,
        "query_launch": True,
    }


@boot_configuration
def launch_tpm_1_2_burst_boot() -> dict:
    """Serial output of one launch through a TPM 1.2 whose FIFO moves 3
    bytes a burst, so bursts end inside the header's fields."""
    return {
        "amd_drtm": True,
        "tpm": "tis-1.2",
        "tpm_burst_limit": 3,
        "launch": LAUNCH_HALTED,
        "query_launch": True,
    }


@boot_configuration
def launch_blocked_boot() -> dict:
    """Serial output of one boot on the launch machine with no TPM, so
    `tpm.present` fails and the gate refuses to launch."""
    return {"amd_drtm": True, "launch": LAUNCH_GATE, "query_launch": True}


DLME_REACHED = 0xD7D70001
"""What drtm's DLME writes first once AMD's SL has jumped to it. A fatal
error in AMD's SL resets the machine instead, which ends the run."""


def _amd_sl_done(codes: list[int]) -> bool:
    return DLME_REACHED in codes


@boot_configuration
def foreign_amd_sl_boot() -> dict:
    """Serial output of one `launch=foreign` boot of the SL
    `DRTM_TEST_FOREIGN_SL` names, with the PSP DRTM service ready and the
    POST codes logged, until the DLME it hands off to halts. `client-smm`
    answers the two SMM commands AMD's client SL sends, which server
    firmware refuses."""
    sl = FOREIGN_SL
    assert sl is not None
    return {
        **_psp_machine(**{"client-smm": "on"}),
        "cpu": "EPYC-Genoa,mmio-conf-base=0xb0000021",
        "amd_fch_pmio": True,
        "strict": False,
        "config": FULL_OUTPUT + "launch=foreign\n",
        "launch": LAUNCH_HALTED,
        "build": lambda: foreign_efi_binary(Path(sl), "foreign-test"),
        "post_codes_done": _amd_sl_done,
        "query_launch": True,
    }


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


def launch_state(name: str) -> dict:
    """What the platform device recorded of boot `name`, from
    `query-amd-drtm`: whether a launch happened, whether SL_DEV is armed and
    which ranges are kept from DMA."""
    state = _LAUNCH_STATE.get(SESSION.boot_log_dir(name))
    assert state is not None, f"QEMU exited before {name} could be queried"
    return state


def _is_trace(line: str) -> bool:
    """Whether a `qemu.log` line is a trace, which starts with its event's
    name, rather than a guest error."""
    event = line.split(" ", 1)[0]
    return any(fnmatch.fnmatchcase(event, pattern) for pattern in TRACES)


def _qemu_lines(name: str) -> list[str]:
    log = SESSION.boot_log_dir(name) / "qemu.log"
    return log.read_text(errors="replace").splitlines(keepends=True)


def qemu_log(name: str) -> str:
    """QEMU's own log of boot `name` without its traces: the guest errors
    and anything else QEMU complained of."""
    return "".join(line for line in _qemu_lines(name) if not _is_trace(line))


def qemu_traces(name: str) -> list[str]:
    """The trace lines of boot `name`, in order."""
    return [line.rstrip("\n") for line in _qemu_lines(name) if _is_trace(line)]


def completed(capture: str) -> bool:
    """Whether the run reached its end, in whichever form it printed."""
    return SUMMARY_RECORD in capture or CHECKS_COMPLETE in capture


def verdicts(capture: str, identifier: str) -> list[str]:
    """Just the tags from `results`, for the common case of not caring what
    the detail said.
    """
    return [tag for tag, _ in results(capture, identifier)]
