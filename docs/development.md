# Development

## Toolchain setup

Install Rust via [rustup](https://rustup.rs/), then add the UEFI target:

```sh
rustup target add x86_64-unknown-uefi
```

If your root filesystem is space-constrained (common on small VMs), rustup
respects `RUSTUP_HOME`/`CARGO_HOME` to install elsewhere instead of the
default `~/.rustup`/`~/.cargo`. Set those before running the installer if
needed.

System packages needed alongside it:

- `build-essential`: `rustc`'s host build scripts and proc-macros (e.g.
  `proc-macro2`, `quote`) compile for the host target and need a C linker
  (`cc`) even though the final binary targets `x86_64-unknown-uefi`, which
  doesn't need one.
- `python3.12-venv` (or whatever matches the system Python): to create
  `.venv`, see `docs/testing.md`.
- `qemu-system-x86_64`: for the test suite.

## Task runner

Build, lint, and test are wrapped in a [Taskfile](https://taskfile.dev/)
(install `task` via its own instructions):

```sh
task build
task lint
task test
```

`task test` runs `pytest` from whatever's on `PATH`, so activate the venv
(`source .venv/bin/activate`, see `docs/testing.md`) before running it.

These just wrap the commands below, so using them directly works too.

## Building

```sh
cargo build
```

Produces `target/x86_64-unknown-uefi/debug/drtm.efi`. The target is fixed in
`.cargo/config.toml`, so no `--target` flag is needed.

## Linting

Formatting and linting (Rust, Python, Markdown, spelling, license headers,
generic hygiene checks) run via [pre-commit](https://pre-commit.com/), config
in `.pre-commit-config.yaml`:

```sh
apt install pre-commit    # or: pipx install pre-commit
pre-commit install        # wires it into git, runs on every commit
pre-commit run --all-files    # run everything once, e.g. after cloning
```

`cargo-fmt` and `cargo-clippy` hooks shell out to the local toolchain
directly, so `rustup component add rustfmt clippy` is needed once too.

## Testing

See `docs/testing.md`.

## Layout

- `src/`: the Rust binary.
- `qemu-tests/`: the pytest-based QEMU test suite (kept out of Cargo's own
  `tests/` directory to avoid confusion between the two).
- `docs/`: this folder.
- `Taskfile.yml`, `.pre-commit-config.yaml`: see above.
