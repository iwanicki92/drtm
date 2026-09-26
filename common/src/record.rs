// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! What a line of output looks like, in either of the two forms.
//!
//! Depends on nothing but [`core::fmt::Write`], so `drtm` under UEFI and the
//! Secure Loader after `SKINIT` print the same records through sinks of
//! their own. What differs between the two is the transport, not the format.

use core::fmt::{self, Write};

/// The two output forms, one run at a time.
///
/// They reach the same person on the same wire, so emitting both would show
/// every event twice. JSON therefore has to carry everything the prose does,
/// down to the detail text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Json,
}

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
    pub const ALL: [Self; 5] = [Self::Fail, Self::Warn, Self::Pass, Self::Info, Self::Skip];

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

/// What the checks added up to: a count per tag and the worst verdict.
pub struct Summary {
    pub counts: [(Tag, u32); Tag::ALL.len()],
    pub verdict: Tag,
}

impl Summary {
    /// The tally of `count`, the number of results each tag had. A skipped
    /// check is not a verdict, and neither is a reported fact, so neither
    /// can decide this one.
    pub fn new(count: impl Fn(Tag) -> u32) -> Self {
        let counts = Tag::ALL.map(|tag| (tag, count(tag)));
        let verdict = counts
            .iter()
            .find(|&&(tag, count)| count > 0 && matches!(tag, Tag::Fail | Tag::Warn))
            .map_or(Tag::Pass, |&(tag, _)| tag);
        Self { counts, verdict }
    }
}

/// Writes one freeform message. `source` is where it came from, as
/// `file.rs:LINE`, carried only at the developer levels. `\r\n` because
/// this is a UART, not a terminal that translates newlines.
pub fn log(
    sink: &mut dyn Write,
    format: Format,
    source: Option<(&str, u32)>,
    message: fmt::Arguments,
) -> fmt::Result {
    match format {
        Format::Text => match source {
            None => write!(sink, "drtm: {message}\r\n"),
            Some((file, line)) => write!(sink, "drtm: {file}:{line}: {message}\r\n"),
        },
        // One self-contained object per line, so a line lost to noise costs
        // one record. The `drtm:` prefix stays, since firmware writes to the
        // same UART and a parser has to find its own lines without trying
        // to parse every one.
        Format::Json => {
            sink.write_str(r#"drtm: {"rec":"log""#)?;
            if let Some((file, line)) = source {
                write!(sink, r#","src":"{file}:{line}""#)?;
            }
            sink.write_str(r#","msg":""#)?;
            write!(Escape(sink), "{message}")?;
            sink.write_str("\"}\r\n")
        }
    }
}

/// Writes one check result. The text form is `[TAG] id: detail`, and the
/// JSON form makes its three fields addressable instead of leaving a parser
/// to split the same line up again.
pub fn check(
    sink: &mut dyn Write,
    format: Format,
    tag: Tag,
    id: &str,
    fields: &[(&str, Value)],
    detail: fmt::Arguments,
) -> fmt::Result {
    match format {
        // Fields say again what the detail already spells out, so prose
        // leaves them out.
        Format::Text => write!(sink, "drtm: [{}] {id}: {detail}\r\n", tag.name()),
        // Identifiers and field names are literals in the caller's source,
        // dotted ASCII throughout, so only the values can need escaping.
        // Fields keep an object of their own, so a check naming one `id` or
        // `result` cannot shadow the record's own keys.
        Format::Json => {
            write!(
                sink,
                r#"drtm: {{"rec":"check","id":"{id}","result":"{}""#,
                tag.result()
            )?;
            if let Some(((first, value), rest)) = fields.split_first() {
                write!(sink, r#","fields":{{"{first}":"#)?;
                write_value(sink, value)?;
                for (name, value) in rest {
                    write!(sink, r#","{name}":"#)?;
                    write_value(sink, value)?;
                }
                sink.write_str("}")?;
            }
            sink.write_str(r#","detail":""#)?;
            write!(Escape(sink), "{detail}")?;
            sink.write_str("\"}\r\n")
        }
    }
}

/// Writes one record of named values, flat, since both are literals in the
/// caller's source and cannot collide. JSON only: prose has no form for it,
/// so a text-mode run writes nothing.
pub fn record(
    sink: &mut dyn Write,
    format: Format,
    kind: &str,
    fields: &[(&str, Value)],
) -> fmt::Result {
    if format != Format::Json {
        return Ok(());
    }
    write!(sink, r#"drtm: {{"rec":"{kind}""#)?;
    for (name, value) in fields {
        write!(sink, r#","{name}":"#)?;
        write_value(sink, value)?;
    }
    sink.write_str("}\r\n")
}

/// Writes the tally a run ends with.
///
/// Both forms carry it, so a person reading prose does not have to count
/// the lines back. `checks complete` leads the text form because the QEMU
/// harness reads until it, as it does until the record in JSON.
pub fn summary(sink: &mut dyn Write, format: Format, summary: &Summary) -> fmt::Result {
    match format {
        Format::Text => {
            write!(
                sink,
                "drtm: checks complete, verdict {}",
                summary.verdict.result()
            )?;
            let mut separator = ": ";
            for (tag, count) in summary.counts {
                write!(sink, "{separator}{count} {}", tag.result())?;
                separator = ", ";
            }
            sink.write_str("\r\n")
        }
        Format::Json => {
            sink.write_str(r#"drtm: {"rec":"summary""#)?;
            for (tag, count) in summary.counts {
                write!(sink, r#","{}":{count}"#, tag.result())?;
            }
            write!(sink, r#","verdict":"{}"}}"#, summary.verdict.result())?;
            sink.write_str("\r\n")
        }
    }
}

fn write_value(sink: &mut dyn Write, value: &Value) -> fmt::Result {
    match value {
        Value::Count(count) => write!(sink, "{count}"),
        Value::Hex(bits) => write!(sink, "\"{bits:#x}\""),
        Value::Flag(flag) => sink.write_str(if *flag { "true" } else { "false" }),
        Value::Bytes(bytes) => {
            sink.write_str("\"")?;
            write_hex(sink, bytes)?;
            sink.write_str("\"")
        }
        Value::Text(text) => {
            sink.write_str("\"")?;
            write!(Escape(sink), "{text}")?;
            sink.write_str("\"")
        }
    }
}

/// `bytes` as hexadecimal digits, a chunk at a time.
///
/// Buffered rather than formatted byte by byte: a sink is a firmware call
/// per write, and a dumped ACPI table is a hundred thousand bytes.
pub fn write_hex(sink: &mut dyn Write, bytes: &[u8]) -> fmt::Result {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    for chunk in bytes.chunks(64) {
        let mut text = [0u8; 128];
        for (pair, byte) in text.as_chunks_mut::<2>().0.iter_mut().zip(chunk) {
            pair[0] = DIGITS[usize::from(byte >> 4)];
            pair[1] = DIGITS[usize::from(byte & 0xF)];
        }
        let Ok(text) = str::from_utf8(&text[..chunk.len() * 2]) else {
            return Err(fmt::Error);
        };
        sink.write_str(text)?;
    }
    Ok(())
}

/// Forwards to `sink` escaping what a JSON string cannot hold literally.
/// Nothing here buffers, so a record is written as it is formatted.
struct Escape<'a, W: Write + ?Sized>(&'a mut W);

impl<W: Write + ?Sized> Write for Escape<'_, W> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let mut rest = text;
        while let Some(at) = rest.find(|c: char| c == '"' || c == '\\' || c < ' ') {
            let (before, tail) = rest.split_at(at);
            self.0.write_str(before)?;
            let mut characters = tail.chars();
            match characters.next() {
                Some('"') => self.0.write_str("\\\"")?,
                Some('\\') => self.0.write_str("\\\\")?,
                // Control characters, which only a malformed read would
                // produce here. `\u` covers every one of them.
                Some(control) => write!(self.0, "\\u{:04x}", control as u32)?,
                None => break,
            }
            rest = characters.as_str();
        }
        self.0.write_str(rest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::String;

    fn text(emit: impl FnOnce(&mut String) -> fmt::Result) -> String {
        let mut out = String::new();
        emit(&mut out).unwrap();
        out
    }

    #[test]
    fn log_text_with_and_without_source() {
        assert_eq!(
            text(|s| log(s, Format::Text, None, format_args!("hello"))),
            "drtm: hello\r\n"
        );
        assert_eq!(
            text(|s| log(s, Format::Text, Some(("a.rs", 7)), format_args!("x"))),
            "drtm: a.rs:7: x\r\n"
        );
    }

    #[test]
    fn log_json_escapes_the_message() {
        assert_eq!(
            text(|s| log(s, Format::Json, None, format_args!("a\"b\\c\n"))),
            "drtm: {\"rec\":\"log\",\"msg\":\"a\\\"b\\\\c\\u000a\"}\r\n"
        );
    }

    #[test]
    fn check_in_both_forms() {
        let fields = [("n", Value::Count(3)), ("at", Value::Hex(0x10))];
        assert_eq!(
            text(|s| check(
                s,
                Format::Text,
                Tag::Warn,
                "a.b",
                &fields,
                format_args!("d")
            )),
            "drtm: [WARN] a.b: d\r\n"
        );
        assert_eq!(
            text(|s| check(
                s,
                Format::Json,
                Tag::Warn,
                "a.b",
                &fields,
                format_args!("d")
            )),
            "drtm: {\"rec\":\"check\",\"id\":\"a.b\",\"result\":\"warn\",\
             \"fields\":{\"n\":3,\"at\":\"0x10\"},\"detail\":\"d\"}\r\n"
        );
    }

    #[test]
    fn record_is_json_only() {
        let fields = [("b", Value::Bytes(&[0xab, 0x01])), ("f", Value::Flag(true))];
        assert_eq!(text(|s| record(s, Format::Text, "k", &fields)), "");
        assert_eq!(
            text(|s| record(s, Format::Json, "k", &fields)),
            "drtm: {\"rec\":\"k\",\"b\":\"ab01\",\"f\":true}\r\n"
        );
    }

    #[test]
    fn summary_verdict_is_the_worst_tag() {
        let tally = Summary::new(|tag| match tag {
            Tag::Pass => 2,
            Tag::Warn => 1,
            Tag::Skip => 4,
            _ => 0,
        });
        assert_eq!(tally.verdict, Tag::Warn);
        assert_eq!(
            text(|s| summary(s, Format::Text, &tally)),
            "drtm: checks complete, verdict warn: 0 fail, 1 warn, 2 pass, \
             0 info, 4 skip\r\n"
        );
        assert_eq!(
            text(|s| summary(s, Format::Json, &tally)),
            "drtm: {\"rec\":\"summary\",\"fail\":0,\"warn\":1,\"pass\":2,\
             \"info\":0,\"skip\":4,\"verdict\":\"warn\"}\r\n"
        );
        assert_eq!(Summary::new(|_| 0).verdict, Tag::Pass);
    }
}
