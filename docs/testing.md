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
QEMU configuration and yields that boot's complete serial output as a string,
which tests then assert against. A configuration needing different QEMU
arguments gets its own fixture calling `boot_and_capture`, rather than the
existing ones growing parameters.

| Fixture        | Configuration                                      |
|----------------|----------------------------------------------------|
| `default_boot` | No TPM device                                      |
| `tpm_tis_boot` | `swtpm`-backed TPM 2.0 on the FIFO (TIS) interface |
| `tpm_crb_boot` | `swtpm`-backed TPM 2.0 on the CRB interface        |

All three use a CPU model reporting neither AMD nor `SKINIT`, since QEMU
will not set that feature bit (see above).

The two TPM configurations exist because the check reports a different
interface for each, and getting that wrong is not hypothetical: reading
`TPM_INTF_CAPABILITY` before establishing the active interface made a CRB
TPM report as `V1_2 Tis`, since offset `0x14` holds `TPM_DATA_CSUM` under
the CRB layout. Reverting that fix turns `test_tpm_check_detects_a_crb_tpm`
red, which is the point of testing both.

The trade-off is that tests sharing a boot fail together if that boot does,
which is the honest result: they were never independent, they were repeated
boots of identical arguments asserting on identical output.

Each boot reads serial output until `drtm: checks complete`, the last line
`src/main.rs` prints. Reaching that marker is what proves the binary got past
every check instead of hanging or faulting midway, so the marker must stay in
step with the code that emits it.

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

`DRTM_QEMU_ACCEL` picks the accelerator. `tcg` is the default, `kvm` forces
KVM, and `auto` uses KVM whenever `/dev/kvm` is usable.

TCG is the default because KVM measured slower on every machine tried. On a
nested-virtualisation development host it is roughly 2x slower:

| Mode | Boot  | Full suite |
|------|-------|------------|
| TCG  | 11.0s | 11.4s      |
| KVM  | 22.7s | 23.0s      |

On bare metal the gap nearly closes, but KVM is still marginally behind (a
little over half a second on the suite). The guest boots with `smm=on` and
Dasharo's firmware does substantial SMM setup, so nested, every SMM
transition traps out through both hypervisor layers while TCG emulates it in
userspace with no exits at all.

This is a measurement, not a law: `kvm` and `auto` exist so the comparison
can be re-run somewhere it may not hold. Each boot records its
`qemu-system-x86_64` invocation to `qemu-args.txt` in its log folder, so two
runs compared against each other show what actually differed.

## Timeouts

A stuck `drtm.efi` fails the run rather than wedging it. Layers, innermost
first:

- `read_serial_until` gives up after its timeout and raises with the serial
    output captured so far. Boot waits pass `BOOT_TIMEOUT` (120s), which is
    deliberately far above the ~23s a real boot takes: reporting a genuine
    hang late costs one wait, while a tight limit turns a loaded machine into
    a flaky failure that looks identical to a hang.
- `QemuVm.__exit__` always tears QEMU down (`terminate`, then `kill` after
    5s), so a timed-out test leaves no stray process.
- `_connect_serial` waits at most 5s for QEMU to open its serial port,
    `QmpClient` bounds every socket operation at 10s, and the `cargo build`
    in the `efi_binary` fixture is capped at `BUILD_TIMEOUT` (300s).
- `pytest-timeout` backstops anything the above misses, at 300s per test
    with the `signal` method. `signal` matters: it raises inside the test, so
    pytest still reports the failure and still runs the hook that writes
    `results.txt`. The `thread` method calls `os._exit()` and loses both.

## Logs

Each pytest invocation gets its own numbered, gitignored folder under
`qemu-tests/logs/`, e.g. `0007-2026-09-13/`, holding:

- `timestamp.txt`: the exact run time.
- `results.txt`: every test's outcome and duration, the session's exit
    status, and the full traceback for anything that failed. A log folder
    therefore records how the run went, not only what the guest printed.
- One subfolder per boot, e.g. `boot-default/`, with that boot's captured
    `serial.log` and `qemu.log`, plus the `qemu-args.txt` it was launched
    with. Boots with a TPM also get `swtpm.log`.

`swtpm.log` is empty after a healthy run, which is expected rather than a
sign of a broken capture. swtpm writes its errors straight to stderr, which
that file captures, but emits nothing otherwise. Debug tracing is separate
and off by default: set `DRTM_SWTPM_LOG_LEVEL` to turn it on. The only
threshold swtpm documents is that level 5 and above activates libtpms
debug logging, and level 20 dumps every TPM command and response, around
38K per boot.

Durations in `results.txt` sum a test's setup, call and teardown phases, so a
shared boot's cost shows against the first test that triggered it rather than
every test reporting 0.0s.

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

QEMU does not support real DRTM launches. Real DRTM-path code will either
need to be stubbed out for QEMU builds (a no-DRTM test build variant) or is
expected to fault/crash under QEMU. Which of these applies depends on how
the DRTM instructions actually behave when executed in QEMU, and should be
verified rather than assumed. Dasharo's own TrenchBoot QEMU tests suggest the
failure is graceful and happens before the DRTM instruction executes (GRUB
reports "secure launch not enabled" rather than the VM crashing), but this
should be confirmed for our own launch path rather than assumed.

Dasharo's `trenchboot/*.robot` tests are a useful model for DRTM-aware test
structure later. They run against a fully booted Linux and read PCRs from
`/sys/class/tpm/tpm0/pcr-*` and the event log from
`/sys/kernel/security/slaunch/eventlog`, though not applicable verbatim
since we don't boot a full OS.

A separate future project may add simple DRTM emulation to QEMU itself.
That is out of scope here.
