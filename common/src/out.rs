// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Output once UEFI is gone: every record to every UART in a sink list, in
//! the form and at the level `drtm.conf` chose, and the tally a summary
//! reports. The preamble writes through it after `ExitBootServices()`, and
//! the Secure Loader throughout.

use core::fmt::{self, Write};

use crate::params::MAX_SINKS;
use crate::record::{self, Format, Summary, Tag, Value};
use crate::uart::Uart;

/// `log::LevelFilter` numbering, which the parameter page uses.
pub const ERROR: u8 = 1;
pub const WARN: u8 = 2;
pub const INFO: u8 = 3;
pub const DEBUG: u8 = 4;

/// Every sink at once, as one `fmt::Write`.
pub struct Sinks(pub [Option<Uart>; MAX_SINKS]);

impl Write for Sinks {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for uart in self.0.iter_mut().flatten() {
            uart.write_str(text)?;
        }
        Ok(())
    }
}

pub struct Out {
    sinks: Sinks,
    format: Format,
    level: u8,
    counts: [u32; Tag::ALL.len()],
}

impl Out {
    pub fn new(sinks: [Option<Uart>; MAX_SINKS], format: Format, level: u8) -> Self {
        Self {
            sinks: Sinks(sinks),
            format,
            level,
            counts: [0; Tag::ALL.len()],
        }
    }

    /// Reports one result and counts it for the summary. Printed at the
    /// level drtm's own checks are: failures at error, warnings at warn.
    pub fn check(&mut self, tag: Tag, id: &str, fields: &[(&str, Value)], detail: fmt::Arguments) {
        self.counts[tag as usize] += 1;
        self.unchecked(tag, id, fields, detail);
    }

    /// Reports a result that the summary does not count, such as what
    /// follows it.
    pub fn unchecked(
        &mut self,
        tag: Tag,
        id: &str,
        fields: &[(&str, Value)],
        detail: fmt::Arguments,
    ) {
        let level = match tag {
            Tag::Fail => ERROR,
            Tag::Warn => WARN,
            _ => INFO,
        };
        if level <= self.level {
            let _ = record::check(&mut self.sinks, self.format, tag, id, fields, detail);
        }
    }

    /// Reports a result at debug level, uncounted: progress rather than a
    /// verdict.
    pub fn progress(&mut self, id: &str, fields: &[(&str, Value)], detail: fmt::Arguments) {
        if DEBUG <= self.level {
            let _ = record::check(&mut self.sinks, self.format, Tag::Info, id, fields, detail);
        }
    }

    pub fn debug(&mut self, message: fmt::Arguments) {
        if DEBUG <= self.level {
            let _ = record::log(&mut self.sinks, self.format, None, message);
        }
    }

    pub fn summary(&mut self) {
        let summary = Summary::new(|tag| self.counts[tag as usize]);
        let _ = record::summary(&mut self.sinks, self.format, &summary);
    }
}
