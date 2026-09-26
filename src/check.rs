// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! How a check states its result.
//!
//! One shape for all of them: a verdict, a stable identifier and the detail
//! behind it, as `[PASS] amd.iommu: capability block at 0x40`. Identifiers
//! are dotted with the area first, so tests can key on verdict and identifier
//! and rewording the detail is not a behaviour change.
//!
//! Results do not go through `log!`, because the record carries the tag and
//! the identifier as fields of their own. What they keep from it is the
//! level each used to log at, so `level=` still decides what prints.
//!
//! A check whose result is a value rather than a verdict names that value
//! as a field too, so a reader of the JSON form does not have to pick it
//! back out of the sentence. The prose is unchanged by it.

use core::fmt;
use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};

use log::Level;

use crate::logging;

pub use common::record::{Summary, Tag, Value};

/// The level a result is filtered at. A failure is an error and a warning a
/// warning, so a run cut down to `level=error` still shows what stops a
/// launch.
const fn level(tag: Tag) -> Level {
    match tag {
        Tag::Fail => Level::Error,
        Tag::Warn => Level::Warn,
        _ => Level::Info,
    }
}

/// A plain count, written as a number.
pub fn count(value: impl Into<u64>) -> Value<'static> {
    Value::Count(value.into())
}

/// An address, offset or register value, written as a hexadecimal string.
pub fn hex(value: impl Into<u64>) -> Value<'static> {
    Value::Hex(value.into())
}

/// A yes or no, written as a JSON boolean.
pub fn flag(value: bool) -> Value<'static> {
    Value::Flag(value)
}

/// Raw bytes, written as one run of hexadecimal digits. What the platform
/// dump carries, since laying them out for a person is a reader's job.
pub fn bytes(value: &[u8]) -> Value<'_> {
    Value::Bytes(value)
}

/// Anything with a `Display`, as `text!("{function}")`. A macro because the
/// formatted arguments live only as long as the call they are made in.
macro_rules! check_text {
    ($($arguments:tt)*) => {
        $crate::check::Value::Text(format_args!($($arguments)*))
    };
}

/// One counter per tag, indexed by the tag's own discriminant.
static COUNTS: [AtomicU32; Tag::ALL.len()] = [const { AtomicU32::new(0) }; Tag::ALL.len()];

/// The results the launch gate reads back, by identifier, and which tags
/// each reported, one bit per tag.
const TRACKED: [&str; 5] = [
    "amd.skinit",
    "cpu.machine_check",
    "cpu.bsp",
    "amd.slb",
    "tpm.present",
];
static REPORTED: [AtomicU8; TRACKED.len()] = [const { AtomicU8::new(0) }; TRACKED.len()];

/// Whether `id`, one of the results the gate reads, reported `tag` at least
/// once this run.
pub fn reported(id: &str, tag: Tag) -> bool {
    TRACKED
        .iter()
        .position(|&tracked| tracked == id)
        .is_some_and(|at| REPORTED[at].load(Ordering::Relaxed) & (1 << tag as u8) != 0)
}

/// Reports a result that no summary counts, because it follows the one
/// the checks end with. Filtered at its tag's level all the same.
pub fn after_summary(tag: Tag, id: &str, fields: &[(&str, Value)], detail: fmt::Arguments) {
    if level(tag) <= log::max_level() {
        logging::check(tag, id, fields, detail);
    }
}

/// Reports one result. Called by the macros below rather than directly.
///
/// Counted whether or not it prints, so a run at `level=error` still ends
/// with a summary that agrees with a run at `level=info`.
pub fn report(tag: Tag, id: &str, fields: &[(&str, Value)], detail: fmt::Arguments) {
    COUNTS[tag as usize].fetch_add(1, Ordering::Relaxed);
    if let Some(at) = TRACKED.iter().position(|&tracked| tracked == id) {
        REPORTED[at].fetch_or(1 << tag as u8, Ordering::Relaxed);
    }
    if level(tag) <= log::max_level() {
        logging::check(tag, id, fields, detail);
    }
}

/// Ends the run, reporting the tally and the worst verdict in it.
pub fn summary() {
    logging::summary(&Summary::new(|tag| {
        COUNTS[tag as usize].load(Ordering::Relaxed)
    }));
}

/// Shared body of the tag macros below: an id, optional `name = value`
/// fields, then the detail as `format!` arguments.
macro_rules! check_report {
    ($tag:ident, $id:literal, $($field:ident = $value:expr),+ ; $($detail:tt)*) => {
        $crate::check::report(
            $crate::check::Tag::$tag,
            $id,
            &[$((stringify!($field), $value)),+],
            format_args!($($detail)*),
        )
    };
    ($tag:ident, $id:literal, $($detail:tt)*) => {
        $crate::check::report($crate::check::Tag::$tag, $id, &[], format_args!($($detail)*))
    };
}

/// The platform meets this precondition.
macro_rules! check_pass {
    ($($arguments:tt)*) => {
        $crate::check::check_report!(Pass, $($arguments)*)
    };
}

/// The platform does not meet it, and no dynamic launch can happen here.
macro_rules! check_fail {
    ($($arguments:tt)*) => {
        $crate::check::check_report!(Fail, $($arguments)*)
    };
}

/// Worth knowing, but not a failed precondition: a launch would still go
/// ahead, with something it would rather have had.
macro_rules! check_warn {
    ($($arguments:tt)*) => {
        $crate::check::check_report!(Warn, $($arguments)*)
    };
}

/// The check did not run. Not an anomaly, so it sits at `info` with the
/// results rather than among the warnings.
macro_rules! check_skip {
    ($($arguments:tt)*) => {
        $crate::check::check_report!(Skip, $($arguments)*)
    };
}

/// A fact about the platform that gates nothing, for a check that reports
/// rather than judges. Earns its own tag by not claiming a verdict it cannot
/// support, and can become a pass or a fail once it can.
macro_rules! check_info {
    ($($arguments:tt)*) => {
        $crate::check::check_report!(Info, $($arguments)*)
    };
}

// Defined under `check_` names and renamed here, because a plain
// `macro_rules! warn` cannot be re-exported: the bare name is ambiguous with
// the built-in `warn` attribute.
pub(crate) use {
    check_fail as fail, check_info as info, check_pass as pass, check_report, check_skip as skip,
    check_text as text, check_warn as warn,
};
