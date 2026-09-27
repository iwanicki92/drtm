// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! drtm's Secure Loader: the code `SKINIT` measures and starts.
//!
//! It reads the DRTM PCRs, judges whether the TPM measured this image and
//! reports on the sinks the parameter page lists. Then it measures drtm's
//! DLME and jumps to it, which judges the rest and halts. The SL wakes no
//! processor.
//!
//! It is built once per launch path, the `classic` or `psp` feature, and
//! the preamble writes the image of the path it takes into the SLB.

#![no_std]
#![no_main]

#[cfg(not(any(
    all(feature = "classic", not(feature = "psp")),
    all(feature = "psp", not(feature = "classic"))
)))]
compile_error!("build the SL with exactly one of the classic and psp features");

mod entry;
mod handoff;
#[cfg(feature = "psp")]
mod psp;

use core::fmt::Write;
use core::panic::PanicInfo;

use common::params::{self, Params};
use common::pcr::read;
use common::record::Tag;
use common::transport::Tpm;
use common::uart;
#[cfg(feature = "classic")]
use {
    common::params::{Family, Handoff, PspDrtm, Sender},
    common::pcr,
    common::record::Value,
    common::tpm::Bank,
    common::verdict::{self, Pcr17},
    handoff::Measured,
};

use common::out::Out;

// The parameter page, sized from the one definition the preamble writes
// it by. `sl.ld` places it after the image and before `.bss`.
core::arch::global_asm!(
    ".section .sl.params, \"aw\", @nobits",
    ".global __params",
    "__params:",
    ".skip {size}",
    ".previous",
    size = const params::SIZE,
);

unsafe extern "C" {
    static __params: [u8; params::SIZE];
    static __stack_bottom: u8;
    static __stack_top: u8;
}

/// How much of the stack, which the entry zeroed, the run wrote to.
fn stack_used() -> usize {
    let (bottom, top) = (
        &raw const __stack_bottom as usize,
        &raw const __stack_top as usize,
    );
    // Safety: the stack lies in the SLB, which the page tables map, and the
    // bytes below the one in use are only read.
    let untouched = (bottom..top)
        .take_while(|&at| unsafe { core::ptr::read_volatile(at as *const u8) } == 0)
        .count();
    top - bottom - untouched
}

/// The locality a Secure Loader extends PCR 17 and 18 from.
pub(crate) const SL_LOCALITY: u8 = 2;

fn params() -> Result<Params, params::Invalid> {
    // Safety: the page lies inside the SLB, which the page tables map, and
    // nothing writes it while the SL runs.
    Params::decode(unsafe { &__params })
}

fn halt() -> ! {
    loop {
        // Safety: stopping the processor with interrupts off is the end of
        // the run, and GIF is still clear from SKINIT.
        unsafe { core::arch::asm!("cli", "hlt", options(nomem, nostack)) };
    }
}

#[unsafe(no_mangle)]
extern "C" fn sl_main(slb: u32) -> ! {
    let params = match params() {
        Ok(params) => params,
        Err(why) => {
            let mut com1 = uart::COM1;
            let _ = write!(
                com1,
                "drtm: [FAIL] launch.params: the parameter page at SLB \
                 {slb:#x} is not valid ({why}), halting\r\n"
            );
            halt();
        }
    };
    let mut out = Out::new(params.sinks, params.format, params.level);
    out.debug(format_args!("SL running, SLB at {slb:#x}"));

    #[cfg(feature = "classic")]
    let next = Some(classic(&mut out, &params));
    #[cfg(feature = "psp")]
    let next = match &params.psp {
        Some(psp) => Some(psp::run(&mut out, &params, psp, slb)),
        None => {
            out.unchecked(
                Tag::Fail,
                "launch.params",
                &[],
                format_args!("the parameter page names no PSP service for this PSP SL"),
            );
            None
        }
    };

    out.debug(format_args!("stack used: {} bytes", stack_used()));
    if let Some((handoff, measured)) = next {
        handoff::jump(&mut out, &params, handoff, &measured, slb);
    }
    out.summary();
    out.unchecked(
        Tag::Info,
        "launch.halted",
        &[],
        format_args!("the machine is halted, power-cycle it before the next attempt"),
    );
    halt();
}

/// The classic path: the PCRs `SKINIT` left, judged against the build,
/// then the DLME measured into PCR 17.
#[cfg(feature = "classic")]
fn classic(out: &mut Out, params: &Params) -> (Handoff, Measured) {
    let (tpm, locality_granted) = take_locality(out, params);
    match params.family {
        Family::V2_0 => {
            for bank in Bank::ALL {
                if params.banks & bank.bit() != 0 {
                    report_bank(out, params, &tpm, bank);
                }
            }
        }
        Family::V1_2 => report_bank(out, params, &tpm, Bank::Sha1),
    }
    let measured = handoff::measure(params);
    let extended_banks = extend_dlme(out, params, &tpm, &measured);
    if locality_granted {
        tpm.relinquish();
    }
    let handoff = Handoff {
        from: Sender::Classic,
        extended_banks,
        ..Handoff::FOREIGN
    };
    (handoff, measured)
}

/// Extends PCR 17 with the DLME in the SHA-1 and SHA-256 banks, the ones
/// the SL hashes, where they are active. Returns the banks extended. Kept
/// out of `classic`, whose frame would otherwise hold its buffers while
/// the banks are read.
#[cfg(feature = "classic")]
#[inline(never)]
fn extend_dlme(out: &mut Out, params: &Params, tpm: &Tpm, measured: &Measured) -> u32 {
    if !measured.matches {
        // The handoff reports it, and does not jump.
        return 0;
    }
    if tpm.locality() != SL_LOCALITY {
        out.check(
            Tag::Skip,
            "launch.dlme_extend",
            &[],
            format_args!(
                "the DLME is not extended into PCR 17, locality {SL_LOCALITY} was not granted"
            ),
        );
        return 0;
    }
    let active = match params.family {
        Family::V2_0 => params.banks,
        Family::V1_2 => Bank::Sha1.bit(),
    };
    let both: [(Bank, &[u8]); 2] = [
        (Bank::Sha1, &measured.sha1),
        (Bank::Sha256, &measured.sha256),
    ];
    let mut digests = both;
    let mut count = 0;
    for pair in both {
        if active & pair.0.bit() != 0 {
            digests[count] = pair;
            count += 1;
        }
    }
    let banks = digests[..count]
        .iter()
        .fold(0, |bits, (bank, _)| bits | bank.bit());
    let unhashed = active & !(Bank::Sha1.bit() | Bank::Sha256.bit());
    let fields = [
        ("banks", Value::Hex(banks.into())),
        ("unhashed", Value::Hex(unhashed.into())),
        ("sha1", Value::Bytes(&measured.sha1)),
        ("sha256", Value::Bytes(&measured.sha256)),
    ];
    let rest = if unhashed == 0 {
        ""
    } else {
        ", the other active banks are not extended, since the SL hashes SHA-1 and SHA-256 alone"
    };
    if count == 0 {
        out.check(
            Tag::Warn,
            "launch.dlme_extend",
            &fields,
            format_args!(
                "PCR 17 is not extended with the DLME, no SHA-1 or SHA-256 bank is active \
                 and the SL hashes those alone"
            ),
        );
        return 0;
    }
    match pcr::extend17(tpm, params.family, &digests[..count]) {
        Ok(()) => {
            out.check(
                Tag::Pass,
                "launch.dlme_extend",
                &fields,
                format_args!("PCR 17 extended with the DLME's digest{rest}"),
            );
            banks
        }
        Err(failure) => {
            out.check(
                Tag::Fail,
                "launch.dlme_extend",
                &fields,
                format_args!("PCR 17 was not extended with the DLME ({failure})"),
            );
            0
        }
    }
}

/// The TPM at `locality`.
pub(crate) fn tpm(params: &Params, locality: u8) -> Tpm {
    // Safety: the SL's page tables identity map the low 4 GiB, which holds
    // the TPM windows and any CRB buffer the transport will touch.
    unsafe { Tpm::new(params.interface, locality, params.tsc_per_ms) }
}

/// Asks for locality 2 and reports whether it came. Without it the PCRs
/// are read at locality 0, which every TPM allows.
#[cfg(feature = "classic")]
fn take_locality(out: &mut Out, params: &Params) -> (Tpm, bool) {
    let access: [u32; 5] = core::array::from_fn(|locality| tpm(params, locality as u8).access());
    out.debug(format_args!(
        "TPM access by locality: {:#x} {:#x} {:#x} {:#x} {:#x}",
        access[0], access[1], access[2], access[3], access[4]
    ));

    let wanted = tpm(params, SL_LOCALITY);
    if wanted.request().is_ok() {
        out.check(
            Tag::Pass,
            "launch.locality",
            &[("locality", Value::Count(SL_LOCALITY.into()))],
            format_args!("locality {SL_LOCALITY} granted"),
        );
        return (wanted, true);
    }
    let psp = if params.psp_drtm == PspDrtm::Ready {
        ", and the PSP DRTM service is on, which keeps it locked until a \
         PSP launch"
    } else {
        ""
    };
    out.check(
        Tag::Warn,
        "launch.locality",
        &[
            ("locality", Value::Count(SL_LOCALITY.into())),
            (
                "access",
                Value::Hex(access[usize::from(SL_LOCALITY)].into()),
            ),
        ],
        format_args!(
            "locality {SL_LOCALITY} not granted, so a Secure Loader could \
             not extend PCRs 17 to 22{psp}"
        ),
    );
    let fallback = tpm(params, 0);
    let granted = fallback.request().is_ok();
    (fallback, granted)
}

/// Reads one bank and judges it.
#[cfg(feature = "classic")]
fn report_bank(out: &mut Out, params: &Params, tpm: &Tpm, bank: Bank) {
    let name = bank.name();
    let locality = Value::Count(tpm.locality().into());
    let pcrs = match read(tpm, params.family, bank) {
        Ok(pcrs) => pcrs,
        Err(failure) => {
            out.check(
                Tag::Fail,
                "launch.pcr_read",
                &[
                    ("bank", Value::Text(format_args!("{name}"))),
                    ("locality", locality),
                ],
                format_args!(
                    "the {name} bank did not read at locality {} ({failure}), \
                     so the launch cannot be verified in it",
                    tpm.locality()
                ),
            );
            return;
        }
    };

    let expected = params.expected(bank);
    let value = pcrs[0].as_bytes();
    let fields = [
        ("bank", Value::Text(format_args!("{name}"))),
        ("locality", Value::Count(tpm.locality().into())),
        ("pcr17", Value::Bytes(value)),
        ("expected", Value::Bytes(expected.unwrap_or(&[]))),
    ];
    let at = tpm.locality();
    match verdict::pcr17(&pcrs, expected) {
        Pcr17::Expected => out.check(
            Tag::Pass,
            "launch.pcr17",
            &fields,
            format_args!("{name} PCR 17 holds this image's measurement (read at locality {at})"),
        ),
        Pcr17::Reset => out.check(
            Tag::Fail,
            "launch.pcr17",
            &fields,
            format_args!(
                "{name} PCR 17 is all ones, no hash transaction reached the TPM \
                 (read at locality {at})"
            ),
        ),
        Pcr17::Other => out.check(
            Tag::Fail,
            "launch.pcr17",
            &fields,
            format_args!("{name} PCR 17 is not this image's measurement (read at locality {at})"),
        ),
        Pcr17::Unjudged => out.check(
            Tag::Info,
            "launch.pcr17",
            &fields[..3],
            format_args!(
                "{name} PCR 17 read at locality {at}, and the build computes no \
                 {name} value to judge it against"
            ),
        ),
    }

    let rest = [
        ("bank", Value::Text(format_args!("{name}"))),
        ("pcr18", Value::Bytes(pcrs[1].as_bytes())),
        ("pcr19", Value::Bytes(pcrs[2].as_bytes())),
        ("pcr20", Value::Bytes(pcrs[3].as_bytes())),
        ("pcr21", Value::Bytes(pcrs[4].as_bytes())),
        ("pcr22", Value::Bytes(pcrs[5].as_bytes())),
    ];
    if verdict::pcr18_22_reset(&pcrs) {
        out.check(
            Tag::Pass,
            "launch.pcr18_22",
            &rest,
            format_args!("{name} PCR 18 to 22 are zero, as the launch left them"),
        );
    } else {
        out.check(
            Tag::Fail,
            "launch.pcr18_22",
            &rest,
            format_args!("{name} PCR 18 to 22 are not all zero"),
        );
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match params() {
        Ok(params) => {
            let mut out = Out::new(params.sinks, params.format, params.level);
            out.unchecked(
                Tag::Fail,
                "launch.panic",
                &[],
                format_args!("the Secure Loader panicked: {}", info.message()),
            );
        }
        Err(_) => {
            let mut com1 = uart::COM1;
            let _ = write!(com1, "drtm: the Secure Loader panicked\r\n");
        }
    }
    halt();
}
