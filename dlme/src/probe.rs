// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! A device write into a range the TMR covers: the PIT's interrupt, which
//! the IO-APIC delivers as a write into the `0xFEE0_0000` window.
//!
//! IF is clear in the DLME, and GIF too after `SKINIT`, so an interrupt that
//! arrives stays pending in the local APIC's IRR and needs no handler. The probe
//! reads the IRR, through the xAPIC page or the x2APIC MSRs.

use core::arch::asm;
use core::ptr;

use common::clock::Deadline;
use common::params::Pit;

const IA32_APIC_BASE: u32 = 0x1B;
const APIC_ENABLE: u64 = 1 << 11;
const APIC_X2APIC: u64 = 1 << 10;
const APIC_BASE_MASK: u64 = 0x000F_FFFF_FFFF_F000;

const LAPIC_ID: u32 = 0x20;
const LAPIC_SVR: u32 = 0xF0;
const LAPIC_ISR: u32 = 0x100;
const LAPIC_IRR: u32 = 0x200;
const SVR_ENABLE: u32 = 1 << 8;

/// IO-APIC: the select register, the window 0x10 past it, and each pin's
/// redirection entry as two registers from 0x10.
const IOAPIC_WINDOW: usize = 0x10;
const IOAPIC_REDIRECTION: u32 = 0x10;
const RTE_ACTIVE_LOW: u32 = 1 << 13;
const RTE_LEVEL: u32 = 1 << 15;
const RTE_MASKED: u32 = 1 << 16;

/// The PIT's channel 0 as a rate generator at about 1 kHz: 1.193182 MHz
/// divided by this.
const PIT_COMMAND: u16 = 0x43;
const PIT_CHANNEL0: u16 = 0x40;
const PIT_RATE: u8 = 0x34;
const PIT_STOP: u8 = 0x30;
const PIT_DIVISOR: u16 = 1193;

/// How long each half of the probe watches the IRR.
pub const WATCH_MS: u64 = 20;

fn rdmsr(msr: u32) -> u64 {
    let (low, high): (u32, u32);
    // Safety: the MSRs read here are architectural where a local APIC is,
    // and `SKINIT` requires one.
    unsafe {
        asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high,
             options(nomem, nostack, preserves_flags));
    }
    u64::from(low) | (u64::from(high) << 32)
}

fn wrmsr(msr: u32, value: u64) {
    // Safety: only the x2APIC's SVR is written, in x2APIC mode.
    unsafe {
        asm!("wrmsr", in("ecx") msr, in("eax") value as u32, in("edx") (value >> 32) as u32,
             options(nostack, preserves_flags));
    }
}

fn outb(port: u16, value: u8) {
    // Safety: the PIT's ports, which nothing else drives while the DLME runs.
    unsafe { asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack)) };
}

/// The local APIC, in whichever mode firmware left it.
enum Lapic {
    X(usize),
    X2,
}

impl Lapic {
    fn read(&self, offset: u32) -> u32 {
        match self {
            // Safety: the xAPIC page lies below 4 GiB, checked when it was
            // found, which the DLME identity maps.
            Self::X(base) => unsafe { ptr::read_volatile((base + offset as usize) as *const u32) },
            Self::X2 => rdmsr(0x800 + (offset >> 4)) as u32,
        }
    }

    fn write(&self, offset: u32, value: u32) {
        match self {
            // Safety: as for `read`.
            Self::X(base) => unsafe {
                ptr::write_volatile((base + offset as usize) as *mut u32, value);
            },
            Self::X2 => wrmsr(0x800 + (offset >> 4), value.into()),
        }
    }

    fn id(&self) -> u32 {
        match self {
            Self::X(_) => self.read(LAPIC_ID) >> 24,
            Self::X2 => self.read(LAPIC_ID),
        }
    }

    fn bit(&self, base: u32, vector: u8) -> bool {
        let vector = u32::from(vector);
        self.read(base + (vector / 32) * 0x10) & (1 << (vector % 32)) != 0
    }
}

/// The probe, armed but for the PIT.
pub struct Probe {
    lapic: Lapic,
    ioapic: usize,
    pin: u8,
    pub vector: u8,
    pub apic_id: u32,
    /// Whether the local APIC was software-disabled and had to be enabled.
    pub enabled_apic: bool,
    tsc_per_ms: u64,
}

impl Probe {
    fn ioapic_write(&self, register: u32, value: u32) {
        // Safety: the MADT names the IO-APIC here, below 4 GiB, which the
        // parameter page checked and the DLME identity maps.
        unsafe {
            ptr::write_volatile(self.ioapic as *mut u32, register);
            ptr::write_volatile((self.ioapic + IOAPIC_WINDOW) as *mut u32, value);
        }
    }

    /// Points the PIT's pin at a free vector on this processor and starts
    /// the PIT.
    pub fn arm(pit: &Pit, tsc_per_ms: u64) -> Result<Self, &'static str> {
        let base = rdmsr(IA32_APIC_BASE);
        if base & APIC_ENABLE == 0 {
            return Err("the local APIC is disabled in IA32_APIC_BASE");
        }
        let lapic = if base & APIC_X2APIC != 0 {
            Lapic::X2
        } else {
            let address = base & APIC_BASE_MASK;
            if address >= 1 << 32 {
                return Err("the local APIC's page is above 4 GiB");
            }
            Lapic::X(address as usize)
        };
        let apic_id = lapic.id();
        if apic_id > 0xFF {
            return Err("this processor's APIC ID does not fit an IO-APIC destination");
        }
        let svr = lapic.read(LAPIC_SVR);
        let enabled_apic = svr & SVR_ENABLE == 0;
        if enabled_apic {
            lapic.write(LAPIC_SVR, svr | SVR_ENABLE);
        }
        let vector = (0x40..0xF0)
            .find(|&v| !lapic.bit(LAPIC_IRR, v) && !lapic.bit(LAPIC_ISR, v))
            .ok_or("no free vector")?;

        let probe = Self {
            lapic,
            ioapic: pit.ioapic as usize,
            pin: pit.pin,
            vector,
            apic_id,
            enabled_apic,
            tsc_per_ms,
        };
        let entry = IOAPIC_REDIRECTION + 2 * u32::from(pit.pin);
        let low = u32::from(vector)
            | if pit.active_low { RTE_ACTIVE_LOW } else { 0 }
            | if pit.level { RTE_LEVEL } else { 0 };
        probe.ioapic_write(entry + 1, apic_id << 24);
        probe.ioapic_write(entry, low);
        outb(PIT_COMMAND, PIT_RATE);
        let [low, high] = PIT_DIVISOR.to_le_bytes();
        outb(PIT_CHANNEL0, low);
        outb(PIT_CHANNEL0, high);
        Ok(probe)
    }

    /// Whether the vector turns up in the IRR within [`WATCH_MS`].
    pub fn watch(&self) -> bool {
        let deadline = Deadline::after(WATCH_MS, self.tsc_per_ms);
        while !deadline.passed() {
            if self.lapic.bit(LAPIC_IRR, self.vector) {
                return true;
            }
            core::hint::spin_loop();
        }
        self.lapic.bit(LAPIC_IRR, self.vector)
    }

    /// Masks the pin and stops the PIT.
    pub fn disarm(self) {
        self.ioapic_write(IOAPIC_REDIRECTION + 2 * u32::from(self.pin), RTE_MASKED);
        outb(PIT_COMMAND, PIT_STOP);
    }
}
