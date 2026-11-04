<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Development

## Toolchain setup

Install Rust via [rustup](https://rustup.rs/), then add the two targets. `drtm`
itself is built for UEFI, and the Secure Loader and the DLME embedded in it are
built for bare metal:

```sh
rustup target add x86_64-unknown-uefi x86_64-unknown-none
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
- `swtpm` and [our QEMU build](testing.md#the-qemu-build): for the test suite.

## Building

```sh
cargo build
```

Produces `target/x86_64-unknown-uefi/debug/drtm.efi`. The target is fixed in
`.cargo/config.toml`, so no `--target` flag is needed. `build.rs` first builds
the Secure Loader twice with its own `sl` profile into a target directory under
`OUT_DIR`, once per launch path, then the DLME with the same profile. It
flattens each build into the image that is measured, and fails the build if an
image breaks the size or layout limits of the SLB or the DLME's block. It then
computes the PCR values the launch should leave. It also copies the images to
`target/sl-classic.bin`, `target/sl-psp.bin` and `target/dlme.bin`, where the
QEMU tests recompute those values.

## Task runner

Build, lint, and test are wrapped in a [Taskfile](https://taskfile.dev/)
(install `task` via its own instructions):

```sh
task build
task lint
task test
```

These just wrap the commands on this page, so using them directly works too.
Arguments after `--` reach the wrapped command, as in `task build -- --features
sink-console` or `task lint -- cargo-clippy`. `docs` takes none, because its
script does not.

## Crates

The repository is a Cargo workspace. The root package is `drtm`, the EFI binary,
and the workspace's only default member, so a plain `cargo build` or
`cargo clippy` builds that and nothing else.

`sl` is the [Secure Loader][glossary], which runs after `SKINIT`. `build.rs`
builds it twice, once per launch path, through its `classic` and `psp` features.
`dlme` is the DLME the Secure Loader hands off to, and `build.rs` builds it
once. Both are built for `x86_64-unknown-none` and embedded in `drtm.efi`.

`common` holds what `drtm`, `sl` and `dlme` share, such as the record format and
the TPM commands, and needs neither UEFI nor a particular processor mode. Its
unit tests run on the host:

```sh
cargo test-host
```

`test-host` is an alias in `.cargo/config.toml`, since the configured default
target is UEFI and a bare `cargo test` would build for it. It also runs the
tests of `sign`, which `build.rs` uses to sign the Secure Loader for the PSP
DRTM service, with the key `keys/test/README.md` describes.

## Layout

- `src/`: the Rust binary. `src/launch/` is the gate and the preamble.
- `sl/`: the Secure Loader, with its linker script `sl.ld`.
- `dlme/`: the DLME every launch hands off to, with its linker script `dlme.ld`.
- `common/`: what `drtm`, the Secure Loader and the DLME share.
- `qemu-tests/`: the pytest-based QEMU test suite (kept out of Cargo's own
    `tests/` directory to avoid confusion between the two). The harness it boots
    with is the `drtmtest` package from the `drtm-tests` repository.
- `scripts/`: standalone helper scripts, e.g. dependency doc generation.
- `tools/fwscan/`: a host tool that looks for an SL key allowlist in a firmware
    image's PSP DRTM TA (Python, uv, see its README).
- `docs/`: this folder.
- `Taskfile.yml`, `.pre-commit-config.yaml`: see [Task runner](#task-runner) and
    [Linting](#linting).

## Build features

Log records go to the serial device by default, because that is what an operator
running this by hand is usually watching. The `sink-console` feature adds the
UEFI text console as a second output, called a [sink][glossary]. With
`--no-default-features` the console replaces the serial device. To build each
combination, run:

```sh
cargo build --features sink-console                      # both
cargo build --no-default-features --features sink-console  # console only
```

Enabling both is right only where the two are separate devices. Firmware that
mirrors its console to the serial port, which the test suite's does, would then
show every line twice. The QEMU suite only ever builds the default, so the other
two combinations are checked by `cargo build` and nothing else.

`DRTM_LOG_LEVEL`, `DRTM_LOG_FORMAT` and `DRTM_LAUNCH` set a run's starting
configuration at build time, for a binary copied to an ESP with no `drtm.conf`
beside it. To set them, run:

```sh
DRTM_LOG_LEVEL=trace DRTM_LOG_FORMAT=json DRTM_LAUNCH=psp cargo build
```

Each takes the same values as its `drtm.conf` key. `DRTM_EXTEND_TIMEOUT_MS` has
no key. It sets how many milliseconds the PSP Secure Loader waits for
`EXTEND_OSSL_DIGEST`, 10000 unless given:

```sh
DRTM_EXTEND_TIMEOUT_MS=180000 cargo build
```

An unusable value fails the build, as does `DRTM_LAUNCH=foreign` without
`DRTM_FOREIGN_SL`, since nothing later in the run could report it. Cargo
rebuilds when a variable changes, so to return to the defaults, run
`cargo build` with the variable unset. A `drtm.conf` on the machine still
overrides whatever was built in.

## Runtime configuration

`drtm.conf` sits beside the binary on the ESP, next to `BOOTX64.EFI`, and is
read at startup. It is optional: with no file the defaults apply.

```text
# how much to print: off, error, warn, info, debug, trace
level=info
# what it looks like: text or json
format=text
# whether and how the run ends in a launch: auto, psp, skinit, foreign or off
launch=auto
```

`level=info` is the default and prints one line per check. `debug` adds the PCI
root bridge count. `trace` adds the TPM's per-locality access registers and,
with `format=json`, the platform dump below. A build made with `DRTM_LOG_LEVEL`
starts at that level instead, and this file overrides it either way.

`format=text` is the default and prints prose for the person watching the serial
console. `format=json` prints one JSON object per line instead, still behind the
`drtm:` prefix, so a parser can find its own lines on a UART the firmware also
writes to:

```text
drtm: {"rec":"check","id":"tpm.present","result":"pass","fields":{"family":"V2_0","interface":"Crb"},"detail":"locality 0 answered, V2_0 Crb"}
drtm: {"rec":"log","src":"pci.rs:262","msg":"[SCAN] PCI root bridges: 1"}
drtm: {"rec":"summary","fail":1,"warn":2,"pass":4,"info":2,"skip":2,"verdict":"fail"}
```

A check that reports a value also puts it in the `fields` object, so a parser
does not have to extract it from the `detail` text. Each check picks its own
field names. They sit in a separate object, so a field called `id` or `result`
cannot overwrite the record's own keys. Counts are written as numbers, while
addresses and register values are `"0x..."` strings, since a 64-bit value does
not survive every JSON parser as a number.

The two forms are exclusive, because they reach the same person over the same
wire and printing both would show everything twice. So the JSON form carries
everything the prose does: a `check` record per result, a `log` record for
anything the checks print alongside them, and a closing `summary` record.

Both forms end with the same tally, the prose one as a line:

```text
drtm: checks complete, verdict fail: 2 fail, 3 warn, 4 pass, 1 info, 2 skip
```

The verdict is `fail` if any check failed, `warn` if any warned, and `pass`
otherwise, since a skipped check and a reported fact decide nothing. The counts
hold what ran rather than what printed, so lowering the level does not move
them.

`launch=auto` is the default. Once the checks allow a launch, the run goes
through `SKINIT` into the Secure Loader, and also uses the PSP DRTM service
where the platform offers it. The Secure Loader reports and hands off to the
DLME, which reports and halts the machine. With the defaults, a missing
`drtm.conf` launches too. `launch=psp` insists on the PSP path and
`launch=skinit` never takes it. `launch=off` stops after the summary. See
`docs/design.md` for what the gate blocks on and how the path is chosen.
`launch=foreign` exists only in a build made with `DRTM_FOREIGN_SL`, see
`docs/foreign-sl.md`.

A line that is not `key=value`, an unknown key and an unusable value are each
reported as a `config` warning naming the line, and the default is kept. A file
this tool cannot read is never a reason to refuse to run, since producing
diagnostics on a machine that is already misbehaving is the point.

## Platform dump

`format=json` with `level=trace` adds the raw platform state a hardware
investigation needs: firmware version, SMBIOS, boot variables, the firmware
component inventory, CPUID, the processors firmware started, every ACPI table,
PCI configuration space, TPM registers, MSRs, the registers of any IOMMU
firmware enabled, and the Secure Processor registers the ASPT names. The dump
needs both settings, because it has no prose form and would bury the results in
the prose output.

```text
drtm: {"rec":"dump","at":"acpi"}
drtm: {"rec":"acpi","sig":"FACP","addr":"0x3f9f8000","len":244,"csum":true,"hex":"46414350..."}
drtm: {"rec":"msr","id":"0xc0010058","name":"MMIO_CONF_BASE","value":"0x3ffb00000021"}
drtm: {"rec":"config","fn":"0000:00:00.0","len":4096,"hex":"86802900...","fill":"0xff"}
```

The dump records every value whole and unfiltered, with two exceptions:

- A trailing run of identical bytes in PCI configuration space is replaced by
    the `fill` field, which holds that byte. `len` gives the full length, so the
    bytes can be rebuilt.
- A DSDT or SSDT larger than 4 KiB is reduced to its header and marked `cut`,
    because nothing here interprets AML and one DSDT is larger than every other
    table together. `len` still gives the table's true size.

A `dump` record opens each section of the dump. Within a section, the reads
happen in the order the source code lists them. If a machine dies mid-dump, the
last record shows where it stopped.

Nothing is rendered for a person, since hexadecimal digits cost half what a
hexdump does on the wire. A script turning a capture back into hexdumps can be
written the day someone needs to read one by eye.

## Linting

Formatting and linting (Rust, Python, Markdown, spelling, license headers,
generic hygiene checks) run via [pre-commit](https://pre-commit.com/), config in
`.pre-commit-config.yaml`:

```sh
apt install pre-commit    # or: pipx install pre-commit
pre-commit install        # wires it into git, runs on every commit
pre-commit run --all-files    # run everything once, e.g. after cloning
```

`cargo-fmt` and `cargo-clippy` hooks shell out to the local toolchain directly,
so `rustup component add rustfmt clippy` is needed once too. Clippy runs five
times: `drtm` for UEFI, `sl` for `x86_64-unknown-none` once per launch path,
`dlme` for the same target, and `common` on the host with its tests.

## Licensing

The project is BSD-3-Clause (`LICENSE`, a symlink into `LICENSES/`) and aims for
[REUSE](https://reuse.software/) compliance, checked by the `reuse` pre-commit
hook. New files need an `SPDX-FileCopyrightText` / `SPDX-License-Identifier`
header (see any existing file for the style). For ones that cannot carry a
header (generated lockfiles, `NOTICE`), add an entry to `REUSE.toml` instead.

## Testing

See `docs/testing.md`.

## Dependency docs

`task docs` (`scripts/gen-dependency-docs.sh`) regenerates offline `rustdoc`
HTML for `drtm`'s dependencies into `docs/dependency-docs` (gitignored, since
it's generated and can be rebuilt any time). It also builds an `index.html`
linking to each documented crate, styled to match rustdoc's own pages, and adds
a link back to that index on each crate's root page.

[glossary]: design.md#glossary
