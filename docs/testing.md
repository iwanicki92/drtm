<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: Apache-2.0
-->

# Testing

A QEMU-based test suite exists from the start, even before real DRTM launches
are supported, to prove the UEFI application itself boots and runs correctly.

## Running

`cargo` must be on `PATH` (the test fixture builds the binary itself). Test
dependencies are managed by [uv](https://docs.astral.sh/uv/) via
`qemu-tests/pyproject.toml`, no separate venv setup needed:

```sh
uv run --project qemu-tests pytest qemu-tests
```

`uv` creates and reuses `qemu-tests/.venv` on its own.

The first run downloads Dasharo's QEMU Q35 firmware image into
`qemu-tests/dl-cache/` (a few MB, gitignored, cached after that).

## How the VM is set up

Booting takes around 11 seconds, so tests do not boot QEMU themselves. A
session-scoped fixture in `qemu-tests/conftest.py` boots once per distinct
QEMU configuration and yields that boot's serial output as a string, which
tests assert against. A configuration needing different QEMU arguments gets
its own fixture calling `boot_and_capture`, rather than the existing ones
growing parameters. Tests sharing a boot fail together if it does.

| Fixture        | Configuration                                      |
|----------------|----------------------------------------------------|
| `default_boot` | No TPM device, Intel CPU model (`Skylake-Client`)  |
| `amd_cpu_boot` | No TPM device, AMD CPU model (`EPYC`)              |
| `tpm_tis_boot` | `swtpm`-backed TPM 2.0 on the FIFO (TIS) interface |
| `tpm_crb_boot` | `swtpm`-backed TPM 2.0 on the CRB interface        |

No configuration reports `SKINIT` support, since QEMU will not set that
feature bit on any CPU model (see [DRTM under QEMU](#drtm-under-qemu)). The
CPU model still decides the vendor string, so `default_boot` and
`amd_cpu_boot` cover both reasons the check can fail: a CPU that is not AMD
at all, and an AMD CPU without the feature. Both TPM interfaces are tested
because the check reports them differently, and an earlier bug got CRB
wrong while passing on TIS.

Each boot reads serial output until `drtm: checks complete`, the last line
`src/main.rs` prints, which proves the binary got past every check rather
than hanging midway. The UEFI console hard-wraps at 80 columns mid-word, so
`boot_and_capture` rejoins split lines before tests see them (`serial.log`
keeps the raw capture), and the marker itself has to stay under that width.

For each boot, `qemu-tests/qemu_vm.py` builds a scratch working directory
(under the system temp dir, removed on exit) containing:

- `esp/EFI/BOOT/BOOTX64.EFI`: a copy of the built `drtm.efi`, placed where
    UEFI's removable-media boot path picks it up automatically, so it needs
    no boot entry or bootloader.
- `firmware.rom`: a copy of the cached Dasharo firmware image, with its
    variable store (the first 256 bytes) zeroed so settings like boot order
    reset to defaults on every run.

It then launches `qemu-system-x86_64` on a Q35 machine, with that ESP
attached as a FAT-formatted drive and that firmware image as pflash. Serial
is exposed on a raw TCP port (see below) and QMP on a short-lived Unix
socket, both used by the test to interact with the VM.

## Acceleration

`DRTM_QEMU_ACCEL` picks the accelerator: `tcg` (the default), `kvm`, or
`auto` (KVM whenever `/dev/kvm` is usable). TCG is the default because KVM
measured slower on every machine tried, roughly 2x on a
nested-virtualisation host:

| Mode | Boot  | Full suite |
|------|-------|------------|
| TCG  | 11.0s | 11.4s      |
| KVM  | 22.7s | 23.0s      |

On bare metal KVM is still marginally behind, by a little over half a second
on the suite. The guest boots with `smm=on` and Dasharo's firmware does
substantial SMM setup, so nested, every SMM transition traps through both
hypervisor layers while TCG has no exits at all.

This is a measurement, not a law: `kvm` and `auto` exist so it can be re-run
somewhere it may not hold. Each boot records its invocation to
`qemu-args.txt`, so two runs compared show what actually differed.

## Timeouts

A stuck `drtm.efi` fails the run rather than wedging it. Layers, innermost
first:

- `read_serial_until` raises with the output captured so far. Boot waits use
    `BOOT_TIMEOUT` (120s), far above the ~23s a real boot takes, so a loaded
    machine does not fail looking like a hang.
- `QemuVm.__exit__` always tears QEMU down (`terminate`, then `kill` after
    5s), so a timed-out test leaves no stray process.
- `_connect_serial` waits at most 5s for QEMU to open its serial port,
    `QmpClient` bounds every socket operation at 10s, and the `cargo build`
    in the `efi_binary` fixture is capped at `BUILD_TIMEOUT` (300s).
- `pytest-timeout` backstops the rest at 300s per test, with the `signal`
    method so the failure is still reported and `results.txt` still written.
    The `thread` method calls `os._exit()` and loses both.

## Logs

Each pytest invocation gets its own numbered, gitignored folder under
`qemu-tests/logs/`, e.g. `0007-2026-09-13/`, holding:

- `timestamp.txt`: the exact run time.
- `results.txt`: every test's outcome and duration, the session's exit
    status, and the full traceback for anything that failed. Durations sum
    the setup, call and teardown phases, so a shared boot's cost shows
    against the test that triggered it rather than every test reading 0.0s.
- One subfolder per boot, e.g. `boot-default/`, with that boot's captured
    `serial.log` and `qemu.log`, plus the `qemu-args.txt` it was launched
    with. Boots with a TPM also get `swtpm.log`.

`swtpm.log` is empty after a healthy run: swtpm writes errors to stderr,
which that file captures, and emits nothing otherwise. Debug tracing is
separate, via `DRTM_SWTPM_LOG_LEVEL`, where level 5 and above activates
libtpms logging and 20 dumps every command (~38K per boot).

Old runs are never pruned automatically, so clean out `qemu-tests/logs/` by
hand once in a while. The run number is 4 digits; if it's exhausted, delete
old runs and try again.

## Why Dasharo's firmware, not OVMF

Firmware for QEMU testing is Dasharo's coreboot+UEFI build for QEMU Q35 (from
the `Dasharo/coreboot` releases, `qemu_q35_all_menus.rom`), not stock OVMF.
It's a more feature-complete, open-source UEFI implementation, already used
by Dasharo to test TrenchBoot itself, which leaves room to grow into
TPM/DRTM-adjacent QEMU testing later without switching firmware.

## Why pytest, not Robot Framework

RF's test-case syntax is `.robot` files, which is more ceremony than this
project wants. QEMU is driven directly from Python over its serial console,
exposed as a raw TCP socket (`-serial tcp:...`, not `telnet:...`, to avoid
depending on Python's removed/deprecated `telnetlib`, which RF's own
`Telnet` library needs), plus its QMP socket for VM control (reset, quit).
See `qemu-tests/qemu_vm.py`.

`qemu-tests/qmp_client.py` is adapted from
Dasharo/open-source-firmware-validation's `lib/QemuMonitor.py` (Apache-2.0,
see its SPDX header and this repo's `NOTICE` file), with the
Robot-Framework-specific bits stripped.

## DRTM under QEMU

QEMU does not support real DRTM launches, and this is deliberate rather than
a gap that will close. `+skinit` is refused on every CPU model (TCG and KVM
both report the feature unsupported), and QEMU's TCG decoder raises `#UD`
for `SKINIT` unconditionally, under the comment "If not intercepted, not
implemented" (QEMU 8.2.2, `target/i386/tcg/translate.c`).

So code reaching the instruction takes an undefined opcode exception, not a
crash, and no separate no-DRTM build variant is needed to avoid one. Nothing
past that point is testable here either, so a launch path has to keep the
surrounding work (SLB construction, memory layout, header validation)
exercisable without executing `SKINIT`. Dasharo's own TrenchBoot QEMU tests
never get that far: GRUB stops at its own precondition checks.

Dasharo's `trenchboot/*.robot` tests are a useful model for DRTM-aware test
structure later. They run against a fully booted Linux and read PCRs from
`/sys/class/tpm/tpm0/pcr-*` and the event log from
`/sys/kernel/security/slaunch/eventlog`, though not applicable verbatim
since we don't boot a full OS.

A separate future project may add simple DRTM emulation to QEMU itself.
That is out of scope here.
