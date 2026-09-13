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
- `qemu-system-x86_64` and [uv](https://docs.astral.sh/uv/), for the test
    suite.

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

## More

- [docs/design.md](docs/design.md): target scope, architecture, and DRTM
    launch design decisions.
- [docs/testing.md](docs/testing.md): how the QEMU test suite works and why.
- [docs/development.md](docs/development.md): toolchain setup and repo
    layout.

## License

Apache-2.0, see [LICENSE](LICENSE). `qemu-tests/qmp_client.py` is adapted
from third-party Apache-2.0 code, see [NOTICE](NOTICE).
