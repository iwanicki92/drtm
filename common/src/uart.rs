// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! A 16550 UART driven by its registers, the one output sink the Secure
//! Loader has. Firmware configured the line, so this only writes bytes.

use core::arch::asm;
use core::fmt;

/// Where a UART's registers are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Space {
    Io = 1,
    Mmio = 2,
}

/// One 16550: where its registers start and how wide each is, 1 byte or 4.
/// Registers sit `width` bytes apart, as SPCR's access size implies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uart {
    pub space: Space,
    pub base: u64,
    pub width: u8,
}

/// The legacy COM1 port, which a board without SPCR most likely wires its
/// console to, and where the Secure Kernel Loader writes.
pub const COM1: Uart = Uart {
    space: Space::Io,
    base: 0x3F8,
    width: 1,
};

const THR: u64 = 0;
const SCR: u64 = 7;
const LSR: u64 = 5;
const LSR_THRE: u8 = 1 << 5;

/// How many line status reads to wait for the transmitter before writing
/// anyway, so a UART that never drains slows the run but cannot stop it.
const SPIN_LIMIT: u32 = 100_000;

impl Uart {
    fn read(&self, register: u64) -> u8 {
        let at = self.base + register * u64::from(self.width);
        match (self.space, self.width) {
            (Space::Io, _) => {
                let value: u8;
                // Safety: port I/O touches no memory, and the port was chosen
                // as this UART's.
                unsafe {
                    asm!("in al, dx", in("dx") at as u16, out("al") value,
                         options(nomem, nostack, preserves_flags));
                }
                value
            }
            // Safety: the MMIO address is this UART's register, below 4 GiB
            // as the sink list requires, and identity-mapped wherever this
            // runs.
            (Space::Mmio, 4) => unsafe { (at as *const u32).read_volatile() as u8 },
            // Safety: as above.
            (Space::Mmio, _) => unsafe { (at as *const u8).read_volatile() },
        }
    }

    fn write(&self, register: u64, value: u8) {
        let at = self.base + register * u64::from(self.width);
        match (self.space, self.width) {
            // Safety: as for `read`.
            (Space::Io, _) => unsafe {
                asm!("out dx, al", in("dx") at as u16, in("al") value,
                     options(nomem, nostack, preserves_flags));
            },
            // Safety: as for `read`.
            (Space::Mmio, 4) => unsafe { (at as *mut u32).write_volatile(u32::from(value)) },
            // Safety: as for `read`.
            (Space::Mmio, _) => unsafe { (at as *mut u8).write_volatile(value) },
        }
    }

    pub fn put(&self, byte: u8) {
        let mut spins = 0;
        while self.read(LSR) & LSR_THRE == 0 && spins < SPIN_LIMIT {
            core::hint::spin_loop();
            spins += 1;
        }
        self.write(THR, byte);
    }

    /// Whether a 16550 answers here: its scratch register keeps what was
    /// written, which an empty port reading all ones does not. What was
    /// there before goes back.
    pub fn answers(&self) -> bool {
        let saved = self.read(SCR);
        let answered = [0x5A, 0xA5].iter().all(|&pattern| {
            self.write(SCR, pattern);
            self.read(SCR) == pattern
        });
        self.write(SCR, saved);
        answered
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            self.put(byte);
        }
        Ok(())
    }
}
