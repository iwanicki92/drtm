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

## What it does

`drtm` is a UEFI binary that checks whether an AMD platform meets the
preconditions of a dynamic launch, and can dump the platform state those checks
read. It then launches its own [Secure Loader][glossary] with the `SKINIT`
instruction. Where the platform offers AMD's [PSP DRTM service][glossary], the
launch also uses that service.

The Secure Loader reads the DRTM PCRs and reports whether the TPM and the
service measured it. It then hands off to `drtm`'s own [DLME][glossary], which
compares the PCRs with the values its measurement should leave and halts the
machine. The machine then needs a power cycle.

`launch=` in `drtm.conf` picks the path: `auto` (the default), `psp`, `skinit`,
or `off` to stop a run after the checks. See [docs/design.md](docs/design.md)
for how it works and what it leaves out.

## Requirements

- Rust via [rustup](https://rustup.rs/), with the `x86_64-unknown-uefi` and
    `x86_64-unknown-none` targets added.
- For the test suite:
    - [our QEMU build](docs/testing.md#the-qemu-build), since a distribution
        QEMU can't run it
    - `swtpm`, which backs the emulated TPM the TPM-detection tests need
    - [uv](https://docs.astral.sh/uv/)

See [docs/development.md](docs/development.md) for setup details.

## Usage

```sh
task build   # cargo build -> target/x86_64-unknown-uefi/debug/drtm.efi
task lint    # pre-commit run --all-files
task test    # uv run --project qemu-tests pytest qemu-tests (builds the binary)
task docs    # scripts/gen-dependency-docs.sh -> docs/dependency-docs
```

`task` is optional. Each wraps a plain command documented in
[docs/development.md](docs/development.md) or, for the tests,
[docs/testing.md](docs/testing.md). Anything after `--` reaches that command
(except for `docs`), so `task build -- --features sink-console` and
`task test -- -k tpm` work.

## Configuration

`drtm.conf` sits beside the binary on the ESP and is read at startup. It is
optional: with no file the defaults apply. One `key=value` per line, `#`
starting a comment, and anything wrong in it is reported and the default kept.

| Key      | Values                                           | Default |
|----------|--------------------------------------------------|---------|
| `level`  | `off`, `error`, `warn`, `info`, `debug`, `trace` | `info`  |
| `format` | `text`, `json`                                   | `text`  |
| `launch` | `auto`, `psp`, `skinit`, `foreign`, `off`        | `auto`  |

The three keys can also be set at build time, for a binary copied to an ESP with
no `drtm.conf` beside it. A `drtm.conf` on the machine overrides them. Two more
settings exist only at build time: where log records go, and how long the Secure
Loader on the PSP path waits for an answer to `EXTEND_OSSL_DIGEST`, the service
command that measures the DLME. The table below lists the environment variables
and cargo features for all of them.

| Setting                        | Values                    | Default |
|--------------------------------|---------------------------|---------|
| `DRTM_LOG_LEVEL`               | the names `level=` takes  | `info`  |
| `DRTM_LOG_FORMAT`              | the names `format=` takes | `text`  |
| `DRTM_LAUNCH`                  | the names `launch=` takes | `auto`  |
| `DRTM_EXTEND_TIMEOUT_MS`       | milliseconds, above 0     | `10000` |
| `sink-serial` (cargo feature)  | on, off                   | on      |
| `sink-console` (cargo feature) | on, off                   | off     |

```sh
DRTM_LOG_LEVEL=trace DRTM_LOG_FORMAT=json task build -- --features sink-console
```

An unusable value for any of the environment variables fails the build, because
a bad built-in value could not be reported later in the run. The `sink-*`
features choose where log records go. Enable both only if the serial port and
the UEFI text console are separate devices, otherwise every line prints twice.
See [docs/development.md](docs/development.md) for the rest.

`launch=foreign` needs a debug build made with `DRTM_FOREIGN_SL` set to a Secure
Loader signed by someone else, such as AMD's client or EPYC SL from Lenovo's
Windows "AMD DRTM Boot Driver" packages. It launches that SL in place of
`drtm`'s own. That SL reports only through POST codes, until it hands off to
`drtm`'s DLME, which reports as usual. See
[docs/foreign-sl.md](docs/foreign-sl.md).

## Firmware scan

On some platforms the PSP DRTM service accepts a Secure Loader only if its
signing key is on a list in the [DRTM TA][glossary]. `drtm`'s own key is never
on that list. [tools/fwscan](tools/fwscan/README.md) checks a firmware image for
such a list before it is flashed:

```sh
uv run --project tools/fwscan fwscan IMAGE...
```

It gives each image one of these verdicts:

- `allowlist`: the TA compares the SL's [key token][glossary] with digests
    stored in it. The report gives the keys it recognises.
- `no-allowlist`: the TA compares only digests of other keys.
- `encrypted`: the TA cannot be read.
- `inconclusive`: the TA is plaintext, but no such compare was found in it.
- `no-drtm-ta`: the image has no DRTM TA.

Newer client firmware carries such a list, see
[docs/client-sl-key-allowlist.md](docs/client-sl-key-allowlist.md). Server
firmware ships its TA encrypted, so a scan can't tell, and some server firmware
does refuse `drtm`'s key.

## More

- [docs/design.md](docs/design.md): target scope, architecture, and DRTM launch
    design decisions.
- [docs/testing.md](docs/testing.md): how the QEMU test suite works and why.
- [docs/development.md](docs/development.md): toolchain setup and repo layout.
- [docs/foreign-sl.md](docs/foreign-sl.md): launching AMD's own SLs, and what it
    expects from the OS loader.

## Acknowledgements

Two ideas in `drtm` come from the talk [A Practical Security Assessment of DRTM
on AMD-based Systems][talk] by Christian Werling and Julian Prein at the Boot
Security Mastery Conference 2026 ([slides][talk-slides]):

- Reusing the Secure Loader that Windows launches. Newer client firmware most
    likely launches only SLs signed with a key on its built-in list. AMD's SL is
    signed with a listed key and `drtm`'s own SL is not, so `drtm` launches AMD's
    SL, which then jumps to `drtm`'s code. See
    [docs/foreign-sl.md](docs/foreign-sl.md).
- Checking which firmware versions accept only Secure Loaders signed with a
    listed key. See
    [docs/client-sl-key-allowlist.md](docs/client-sl-key-allowlist.md).

## License

BSD-3-Clause, see [LICENSE](LICENSE).

[glossary]: docs/design.md#glossary
[talk]: https://cfp.3mdeb.com/bsmconf/talk/ZJWHCU/
[talk-slides]: https://cfp.3mdeb.com/media/bsmconf/submissions/ZJWHCU/resources/20260924-drtm-on-amd-slides_5B8J8mF.pdf
