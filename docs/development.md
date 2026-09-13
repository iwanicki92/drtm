# Development

## Toolchain setup

Install Rust via [rustup](https://rustup.rs/), then add the UEFI target:

```sh
rustup target add x86_64-unknown-uefi
```

If your root filesystem is space-constrained (common on small VMs), rustup
respects `RUSTUP_HOME`/`CARGO_HOME` to install elsewhere instead of the
default `~/.rustup`/`~/.cargo` — set those before running the installer if
needed.

System packages needed alongside it:

- `build-essential` — `rustc`'s host build scripts and proc-macros (e.g.
  `proc-macro2`, `quote`) compile for the host target and need a C linker
  (`cc`) even though the final binary targets `x86_64-unknown-uefi`, which
  doesn't need one.
- `python3.12-venv` (or whatever matches the system Python) — to create
  `.venv-vm`, see `docs/testing.md`.
- `qemu-system-x86_64` — for the test suite.

## Building

```sh
cargo build
```

Produces `target/x86_64-unknown-uefi/debug/drtm.efi`. The target is fixed in
`.cargo/config.toml`, so no `--target` flag is needed.

## Testing

See `docs/testing.md`.

## Layout

- `src/` — the Rust binary.
- `qemu-tests/` — the pytest-based QEMU test suite (kept out of Cargo's own
  `tests/` directory to avoid confusion between the two).
- `docs/` — this folder.
