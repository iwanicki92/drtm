// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! Everything from a clear gate to `SKINIT`.
//!
//! What can fail with a readable error happens before
//! `ExitBootServices()`. After it there is no allocation and no UEFI
//! logger, and progress goes out as `launch.preamble` records through the
//! sink list at debug level.

use alloc::format;

use core::arch::asm;
use core::arch::x86_64::__cpuid;
use core::ptr;
use core::time::Duration;

use common::clock;
use common::out::Out;
use common::params::{self, Family, Params, Path, Psp};
use common::record::{Tag, Value};
use common::sha256;
use common::tpm::Bank;
use common::transport::{Interface, Tpm};
use log::LevelFilter;
use uefi::boot::{self, AllocateType, MemoryType, OpenProtocolAttributes, OpenProtocolParams};
use uefi::mem::memory_map::MemoryAttribute;
use uefi::proto::security::MemoryProtection;

use super::gate::Sinks;
use super::psp::{self, Service};
use super::{image, pit, trampoline};
use crate::amd::aspt;
use crate::config::Config;
use crate::tpm::{self, Probe};
use crate::{acpi, check};

const SLB_SIZE: usize = 0x10000;
const PAGE: usize = 0x1000;
const BELOW_4GIB: u64 = u32::MAX as u64;

// The SL reserves its parameter page from the same `params::SIZE`, which
// build.rs read back from its ELF.
const _: () = assert!(image::PARAMS_SIZE == params::SIZE);

/// Pages holding a 64 KiB block on a 2 MiB boundary wherever the firmware
/// puts them. The SL maps its SLB with one 2 MiB page from that boundary.
const SLB_ALIGN: usize = 2 << 20;
const SLB_PAGES: usize = (SLB_ALIGN + SLB_SIZE - PAGE) / PAGE;

/// The alignment the guide's appendix E.1 asks of the MLE the PSP path
/// extends, which starts at the SLB.
const MLE_ALIGN: u64 = 64 << 20;

/// `IA32_APIC_BASE`: the xAPIC's MMIO base in bits 51:12, x2APIC mode in
/// bit 10. The ICR is at offset 0x300 in xAPIC mode and `MSR 0x830` in
/// x2APIC mode.
const IA32_APIC_BASE: u32 = 0x1B;
const APIC_X2APIC: u64 = 1 << 10;
const APIC_BASE_MASK: u64 = 0x000F_FFFF_FFFF_F000;
const XAPIC_ICR_LOW: u64 = 0x300;
const X2APIC_ICR: u32 = 0x830;
const ICR_DELIVERY_PENDING: u32 = 1 << 12;

/// INIT, asserted, to all processors but this one: the destination
/// shorthand in bits 19:18, level in 14 and delivery mode in 10:8.
const ICR_INIT_ALL_BUT_SELF: u32 = (0b11 << 18) | (1 << 14) | (0b101 << 8);

/// How long the APs get to reach INIT state. `SKINIT` may need up to 1000
/// cycles to see it (APM Vol 2, section 15.27.8), so this is ample.
const PARK_WAIT_MS: u64 = 10;

/// `CR0.CD`, `CR0.NW` and `CR0.NE`, set as `grub_skinit_prepare_cpu()` sets
/// them: caches on and native FPU error reporting.
const CR0_CD: u64 = 1 << 30;
const CR0_NW: u64 = 1 << 29;
const CR0_NE: u64 = 1 << 5;

/// Ticks of the time-stamp counter per millisecond, against UEFI's timer.
pub fn calibrate_tsc() -> u64 {
    let start = clock::tsc();
    boot::stall(Duration::from_millis(10));
    ((clock::tsc() - start) / 10).max(1)
}

/// Allocates `pages` below 4 GiB.
fn allocate(memory: MemoryType, pages: usize) -> Option<*mut u8> {
    boot::allocate_pages(AllocateType::MaxAddress(BELOW_4GIB), memory, pages)
        .ok()
        .map(|block| block.as_ptr())
}

/// A 64 KiB block on a 64 MiB boundary below 4 GiB, the first one free.
fn allocate_aligned() -> Option<*mut u8> {
    (1..(1u64 << 32) / MLE_ALIGN).find_map(|slot| {
        boot::allocate_pages(
            AllocateType::Address(slot * MLE_ALIGN),
            MemoryType::LOADER_DATA,
            SLB_SIZE / PAGE,
        )
        .ok()
        .map(|block| block.as_ptr())
    })
}

/// Makes `page` executable and writable where the firmware protects its
/// allocations through `EFI_MEMORY_ATTRIBUTE_PROTOCOL`. Firmware without
/// the protocol leaves new loader code executable, so there is nothing to
/// do.
fn make_executable(page: *mut u8) -> uefi::Result {
    let Ok(handle) = boot::get_handle_for_protocol::<MemoryProtection>() else {
        return Ok(());
    };
    // Safety: the handle came from `get_handle_for_protocol` for this
    // protocol, and `GetProtocol` leaves its driver alone.
    let protection = unsafe {
        boot::open_protocol::<MemoryProtection>(
            OpenProtocolParams {
                handle,
                agent: boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    }?;
    let start = page as u64;
    protection.clear_memory_attributes(
        start..start + PAGE as u64,
        MemoryAttribute::EXECUTE_PROTECT | MemoryAttribute::READ_ONLY,
    )
}

fn read_msr(msr: u32) -> u64 {
    // Safety: every MSR read here is architectural on a CPU the gate let
    // through: IA32_APIC_BASE with a local APIC.
    unsafe { crate::msr::read(msr) }
}

/// Sends every other processor INIT and waits for it to take.
///
/// # Safety
///
/// Boot services must be gone: the APs run firmware code until now.
unsafe fn park_aps(tsc_per_ms: u64) {
    if __cpuid(1).edx & (1 << 9) == 0 {
        return;
    }
    let apic = read_msr(IA32_APIC_BASE);
    if apic & APIC_X2APIC != 0 {
        // Safety: x2APIC mode is on, so the ICR is this MSR, and the
        // shorthand needs no destination in the high half.
        unsafe {
            asm!("wrmsr", in("ecx") X2APIC_ICR, in("eax") ICR_INIT_ALL_BUT_SELF, in("edx") 0,
                 options(nostack, preserves_flags));
        }
    } else {
        let icr = ((apic & APIC_BASE_MASK) + XAPIC_ICR_LOW) as *mut u32;
        // Safety: the local APIC's registers are at this base, identity
        // mapped by the firmware, and the ICR low word is a 32-bit register.
        unsafe { icr.write_volatile(ICR_INIT_ALL_BUT_SELF) };
        let deadline = clock::Deadline::after(PARK_WAIT_MS, tsc_per_ms);
        // Safety: as for the write.
        while unsafe { icr.read_volatile() } & ICR_DELIVERY_PENDING != 0 && !deadline.passed() {
            core::hint::spin_loop();
        }
    }
    clock::delay(PARK_WAIT_MS, tsc_per_ms);
}

/// Caches on and native FPU errors, as `SKINIT` wants the core, and
/// interrupts off from here on.
fn prepare_cpu() {
    // Safety: clearing IF and the cache-disable bits and setting NE changes
    // nothing a running UEFI-less program relies on.
    unsafe {
        asm!("cli", options(nomem, nostack));
        let mut cr0: u64;
        asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
        cr0 = (cr0 & !(CR0_CD | CR0_NW)) | CR0_NE;
        asm!("mov cr0, {}", in(reg) cr0, options(nostack, preserves_flags));
    }
}

/// Prepares and issues `SKINIT`. Returns only where something before
/// `ExitBootServices()` failed, which it reports.
pub fn launch(config: &Config, sinks: Sinks, service: Option<Service>, tsc_per_ms: u64) {
    let fail = |detail: &str| {
        check::after_summary(
            Tag::Fail,
            "launch.preamble",
            &[],
            format_args!("{detail}, not launching"),
        );
    };
    let Probe::Present { family, interface } = tpm::probe() else {
        fail("no TPM answers any more");
        return;
    };
    // The PSP path's extend starts at the SLB, so it goes on a 64 MiB
    // boundary where one is free.
    let aligned = service.as_ref().and_then(|_| allocate_aligned());
    let (Some(block), Some(page)) = (
        aligned.or_else(|| allocate(MemoryType::LOADER_DATA, SLB_PAGES)),
        allocate(MemoryType::LOADER_CODE, 1),
    ) else {
        fail("no memory below 4 GiB for the SLB and the trampoline");
        return;
    };
    if let Err(error) = make_executable(page) {
        fail(&format!(
            "the trampoline page could not be made executable ({:?})",
            error.status()
        ));
        return;
    }
    let slb = (block as usize).next_multiple_of(SLB_ALIGN) as *mut u8;

    let params = Params {
        format: config.format,
        level: config.level as usize as u8,
        family: match family {
            tpm::Family::V1_2 => Family::V1_2,
            tpm::Family::V2_0 => Family::V2_0,
        },
        interface: match interface {
            tpm::Interface::Tis => Interface::Tis,
            tpm::Interface::Crb => Interface::Crb,
        },
        // SHA-1 and SHA-256 where the firmware does not say, the two banks
        // a launch is judged in.
        banks: tpm::active_banks().unwrap_or(Bank::Sha1.bit() | Bank::Sha256.bit()),
        psp_drtm: aspt::drtm_state(),
        tsc_per_ms,
        sinks: sinks.uarts,
        expected_sha1: image::EXPECTED_SHA1,
        expected_sha256: image::EXPECTED_SHA256,
        path: if service.is_some() {
            Path::Psp
        } else {
            Path::Classic
        },
        psp: service.as_ref().map(|service| Psp {
            mailbox: service.mailbox.address() as u64,
            source: service.source,
            capability: service.capability,
            tmr_base: psp::TMR_BASE,
            tmr_size: psp::TMR_SIZE,
            pcr17: image::PSP_PCR17,
            pcr18: image::PSP_PCR18,
        }),
        pit: service.as_ref().and_then(|_| pit::find()),
        ivrs_sha256: match acpi::find(b"IVRS") {
            Ok(Some(ivrs)) if service.is_some() => Some(sha256::digest(ivrs)),
            _ => None,
        },
    };
    // Safety: `slb` is 64 KiB inside the block just allocated, and `page`
    // a page of loader code below 4 GiB, neither used by anything else.
    unsafe {
        ptr::write_bytes(slb, 0, SLB_SIZE);
        ptr::copy_nonoverlapping(image::BYTES.as_ptr(), slb, image::BYTES.len());
        let page_bytes = params.encode();
        ptr::copy_nonoverlapping(
            page_bytes.as_ptr(),
            slb.add(image::PARAMS_OFFSET),
            params::SIZE,
        );
        trampoline::install(page);
    }

    check::after_summary(
        Tag::Info,
        "launch.skinit",
        &[
            ("slb", Value::Hex(slb as u64)),
            ("measured", Value::Count(image::MEASURED as u64)),
            ("expected_sha1", Value::Bytes(&image::EXPECTED_SHA1)),
            ("expected_sha256", Value::Bytes(&image::EXPECTED_SHA256)),
        ],
        format_args!(
            "SLB at {:#x}, {} bytes measured, exiting boot services",
            slb as u64,
            image::MEASURED
        ),
    );
    if let Some(psp) = &params.psp {
        let pit = params.pit;
        check::after_summary(
            Tag::Info,
            "launch.psp_service",
            &[
                ("mailbox", Value::Hex(psp.mailbox)),
                ("source", Value::Text(format_args!("{}", psp.source.name()))),
                (
                    "capability",
                    Value::Text(format_args!(
                        "{:#x} {:#x} {:#x}",
                        psp.capability[0], psp.capability[1], psp.capability[2]
                    )),
                ),
                ("tmr_base", Value::Hex(psp.tmr_base)),
                ("tmr_size", Value::Hex(psp.tmr_size)),
                (
                    "slb_aligned",
                    Value::Flag((slb as u64).is_multiple_of(MLE_ALIGN)),
                ),
                ("ioapic", Value::Hex(pit.map_or(0, |pit| pit.ioapic.into()))),
                ("pit_pin", Value::Count(pit.map_or(0, |pit| pit.pin.into()))),
                ("ivrs", Value::Flag(params.ivrs_sha256.is_some())),
                ("expected_pcr17", Value::Bytes(&psp.pcr17)),
                ("expected_pcr18", Value::Bytes(&psp.pcr18)),
            ],
            format_args!(
                "the service at {:#x} from the {} is kicked, TMR 0 covers \
                 [{:#x}, {:#x}), the SLB is {}on a 64 MiB boundary",
                psp.mailbox,
                psp.source.name(),
                psp.tmr_base,
                psp.tmr_base + psp.tmr_size,
                if (slb as u64).is_multiple_of(MLE_ALIGN) {
                    ""
                } else {
                    "not "
                }
            ),
        );
    }

    // Nothing may reach the UEFI logger from here, so nothing is let
    // through to it.
    log::set_max_level(LevelFilter::Off);
    // Safety: nothing borrowed from boot services is used past this call.
    let map = unsafe { boot::exit_boot_services(None) };
    // Its drop would log, and it has nowhere to be freed to anyway.
    core::mem::forget(map);

    let mut out = Out::new(params.sinks, params.format, params.level);
    let step = |out: &mut Out, detail: &str| {
        out.progress("launch.preamble", &[], format_args!("{detail}"));
    };
    step(&mut out, "boot services exited");

    // The firmware's TCG2 driver measured the exit into PCR 5 and may
    // still hold locality 0. The launch starts with none active.
    // Safety: the firmware's page tables, still in use, identity map the
    // TPM windows, and a relinquish touches no CRB buffer.
    unsafe { Tpm::new(params.interface, 0, tsc_per_ms) }.relinquish();
    step(&mut out, "locality 0 relinquished");

    prepare_cpu();
    step(&mut out, "caches on, interrupts off");

    // Safety: boot services are gone.
    unsafe { park_aps(tsc_per_ms) };
    step(&mut out, "INIT sent to every other processor");

    step(&mut out, "entering the trampoline, SKINIT follows");
    // Safety: the trampoline is installed, the SLB written, boot services
    // gone, the APs parked and interrupts off.
    unsafe { trampoline::enter(page, slb as u32, params.sinks[0]) }
}
