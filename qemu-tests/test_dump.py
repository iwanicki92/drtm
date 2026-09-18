# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Exercises the platform dump (`src/dump/`).

Raw state for a hardware run, gathered only where `format=json` and
`level=trace` both hold: it has no prose form, and it is far larger than the
results it would otherwise bury.

The tests hold it to being complete and well formed rather than to any
particular value, since what a table or register holds is the platform's
business and changes with the machine.
"""

from conftest import dumped


def test_the_dump_runs_to_its_end(default_boot: str):
    # A record either side of the whole thing, so a machine that dies on a
    # read says which one it was on rather than going quiet.
    assert [record["at"] for record in dumped(default_boot, "dump")] == [
        "start",
        "end",
    ]


def test_text_mode_carries_no_dump(tpm_crb_boot: str):
    # The boot with no `drtm.conf`, so it runs in the default prose form.
    # Records there would break every line a person reads.
    assert dumped(tpm_crb_boot, "dump") == []
    assert dumped(tpm_crb_boot, "acpi") == []


def test_acpi_tables_are_dumped_whole(default_boot: str):
    # `len` is the header's own length field and `hex` the bytes read at the
    # address, so the two agreeing is what says the table came over intact.
    tables = [table for table in dumped(default_boot, "acpi") if "hex" in table]
    assert tables, "no ACPI table was dumped"
    for table in tables:
        assert len(table["hex"]) == 2 * table["len"], table["sig"]


def test_the_root_table_and_its_entries_are_all_there(default_boot: str):
    # QEMU's q35 publishes these, and the FADT's own two targets are reached
    # through it rather than listed in it.
    signatures = {table["sig"] for table in dumped(default_boot, "acpi")}
    assert {"RSDP", "XSDT", "FACP", "APIC", "DSDT", "FACS"} <= signatures


def test_dumped_tables_checksum(default_boot: str):
    # A table whose bytes do not sum to zero was misread, which is the one
    # thing a reader cannot tell from the hexadecimal alone.
    for table in dumped(default_boot, "acpi"):
        assert table.get("csum", True), table["sig"]
