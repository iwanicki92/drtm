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
