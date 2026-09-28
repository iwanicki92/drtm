<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Testing

A QEMU-based test suite exists from the start, even before real DRTM launches
are supported, to prove the UEFI application itself boots and runs correctly.

## Running

`cargo` must be on `PATH` (the test fixture builds the binary itself). Test
dependencies are managed by [uv](https://docs.astral.sh/uv/) via
`qemu-tests/pyproject.toml`, no separate venv setup needed. The QEMU
harness itself, the `drtmtest` package, lives in the `drtm-tests`
repository and is taken from a checkout beside this one, `../drtm-tests`,
until that repository has a remote to pin instead.

```sh
task test
# or
uv run --project qemu-tests pytest qemu-tests
```

`uv` creates and reuses `qemu-tests/.venv` on its own.

The first run downloads Dasharo's QEMU Q35 firmware image into
`qemu-tests/dl-cache/` (a few MB, gitignored, cached after that).

### The QEMU build

The suite needs our own QEMU, not a distribution one. Nine checks reach
outcomes upstream cannot produce: `amd.skinit` passing needs a CPU allowed
to report the feature bit, `amd.iommu` passing needs an `amd-iommu` whose
capability reads enabled, `amd.mmconfig` and `amd.mmconfig_table` need
`MSR 0xC0010058` to hold a value at all, `amd.psp` needs a device with a
Secure Processor ID, `amd.psp_drtm` needs that device's mailbox registers
and the table naming where they are, `amd.drtm_device` finding anything
needs the SSDT that describes it, `amd.iommu_table` warning needs an IOMMU
with no IVRS table, and `amd.northbridge` finding a function needs the
northbridge the launch machine adds.
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

Booting takes around 5 seconds, so tests do not boot QEMU themselves. Each
configuration in `qemu-tests/conftest.py` carries a `@boot_configuration`
decorator, which turns it into a session fixture of the same name yielding
that boot's serial output as a string. A configuration needing different
QEMU arguments is a new function returning them, rather than the existing
ones growing parameters. Tests sharing a boot fail together if it does.

Collection starts those boots rather than the fixtures doing it: once the
tests to run are known, every configuration among them is submitted to a
pool of `os.cpu_count()` threads, in the order the tests ask for them, and
the fixture then waits on its result. So a session's wall time tracks the
slowest batch rather than the number of configurations, and `-k` or a single
test file still boots only what it names.

| Fixture                    | Configuration                                                                 | `drtm.conf`                  |
|----------------------------|-------------------------------------------------------------------------------|------------------------------|
| `default_boot`             | No TPM device, `EPYC,+skinit`, window below 4 GiB                             | `level=trace`, `format=json` |
| `intel_cpu_boot`           | No TPM device, Intel CPU model (`Skylake-Client`)                             | `level=trace`, `format=json` |
| `tpm_tis_boot`             | `swtpm`-backed TPM 2.0 on the FIFO (TIS) interface                            | `level=trace`, `format=json` |
| `tpm_crb_boot`             | `swtpm`-backed TPM 2.0 on the CRB interface                                   | none                         |
| `amd_iommu_boot`           | `amd-iommu` disabled, `EPYC`, second root bridge                              | `level=trace`, `format=json` |
| `amd_iommu_enabled_boot`   | `amd-iommu` enabled by firmware, `amd-psp`, window as mapped                  | `level=verbose`              |
| `mmconfig_above_4gib_boot` | Window crossing 4 GiB, family 1Ah, `VM_CR` locked, PSP not ready, DRTM device | `level=trace`, `format=json` |
| `machine_check_error_boot` | Machine-check error, IOMMU not in ACPI, PSP no ASPT                           | `level=trace`, `format=json` |
| `amd_drtm_boot`            | `amd-drtm=on` strict, two processors, TIS TPM                                 | `level=trace`, `format=json` |
| `unlisted_psp_boot`        | `amd-psp` with an ID no loader's table lists, `VM_CR` as a launch leaves it   | `level=trace`, `format=json` |

`level=verbose` is not a level, and that boot carries it on purpose.

`default_boot` is also the only one keeping QEMU's display adapter and
network card. Initialising them costs 1.3s, and no test asserts on either,
but it is the boot the scan tests read and they want a populated bus.

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
differs in the capability, adds a Secure Processor whose DRTM service is
ready, and is the one boot whose MSR names the window the firmware mapped,
so the MCFG check agrees there and warns everywhere else.
`mmconfig_above_4gib_boot` moves the window, and reports CPUID family 1Ah,
which is the one family the platform dump reads AMD's own MSRs on: every
other boot is `EPYC`, family 17h, where that list is refused. Its `VM_CR`
starts locked with SVM still enabled, as that family's firmware leaves it,
which is the one place the check reports a lock. It carries a Secure
Processor whose DRTM service is not ready too, which is the form that
prints the mailbox address as a field rather than in a sentence, and it is
the one boot publishing the DRTM ACPI device.
`machine_check_error_boot` starts QEMU paused and logs a corrected error in
bank 1 through its `mce` monitor command before the firmware runs. It also
carries an enabled IOMMU that ACPI leaves out, and a Secure Processor with
no ASPT, since its tests only read results.
`amd_drtm_boot` is the machine a launch runs on, and the one boot held to
more than its serial output. The platform device runs strict, so a broken
launch rule would stop the VM as a panic and fail the boot, QEMU's guest
error log is kept and asserted empty, and `query-amd-drtm` is asked once
the run completes, through `launch_state`, to confirm the checks launched
nothing and armed nothing. It has a second processor and a TIS TPM, so the
processor list and the BSP check are read with an AP present.
`unlisted_psp_boot` carries the one Secure Processor found by class rather
than by ID, and starts with the three `VM_CR` bits `SKINIT` sets still in
place, so it is where the check warns about `R_INIT`.

The binary reads `drtm.conf` from beside itself on the ESP, so `QemuVm`
writes one into the scratch ESP. Most boots raise the level to `trace` and
ask for `format=json`, which is what the platform dump takes and what lets
the assertions key on record fields rather than wording. Two boots only
read results, which print at any level, so they carry the other config
states: `tpm_crb_boot` has no file at all and `amd_iommu_enabled_boot` an
unusable level, which must be refused without stopping the run.
`test_config.py` asserts on those two, and having no file also leaves
`tpm_crb_boot` in the default text form, which is what keeps the
human-readable output under test. `tpm_tis_boot` stays at `trace` for its
per-locality `TPM_ACCESS` line.

`conftest.py` reads results through `results` and `verdicts`, which take
either form, so a fixture can change form without its tests changing.
`fields` reads the values a check named, which is how a test asserting on an
address or a count avoids a regular expression over the detail text.
`records` parses a JSON-mode capture and refuses a `drtm:` line that is not a
usable record, since firmware shares this UART but never writes that prefix.
`dumped` takes one kind of platform dump record, such as `device` or `msr`,
with no prose fallback, since the dump has no prose form.

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

For each boot, `qemu-tests/machine.py` builds a scratch ESP (under the
system temp dir, removed on exit) holding `EFI/BOOT/BOOTX64.EFI`, a copy
of the built `drtm.efi`, placed where UEFI's removable-media boot path
picks it up automatically, so it needs no boot entry or bootloader, and
the boot's `drtm.conf` beside it. The same module turns a configuration's
keyword arguments into the QEMU options for that boot: the machine, the
CPU model and the devices only our build has.

`drtmtest.qemu_vm` runs it: a copy of the warmed Dasharo firmware image
as pflash, variable store included, that ESP as a FAT-formatted drive, an
`swtpm` when the boot asks for a TPM, serial on a raw TCP port (see below)
and QMP on a short-lived Unix socket. Nothing here depends on the settings
the store holds, since the binary is found as removable media rather than
through a boot entry.

The warmed image is the download after one boot, cached beside it under the
download's hash. Populating the variable store is 2.8s and the firmware only
does it on an image that has never booted, so the first boot of a fresh
clone makes it and every boot after starts from a store that is already
there. Deleting `qemu-tests/dl-cache/` costs one boot, not correctness.

The firmware's boot prompt is answered with a carriage return as soon as it
appears, which boots the default entry at once rather than after the boot
manager's 3s timeout. Not seeing that prompt fails the boot: the firmware
release is pinned, so its absence means something changed. QEMU's serial
server therefore waits for its connection before the machine starts, since
that prompt is the first thing printed and nothing may be dropped ahead of
it.

## Acceleration

`DRTM_QEMU_ACCEL` picks the accelerator: `tcg` (the default), `kvm`, or
`auto` (KVM whenever `/dev/kvm` is usable). TCG is the default because KVM
measured slower on every machine tried, roughly 2x on a
nested-virtualisation host, where one boot took 11.0s under TCG against
22.7s under KVM. On bare metal KVM was still marginally behind. The guest
boots with `smm=on` and Dasharo's firmware does substantial SMM setup, so
nested, every SMM transition traps through both hypervisor layers while TCG
has no exits at all.

That was a one-off comparison and is not kept current, since only TCG is
used. Per-boot figures are quoted rather than whole-session ones, which
would go stale every time a boot fixture is added. `kvm` and `auto` remain
so the comparison can be re-run somewhere it may not hold. Each boot records
its invocation to `qemu-args.txt`, so two runs compared show what actually
differed.

## Timeouts

A stuck `drtm.efi` fails the run rather than wedging it. Layers, innermost
first:

- `QemuVm.expect` raises with the output captured since its last match.
    Boot waits use `BOOT_TIMEOUT` (120s), far above the ~5s a real boot
    takes, so a loaded machine does not fail looking like a hang. A console
    silent for `IDLE_TIMEOUT` (90s) or a VM stopped as panicked ends the
    wait earlier, with that reason.
- `QemuVm.__exit__` always tears QEMU down (`terminate`, then `kill` after
    5s), so a timed-out test leaves no stray process.
- The serial connection waits at most 10s for QEMU to open its port,
    `QmpClient` bounds every socket operation at 10s, and the `cargo build`
    in `built_efi_binary` is capped at `BUILD_TIMEOUT` (300s).
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
- One subfolder per boot, named after its configuration, e.g. `default_boot`
    to `boot-default/`, with that boot's captured `serial.log`, the
    process's own output in `qemu-stderr.log`, its `-D` log in `qemu.log`,
    and the `qemu-args.txt` it was launched with. Boots with a TPM also get
    `swtpm.log`, and the run that warms a firmware image gets
    `boot-warm-firmware/`.

On the launch machine `qemu.log` holds the guest errors and the launch
traces, the same `amd_drtm_*`, `amd_psp_*` and `x86_*` set the boot matrix
turns on. `qemu_log()` hands tests the guest errors alone, so a healthy
launch still reads empty, and `qemu_traces()` the trace lines.

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
See `drtmtest.qemu_vm` in the `drtm-tests` repository, which also carries
the QMP client and its Apache-2.0 attribution.

## DRTM under QEMU

Upstream QEMU models no part of a real DRTM launch. Our build does, from
`SKINIT` through the locality 4 hash to the northbridge register the Secure
Kernel Loader releases the block through, and boots TrenchBoot's Xen with
it. That end-to-end test lives in the QEMU tree rather than here, since it
exercises the emulator and not this binary.

Every boot here runs with `launch=off` unless it asks for a launch, which
`make_esp` writes into its `drtm.conf`, so the checks run on a platform that
could launch and is not asked to. The launch boots in `test_launch.py` read on
past the checks' summary to the DLME's `launch.halted`: one through a
TIS TPM that must measure the image in SHA-1 and SHA-256, one with the PSP DRTM
service holding localities 1 to 4, one on the defaults with a SHA-384 bank, one
through a TPM 1.2, one through a CRB TPM that fails `tpm.locality` and launches
anyway, and one the gate blocks. Those in `test_psp_launch.py` take the PSP
path: one where every step passes and the probe sees the TMR block the PIT's
interrupt, one with another SPLT version, one whose service is not ready so
`auto` falls back, two that `launch=psp` blocks, one whose `TMR_SETUP`, one
whose `LAUNCH` and one whose extend times out, one built with a longer
`DRTM_EXTEND_TIMEOUT_MS` that waits the same extend out, one without an
IVRS, and five whose `LAUNCH` the service refuses. Four of those patch a
field of the signed image inside the ESP's copy of `drtm.efi`, through
`machine.patch_sl`, and one raises the platform's patch level. The tests
recompute PCR 17 and 18 from the measured part of `target/sl-psp.bin`, its
key token and `target/dlme.bin` rather than trusting what drtm printed.

`test_foreign_sl_amd.py` runs only when `DRTM_TEST_FOREIGN_SL` names one of
AMD's SLs. It builds a second binary embedding it, boots it with
`launch=foreign`, and reads the POST codes `isa-post-code` logged to
`post-codes.txt` in the boot's log directory, since that SL writes nothing
else, then the DLME's report on serial. What the boot needs from QEMU is in
`docs/foreign-sl.md`.
What each check can prove, and what the build still models loosely, is in
`qemu-drtm-support.md`. Dasharo's own TrenchBoot QEMU tests run on upstream and
never get further than GRUB's precondition checks.

Dasharo's `trenchboot/*.robot` tests are a useful model for DRTM-aware test
structure later. They run against a fully booted Linux and read PCRs from
`/sys/class/tpm/tpm0/pcr-*` and the event log from
`/sys/kernel/security/slaunch/eventlog`, though not applicable verbatim
since we don't boot a full OS.

## Boot time

Almost all of a session is firmware rather than anything being tested, so the
boots are where the time is. One is now ~5s, some 8s less than it was, and
the configurations boot together, as many at a time as the machine has
cores, rather than one after another. Measured under TCG against the pinned
firmware release, each figure below being what a run lost when that change
landed, over the boots it applies to.

| Change                                 | Per boot |
|----------------------------------------|----------|
| VGA and NIC dropped from most boots    | 2.4s     |
| Boot prompt answered, not waited out   | 3.0s     |
| Firmware image warmed once per release | 2.8s     |

Keeping the populated variable store saves nothing by itself, but it is what
lets the warmed image be cached at all. `default_boot` keeps the two devices,
since its tests are the ones reading a populated bus.
