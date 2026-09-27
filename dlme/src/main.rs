// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! drtm's DLME: what every launch path ends in, drtm's classic and PSP SLs
//! and a foreign SL alike.
//!
//! It judges the PCRs the launch and its own measurement left, releases the
//! TMR where the PSP set one up, prints the summary of the whole launch,
//! the SL's results included, and halts.

#![no_std]
#![no_main]

mod amd;
mod entry;
mod probe;

use core::fmt::Write;
use core::panic::PanicInfo;

use common::out::Out;
use common::params::{self, Handoff, Params, Sender};
use common::pcr;
use common::psp::{self, Command, Mailbox};
use common::record::{Tag, Value};
use common::sha256;
use common::tpm::{Bank, DrtmPcrs};
use common::transport::Tpm;
use common::uart;

use probe::Probe;

// The parameter page, sized from the one definition the preamble writes
// it by. `dlme.ld` places it right after the image.
core::arch::global_asm!(
    ".section .dlme.params, \"aw\", @nobits",
    ".global __params",
    "__params:",
    ".skip {size}",
    ".previous",
    size = const params::SIZE,
);

unsafe extern "C" {
    static __params: [u8; params::SIZE];
}

fn page() -> &'static [u8; params::SIZE] {
    // Safety: the page lies in the DLME's block, which the page tables map,
    // and nothing writes it while the DLME runs.
    unsafe { &__params }
}

fn halt() -> ! {
    loop {
        // Safety: stopping the processor with interrupts off is the end of
        // the run.
        unsafe { core::arch::asm!("cli", "hlt", options(nomem, nostack)) };
    }
}

#[unsafe(no_mangle)]
extern "C" fn dlme_main(block: u32) -> ! {
    let (params, handoff) = match Params::decode(page())
        .and_then(|params| Handoff::read(page()).map(|handoff| (params, handoff)))
    {
        Ok(both) => both,
        Err(why) => {
            let mut com1 = uart::COM1;
            let _ = write!(
                com1,
                "drtm: [FAIL] launch.params: the parameter page of the DLME at \
                 {block:#x} is not valid ({why}), halting\r\n"
            );
            halt();
        }
    };
    let mut out = Out::resume(params.sinks, params.format, params.level, handoff.counts);
    let by = match handoff.from {
        Sender::Foreign => "a foreign SL",
        Sender::Classic => "drtm's classic SL",
        Sender::Psp => "drtm's PSP SL",
    };
    out.unchecked(
        Tag::Info,
        "launch.dlme",
        &[
            ("block", Value::Hex(block.into())),
            ("from", Value::Text(format_args!("{by}"))),
        ],
        format_args!("the DLME at {block:#x} runs, handed off by {by}"),
    );

    match handoff.from {
        Sender::Classic => classic(&mut out, &params, &handoff),
        Sender::Psp | Sender::Foreign => {
            let pcrs = match params.psp {
                Some(psp) => judge_psp(&mut out, &params, &psp, &handoff),
                None => {
                    out.check(
                        Tag::Fail,
                        "launch.params",
                        &[],
                        format_args!("the parameter page names no PSP service for a PSP launch"),
                    );
                    None
                }
            };
            if handoff.from == Sender::Foreign {
                amd::report(&mut out, &params, block.into(), pcrs.as_ref());
            }
            release(&mut out, &params, &handoff);
        }
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

/// The TPM at locality 0, which every TPM allows, and whether it came.
fn tpm(params: &Params) -> (Tpm, bool) {
    // Safety: the page tables identity map the low 4 GiB, which holds the
    // TPM windows and any CRB buffer the transport will touch.
    let tpm = unsafe { Tpm::new(params.interface, 0, params.tsc_per_ms) };
    let granted = tpm.request().is_ok();
    (tpm, granted)
}

/// After drtm's classic SL: PCR 17 holds the `SKINIT` measurement, then
/// the DLME's in each bank the SL extended.
fn classic(out: &mut Out, params: &Params, handoff: &Handoff) {
    let (tpm, granted) = tpm(params);
    let banks = match params.family {
        params::Family::V2_0 => &Bank::ALL[..],
        params::Family::V1_2 => &[Bank::Sha1][..],
    };
    for &bank in banks
        .iter()
        .filter(|bank| params.banks & bank.bit() != 0 || params.family == params::Family::V1_2)
    {
        let name = bank.name();
        let pcrs = match pcr::read(&tpm, params.family, bank) {
            Ok(pcrs) => pcrs,
            Err(failure) => {
                out.check(
                    Tag::Fail,
                    "launch.dlme_pcr17",
                    &[("bank", Value::Text(format_args!("{name}")))],
                    format_args!("the {name} bank did not read ({failure})"),
                );
                continue;
            }
        };
        let extended = handoff.extended_banks & bank.bit() != 0;
        let expected = match (bank, extended) {
            (Bank::Sha1, true) => Some(&params.dlme.pcr17_sha1[..]),
            (Bank::Sha256, true) => Some(&params.dlme.pcr17_sha256[..]),
            (_, false) => params.expected(bank),
            _ => None,
        };
        let value = pcrs[0].as_bytes();
        let what = if extended {
            "the SL's measurement, then the DLME's"
        } else {
            "the SL's measurement alone, since the SL did not extend the DLME into it"
        };
        let fields = [
            ("bank", Value::Text(format_args!("{name}"))),
            ("pcr17", Value::Bytes(value)),
            ("extended", Value::Flag(extended)),
            ("expected", Value::Bytes(expected.unwrap_or(&[]))),
        ];
        match expected {
            Some(expected) if value == expected => out.check(
                Tag::Pass,
                "launch.dlme_pcr17",
                &fields,
                format_args!("{name} PCR 17 holds {what}"),
            ),
            Some(_) => out.check(
                Tag::Fail,
                "launch.dlme_pcr17",
                &fields,
                format_args!("{name} PCR 17 does not hold {what}"),
            ),
            None => out.check(
                Tag::Info,
                "launch.dlme_pcr17",
                &fields[..3],
                format_args!(
                    "{name} PCR 17 read, and the build computes no {name} value to judge it"
                ),
            ),
        }
    }
    if granted {
        tpm.relinquish();
    }
}

/// SHA-256 `pcr` extended with the DLME, as `EXTEND_OSSL_DIGEST` does: the
/// image's digest, then the marker's.
fn with_dlme(pcr: &[u8; 32], params: &Params) -> [u8; 32] {
    let pcr = sha256::extend(pcr, &params.dlme.sha256);
    sha256::extend(&pcr, &sha256::digest(psp::EXTEND_MARKER))
}

/// After a PSP launch: PCR 17 and 18 as `LAUNCH` and the DLME's extend
/// left them. After a foreign SL both are `info`: its `LAUNCH` words never
/// reach the DLME, and its extends are its own. Returns the PCRs, where they
/// read.
fn judge_psp(
    out: &mut Out,
    params: &Params,
    psp: &params::Psp,
    handoff: &Handoff,
) -> Option<DrtmPcrs> {
    let (tpm, granted) = tpm(params);
    let pcrs = pcr::read(&tpm, params.family, Bank::Sha256);
    if granted {
        tpm.relinquish();
    }
    let pcrs = match pcrs {
        Ok(pcrs) => pcrs,
        Err(failure) => {
            out.check(
                Tag::Fail,
                "launch.dlme_pcr17",
                &[("bank", Value::Text(format_args!("sha256")))],
                format_args!("the sha256 bank did not read ({failure})"),
            );
            return None;
        }
    };
    let foreign = handoff.from == Sender::Foreign;
    let extended = foreign || handoff.dlme_extended;
    let unknown = if foreign {
        None
    } else if handoff.launch.is_none() {
        Some("LAUNCH did not succeed, so no value is expected past the SL's")
    } else if handoff.extend_unanswered {
        Some("EXTEND_OSSL_DIGEST got no answer, so whether it measured the DLME is unknown")
    } else {
        None
    };
    if let Some(why) = unknown {
        for (id, pcr) in [
            ("launch.dlme_pcr17", &pcrs[0]),
            ("launch.dlme_pcr18", &pcrs[1]),
        ] {
            out.check(
                Tag::Skip,
                id,
                &[("value", Value::Bytes(pcr.as_bytes()))],
                format_args!("{why}"),
            );
        }
        return Some(pcrs);
    }
    let what = if extended {
        ", then the DLME"
    } else {
        ", and not the DLME, which the SL's extend did not measure"
    };

    let pcr17 = pcrs[0].as_bytes();
    match handoff.launch {
        Some((anti_rollback, tsme)) => {
            let mut words = [0; 8];
            words[..4].copy_from_slice(&tsme.to_le_bytes());
            words[4..].copy_from_slice(&anti_rollback.to_le_bytes());
            let mut expected = sha256::extend(&psp.pcr17, &sha256::digest(&words));
            if extended {
                expected = with_dlme(&expected, params);
            }
            let ok = pcr17 == expected;
            out.check(
                if ok { Tag::Pass } else { Tag::Fail },
                "launch.dlme_pcr17",
                &[
                    ("bank", Value::Text(format_args!("sha256"))),
                    ("pcr17", Value::Bytes(pcr17)),
                    ("expected", Value::Bytes(&expected)),
                ],
                format_args!(
                    "sha256 PCR 17 {} the SL, SPLT version 0 and the fuse words{what}",
                    if ok { "holds" } else { "does not hold" }
                ),
            );
        }
        None => out.check(
            Tag::Info,
            "launch.dlme_pcr17",
            &[
                ("bank", Value::Text(format_args!("sha256"))),
                ("pcr17", Value::Bytes(pcr17)),
            ],
            format_args!(
                "sha256 PCR 17 read, its last LAUNCH extend takes words only the \
                 foreign SL saw, so it is not judged"
            ),
        ),
    }

    let expected = if extended {
        with_dlme(&psp.pcr18, params)
    } else {
        psp.pcr18
    };
    let pcr18 = pcrs[1].as_bytes();
    let ok = pcr18 == expected;
    // A foreign SL may extend more than the one measurement assumed here.
    let tag = match (foreign, ok) {
        (true, _) => Tag::Info,
        (false, true) => Tag::Pass,
        (false, false) => Tag::Fail,
    };
    out.check(
        tag,
        "launch.dlme_pcr18",
        &[
            ("bank", Value::Text(format_args!("sha256"))),
            ("pcr18", Value::Bytes(pcr18)),
            ("expected", Value::Bytes(&expected)),
            ("matches", Value::Flag(ok)),
        ],
        format_args!(
            "sha256 PCR 18 {} the SL's key token and SecPatchLevel{what}",
            if ok { "holds" } else { "does not hold" }
        ),
    );
    Some(pcrs)
}

/// `TMR_RELEASE`, the last command of a launch, and a device write that
/// shows the TMR blocked before it and not after.
fn release(out: &mut Out, params: &Params, handoff: &Handoff) {
    let Some(psp) = params.psp else {
        return;
    };
    let unsent = if handoff.mailbox_dead {
        Some("the mailbox stopped answering in the SL")
    } else if handoff.from == Sender::Psp && handoff.launch.is_none() {
        Some("LAUNCH did not succeed")
    } else {
        None
    };
    if let Some(why) = unsent {
        out.check(
            Tag::Skip,
            "launch.psp_tmr",
            &[],
            format_args!("TMR_RELEASE not sent, {why}"),
        );
        return;
    }
    // Safety: the parameter page checked the address is below 4 GiB, which
    // the DLME identity maps, and the preamble found the service there.
    let mailbox = unsafe { Mailbox::new(psp.mailbox as usize, params.tsc_per_ms) };
    let armed = match params.pit {
        Some(pit) => Probe::arm(&pit, params.tsc_per_ms).map_err(Some),
        None => Err(None),
    };
    let before = armed.as_ref().ok().map(Probe::watch);
    let answer = match mailbox.command(Command::TmrRelease, 0, [0; 3], psp::TIMEOUT_MS) {
        Ok(answer) => answer,
        Err(error) => {
            let register = mailbox.register();
            out.check(
                Tag::Fail,
                "launch.psp_tmr",
                &[("c2pmsg_72", Value::Hex(register.into()))],
                format_args!(
                    "TMR_RELEASE got no answer in {} ms ({}, C2PMSG_72 {register:#010x})",
                    psp::TIMEOUT_MS,
                    error.name()
                ),
            );
            if let Ok(probe) = armed {
                probe.disarm();
            }
            return;
        }
    };
    out.check(
        if answer.ok() { Tag::Pass } else { Tag::Fail },
        "launch.psp_tmr",
        &[("status", Value::Hex(answer.status.into()))],
        format_args!("TMR_RELEASE answers {}", psp::status_name(answer.status)),
    );

    let probe = match armed {
        Ok(probe) => probe,
        Err(why) => {
            out.check(
                Tag::Info,
                "launch.psp_probe",
                &[],
                format_args!(
                    "no probe: {}",
                    why.unwrap_or("the MADT names no IO-APIC pin for the PIT")
                ),
            );
            return;
        }
    };
    let after = probe.watch();
    let fields = [
        ("vector", Value::Hex(probe.vector.into())),
        ("apic_id", Value::Count(probe.apic_id.into())),
        ("enabled_apic", Value::Flag(probe.enabled_apic)),
        ("before", Value::Flag(before == Some(true))),
        ("after", Value::Flag(after)),
    ];
    probe.disarm();
    match (before == Some(true), after) {
        (false, true) => out.check(
            Tag::Pass,
            "launch.psp_probe",
            &fields,
            format_args!("the PIT's interrupt was blocked, and arrived after TMR_RELEASE"),
        ),
        (true, _) => out.check(
            Tag::Fail,
            "launch.psp_probe",
            &fields,
            format_args!("the PIT's interrupt arrived under the TMR, which it covers"),
        ),
        (false, false) => out.check(
            Tag::Info,
            "launch.psp_probe",
            &fields,
            format_args!("the PIT's interrupt never arrived, the probe shows nothing"),
        ),
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match Params::decode(page()) {
        Ok(params) => {
            let mut out = Out::new(params.sinks, params.format, params.level);
            out.unchecked(
                Tag::Fail,
                "launch.panic",
                &[],
                format_args!("the DLME panicked: {}", info.message()),
            );
        }
        Err(_) => {
            let mut com1 = uart::COM1;
            let _ = write!(com1, "drtm: the DLME panicked\r\n");
        }
    }
    halt();
}
