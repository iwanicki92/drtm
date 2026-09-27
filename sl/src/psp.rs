// SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
//
// SPDX-License-Identifier: BSD-3-Clause

//! The PSP path: `LAUNCH`, then every command the guide gives a loader
//! after it, in the order of its section 2 (AMD publication 58453), each
//! judged from the TPM, the service's event log or a device write.
//!
//! Every step runs unless something makes it impossible, so one run shows
//! every problem. A mailbox that stops answering ends the mailbox steps,
//! and what needs only the TPM still runs.

use core::slice;

use common::out::Out;
use common::params::{Family, Params, Psp};
use common::psp::{self, Answer, Command, Mailbox};
use common::record::{Tag, Value};
use common::sha256;
use common::tcg_log::{self, Log};
use common::tpm::{Bank, DrtmPcrs};
use common::transport::Tpm;

use common::params::{Handoff, Sender};

use crate::handoff::{self, Measured};
use crate::{SL_LOCALITY, read, tpm};

/// What `ALLOCATE_SHARED_MEMORY` asks for.
const SHARED_REQUEST: u32 = 0x10000;

/// A region the service hands back is read only if it is below 4 GiB and
/// no larger than this.
const MAX_REGION: u32 = 1 << 20;

/// The SLB `SKINIT` protects, which holds the SL's code, stack and page
/// tables.
const SLB_LEN: u64 = 0x10000;

/// The guide's appendix E.1 alignment for the MLE.
const MLE_ALIGN: u64 = 64 << 20;

unsafe extern "C" {
    static __sl_start: u8;
    static __params: u8;
}

struct Session<'a> {
    out: &'a mut Out,
    params: &'a Params,
    psp: &'a Psp,
    mailbox: Mailbox,
    /// The command that timed out, after which nothing more is sent.
    dead: Option<Command>,
    launched: bool,
    tpm: Tpm,
    held: bool,
}

/// `[base, base + len)` inside `[at, at + size)`.
fn inside(base: u64, len: u64, at: u64, size: u64) -> bool {
    base >= at && base.saturating_add(len) <= at.saturating_add(size)
}

/// A region the SL can read, as a slice.
fn readable(base: u64, len: u32) -> Option<&'static [u8]> {
    if base == 0 || len == 0 || len > MAX_REGION || base + u64::from(len) > 1 << 32 {
        return None;
    }
    // Safety: the service names this memory as a region it filled for x86,
    // below 4 GiB, which the SL identity maps, and nothing writes it while
    // the SL reads.
    Some(unsafe { slice::from_raw_parts(base as *const u8, len as usize) })
}

impl Session<'_> {
    /// Sends one command, or reports it skipped after a timeout. A timeout
    /// is a `fail` of `id` and ends the mailbox steps.
    fn command(&mut self, id: &str, command: Command, arguments: [u32; 3]) -> Option<Answer> {
        let name = command.name();
        if let Some(dead) = self.dead {
            self.out.check(
                Tag::Skip,
                id,
                &[],
                format_args!("{name} not sent, {} got no answer", dead.name()),
            );
            return None;
        }
        let timeout = if command == Command::ExtendOsslDigest {
            psp::EXTEND_TIMEOUT_MS
        } else {
            psp::TIMEOUT_MS
        };
        self.out
            .debug(format_args!("{name} sent, waiting up to {timeout} ms"));
        match self.mailbox.command(command, 0, arguments, timeout) {
            Ok(answer) => {
                self.out.debug(format_args!(
                    "{name} answers {} ({:#x}), C2PMSG_93 to 95 {:#010x} {:#010x} {:#010x}",
                    psp::status_name(answer.status),
                    answer.status,
                    answer.results[0],
                    answer.results[1],
                    answer.results[2]
                ));
                self.seize(command);
                Some(answer)
            }
            Err(error) => {
                self.dead = Some(command);
                let register = self.mailbox.register();
                self.out.check(
                    Tag::Fail,
                    id,
                    &[
                        ("command", Value::Text(format_args!("{name}"))),
                        ("c2pmsg_72", Value::Hex(register.into())),
                    ],
                    format_args!(
                        "{name} got no answer in {timeout} ms ({}, C2PMSG_72 \
                         {register:#010x}), nothing more is sent",
                        error.name()
                    ),
                );
                None
            }
        }
    }

    /// Whether the command took the SL's locality, as the service does
    /// when it extends, and the locality asked for again if so.
    fn seize(&mut self, command: Command) {
        if !self.held {
            return;
        }
        let seized = self.tpm.seized();
        let active = self.tpm.active();
        let locality = self.tpm.locality();
        self.out.check(
            Tag::Info,
            "launch.psp_seize",
            &[
                ("command", Value::Text(format_args!("{}", command.name()))),
                ("locality", Value::Count(locality.into())),
                ("active", Value::Flag(active)),
                ("seized", Value::Flag(seized)),
            ],
            format_args!(
                "after {}, locality {locality} is {} and beenSeized is {}",
                command.name(),
                if active { "still active" } else { "lost" },
                if seized { "set" } else { "clear" }
            ),
        );
        if seized {
            self.tpm.clear_seized();
        }
        if !active && self.tpm.request().is_err() {
            self.held = false;
        }
    }

    /// A status after a failed `LAUNCH`, which the guide's appendix A says
    /// each command answers, reported as a fact.
    fn declined(&mut self, id: &str, command: Command, answer: &Answer, expected: u16) {
        let status = answer.status;
        self.out.check(
            Tag::Info,
            id,
            &[
                ("status", Value::Hex(status.into())),
                ("expected", Value::Hex(expected.into())),
            ],
            format_args!(
                "{} after the failed LAUNCH answers {}, appendix A gives {}",
                command.name(),
                psp::status_name(status),
                psp::status_name(expected)
            ),
        );
    }

    fn pcrs(&mut self, bank: Bank) -> Option<DrtmPcrs> {
        read(&self.tpm, Family::V2_0, bank).ok()
    }
}

/// Runs the PSP path, from `LAUNCH` to handing the TPM back, and returns
/// what the DLME is to know.
pub fn run(out: &mut Out, params: &Params, psp: &Psp, slb: u32) -> (Handoff, Measured) {
    // Safety: the parameter page checked the address is below 4 GiB, which
    // the SL identity maps, and the preamble found the service there.
    let mailbox = unsafe { Mailbox::new(psp.mailbox as usize, params.tsc_per_ms) };
    let mut session = Session {
        out,
        params,
        psp,
        mailbox,
        dead: None,
        launched: false,
        tpm: tpm(params, 0),
        held: false,
    };
    let fuses = launch(&mut session);
    locality(&mut session);
    // A refused `LAUNCH`, read once a locality is held.
    if !session.launched && session.dead.is_none() {
        cap(&mut session);
    }
    judge_pcrs(&mut session, fuses);
    let measured = handoff::measure(params);
    let dlme_extended = extend(&mut session, &measured);
    let extend_unanswered = session.dead == Some(Command::ExtendOsslDigest);
    tmr_descriptors(&mut session);
    let shared = shared_memory(&mut session, slb);
    tcg_log(&mut session, slb, shared);
    ivrs(&mut session, shared);
    teardown(&mut session);
    if session.held {
        session.tpm.relinquish();
    }
    let handoff = Handoff {
        from: Sender::Psp,
        launch: fuses,
        mailbox_dead: session.dead.is_some(),
        dlme_extended,
        extend_unanswered,
        ..Handoff::FOREIGN
    };
    (handoff, measured)
}

/// Step 1. Returns the anti-rollback and TSME words `LAUNCH` returns.
fn launch(s: &mut Session) -> Option<(u32, u32)> {
    let answer = s.command("launch.psp_launch", Command::Launch, [0; 3])?;
    if answer.ok() {
        s.launched = true;
        let [anti_rollback, tsme, _] = answer.results;
        s.out.check(
            Tag::Pass,
            "launch.psp_launch",
            &[
                ("status", Value::Hex(0)),
                ("anti_rollback", Value::Hex(anti_rollback.into())),
                ("tsme", Value::Hex(tsme.into())),
            ],
            format_args!(
                "LAUNCH accepted this Secure Loader, anti-rollback {anti_rollback:#x}, \
                 TSME {tsme:#x}"
            ),
        );
        return Some((anti_rollback, tsme));
    }
    let status = answer.status;
    s.out.check(
        Tag::Fail,
        "launch.psp_launch",
        &[("status", Value::Hex(status.into()))],
        format_args!(
            "LAUNCH answers {} ({status:#x}), the service refused this Secure Loader",
            psp::status_name(status)
        ),
    );
    None
}

/// After a refused `LAUNCH`, PCR 18 to 20 should hold the cap the guide's
/// step 4f describes: zero extended with all ones.
fn cap(s: &mut Session) {
    let Some(pcrs) = s.pcrs(Bank::Sha256) else {
        s.out.check(
            Tag::Info,
            "launch.psp_cap",
            &[],
            format_args!("the SHA-256 bank did not read, so the cap is not known"),
        );
        return;
    };
    let capped = sha256::extend(&[0; 32], &[0xFF; 32]);
    let ok = pcrs[1..4].iter().all(|pcr| pcr.as_bytes() == capped);
    let fields = [
        ("pcr18", Value::Bytes(pcrs[1].as_bytes())),
        ("pcr19", Value::Bytes(pcrs[2].as_bytes())),
        ("pcr20", Value::Bytes(pcrs[3].as_bytes())),
        ("capped", Value::Flag(ok)),
    ];
    if ok {
        s.out.check(
            Tag::Pass,
            "launch.psp_cap",
            &fields,
            format_args!("SHA-256 PCR 18 to 20 are capped, zero extended with all ones"),
        );
    } else {
        s.out.check(
            Tag::Fail,
            "launch.psp_cap",
            &fields,
            format_args!(
                "SHA-256 PCR 18 to 20 are not capped, which the guide's step 4f \
                 says a refused LAUNCH does"
            ),
        );
    }
}

/// Step 2: locality 2, which a successful `LAUNCH` unlocks.
fn locality(s: &mut Session) {
    let wanted = tpm(s.params, SL_LOCALITY);
    let access = wanted.access();
    let granted = wanted.request().is_ok();
    let fields = [
        ("locality", Value::Count(SL_LOCALITY.into())),
        ("access", Value::Hex(access.into())),
    ];
    match (granted, s.launched) {
        (true, true) => s.out.check(
            Tag::Pass,
            "launch.locality",
            &fields,
            format_args!("locality {SL_LOCALITY} granted after LAUNCH"),
        ),
        (false, true) => s.out.check(
            Tag::Fail,
            "launch.locality",
            &fields,
            format_args!(
                "locality {SL_LOCALITY} not granted after LAUNCH, which the \
                 guide's step 4f.viii says unlocks it"
            ),
        ),
        (granted, false) => s.out.check(
            Tag::Info,
            "launch.locality",
            &fields,
            format_args!(
                "locality {SL_LOCALITY} {} with no successful LAUNCH",
                if granted { "granted" } else { "not granted" }
            ),
        ),
    }
    if granted {
        s.tpm = wanted;
        s.held = true;
    } else {
        s.tpm = tpm(s.params, 0);
        s.held = s.tpm.request().is_ok();
    }
}

/// Step 2: the PCRs `LAUNCH` extended, against the build's predictions.
fn judge_pcrs(s: &mut Session, fuses: Option<(u32, u32)>) {
    let at = s.tpm.locality();
    let Some(pcrs) = s.pcrs(Bank::Sha256) else {
        s.out.check(
            Tag::Fail,
            "launch.pcr_read",
            &[("bank", Value::Text(format_args!("sha256")))],
            format_args!("the sha256 bank did not read at locality {at}"),
        );
        return;
    };
    let pcr17 = pcrs[0].as_bytes();
    match fuses {
        Some((anti_rollback, tsme)) => {
            let mut words = [0; 8];
            words[..4].copy_from_slice(&tsme.to_le_bytes());
            words[4..].copy_from_slice(&anti_rollback.to_le_bytes());
            let expected = sha256::extend(&s.psp.pcr17, &sha256::digest(&words));
            let ok = pcr17 == expected;
            s.out.check(
                if ok { Tag::Pass } else { Tag::Fail },
                "launch.pcr17",
                &[
                    ("bank", Value::Text(format_args!("sha256"))),
                    ("locality", Value::Count(at.into())),
                    ("pcr17", Value::Bytes(pcr17)),
                    ("expected", Value::Bytes(&expected)),
                ],
                format_args!(
                    "sha256 PCR 17 {} this image, SPLT version 0 and the fuse \
                     words (read at locality {at})",
                    if ok { "holds" } else { "does not hold" }
                ),
            );
            let ok = pcrs[1].as_bytes() == s.psp.pcr18;
            s.out.check(
                if ok { Tag::Pass } else { Tag::Fail },
                "launch.pcr18",
                &[
                    ("bank", Value::Text(format_args!("sha256"))),
                    ("pcr18", Value::Bytes(pcrs[1].as_bytes())),
                    ("expected", Value::Bytes(&s.psp.pcr18)),
                ],
                format_args!(
                    "sha256 PCR 18 {} this image's key token and SecPatchLevel",
                    if ok { "holds" } else { "does not hold" }
                ),
            );
            let zero = pcrs[2..].iter().all(|pcr| pcr.all(0));
            s.out.check(
                if zero { Tag::Pass } else { Tag::Fail },
                "launch.pcr19_22",
                &[
                    ("pcr19", Value::Bytes(pcrs[2].as_bytes())),
                    ("pcr22", Value::Bytes(pcrs[5].as_bytes())),
                ],
                format_args!(
                    "sha256 PCR 19 to 22 are {}",
                    if zero { "zero" } else { "not all zero" }
                ),
            );
        }
        None => {
            // No launch extended anything, so PCR 17 holds what SKINIT left.
            let ok = pcr17 == s.params.expected_sha256;
            s.out.check(
                if ok { Tag::Pass } else { Tag::Fail },
                "launch.pcr17",
                &[
                    ("bank", Value::Text(format_args!("sha256"))),
                    ("locality", Value::Count(at.into())),
                    ("pcr17", Value::Bytes(pcr17)),
                    ("expected", Value::Bytes(&s.params.expected_sha256)),
                ],
                format_args!(
                    "sha256 PCR 17 {} this image's SKINIT measurement alone",
                    if ok { "holds" } else { "does not hold" }
                ),
            );
        }
    }

    // The service extends SHA-256 alone, and no run has read SHA-1 after it.
    if s.params.banks & Bank::Sha1.bit() != 0
        && let Ok(sha1) = read(&s.tpm, Family::V2_0, Bank::Sha1)
    {
        let same = sha1[0].as_bytes() == s.params.expected_sha1;
        s.out.check(
            Tag::Info,
            "launch.pcr17",
            &[
                ("bank", Value::Text(format_args!("sha1"))),
                ("locality", Value::Count(at.into())),
                ("pcr17", Value::Bytes(sha1[0].as_bytes())),
                ("expected", Value::Bytes(&s.params.expected_sha1)),
                ("pcr18", Value::Bytes(sha1[1].as_bytes())),
            ],
            format_args!(
                "sha1 PCR 17 {} the SKINIT measurement alone, the service \
                 extends SHA-256",
                if same { "holds" } else { "is not" }
            ),
        );
    }
}

/// Step 3: the DLME, which is what the guide's MLE is here. Returns
/// whether the service measured it.
/// A DLME that is not the build's goes unmeasured. The handoff reports it,
/// and does not jump. Out of line, so its record adds nothing to `run`'s
/// frame.
#[cold]
#[inline(never)]
fn not_extended(out: &mut Out, measured: &Measured) {
    out.check(
        Tag::Skip,
        "launch.psp_extend",
        &[("digest", Value::Bytes(&measured.sha256))],
        format_args!("EXTEND_OSSL_DIGEST not sent, the DLME in memory is not the build's"),
    );
}

fn extend(s: &mut Session, measured: &Measured) -> bool {
    let (base, len) = (s.params.dlme.block, s.params.dlme.len);
    let aligned = base.is_multiple_of(MLE_ALIGN);
    if !measured.matches {
        not_extended(s.out, measured);
        return false;
    }
    let before = s.pcrs(Bank::Sha256);
    let Some(answer) = s.command(
        "launch.psp_extend",
        Command::ExtendOsslDigest,
        [len, base as u32, (base >> 32) as u32],
    ) else {
        return false;
    };
    if !s.launched {
        s.declined(
            "launch.psp_extend",
            Command::ExtendOsslDigest,
            &answer,
            psp::NOT_SUPPORTED,
        );
        return false;
    }
    let status = answer.status;
    let digest = measured.sha256;
    let fields = |pcr17: &'static [u8]| -> [(&str, Value<'_>); 7] {
        [
            ("base", Value::Hex(base)),
            ("size", Value::Hex(len.into())),
            ("aligned", Value::Flag(aligned)),
            ("matches_build", Value::Flag(measured.matches)),
            ("digest", Value::Bytes(&measured.sha256)),
            ("status", Value::Hex(status.into())),
            ("pcr17", Value::Bytes(pcr17)),
        ]
    };
    let unaligned = if aligned {
        ""
    } else {
        ", off the 64 MiB boundary of the guide's appendix E.1"
    };
    if !answer.ok() {
        s.out.check(
            Tag::Fail,
            "launch.psp_extend",
            &fields(&[]),
            format_args!(
                "EXTEND_OSSL_DIGEST of the DLME, {len:#x} bytes at {base:#x}, answers {}{unaligned}",
                psp::status_name(answer.status)
            ),
        );
        return false;
    }
    let (Some(before), Some(after)) = (before, s.pcrs(Bank::Sha256)) else {
        s.out.check(
            Tag::Fail,
            "launch.psp_extend",
            &fields(&[]),
            format_args!("EXTEND_OSSL_DIGEST answered, the PCRs did not read"),
        );
        return true;
    };
    let marker = sha256::digest(psp::EXTEND_MARKER);
    let expect = |pcr: &[u8]| {
        let pcr: [u8; 32] = pcr.try_into().unwrap_or([0; 32]);
        sha256::extend(&sha256::extend(&pcr, &digest), &marker)
    };
    let ok = after[0].as_bytes() == expect(before[0].as_bytes())
        && after[1].as_bytes() == expect(before[1].as_bytes());
    let mut fields = fields(&[]);
    fields[6].1 = Value::Bytes(after[0].as_bytes());
    s.out.check(
        if ok { Tag::Pass } else { Tag::Fail },
        "launch.psp_extend",
        &fields,
        format_args!(
            "EXTEND_OSSL_DIGEST of the DLME, {len:#x} bytes at {base:#x}, {} PCR 17 \
             and 18 with it, then AMDSL{unaligned}",
            if ok { "extended" } else { "did not extend" }
        ),
    );
    true
}

/// Step 4: the service's own record of the TMR the preamble set up.
fn tmr_descriptors(s: &mut Session) {
    let Some(answer) = s.command("launch.psp_tmrs", Command::GetTmrDescriptors, [0; 3]) else {
        return;
    };
    if !s.launched {
        s.declined(
            "launch.psp_tmrs",
            Command::GetTmrDescriptors,
            &answer,
            psp::NOT_SUPPORTED,
        );
        return;
    }
    let (base, len) = answer.region();
    let region = [
        ("status", Value::Hex(answer.status.into())),
        ("base", Value::Hex(base)),
        ("size", Value::Hex(len.into())),
    ];
    let Some(bytes) = answer.ok().then(|| readable(base, len)).flatten() else {
        s.out.check(
            Tag::Info,
            "launch.psp_tmrs",
            &region,
            format_args!(
                "GET_TMR_DESCRIPTORS answers {}, {len:#x} bytes at {base:#x}",
                psp::status_name(answer.status)
            ),
        );
        return;
    };
    let mut found = None;
    for (index, chunk) in bytes
        .as_chunks::<{ psp::TMR_DESCRIPTOR_LEN }>()
        .0
        .iter()
        .enumerate()
    {
        let Some(descriptor) = psp::tmr_descriptor(chunk) else {
            continue;
        };
        s.out.debug(format_args!(
            "TMR descriptor {index}: base {:#x}, size {:#x}",
            descriptor.0, descriptor.1
        ));
        if found.is_none() {
            found = psp::tmr_matches(descriptor, s.psp.tmr_base, s.psp.tmr_size);
        }
    }
    match found {
        Some(unit) => s.out.check(
            Tag::Pass,
            "launch.psp_tmrs",
            &region,
            format_args!(
                "GET_TMR_DESCRIPTORS lists TMR 0, its size in {}",
                match unit {
                    psp::TmrSizeUnit::Bytes => "bytes",
                    psp::TmrSizeUnit::Units64K => "64 KiB units",
                }
            ),
        ),
        None => s.out.check(
            Tag::Info,
            "launch.psp_tmrs",
            &region,
            format_args!(
                "GET_TMR_DESCRIPTORS lists {} descriptors, none TMR 0's",
                bytes.len() / psp::TMR_DESCRIPTOR_LEN
            ),
        ),
    }
}

/// Step 5: a buffer the guide calls shared with x86, and does not say what
/// for. Every outcome is a fact.
fn shared_memory(s: &mut Session, slb: u32) -> Option<(u64, u64)> {
    let answer = s.command(
        "launch.psp_shared",
        Command::AllocateSharedMemory,
        [SHARED_REQUEST, 0, 0],
    )?;
    if !s.launched {
        s.declined(
            "launch.psp_shared",
            Command::AllocateSharedMemory,
            &answer,
            psp::OUT_OF_RESOURCES,
        );
        return None;
    }
    let (base, len) = answer.region();
    let in_tmr = inside(base, len.into(), s.psp.tmr_base, s.psp.tmr_size);
    // A buffer over the SLB would overwrite the running SL.
    let over_slb = base < u64::from(slb) + SLB_LEN && u64::from(slb) < base + u64::from(len);
    let written = answer.ok() && !over_slb && write_back(base, len);
    s.out.check(
        Tag::Info,
        "launch.psp_shared",
        &[
            ("status", Value::Hex(answer.status.into())),
            ("base", Value::Hex(base)),
            ("size", Value::Hex(len.into())),
            ("in_tmr", Value::Flag(in_tmr)),
            ("over_slb", Value::Flag(over_slb)),
            ("written", Value::Flag(written)),
        ],
        format_args!(
            "ALLOCATE_SHARED_MEMORY answers {}, {len:#x} bytes at {base:#x}{}",
            psp::status_name(answer.status),
            if over_slb {
                ", over the SLB, so not written"
            } else {
                ""
            }
        ),
    );
    (answer.ok() && len > 0).then_some((base, len.into()))
}

/// Writes a pattern over the buffer and reads it back.
fn write_back(base: u64, len: u32) -> bool {
    if readable(base, len).is_none() {
        return false;
    }
    let pattern = |i: usize| (i as u8) ^ 0xA5;
    for i in 0..len as usize {
        // Safety: `readable` checked the buffer lies below 4 GiB, which the
        // SL identity maps, and the service granted it to x86.
        unsafe { core::ptr::write_volatile((base as usize + i) as *mut u8, pattern(i)) };
    }
    (0..len as usize).all(|i| {
        // Safety: as for the write.
        unsafe { core::ptr::read_volatile((base as usize + i) as *const u8) == pattern(i) }
    })
}

/// Where a region lies, as fields: inside the TMR and the shared buffer.
fn placement(s: &Session, base: u64, len: u32, shared: Option<(u64, u64)>) -> (bool, bool) {
    (
        inside(base, len.into(), s.psp.tmr_base, s.psp.tmr_size),
        shared.is_some_and(|(at, size)| inside(base, len.into(), at, size)),
    )
}

/// Step 6: the service's event log, replayed against the TPM.
fn tcg_log(s: &mut Session, slb: u32, shared: Option<(u64, u64)>) {
    let Some(answer) = s.command("launch.psp_log", Command::GetTcgLogs, [0; 3]) else {
        return;
    };
    if !s.launched {
        s.declined(
            "launch.psp_log",
            Command::GetTcgLogs,
            &answer,
            psp::NOT_SUPPORTED,
        );
        return;
    }
    let (base, len) = answer.region();
    let (in_tmr, in_shared) = placement(s, base, len, shared);
    let fields = [
        ("status", Value::Hex(answer.status.into())),
        ("base", Value::Hex(base)),
        ("size", Value::Hex(len.into())),
        ("in_tmr", Value::Flag(in_tmr)),
        ("in_shared", Value::Flag(in_shared)),
    ];
    if !answer.ok() {
        s.out.check(
            Tag::Fail,
            "launch.psp_log",
            &fields,
            format_args!(
                "GET_TCG_LOGS answers {} after LAUNCH",
                psp::status_name(answer.status)
            ),
        );
        return;
    }
    let Some(log) = readable(base, len) else {
        s.out.check(
            Tag::Info,
            "launch.psp_log",
            &fields,
            format_args!("the log, {len:#x} bytes at {base:#x}, is out of reach"),
        );
        return;
    };
    let events = match Log::parse(log) {
        Ok(events) => events,
        Err(why) => {
            s.out.check(
                Tag::Fail,
                "launch.psp_log",
                &fields,
                format_args!("the log at {base:#x} does not parse ({why})"),
            );
            return;
        }
    };
    let mut first = None;
    for event in events.flatten() {
        s.out.debug(format_args!(
            "log event at {:#x}: PCR {} type {:#x}, {} data bytes",
            event.offset,
            event.pcr,
            event.kind,
            event.data.len()
        ));
        if first.is_none() && event.pcr == 17 {
            first = event.sha256;
        }
    }
    let measured = {
        // Safety: the header's length field is in the SLB, below 4 GiB.
        let length = unsafe { core::ptr::read_volatile((slb as usize + 2) as *const u16) };
        // Safety: as for the length, the image lies in the SLB.
        let image = unsafe { slice::from_raw_parts(slb as usize as *const u8, length.into()) };
        sha256::digest(image)
    };
    let replay = tcg_log::replay(log);
    let now = s.pcrs(Bank::Sha256);
    let matches = match (&replay, &now) {
        (Ok(replay), Some(now)) => replay[0] == now[0].as_bytes() && replay[1] == now[1].as_bytes(),
        _ => false,
    };
    let first_ok = first == Some(measured);
    let ok = matches && first_ok;
    s.out.check(
        if ok { Tag::Pass } else { Tag::Fail },
        "launch.psp_log",
        &fields,
        format_args!(
            "the log {} PCR 17 and 18, its first event {} this image's digest",
            if matches {
                "replays to"
            } else {
                "does not replay to"
            },
            if first_ok {
                "carries"
            } else {
                "does not carry"
            }
        ),
    );
}

/// Step 7: the service's copy of the IVRS, against the one ACPI holds.
fn ivrs(s: &mut Session, shared: Option<(u64, u64)>) {
    let Some(answer) = s.command("launch.psp_ivrs", Command::GetIvrsTableInfo, [0; 3]) else {
        return;
    };
    if !s.launched {
        s.declined(
            "launch.psp_ivrs",
            Command::GetIvrsTableInfo,
            &answer,
            psp::GET_IVRS_TABLE_FAILED,
        );
        return;
    }
    let (base, len) = answer.region();
    let (in_tmr, in_shared) = placement(s, base, len, shared);
    let table = answer.ok().then(|| readable(base, len)).flatten();
    let digest = table.map(sha256::digest);
    let same = digest.is_some() && digest == s.params.ivrs_sha256;
    let fields = [
        ("status", Value::Hex(answer.status.into())),
        ("base", Value::Hex(base)),
        ("size", Value::Hex(len.into())),
        ("in_tmr", Value::Flag(in_tmr)),
        ("in_shared", Value::Flag(in_shared)),
        ("same", Value::Flag(same)),
    ];
    let acpi = if s.params.ivrs_sha256.is_some() {
        "not the IVRS ACPI holds"
    } else {
        "and ACPI has no IVRS"
    };
    if same {
        s.out.check(
            Tag::Pass,
            "launch.psp_ivrs",
            &fields,
            format_args!("GET_IVRS_TABLE_INFO hands out a copy of ACPI's IVRS"),
        );
    } else {
        s.out.check(
            Tag::Info,
            "launch.psp_ivrs",
            &fields,
            format_args!(
                "GET_IVRS_TABLE_INFO answers {}, {len:#x} bytes at {base:#x}, {acpi}",
                psp::status_name(answer.status)
            ),
        );
    }
}

/// Step 8: the TPM handed back, which locks locality 2 again.
fn teardown(s: &mut Session) {
    if s.held {
        s.tpm.relinquish();
        s.held = false;
    }
    let Some(answer) = s.command("launch.psp_teardown", Command::TpmLocalityAccess, [0; 3]) else {
        return;
    };
    if !s.launched {
        s.declined(
            "launch.psp_teardown",
            Command::TpmLocalityAccess,
            &answer,
            psp::NOT_SUPPORTED,
        );
        return;
    }
    let locality2 = tpm(s.params, SL_LOCALITY);
    let refused = locality2.request().is_err();
    if !refused {
        locality2.relinquish();
    }
    let access4 = tpm(s.params, 4).access();
    let fields = [
        ("status", Value::Hex(answer.status.into())),
        ("refused", Value::Flag(refused)),
        ("access4", Value::Hex(access4.into())),
    ];
    let (tag, detail) = match (answer.ok(), refused) {
        (true, true) => (Tag::Pass, "and locality 2 is refused from now on"),
        (true, false) => (
            Tag::Fail,
            "and locality 2 is still granted, which the guide's section 4.8 \
             says it stops",
        ),
        (false, _) => (Tag::Fail, "so the TPM was not handed back"),
    };
    s.out.check(
        tag,
        "launch.psp_teardown",
        &fields,
        format_args!(
            "TPM_LOCALITY_ACCESS answers {}, {detail}, TPM_ACCESS_4 reads {access4:#x}",
            psp::status_name(answer.status)
        ),
    );
}
