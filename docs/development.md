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

`DRTM_LOG_LEVEL` and `DRTM_LOG_FORMAT` set what a run starts at, for a binary
copied to an ESP with no `drtm.conf` beside it:

```sh
DRTM_LOG_LEVEL=trace DRTM_LOG_FORMAT=json cargo build
```

Each takes the same names its `drtm.conf` key does. An unusable value fails
the build, since nothing later in the run could report it, and cargo rebuilds
when a variable changes, so switching back is a plain `cargo build`. A
`drtm.conf` on the machine still overrides whatever was built in.

## Runtime configuration

`drtm.conf` sits beside the binary on the ESP, next to `BOOTX64.EFI`, and is
read at startup. It is optional: with no file the defaults apply.

```text
# how much to print: off, error, warn, info, debug, trace
level=info
# what it looks like: text or json
format=text
```

`level=info` is the default and prints one line per check. `debug` adds the
PCI root bridge count, and `trace` the whole configuration space dump the
IOMMU and Secure Processor checks read. A build made with `DRTM_LOG_LEVEL`
starts at that level instead, and this file overrides it either way.

`format=text` is the default and prints prose for the person watching the
serial console. `format=json` prints one JSON object per line instead, still
behind the `drtm:` prefix, so a parser can find its own lines on a UART the
firmware also writes to:

```text
drtm: {"rec":"check","id":"tpm.present","result":"pass","detail":"locality 0 answered, V2_0 Crb"}
drtm: {"rec":"log","level":"debug","src":"pci.rs:262","msg":"[SCAN] PCI root bridges: 1"}
drtm: {"rec":"summary","fail":1,"warn":2,"pass":4,"info":2,"skip":2,"verdict":"fail"}
```

The two forms are exclusive, because they reach the same person over the same
wire and printing both would show everything twice. So the JSON form carries
everything the prose does: a `check` record per result, a `log` record for
anything the checks print alongside them, and a closing `summary` record with
the tally and the worst verdict in it. `verdict` is `fail` if any check
failed, `warn` if any warned, and `pass` otherwise, and the counts hold what
ran rather than what printed, so lowering the level does not move them.

A line that is not `key=value`, an unknown key and an unusable value are
each reported as a `config` warning naming the line, and the default is kept.
A file this tool cannot read is never a reason to refuse to run, since
producing diagnostics on a machine that is already misbehaving is the point.

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
