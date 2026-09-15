// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! How a check states its result.
//!
//! One shape for all of them: a verdict, a stable identifier and the detail
//! behind it, as `[PASS] amd.iommu: capability block at 0x40`. Identifiers
//! are dotted with the area first, so tests can key on verdict and identifier
//! and rewording the detail is not a behaviour change.

/// The platform meets this precondition.
macro_rules! check_pass {
    ($id:literal, $($detail:tt)*) => {
        log::info!("[PASS] {}: {}", $id, format_args!($($detail)*))
    };
}

/// The platform does not meet it, and no dynamic launch can happen here.
macro_rules! check_fail {
    ($id:literal, $($detail:tt)*) => {
        log::error!("[FAIL] {}: {}", $id, format_args!($($detail)*))
    };
}

/// Worth knowing, but not a failed precondition: a launch would still go
/// ahead, with something it would rather have had.
macro_rules! check_warn {
    ($id:literal, $($detail:tt)*) => {
        log::warn!("[WARN] {}: {}", $id, format_args!($($detail)*))
    };
}

/// The check did not run. Not an anomaly, so it sits at `info` with the
/// results rather than among the warnings.
macro_rules! check_skip {
    ($id:literal, $($detail:tt)*) => {
        log::info!("[SKIP] {}: {}", $id, format_args!($($detail)*))
    };
}

// Defined under `check_` names and renamed here, because a plain
// `macro_rules! warn` cannot be re-exported: the bare name is ambiguous with
// the built-in `warn` attribute.
pub(crate) use {check_fail as fail, check_pass as pass, check_skip as skip, check_warn as warn};
