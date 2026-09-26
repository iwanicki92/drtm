// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Where log output goes and what it looks like.
//!
//! Replaces the logger in `uefi::helpers`, whose format is fixed and whose
//! sink is the text console. The console hard-wraps at 80 columns mid-word,
//! so anything longer arrives split.
//!
//! The record format itself is `common::record`, which the Secure Loader
//! prints through too. This module owns the UEFI transport.

use core::ffi::c_void;
use core::fmt::{self, Write};
use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU8, Ordering};

use log::{Level, LevelFilter, Log, Metadata, Record};

pub use common::record::Format;
use common::record::{self, Summary, Tag, Value};
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

/// Writes one line to every sink this build and platform have. Every kind
/// of record goes through here, since none of them has a reason to own a
/// sink.
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

/// Writes one check result.
pub fn check(tag: Tag, id: &str, fields: &[(&str, Value)], detail: fmt::Arguments) {
    write_line(|sink| record::check(sink, format(), tag, id, fields, detail));
}

/// Writes one platform dump record. JSON only: the dump exists to be
/// parsed, and the hexadecimal it carries would bury the results a person
/// is reading the prose form for.
pub fn record(kind: &str, fields: &[(&str, Value)]) {
    if format() == Format::Json {
        write_line(|sink| record::record(sink, Format::Json, kind, fields));
    }
}

/// Ends the run with what the checks added up to.
pub fn summary(summary: &Summary) {
    write_line(|sink| record::summary(sink, format(), summary));
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        write_line(|sink| record::log(sink, format(), source(record), *record.args()));
    }

    fn flush(&self) {}
}
