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


def test_the_cpu_identity_is_named(default_boot: str):
    # The fold of the extended family and model fields, which decides which
    # register lists apply. QEMU's plain `EPYC` is family 17h.
    (cpu,) = dumped(default_boot, "cpu")
    assert cpu["vendor"] == "AuthenticAMD"
    assert int(cpu["family"], 16) == 0x17


def test_cpuid_leaves_are_dumped(default_boot: str):
    # Leaf 0 is what every other leaf count comes from, and the extended
    # range is where the AMD features live.
    leaves = {int(leaf["leaf"], 16) for leaf in dumped(default_boot, "cpuid")}
    assert {0x0, 0x1, 0x8000_0000, 0x8000_0001} <= leaves


def test_subleaf_leaves_are_read_per_subleaf(default_boot: str):
    # Leaf 7 reports different words per subleaf, so reading it once would
    # silently lose whatever the others hold.
    subleaves = [
        leaf["subleaf"]
        for leaf in dumped(default_boot, "cpuid")
        if leaf["leaf"] == "0x7"
    ]
    assert subleaves == sorted(set(subleaves))
    assert subleaves[0] == 0


def test_every_processor_firmware_started_is_listed(default_boot: str):
    # MP services is how a launch would park the APs, so the dump has to
    # agree with it about how many there are.
    (counts,) = dumped(default_boot, "processors")
    listed = dumped(default_boot, "processor")
    assert len(listed) == counts["total"]
    assert sum(1 for cpu in listed if cpu["bsp"]) == 1


def test_msrs_are_announced_before_they_are_read(default_boot: str):
    # A read that faults leaves its announcement and no value, so a hung
    # machine still says which register killed it. Every read here survives,
    # so each announcement is followed by its value.
    reads = [msr for msr in dumped(default_boot, "msr") if "id" in msr]
    assert reads, "no MSR was read"
    for announced, value in zip(reads[::2], reads[1::2], strict=True):
        assert "value" not in announced
        assert value["id"] == announced["id"]
        assert "value" in value


def test_the_amd_msr_list_is_guarded_by_family(default_boot: str):
    # The list is verified against one family's PPR, and a register another
    # family does not implement raises #GP, which hangs the machine. No QEMU
    # model here is family 1Ah, so every boot must refuse the list.
    (refused,) = [msr for msr in dumped(default_boot, "msr") if "documented" in msr]
    assert refused["documented"] is False
    assert int(refused["family"], 16) == 0x17
    read = {msr["name"] for msr in dumped(default_boot, "msr") if "name" in msr}
    assert "MMIO_CONF_BASE" not in read


def test_architectural_msrs_are_read_whatever_the_family(default_boot: str):
    # The family guard covers AMD's own list. What CPUID reports as present
    # is present, so those reads stand on the feature bit instead.
    read = {msr["name"] for msr in dumped(default_boot, "msr") if "name" in msr}
    assert {"APIC_BASE", "PAT", "MTRRCAP", "MTRR_DEF_TYPE"} <= read


def test_msrs_are_read_on_a_non_amd_cpu(intel_cpu_boot: str):
    # Nothing in the architectural set is AMD's, so an Intel boot dumps it
    # and skips the vendor list without saying anything about a family.
    read = {msr["name"] for msr in dumped(intel_cpu_boot, "msr") if "name" in msr}
    assert "APIC_BASE" in read
    assert not [msr for msr in dumped(intel_cpu_boot, "msr") if "documented" in msr]
