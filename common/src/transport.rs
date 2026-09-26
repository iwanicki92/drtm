// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Talking to the TPM through its registers, with no firmware driver: what
//! the Secure Loader has once UEFI is gone, and how the preamble lets go of
//! locality 0 after `ExitBootServices()`.
//!
//! Register layouts are the TCG PC Client Platform TPM Profile's (PTP),
//! FIFO in section 6.5.2 and CRB in section 6.5.3.

use core::ptr;

use crate::clock::Deadline;

/// Base of the locality 0 window on the PC platform, fixed by the chipset
/// (PTP section 6.5.2). Each locality has 4 KiB of its own after it.
pub const TPM_BASE: usize = 0xFED4_0000;

/// Start of `locality`'s window.
pub const fn locality_base(locality: u8) -> usize {
    TPM_BASE + ((locality as usize) << 12)
}

/// The two register interfaces a TPM can present.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    Tis,
    Crb,
}

/// Why a command did not get an answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The locality was not granted in time.
    Locality,
    /// The TPM did not become ready for a command.
    NotReady,
    /// The TPM stopped taking bytes before the command ended, or wanted
    /// more after it.
    Transfer,
    /// No response within the time allowed.
    Timeout,
    /// The response is larger than the buffer, or its header is not one.
    Response,
    /// A CRB buffer the SL cannot reach, above 4 GiB.
    Buffer,
}

// FIFO registers and bits (PTP tables 19 and 31, and section 6.5.2.5).
const TIS_ACCESS: usize = 0x00;
const TIS_STS: usize = 0x18;
const TIS_BURST: usize = 0x19;
const TIS_FIFO: usize = 0x24;
const ACCESS_VALID: u8 = 1 << 7;
const ACCESS_ACTIVE: u8 = 1 << 5;
const ACCESS_REQUEST: u8 = 1 << 1;
const STS_VALID: u8 = 1 << 7;
const STS_READY: u8 = 1 << 6;
const STS_GO: u8 = 1 << 5;
const STS_DATA: u8 = 1 << 4;
const STS_EXPECT: u8 = 1 << 3;

// CRB registers and bits (PTP tables 20, 37 to 44).
const CRB_LOC_STATE: usize = 0x00;
const CRB_LOC_CTRL: usize = 0x08;
const CRB_CTRL_REQ: usize = 0x40;
const CRB_CTRL_STS: usize = 0x44;
const CRB_CTRL_START: usize = 0x4C;
const CRB_CMD_SIZE: usize = 0x58;
const CRB_CMD_LOW: usize = 0x5C;
const CRB_CMD_HIGH: usize = 0x60;
const CRB_RSP_SIZE: usize = 0x64;
const CRB_RSP_ADDR: usize = 0x68;
const LOC_STATE_VALID: u32 = 1 << 7;
const LOC_STATE_ASSIGNED: u32 = 1 << 1;
const LOC_STATE_ACTIVE_SHIFT: u32 = 2;
const LOC_CTRL_REQUEST: u32 = 1 << 0;
const LOC_CTRL_RELINQUISH: u32 = 1 << 1;
const CTRL_REQ_READY: u32 = 1 << 0;
const CTRL_REQ_IDLE: u32 = 1 << 1;
const CTRL_STS_ERROR: u32 = 1 << 0;

/// PTP's timeouts: A for a locality, B for a state change, C for a status
/// bit to settle, and D for a burst count, and a PCR read finishes well
/// inside B.
const TIMEOUT_A_MS: u64 = 750;
const TIMEOUT_B_MS: u64 = 2000;
const TIMEOUT_C_MS: u64 = 750;
const TIMEOUT_D_MS: u64 = 750;

/// A TPM at one locality.
pub struct Tpm {
    interface: Interface,
    locality: u8,
    tsc_per_ms: u64,
}

/// Bytes in one locality's register window.
const WINDOW: usize = 0x1000;

/// # Safety
///
/// `address` must be device memory that is safe to read a byte from: a
/// TPM register, or a CRB buffer the TPM names.
unsafe fn read8(address: usize) -> u8 {
    // Safety: the caller's contract.
    unsafe { ptr::read_volatile(address as *const u8) }
}

/// # Safety
///
/// As for [`read8`], for a write.
unsafe fn write8(address: usize, value: u8) {
    // Safety: the caller's contract.
    unsafe { ptr::write_volatile(address as *mut u8, value) }
}

/// # Safety
///
/// As for [`read8`], and `address` 4-byte aligned.
unsafe fn read32(address: usize) -> u32 {
    // Safety: the caller's contract.
    unsafe { ptr::read_volatile(address as *const u32) }
}

/// # Safety
///
/// As for [`read32`], for a write.
unsafe fn write32(address: usize, value: u32) {
    // Safety: the caller's contract.
    unsafe { ptr::write_volatile(address as *mut u32, value) }
}

impl Tpm {
    /// The TPM at `locality`, not yet requested. `tsc_per_ms` bounds every
    /// wait.
    ///
    /// # Safety
    ///
    /// The TPM's register windows, and any CRB buffer below 4 GiB it names,
    /// must be mapped as device memory for as long as the `Tpm` is used.
    ///
    /// # Panics
    ///
    /// On a locality above 4, which has no window.
    pub unsafe fn new(interface: Interface, locality: u8, tsc_per_ms: u64) -> Self {
        assert!(locality <= 4, "no TPM locality {locality}");
        Self {
            interface,
            locality,
            tsc_per_ms,
        }
    }

    pub fn locality(&self) -> u8 {
        self.locality
    }

    /// The address of the register at `offset`, which is always inside this
    /// locality's window: the chipset decodes the whole range at any time,
    /// and a TPM that is not there reads all ones rather than faulting.
    fn register(&self, offset: usize, width: usize) -> usize {
        assert!(offset + width <= WINDOW && offset.is_multiple_of(width));
        locality_base(self.locality) + offset
    }

    fn reg8(&self, offset: usize) -> u8 {
        // Safety: `register` keeps the address in the TPM window, which
        // `new`'s caller vouched is mapped.
        unsafe { read8(self.register(offset, 1)) }
    }

    fn set8(&self, offset: usize, value: u8) {
        // Safety: as for `reg8`.
        unsafe { write8(self.register(offset, 1), value) }
    }

    fn reg32(&self, offset: usize) -> u32 {
        // Safety: as for `reg8`, and `register` checks the alignment.
        unsafe { read32(self.register(offset, 4)) }
    }

    fn set32(&self, offset: usize, value: u32) {
        // Safety: as for `reg32`.
        unsafe { write32(self.register(offset, 4), value) }
    }

    fn wait(&self, ms: u64, mut done: impl FnMut() -> bool) -> bool {
        let deadline = Deadline::after(ms, self.tsc_per_ms);
        loop {
            if done() {
                return true;
            }
            if deadline.passed() {
                return done();
            }
            core::hint::spin_loop();
        }
    }

    /// What the locality's access register reads, which requests nothing.
    pub fn access(&self) -> u32 {
        match self.interface {
            Interface::Tis => u32::from(self.reg8(TIS_ACCESS)),
            Interface::Crb => self.reg32(CRB_LOC_STATE),
        }
    }

    /// Whether this locality is the active one.
    pub fn active(&self) -> bool {
        match self.interface {
            Interface::Tis => {
                let access = self.reg8(TIS_ACCESS);
                access != 0xFF && access & ACCESS_VALID != 0 && access & ACCESS_ACTIVE != 0
            }
            Interface::Crb => {
                let state = self.reg32(CRB_LOC_STATE);
                state != u32::MAX
                    && state & LOC_STATE_VALID != 0
                    && state & LOC_STATE_ASSIGNED != 0
                    && (state >> LOC_STATE_ACTIVE_SHIFT) & 0x7 == u32::from(self.locality)
            }
        }
    }

    /// Asks for this locality and waits for it to be granted.
    pub fn request(&self) -> Result<(), Error> {
        if self.active() {
            return Ok(());
        }
        match self.interface {
            Interface::Tis => self.set8(TIS_ACCESS, ACCESS_REQUEST),
            Interface::Crb => self.set32(CRB_LOC_CTRL, LOC_CTRL_REQUEST),
        }
        if self.wait(TIMEOUT_A_MS, || self.active()) {
            Ok(())
        } else {
            Err(Error::Locality)
        }
    }

    /// Gives this locality up. A write only, since the locality it is
    /// written to may not answer reads.
    pub fn relinquish(&self) {
        match self.interface {
            Interface::Tis => self.set8(TIS_ACCESS, ACCESS_ACTIVE),
            Interface::Crb => self.set32(CRB_LOC_CTRL, LOC_CTRL_RELINQUISH),
        }
    }

    /// Sends `command` and reads the answer into `response`, returning its
    /// length. The locality must be active.
    pub fn transmit(&self, command: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        match self.interface {
            Interface::Tis => self.transmit_tis(command, response),
            Interface::Crb => self.transmit_crb(command, response),
        }
    }

    fn sts(&self) -> u8 {
        self.reg8(TIS_STS)
    }

    fn burst(&self) -> usize {
        let low = self.reg8(TIS_BURST);
        let high = self.reg8(TIS_BURST + 1);
        usize::from(u16::from_le_bytes([low, high]))
    }

    fn transmit_tis(&self, command: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        self.set8(TIS_STS, STS_READY);
        if !self.wait(TIMEOUT_B_MS, || self.sts() & STS_READY != 0) {
            return Err(Error::NotReady);
        }

        let mut sent = 0;
        while sent < command.len() {
            let mut burst = 0;
            if !self.wait(TIMEOUT_B_MS, || {
                burst = self.burst();
                burst > 0
            }) {
                return Err(Error::Transfer);
            }
            for &byte in command[sent..].iter().take(burst) {
                self.set8(TIS_FIFO, byte);
                sent += 1;
            }
        }
        // The TPM clears Expect once it has the whole command.
        let settled = self.wait(TIMEOUT_C_MS, || self.sts() & STS_VALID != 0);
        if !settled || self.sts() & STS_EXPECT != 0 {
            return Err(Error::Transfer);
        }

        self.set8(TIS_STS, STS_GO);
        let answered = || self.sts() & (STS_VALID | STS_DATA) == STS_VALID | STS_DATA;
        if !self.wait(TIMEOUT_B_MS, answered) {
            self.set8(TIS_STS, STS_READY);
            return Err(Error::Timeout);
        }
        let received = self.receive_tis(response);
        // Back to idle, which drops anything the TPM still holds.
        self.set8(TIS_STS, STS_READY);
        received
    }

    /// Reads the response a burst at a time, waiting before each for the
    /// TPM to have data and a non-zero burst count, as a FIFO TPM refills
    /// between bursts.
    fn receive_tis(&self, response: &mut [u8]) -> Result<usize, Error> {
        let available = || self.sts() & (STS_VALID | STS_DATA) == STS_VALID | STS_DATA;
        let mut read = 0;
        let mut size = 10;
        while read < size {
            if !self.wait(TIMEOUT_C_MS, available) {
                return Err(Error::Response);
            }
            let mut burst = 0;
            if !self.wait(TIMEOUT_D_MS, || {
                burst = self.burst();
                burst > 0
            }) {
                return Err(Error::Transfer);
            }
            for _ in 0..burst.min(size - read) {
                *response.get_mut(read).ok_or(Error::Response)? = self.reg8(TIS_FIFO);
                read += 1;
            }
            if read >= 10 {
                size = crate::tpm::response_size(response).ok_or(Error::Response)?;
                if size < 10 || size > response.len() {
                    return Err(Error::Response);
                }
            }
        }
        // With the whole response read, the TPM has nothing left to give.
        if !self.wait(TIMEOUT_C_MS, || self.sts() & STS_VALID != 0) || self.sts() & STS_DATA != 0 {
            return Err(Error::Response);
        }
        Ok(size)
    }

    fn transmit_crb(&self, command: &[u8], response: &mut [u8]) -> Result<usize, Error> {
        self.set32(CRB_CTRL_REQ, CTRL_REQ_READY);
        if !self.wait(TIMEOUT_B_MS, || {
            self.reg32(CRB_CTRL_REQ) & CTRL_REQ_READY == 0
        }) {
            return Err(Error::NotReady);
        }

        let command_size = self.reg32(CRB_CMD_SIZE) as usize;
        let command_at =
            u64::from(self.reg32(CRB_CMD_LOW)) | (u64::from(self.reg32(CRB_CMD_HIGH)) << 32);
        let response_size = self.reg32(CRB_RSP_SIZE) as usize;
        let response_at =
            u64::from(self.reg32(CRB_RSP_ADDR)) | (u64::from(self.reg32(CRB_RSP_ADDR + 4)) << 32);
        let reachable =
            |at: u64, len: usize| at.checked_add(len as u64).is_some_and(|end| end <= 1 << 32);
        if !reachable(command_at, command_size) || !reachable(response_at, response_size) {
            return Err(Error::Buffer);
        }
        if command.len() > command_size {
            return Err(Error::Transfer);
        }
        for (i, &byte) in command.iter().enumerate() {
            // Safety: the TPM names this range as its command buffer, and it
            // was checked to lie below 4 GiB, which `new`'s caller maps.
            unsafe { write8(command_at as usize + i, byte) };
        }

        self.set32(CRB_CTRL_START, 1);
        if !self.wait(TIMEOUT_B_MS, || self.reg32(CRB_CTRL_START) & 1 == 0) {
            return Err(Error::Timeout);
        }
        if self.reg32(CRB_CTRL_STS) & CTRL_STS_ERROR != 0 {
            return Err(Error::Timeout);
        }
        let mut header = [0; 10];
        for (i, byte) in header.iter_mut().enumerate() {
            // Safety: as for the command buffer, for the response buffer,
            // and the header is within its checked size.
            *byte = unsafe { read8(response_at as usize + i) };
        }
        let size = crate::tpm::response_size(&header).ok_or(Error::Response)?;
        if size < 10 || size > response.len() || size > response_size {
            return Err(Error::Response);
        }
        for (i, byte) in response[..size].iter_mut().enumerate() {
            // Safety: as for the header, `size` checked against the buffer's.
            *byte = unsafe { read8(response_at as usize + i) };
        }
        self.set32(CRB_CTRL_REQ, CTRL_REQ_IDLE);
        Ok(size)
    }
}
