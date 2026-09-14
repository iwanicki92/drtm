// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

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
use core::fmt::Write;
use core::ptr;
use core::sync::atomic::{AtomicPtr, Ordering};

use log::{Level, LevelFilter, Log, Metadata, Record};
use uefi::boot::{self, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi::proto::console::serial::Serial;
use uefi::{Handle, system};

/// Verbosity until something asks for another. Everything up to `trace` is
/// compiled in, so raising it needs no rebuild.
const DEFAULT_LEVEL: LevelFilter = LevelFilter::Info;

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

static LOGGER: Logger = Logger;

struct Logger;

/// Installs the logger. Call once, after `uefi::helpers::init()`.
pub fn init() {
    if let Ok(handle) = boot::get_handle_for_protocol::<Serial>() {
        SERIAL.store(handle.as_ptr(), Ordering::Relaxed);
    }
    log::set_logger(&LOGGER).expect("a logger is already installed");
    log::set_max_level(DEFAULT_LEVEL);
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

/// Writes one record. `\r\n` because this is a UART, not a terminal that
/// translates newlines.
fn emit(sink: &mut impl Write, record: &Record) -> core::fmt::Result {
    if record.level() <= Level::Info {
        write!(sink, "drtm: {}\r\n", record.args())
    } else {
        // File and line are a developer aid, and these are the developer
        // levels, so the prefix follows the level rather than a build flag.
        write!(
            sink,
            "drtm: {}:{}: {}\r\n",
            file_name(record.file().unwrap_or("?")),
            record.line().unwrap_or(0),
            record.args()
        )
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let mut on_serial = false;
        if SERIAL_SINK && let Some(mut serial) = serial() {
            let _ = emit(&mut *serial, record);
            on_serial = true;
        }
        // Also the fallback: a platform with no serial device would
        // otherwise say nothing at all.
        if CONSOLE_SINK || !on_serial {
            system::with_stdout(|stdout| {
                let _ = emit(stdout, record);
            });
        }
    }

    fn flush(&self) {}
}
