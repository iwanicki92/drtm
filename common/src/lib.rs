// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! What `drtm` and its Secure Loader share: code that needs neither UEFI nor
//! a particular processor mode, so both can link it and the host can test it.

#![no_std]

#[cfg(test)]
extern crate std;

pub mod record;
