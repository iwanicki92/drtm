<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: Apache-2.0
-->

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
- [`uv`](https://docs.astral.sh/uv/): manages the Python test dependencies
  and their virtual environment, see `docs/testing.md`.
- `qemu-system-x86_64`: for the test suite.

## Build features

Log records go to the serial device by default, because that is what an
operator running this by hand is usually watching. `sink-console` adds (or,
with `--no-default-features`, replaces it with) the UEFI text console:

```sh
cargo build --features sink-console                      # both
cargo build --no-default-features --features sink-console  # console only
```

Enabling both is right only where the two are separate devices. Firmware
that mirrors its console to the serial port, which the test suite's does,
would then show every line twice. The QEMU suite only ever builds the
default, so the other two combinations are checked by `cargo build` and
nothing else.

## Task runner

Build, lint, and test are wrapped in a [Taskfile](https://taskfile.dev/)
(install `task` via its own instructions):

```sh
task build
task lint
task test
```

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

## Licensing

The project is Apache-2.0 (`LICENSE`, a symlink into `LICENSES/`) and aims
for [REUSE](https://reuse.software/) compliance, checked by the `reuse`
pre-commit hook. New files need an `SPDX-FileCopyrightText` /
`SPDX-License-Identifier` header (see any existing file for the style); for
ones that cannot carry a header (generated lockfiles, `NOTICE`), add an
entry to `REUSE.toml` instead.

## Testing

See `docs/testing.md`.

## Dependency docs

`task docs` (`scripts/gen-dependency-docs.sh`) regenerates offline `rustdoc`
HTML for `drtm`'s dependencies into `docs/dependency-docs` (gitignored,
since it's generated and can be rebuilt any time). It also builds an
`index.html` linking to each documented crate, styled to match rustdoc's own
pages, and adds a link back to that index on each crate's root page.

## Layout

- `src/`: the Rust binary.
- `qemu-tests/`: the pytest-based QEMU test suite (kept out of Cargo's own
  `tests/` directory to avoid confusion between the two).
- `scripts/`: standalone helper scripts, e.g. dependency doc generation.
- `docs/`: this folder.
- `Taskfile.yml`, `.pre-commit-config.yaml`: see above.
