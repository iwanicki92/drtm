# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""Reports whether a firmware image's DRTM TA holds an SL key allowlist."""

import argparse
import json
import sys
from enum import IntEnum
from pathlib import Path

from fwscan import acpi, allowlist, keys, psp


class Verdict(IntEnum):
    """Per TA and per image, the exit status is the worst over all images."""

    NO_DRTM_TA = 0
    NO_ALLOWLIST = 2
    ENCRYPTED = 3
    INCONCLUSIVE = 4
    ALLOWLIST = 5

    @property
    def label(self) -> str:
        return self.name.lower().replace("_", "-")


EXIT_ERROR = 1

MEANING = {
    Verdict.NO_DRTM_TA: "no DRTM TA (PSP entry type 0x47) in the image",
    Verdict.NO_ALLOWLIST: (
        "the DRTM TA only compares digests of keys it quite likely checks "
        "outside the SL key path, so no SL key allowlist was found"
    ),
    Verdict.ENCRYPTED: "the DRTM TA is encrypted, its key checks cannot be read",
    Verdict.INCONCLUSIVE: "plaintext DRTM TA, but no key allowlist code found in it",
    Verdict.ALLOWLIST: (
        "the DRTM TA only takes SL keys whose token digest it lists, "
        "an SL signed with any other key is expected to be refused"
    ),
}


def scan_ta(ta: psp.Ta, known: list[keys.Key]) -> dict:
    report = {
        "address": ta.address,
        "size": ta.size,
        "version": ta.version,
        "key_id": ta.key_id.hex().upper(),
        "encrypted": ta.encrypted,
        "compressed": ta.compressed,
        "digests": [],
    }
    if ta.image is None:
        report["verdict"] = Verdict.ENCRYPTED
        return report
    for digest in allowlist.find(ta.image):
        report["digests"].append(
            {
                "algorithm": digest.algorithm,
                "digest": digest.value.hex(),
                "offset": digest.offset,
                "keys": [k.name for k in known if k.matches(digest.value)],
            }
        )
    report["verdict"] = verdict(report["digests"], known)
    return report


def verdict(digests: list[dict], known: list[keys.Key]) -> Verdict:
    """A digest counts unless all its keys are known to be checked off the SL path."""
    if not digests:
        return Verdict.INCONCLUSIVE
    off_sl = {k.name for k in known if not k.sl}
    if all(d["keys"] and set(d["keys"]) <= off_sl for d in digests):
        return Verdict.NO_ALLOWLIST
    return Verdict.ALLOWLIST


def scan(path: Path, psptool: str, known: list[keys.Key], hid: bool) -> dict:
    firmware = path.read_bytes()
    tas = [
        scan_ta(ta, known)
        for ta in psp.drtm_tas(firmware, psp.run_psptool(psptool, path))
    ]
    listed = {name for ta in tas for d in ta["digests"] for name in d["keys"]}
    return {
        "image": str(path),
        "verdict": max((ta["verdict"] for ta in tas), default=Verdict.NO_DRTM_TA),
        "drtm_hids": acpi.drtm_hids(firmware) if hid else None,
        "tas": tas,
        "keys": {k.name: k.name in listed for k in known},
    }


def to_json(report: dict) -> dict:
    out = dict(report, verdict=report["verdict"].label)
    out["tas"] = [dict(ta, verdict=ta["verdict"].label) for ta in report["tas"]]
    return out


def print_text(report: dict) -> None:
    verdict = report["verdict"]
    print(f"{report['image']}: {verdict.label}")
    if report["drtm_hids"] is not None:
        print(f"  DRTM device _HID: {', '.join(report['drtm_hids']) or 'none found'}")
    for ta in report["tas"]:
        state = "encrypted" if ta["encrypted"] else "plaintext"
        if ta["compressed"]:
            state += ", compressed"
        print(
            f"  DRTM TA {ta['version']} at {ta['address']:#x}: {state}, "
            f"signed by {ta['key_id']}"
        )
        for d in ta["digests"]:
            names = ", ".join(d["keys"]) or "unknown key"
            print(f"    {d['algorithm']} {d['digest'][:16]}... {names}")
    if verdict == Verdict.ALLOWLIST:
        for name, is_listed in report["keys"].items():
            print(f"  {name}: {'listed' if is_listed else 'not listed'}")
    print(f"  {MEANING[verdict]}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="fwscan",
        description=(
            "Look for an SL key allowlist in the PSP DRTM TA of each firmware "
            "image. Exit status: 0 no DRTM TA, 2 no allowlist, 3 encrypted, "
            "4 inconclusive, 5 allowlist (the worst over all images), 1 on an "
            "error."
        ),
    )
    parser.add_argument("images", nargs="+", type=Path, metavar="IMAGE")
    parser.add_argument(
        "--sl",
        action="append",
        default=[],
        type=Path,
        metavar="FILE",
        help="also look for the key of this $AS1-signed SL, e.g. amddrtm.dll",
    )
    parser.add_argument("--psptool", default="psptool", help="psptool command to run")
    parser.add_argument("--json", action="store_true", help="print JSON")
    parser.add_argument(
        "--no-hid", action="store_true", help="skip the DRTM device _HID search"
    )
    args = parser.parse_args(argv)

    try:
        known = list(keys.KNOWN)
        for path in args.sl:
            known.append(keys.from_token(path.name, keys.sl_token(path.read_bytes())))
        reports = [
            scan(path, args.psptool, known, not args.no_hid) for path in args.images
        ]
    except (OSError, ValueError, psp.PsptoolError) as e:
        print(f"fwscan: {e}", file=sys.stderr)
        return EXIT_ERROR

    if args.json:
        print(json.dumps([to_json(r) for r in reports], indent=2))
    else:
        for report in reports:
            print_text(report)
    return max(r["verdict"] for r in reports)
