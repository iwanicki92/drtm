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
use core::sync::atomic::{AtomicU32, Ordering};

use log::Level;

use crate::logging;

/// What a check concluded. Three of these are verdicts, [`Tag::Skip`] says
/// the check did not run, and [`Tag::Info`] reports a platform fact while
/// gating nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Pass,
    Fail,
    Warn,
    Skip,
    Info,
}

impl Tag {
    /// In the order a summary reads best, worst first.
    const ALL: [Self; 5] = [Self::Fail, Self::Warn, Self::Pass, Self::Info, Self::Skip];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Warn => "WARN",
            Self::Skip => "SKIP",
            Self::Info => "INFO",
        }
    }

    pub const fn result(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Warn => "warn",
            Self::Skip => "skip",
            Self::Info => "info",
        }
    }

    /// The level this result is filtered at. A failure is an error and a
    /// warning a warning, so a run cut down to `level=error` still shows
    /// what stops a launch.
    const fn level(self) -> Level {
        match self {
            Self::Fail => Level::Error,
            Self::Warn => Level::Warn,
            _ => Level::Info,
        }
    }
}

/// One named value in a record, whether a check's or the platform dump's.
/// What a field is decides how it is written: counts are JSON numbers, while
/// addresses and register values are `"0x..."` strings, since a 64-bit value
/// does not survive every JSON parser as a number.
pub enum Value<'a> {
    Count(u64),
    Hex(u64),
    Flag(bool),
    Bytes(&'a [u8]),
    Text(fmt::Arguments<'a>),
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

/// What the checks added up to.
pub struct Summary {
    pub counts: [(Tag, u32); Tag::ALL.len()],
    pub verdict: Tag,
}

/// Reports one result. Called by the macros below rather than directly.
///
/// Counted whether or not it prints, so a run at `level=error` still ends
/// with a summary that agrees with a run at `level=info`.
pub fn report(tag: Tag, id: &str, fields: &[(&str, Value)], detail: fmt::Arguments) {
    COUNTS[tag as usize].fetch_add(1, Ordering::Relaxed);
    if tag.level() <= log::max_level() {
        logging::check(tag, id, fields, detail);
    }
}

/// Ends the run, reporting the tally and the worst verdict in it.
pub fn summary() {
    let counts = Tag::ALL.map(|tag| (tag, COUNTS[tag as usize].load(Ordering::Relaxed)));
    // A skipped check is not a verdict, and neither is a reported fact, so
    // neither can decide this one.
    let verdict = counts
        .iter()
        .find(|&&(tag, count)| count > 0 && matches!(tag, Tag::Fail | Tag::Warn))
        .map_or(Tag::Pass, |&(tag, _)| tag);
    logging::summary(&Summary { counts, verdict });
}

/// The platform meets this precondition.
macro_rules! check_pass {
    ($id:literal, $($field:ident = $value:expr),+ ; $($detail:tt)*) => {
        $crate::check::report(
            $crate::check::Tag::Pass,
            $id,
            &[$((stringify!($field), $value)),+],
            format_args!($($detail)*),
        )
    };
    ($id:literal, $($detail:tt)*) => {
        $crate::check::report($crate::check::Tag::Pass, $id, &[], format_args!($($detail)*))
    };
}

/// The platform does not meet it, and no dynamic launch can happen here.
macro_rules! check_fail {
    ($id:literal, $($field:ident = $value:expr),+ ; $($detail:tt)*) => {
        $crate::check::report(
            $crate::check::Tag::Fail,
            $id,
            &[$((stringify!($field), $value)),+],
            format_args!($($detail)*),
        )
    };
    ($id:literal, $($detail:tt)*) => {
        $crate::check::report($crate::check::Tag::Fail, $id, &[], format_args!($($detail)*))
    };
}

/// Worth knowing, but not a failed precondition: a launch would still go
/// ahead, with something it would rather have had.
macro_rules! check_warn {
    ($id:literal, $($field:ident = $value:expr),+ ; $($detail:tt)*) => {
        $crate::check::report(
            $crate::check::Tag::Warn,
            $id,
            &[$((stringify!($field), $value)),+],
            format_args!($($detail)*),
        )
    };
    ($id:literal, $($detail:tt)*) => {
        $crate::check::report($crate::check::Tag::Warn, $id, &[], format_args!($($detail)*))
    };
}

/// The check did not run. Not an anomaly, so it sits at `info` with the
/// results rather than among the warnings.
macro_rules! check_skip {
    ($id:literal, $($field:ident = $value:expr),+ ; $($detail:tt)*) => {
        $crate::check::report(
            $crate::check::Tag::Skip,
            $id,
            &[$((stringify!($field), $value)),+],
            format_args!($($detail)*),
        )
    };
    ($id:literal, $($detail:tt)*) => {
        $crate::check::report($crate::check::Tag::Skip, $id, &[], format_args!($($detail)*))
    };
}

/// A fact about the platform that gates nothing, for a check that reports
/// rather than judges. Earns its own tag by not claiming a verdict it cannot
/// support, and can become a pass or a fail once it can.
macro_rules! check_info {
    ($id:literal, $($field:ident = $value:expr),+ ; $($detail:tt)*) => {
        $crate::check::report(
            $crate::check::Tag::Info,
            $id,
            &[$((stringify!($field), $value)),+],
            format_args!($($detail)*),
        )
    };
    ($id:literal, $($detail:tt)*) => {
        $crate::check::report($crate::check::Tag::Info, $id, &[], format_args!($($detail)*))
    };
}

// Defined under `check_` names and renamed here, because a plain
// `macro_rules! warn` cannot be re-exported: the bare name is ambiguous with
// the built-in `warn` attribute.
pub(crate) use {
    check_fail as fail, check_info as info, check_pass as pass, check_skip as skip,
    check_text as text, check_warn as warn,
};
