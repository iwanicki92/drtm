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
proving it does. No DRTM launch logic yet — see
[docs/design.md](docs/design.md) for the planned scope.

## Requirements

- Rust via [rustup](https://rustup.rs/), with the `x86_64-unknown-uefi`
    target added.
- `qemu-system-x86_64` and a Python 3 venv, for the test suite.

See [docs/development.md](docs/development.md) for setup details.

## Usage

```sh
cargo build
```

Produces `target/x86_64-unknown-uefi/debug/drtm.efi`.

To run the QEMU smoke test:

```sh
python3 -m venv .venv-vm
.venv-vm/bin/pip install -r qemu-tests/requirements.txt
.venv-vm/bin/pytest qemu-tests/
```

## More

- [docs/design.md](docs/design.md) — target scope, architecture, and DRTM
    launch design decisions.
- [docs/testing.md](docs/testing.md) — how the QEMU test suite works and why.
- [docs/development.md](docs/development.md) — toolchain setup and repo
    layout.
