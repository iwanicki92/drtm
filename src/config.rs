// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

//! `drtm.conf`, read from beside the binary on the volume it was loaded from.
//!
//! One `key=value` per line, `#` starting a comment. A missing file means the
//! defaults. Anything wrong in it is reported and the default kept, since
//! refusing to run over a typo would cost the diagnostics this exists for.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::str;

use log::LevelFilter;
use uefi::boot::{self, ScopedProtocol};
use uefi::fs::PathBuf;
use uefi::proto::device_path::DevicePathNodeEnum;
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::file::{File, FileAttribute, FileMode};
use uefi::{CString16, Status, cstr16};

use crate::check;

/// A config file is a few lines. Anything larger is not one.
const MAX_SIZE: usize = 4096;

/// Verbosity when nothing asks for another: results, not the device dumps.
/// `DRTM_LOG_LEVEL` at build time picks another, for a binary meant to be
/// copied to an ESP on its own. A `level=` in `drtm.conf` still wins.
const DEFAULT_LEVEL: LevelFilter = default_level();

/// Reads `DRTM_LOG_LEVEL`, taking the same names `level=` takes. An
/// unusable value fails the build rather than quietly leaving `info`, since
/// nothing later in the run could report it.
const fn default_level() -> LevelFilter {
    let Some(name) = option_env!("DRTM_LOG_LEVEL") else {
        return LevelFilter::Info;
    };
    match name.as_bytes() {
        b"off" => LevelFilter::Off,
        b"error" => LevelFilter::Error,
        b"warn" => LevelFilter::Warn,
        b"info" => LevelFilter::Info,
        b"debug" => LevelFilter::Debug,
        b"trace" => LevelFilter::Trace,
        _ => panic!("DRTM_LOG_LEVEL must be one of off, error, warn, info, debug, trace"),
    }
}

/// Settings, and whatever went wrong reading them.
pub struct Config {
    pub level: LevelFilter,
    problems: Vec<Problem>,
}

enum Problem {
    Unreadable(Status),
    TooLarge,
    NotText,
    NoEquals { line: usize },
    UnknownKey { line: usize, key: String },
    BadLevel { line: usize, value: String },
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(status) => write!(f, "drtm.conf would not read ({status:?})"),
            Self::TooLarge => write!(f, "drtm.conf is over {MAX_SIZE} bytes"),
            Self::NotText => write!(f, "drtm.conf is not UTF-8 text"),
            Self::NoEquals { line } => write!(f, "line {line} is not key=value"),
            Self::UnknownKey { line, key } => write!(f, "line {line}: no key {key:?}"),
            Self::BadLevel { line, value } => write!(
                f,
                "line {line}: level {value:?} is not one of off, error, warn, \
                 info, debug, trace"
            ),
        }
    }
}

impl Config {
    /// Reads `drtm.conf`. Reports nothing yet, so the level can be applied
    /// before anything is printed under it.
    pub fn load() -> Self {
        let mut config = Self {
            level: DEFAULT_LEVEL,
            problems: Vec::new(),
        };
        let mut buffer = [0; MAX_SIZE + 1];
        match read(&mut buffer) {
            Ok(Some(text)) => config.parse(text),
            Ok(None) => {}
            Err(problem) => config.problems.push(problem),
        }
        config
    }

    /// Reports what was wrong with the file, if anything.
    pub fn report(&self) {
        for problem in &self.problems {
            check::warn!("config", "{problem}, so the default applies");
        }
    }

    fn parse(&mut self, text: &str) {
        // Some editors still start a UTF-8 file with a byte order mark.
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        for (index, line) in text.lines().enumerate() {
            let line_number = index + 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                self.problems.push(Problem::NoEquals { line: line_number });
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "level" => match value.parse() {
                    Ok(level) => self.level = level,
                    Err(_) => self.problems.push(Problem::BadLevel {
                        line: line_number,
                        value: value.to_string(),
                    }),
                },
                _ => self.problems.push(Problem::UnknownKey {
                    line: line_number,
                    key: key.to_string(),
                }),
            }
        }
    }
}

/// Reads the file into `buffer`, or `None` where there is no file to read.
///
/// No file includes nowhere to put one: an image loaded from memory or the
/// network has no path or volume, and that is not a fault to report.
fn read(buffer: &mut [u8]) -> Result<Option<&str>, Problem> {
    let Some(path) = path() else {
        return Ok(None);
    };
    let Ok(mut volume) = boot::get_image_file_system(boot::image_handle()) else {
        return Ok(None);
    };
    let mut root = volume
        .open_volume()
        .map_err(|error| Problem::Unreadable(error.status()))?;
    let file = match root.open(path.as_ref(), FileMode::Read, FileAttribute::empty()) {
        Ok(file) => file,
        Err(error) if error.status() == Status::NOT_FOUND => return Ok(None),
        Err(error) => return Err(Problem::Unreadable(error.status())),
    };
    let Some(mut file) = file.into_regular_file() else {
        return Err(Problem::Unreadable(Status::INVALID_PARAMETER));
    };
    // One byte past the limit, so a file that fills it is told apart from
    // one that exceeds it.
    let length = file
        .read(buffer)
        .map_err(|error| Problem::Unreadable(error.status()))?;
    if length > MAX_SIZE {
        return Err(Problem::TooLarge);
    }
    str::from_utf8(&buffer[..length])
        .map(Some)
        .map_err(|_| Problem::NotText)
}

/// Where `drtm.conf` would be: the directory the binary was loaded from.
fn path() -> Option<PathBuf> {
    let image: ScopedProtocol<LoadedImage> =
        boot::open_protocol_exclusive(boot::image_handle()).ok()?;
    // Firmware may split the path over several file path nodes, one per
    // component, so join every one rather than taking the last.
    let mut binary = PathBuf::new();
    for node in image.file_path()?.node_iter() {
        if let Ok(DevicePathNodeEnum::MediaFilePath(node)) = node.as_enum() {
            binary.push(PathBuf::from(CString16::try_from(&node.path_name()).ok()?));
        }
    }
    let mut config = binary.parent()?;
    config.push(cstr16!("drtm.conf"));
    Some(config)
}
