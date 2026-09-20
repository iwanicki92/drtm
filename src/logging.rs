// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Where log output goes and what it looks like.
//!
//! Replaces the logger in `uefi::helpers`, whose format is fixed and whose
//! sink is the text console. The console hard-wraps at 80 columns mid-word,
//! so anything longer arrives split.
//!
//! [`emit`] holds the record format and depends on nothing but
//! [`core::fmt::Write`], so the post-`SKINIT` stub can reuse it with a sink
//! of its own once it exists. What differs between the two is the transport,
//! not the format.

use core::ffi::c_void;
use core::fmt::{self, Write};
use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU8, Ordering};

use log::{Level, LevelFilter, Log, Metadata, Record};

use crate::check::{Summary, Tag, Value};
use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi::proto::console::serial::Serial;
use uefi::{Handle, system};

/// Which sinks to write each record to, from the `sink-serial` and
/// `sink-console` cargo features.
///
/// Both at once is right only where the two are separate devices. Firmware
/// that mirrors its console to the serial port, as the one the test suite
/// uses does, would then show every line twice. Nothing in the application
/// can detect which kind of platform it is on, hence the build flag.
const SERIAL_SINK: bool = cfg!(feature = "sink-serial");
const CONSOLE_SINK: bool = cfg!(feature = "sink-console");

#[cfg(not(any(feature = "sink-serial", feature = "sink-console")))]
compile_error!("enable at least one of the sink-serial, sink-console features");

/// Serial device to write to, null if the platform offered none.
static SERIAL: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

/// Which of the two forms records take. Text until [`set_format`] says
/// otherwise, so anything printed before `drtm.conf` is read is readable.
static FORMAT: AtomicU8 = AtomicU8::new(Format::Text as u8);

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

/// Picks the form for the rest of the run. Call once, from `drtm.conf`.
pub fn set_format(format: Format) {
    FORMAT.store(format as u8, Ordering::Relaxed);
}

pub fn format() -> Format {
    if FORMAT.load(Ordering::Relaxed) == Format::Json as u8 {
        Format::Json
    } else {
        Format::Text
    }
}

static LOGGER: Logger = Logger;

struct Logger;

/// Installs the logger. Call once, after `uefi::helpers::init()`.
///
/// Everything up to `trace` is compiled in and let through, until `drtm.conf`
/// has been read and sets the level.
pub fn init() {
    if let Ok(handle) = boot::get_handle_for_protocol::<Serial>() {
        SERIAL.store(handle.as_ptr(), Ordering::Relaxed);
    }
    log::set_logger(&LOGGER).expect("a logger is already installed");
    log::set_max_level(LevelFilter::Trace);
}

/// Opens the serial device for one write.
///
/// Opened per record rather than held, so nothing borrowed outlives the
/// call and the handle can live in a `static`.
fn serial() -> Option<ScopedProtocol<Serial>> {
    // Safety: SERIAL holds either null, which yields None, or the pointer
    // `init` took from a live handle. Handles stay valid for all of boot
    // services, and nothing here runs after that.
    let handle = unsafe { Handle::from_ptr(SERIAL.load(Ordering::Relaxed)) }?;
    // Safety: the handle came from `get_handle_for_protocol`. `GetProtocol`
    // leaves the owning driver alone; an exclusive open would disconnect the
    // console driver sharing this UART and silence it for the rest of the boot.
    unsafe {
        boot::open_protocol::<Serial>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    }
    .ok()
}

/// Last component of a module path, so the prefix width does not track the
/// module tree. Moving a file deeper used to shift every wrapped line.
fn file_name(path: &str) -> &str {
    match path.rfind('/') {
        Some(cut) => &path[cut + 1..],
        None => path,
    }
}

/// Where the record came from, as `file.rs:LINE`. Only carried at the
/// developer levels, which is what it is an aid to.
fn source<'a>(record: &'a Record<'a>) -> Option<(&'a str, u32)> {
    (record.level() > Level::Info).then(|| {
        (
            file_name(record.file().unwrap_or("?")),
            record.line().unwrap_or(0),
        )
    })
}

/// Writes one record in the form the run is using. `\r\n` because this is a
/// UART, not a terminal that translates newlines.
fn emit(sink: &mut dyn Write, record: &Record) -> fmt::Result {
    match format() {
        Format::Text => emit_text(sink, record),
        Format::Json => emit_json(sink, record),
    }
}

fn emit_text(sink: &mut dyn Write, record: &Record) -> fmt::Result {
    match source(record) {
        None => write!(sink, "drtm: {}\r\n", record.args()),
        Some((file, line)) => write!(sink, "drtm: {file}:{line}: {}\r\n", record.args()),
    }
}

/// One self-contained object per line, so a line lost to noise costs one
/// record. The `drtm:` prefix stays, since firmware writes to the same UART
/// and a parser has to find its own lines without trying to parse every one.
fn emit_json(sink: &mut dyn Write, record: &Record) -> fmt::Result {
    sink.write_str(r#"drtm: {"rec":"log""#)?;
    if let Some((file, line)) = source(record) {
        write!(sink, r#","src":"{file}:{line}""#)?;
    }
    sink.write_str(r#","msg":""#)?;
    write!(Escape(sink), "{}", record.args())?;
    sink.write_str("\"}\r\n")
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

/// Writes one line to every sink this build and platform have. Both forms
/// go through here, since the structured one has no reason to own a sink.
fn write_line(mut line: impl FnMut(&mut dyn Write) -> fmt::Result) {
    let mut on_serial = false;
    if SERIAL_SINK && let Some(mut serial) = serial() {
        let _ = line(&mut *serial);
        on_serial = true;
    }
    // Also the fallback: a platform with no serial device would otherwise
    // say nothing at all.
    if CONSOLE_SINK || !on_serial {
        system::with_stdout(|stdout| {
            let _ = line(stdout);
        });
    }
}

/// Writes one check result. The text form is what the checks have always
/// printed, and the JSON form makes its three fields addressable instead of
/// leaving a parser to split the same line up again.
pub fn check(tag: Tag, id: &str, fields: &[(&str, Value)], detail: fmt::Arguments) {
    write_line(|sink| match format() {
        // Fields say again what the detail already spells out, so prose
        // leaves them out.
        Format::Text => write!(sink, "drtm: [{}] {id}: {detail}\r\n", tag.name()),
        // Identifiers and field names are literals in this crate's own
        // source, dotted ASCII throughout, so only the values can need
        // escaping. Fields keep an object of their own, so a check naming
        // one `id` or `result` cannot shadow the record's own keys.
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
    });
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
fn write_hex(sink: &mut dyn Write, bytes: &[u8]) -> fmt::Result {
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

/// Writes one platform dump record: a kind and its named values, flat,
/// since both are literals in this crate's source and cannot collide.
///
/// JSON only. The dump exists to be parsed, and the hexadecimal it carries
/// would bury the results a person is reading the prose form for.
pub fn record(kind: &str, fields: &[(&str, Value)]) {
    if format() != Format::Json {
        return;
    }
    write_line(|sink| {
        write!(sink, r#"drtm: {{"rec":"{kind}""#)?;
        for (name, value) in fields {
            write!(sink, r#","{name}":"#)?;
            write_value(sink, value)?;
        }
        sink.write_str("}\r\n")
    });
}

/// Ends the run with what the checks added up to.
///
/// Both forms carry the tally, so a person reading prose does not have to
/// count the lines back. `checks complete` leads the text form because the
/// QEMU harness reads until it, as it does until the record in JSON.
pub fn summary(summary: &Summary) {
    write_line(|sink| match format() {
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
    });
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        write_line(|sink| emit(sink, record));
    }

    fn flush(&self) {}
}
