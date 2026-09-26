<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Test signing key

`sl-test-key.pem` is an RSA-4096 private key, published on purpose. The
build signs the Secure Loader with it for the PSP DRTM service, which on
server parts takes an image signed with any key and extends PCR 18 with
the key token the image carries. Anyone can sign an SL with this key, so a
PCR 18 holding its token says nothing about who built the image.

`sl-test-key.pub` is its public half, which the host tests verify the
signature against.

To sign with another key, point `DRTM_SL_KEY` at a PKCS#8 PEM file of an
RSA-4096 key before `cargo build`. PCR 18 then changes with the key's
token, and `build.rs` predicts the new value.
