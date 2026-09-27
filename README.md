<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# DRTM

A single-binary tool for testing DRTM (Dynamic Root of Trust for
Measurement) and gathering logs to help debug
[TrenchBoot](https://trenchboot.org/) issues, currently focused on UEFI.

Where TrenchBoot splits DRTM support across multiple components (GRUB and
Linux/Xen, or GRUB, SKL, and Linux/Xen), this tool is deliberately simpler:
one binary whose only job is to exercise the DRTM launch and report whether
it worked.

## Status

A UEFI binary that checks whether an AMD platform meets the preconditions of
a dynamic launch, dumps the platform state behind them, and then launches
through `SKINIT` into its own Secure Loader, through AMD's PSP DRTM service
where the platform offers it. The Secure Loader reads the DRTM PCRs, reports
whether the TPM and the service measured it, and halts the machine, which
then needs a power cycle. `launch=` in `drtm.conf` picks the path: `auto`
(the default), `psp`, `skinit`, or `off` to stop a run after the checks.
See [docs/design.md](docs/design.md) for how it works and what it leaves
out.

## Requirements

- Rust via [rustup](https://rustup.rs/), with the `x86_64-unknown-uefi`
    target added.
- `qemu-system-x86_64`, `swtpm` and [uv](https://docs.astral.sh/uv/), for
    the test suite. `swtpm` backs the emulated TPM the TPM-detection tests
    need.

See [docs/development.md](docs/development.md) for setup details.

## Usage

```sh
task build   # cargo build -> target/x86_64-unknown-uefi/debug/drtm.efi
task lint    # pre-commit run --all-files
task test    # uv run pytest qemu-tests/ (builds the binary itself first)
task docs    # cargo doc -> docs/dependency-docs (offline dependency docs)
```

`task` is optional. Each wraps a plain command documented in
[docs/development.md](docs/development.md). Anything after `--` reaches that
command, so `task build -- --features sink-console` and `task test -- -k tpm`
work.

## Configuration

`drtm.conf` sits beside the binary on the ESP and is read at startup. It is
optional: with no file the defaults apply. One `key=value` per line, `#`
starting a comment, and anything wrong in it is reported and the default
kept.

| Key      | Values                                           | Default |
|----------|--------------------------------------------------|---------|
| `level`  | `off`, `error`, `warn`, `info`, `debug`, `trace` | `info`  |
| `format` | `text`, `json`                                   | `text`  |
| `launch` | `auto`, `psp`, `skinit`, `foreign`, `off`        | `auto`  |

The first two can be built in, for a binary copied to an ESP with no
`drtm.conf` beside it, along with where records go. A `drtm.conf` on the
machine still wins over either.

| Setting                        | Values                    | Default |
|--------------------------------|---------------------------|---------|
| `DRTM_LOG_LEVEL`               | the names `level=` takes  | `info`  |
| `DRTM_LOG_FORMAT`              | the names `format=` takes | `text`  |
| `sink-serial` (cargo feature)  | on, off                   | on      |
| `sink-console` (cargo feature) | on, off                   | off     |

```sh
DRTM_LOG_LEVEL=trace DRTM_LOG_FORMAT=json task build -- --features sink-console
```

An unusable value for either variable fails the build, since nothing later in
the run could report it. Enabling both sinks is right only where the two are
separate devices. See [docs/development.md](docs/development.md) for the
rest.

`launch=foreign` needs a debug build made with `DRTM_FOREIGN_SL` naming a
Secure Loader signed by someone else, such as AMD's client SL from Lenovo's
Windows "AMD DRTM Boot Driver" package. It launches that SL in place of
`drtm`'s own, which then reports only through POST codes. See
[docs/foreign-sl.md](docs/foreign-sl.md).

## Firmware scan

The PSP DRTM service on some platforms takes only Secure Loaders signed with
keys its DRTM TA lists, which `drtm`'s own key never is.
[tools/fwscan](tools/fwscan/README.md) checks a firmware image for such a list
before it is flashed:

```sh
uv run --project tools/fwscan fwscan IMAGE...
```

It reports `allowlist` when the TA compares the SL's key token against
digests it carries, naming the keys it recognises, `encrypted` when the TA
cannot be read, `inconclusive` for a plaintext TA without that check, and
`no-drtm-ta`. The client firmware checked so far carries such a list, and
server firmware ships its TA encrypted.

## More

- [docs/design.md](docs/design.md): target scope, architecture, and DRTM
    launch design decisions.
- [docs/testing.md](docs/testing.md): how the QEMU test suite works and why.
- [docs/development.md](docs/development.md): toolchain setup and repo
    layout.
- [docs/foreign-sl.md](docs/foreign-sl.md): launching AMD's client SL, and
    what it expects from the OS loader.

## License

BSD-3-Clause, see [LICENSE](LICENSE).
