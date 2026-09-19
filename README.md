<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: Apache-2.0
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

Early scaffolding: a UEFI binary that boots and logs a line, plus a QEMU test
proving it does. No DRTM launch logic yet. See
[docs/design.md](docs/design.md) for the planned scope.

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
[docs/development.md](docs/development.md).

## Configuration

`drtm.conf` sits beside the binary on the ESP and is read at startup. It is
optional: with no file the defaults apply. One `key=value` per line, `#`
starting a comment, and anything wrong in it is reported and the default
kept.

| Key      | Values                                           | Default |
|----------|--------------------------------------------------|---------|
| `level`  | `off`, `error`, `warn`, `info`, `debug`, `trace` | `info`  |
| `format` | `text`, `json`                                   | `text`  |

The same two can be built in, for a binary copied to an ESP with no
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

## More

- [docs/design.md](docs/design.md): target scope, architecture, and DRTM
    launch design decisions.
- [docs/testing.md](docs/testing.md): how the QEMU test suite works and why.
- [docs/development.md](docs/development.md): toolchain setup and repo
    layout.

## License

Apache-2.0, see [LICENSE](LICENSE). `qemu-tests/qmp_client.py` is adapted
from third-party Apache-2.0 code, see [NOTICE](NOTICE).
