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

### The QEMU build

The suite needs our own QEMU, not a distribution one. Five checks reach
outcomes upstream cannot produce: `amd.skinit` passing needs a CPU allowed
to report the feature bit, `amd.iommu` passing needs an `amd-iommu` whose
capability reads enabled, `amd.mmconfig` needs `MSR 0xC0010058` to hold a
value at all, `amd.psp` needs a device with a Secure Processor ID, and
`amd.iommu_table` warning needs an IOMMU with no IVRS table.
What those patches do, and what is still missing, is in
[qemu-drtm-support.md](qemu-drtm-support.md).

```sh
git clone <fork> qemu
cd qemu && git checkout drtm
mkdir build && cd build
../configure --target-list=x86_64-softmmu --enable-tpm --disable-docs
make -j"$(nproc)"
```

`DRTM_QEMU_BINARY` then points the suite at it, by path or by name:

```sh
DRTM_QEMU_BINARY=../qemu/build/qemu-system-x86_64 \
    uv run --project qemu-tests pytest qemu-tests
```

Unset, it falls back to `qemu-system-x86_64` on `PATH`. Before any test,
the suite starts that binary paused with the options only our build
accepts, and stops with the reason if it turns them down, so a
distribution QEMU fails once rather than in every boot fixture.

## How the VM is set up

Booting takes around 11 seconds, so tests do not boot QEMU themselves. A
session-scoped fixture in `qemu-tests/conftest.py` boots once per distinct
QEMU configuration and yields that boot's serial output as a string, which
tests assert against. A configuration needing different QEMU arguments gets
its own fixture calling `boot_and_capture`, rather than the existing ones
growing parameters. Tests sharing a boot fail together if it does.

| Fixture                    | Configuration                                      | `drtm.conf`                  |
|----------------------------|----------------------------------------------------|------------------------------|
| `default_boot`             | No TPM device, `EPYC,+skinit`, window below 4 GiB  | `level=trace`, `format=json` |
| `intel_cpu_boot`           | No TPM device, Intel CPU model (`Skylake-Client`)  | `level=trace`, `format=json` |
| `tpm_tis_boot`             | `swtpm`-backed TPM 2.0 on the FIFO (TIS) interface | `level=trace`, `format=json` |
| `tpm_crb_boot`             | `swtpm`-backed TPM 2.0 on the CRB interface        | none                         |
| `amd_iommu_boot`           | `amd-iommu` disabled, `EPYC`, second root bridge   | `level=trace`, `format=json` |
| `amd_iommu_enabled_boot`   | `amd-iommu` enabled by firmware, `amd-psp`         | `level=verbose`              |
| `mmconfig_above_4gib_boot` | MMIO configuration window crossing 4 GiB           | `level=trace`, `format=json` |
| `machine_check_error_boot` | Machine-check error from reset, IOMMU not in ACPI  | `level=trace`, `format=json` |

`level=verbose` is not a level, and that boot carries it on purpose.

The CPU model decides the vendor string, and that selects which checks run
at all: `default_boot` is AMD so every check runs, while `intel_cpu_boot`
covers the path where the AMD checks are skipped and the vendor-independent
ones still run. Both TPM interfaces are tested because the check reports
them differently, and an earlier bug got CRB wrong while passing on TIS.

Several boots exist only to reach a second outcome of one check.
`default_boot` asks for `+skinit` and `amd_iommu_boot` does not, so the
`SKINIT` check is exercised both ways at no cost, and `amd_iommu_boot` also
leaves the configuration window disabled, which is where that check's third
outcome comes from. It also carries a second root bridge, as a server has,
so the scan is held to reading each function once. The enabled IOMMU boot
differs in the capability and adds a Secure Processor, and
`mmconfig_above_4gib_boot` differs only in where the window sits.
`machine_check_error_boot` starts QEMU paused and logs a corrected error in
bank 1 through its `mce` monitor command before the firmware runs. It also
carries an enabled IOMMU that ACPI leaves out, since its tests only read
results.

The binary reads `drtm.conf` from beside itself on the ESP, so `QemuVm`
writes one into the scratch ESP. Most boots raise the level to `trace`,
because the `[SCAN]` dump the PCI tests assert on sits below the default
`info`, and ask for `format=json` so the assertions key on record fields
rather than wording. Two boots only read results, which print at any level,
so they carry the other config states: `tpm_crb_boot` has no file at all and
`amd_iommu_enabled_boot` an unusable level, which must be refused without
stopping the run. `test_config.py` asserts on those two, and having no file
also leaves `tpm_crb_boot` in the default text form, which is what keeps the
human-readable output under test. `tpm_tis_boot` stays at `trace` for its
per-locality `TPM_ACCESS` line.

`conftest.py` reads results through `results` and `verdicts`, which take
either form, so a fixture can change form without its tests changing.
`records` parses a JSON-mode capture and refuses a `drtm:` line that is not a
usable record, since firmware shares this UART but never writes that prefix.

Each boot reads serial output until the marker its form ends with: the
`summary` record under `format=json`, and the `drtm: checks complete` line
in text mode. Reaching it proves the binary got past every check rather than hanging
midway. The binary writes to the serial device itself rather than through the
UEFI text console, which hard-wraps at 80 columns mid-word, so a line arrives
whole however long it is.

## What tests assert on

Every check reports as `[TAG] <identifier>: <detail>`, where the identifier
is a stable dotted name with the area first (`cpu.bsp`, `amd.iommu`) and the
tag is one of five. `PASS`, `FAIL` and `WARN` are verdicts, `SKIP` means the
check did not run, and `INFO` is for a check that reports a platform fact
without gating anything. `src/check.rs` holds the macros that produce them.

Tests key on the tag and the identifier, through `results` and `verdicts` in
`conftest.py`, so rewording a check's detail text is not a test failure. A
test that needs a value the check computed, such as the SLB address or the
machine-check bank count, scrapes it out of the detail and says so. An
identifier that reports nothing yields an empty list, which is how a skipped
phase or a vendor gate is asserted.

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

QEMU models no part of a real DRTM launch, so a launch path has to keep the
surrounding work (SLB construction, memory layout, header validation)
exercisable without executing `SKINIT`. Which pieces are missing, and what
each check can therefore prove here, is in `qemu-drtm-support.md`. Dasharo's
own TrenchBoot QEMU tests never get further: GRUB stops at its own
precondition checks.

Dasharo's `trenchboot/*.robot` tests are a useful model for DRTM-aware test
structure later. They run against a fully booted Linux and read PCRs from
`/sys/class/tpm/tpm0/pcr-*` and the event log from
`/sys/kernel/security/slaunch/eventlog`, though not applicable verbatim
since we don't boot a full OS.

Adding DRTM emulation to QEMU is out of scope here. `qemu-drtm-support.md`
records what it would take, for whoever picks it up.
