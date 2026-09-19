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

import re

from conftest import dumped


def test_the_dump_runs_to_its_end(default_boot: str):
    # A marker before each section and one after the last, so a machine that
    # dies on a read says where rather than going quiet.
    assert [record["at"] for record in dumped(default_boot, "dump")] == [
        "firmware",
        "cpuid",
        "processors",
        "acpi",
        "tpm",
        "pci",
        "msr",
        "iommu",
        "psp",
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


def test_every_msr_read_reports_its_value(default_boot: str):
    # One record per register, in the order the source reads them, which is
    # what locates the read a hung machine died on.
    reads = [msr for msr in dumped(default_boot, "msr") if "id" in msr]
    assert reads, "no MSR was read"
    for msr in reads:
        assert "value" in msr, msr["name"]


def test_the_amd_msr_list_is_guarded_by_family(default_boot: str):
    # The list is verified against one family's PPR, and a register another
    # family does not implement raises #GP, which hangs the machine. No QEMU
    # model here is family 1Ah, so every boot must refuse the list.
    (refused,) = [msr for msr in dumped(default_boot, "msr") if "documented" in msr]
    assert refused["documented"] is False
    assert int(refused["family"], 16) == 0x17
    read = {msr["name"] for msr in dumped(default_boot, "msr") if "name" in msr}
    assert "MMIO_CONF_BASE" not in read


def test_the_amd_msr_list_runs_on_the_family_it_documents(
    mmconfig_above_4gib_boot: str,
):
    # The other side of that guard, reached by asking QEMU for family 1Ah.
    # `MMIO_CONF_BASE` is the one register in the list this guest really
    # holds, so it reading back what QEMU was told is what says these are
    # reads rather than records. The rest answer zero, since QEMU returns
    # that for an MSR it does not model instead of raising #GP.
    msrs = dumped(mmconfig_above_4gib_boot, "msr")
    (cpu,) = dumped(mmconfig_above_4gib_boot, "cpu")
    assert int(cpu["family"], 16) == 0x1A
    assert not [msr for msr in msrs if "documented" in msr]
    values = {msr["name"]: msr["value"] for msr in msrs if "name" in msr}
    assert {"PATCH_LEVEL", "SYSCFG", "HWCR", "SMM_BASE"} <= values.keys()
    assert int(values["MMIO_CONF_BASE"], 16) == 0xF800_0021


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


def test_configuration_space_is_dumped_per_function(default_boot: str):
    # Every function the scan found gets its space read. `len` is what came
    # back, and a record holding less than that says which byte the tail
    # repeats, so nothing is lost by leaving it out.
    spaces = {
        space["fn"]: space for space in dumped(default_boot, "config") if "hex" in space
    }
    assert spaces.keys() == {device["fn"] for device in dumped(default_boot, "device")}
    for space in spaces.values():
        read = len(space["hex"]) // 2
        assert read <= space["len"], space["fn"]
        assert read == space["len"] or "fill" in space, space["fn"]


def test_unimplemented_space_is_not_spelled_out(default_boot: str):
    # The Q35 host bridge answers all ones past its header, which as digits
    # is eight kilobytes of serial time saying nothing.
    ((bridge,)) = [
        space
        for space in dumped(default_boot, "config")
        if space.get("fn") == "0000:00:00.0" and "hex" in space
    ]
    assert bridge["len"] == 0x1000
    assert int(bridge["fill"], 16) == 0xFF
    assert len(bridge["hex"]) // 2 < bridge["len"]


def test_the_host_bridge_data_registers_are_left_alone(default_boot: str):
    # Reading one reaches whatever register firmware last selected through
    # its index half, so the dump substitutes zeros and says where.
    skipped = [
        space["at"]
        for space in dumped(default_boot, "config")
        if space.get("read") is False
    ]
    assert [int(offset, 16) for offset in skipped] == [0x64, 0xBC, 0xC8]


def test_an_enabled_iommu_reports_its_registers(machine_check_error_boot: str):
    # Firmware enabled this one, so its register block is mapped and the
    # control register reads back rather than the dump refusing.
    registers = {
        register["name"]: register["value"]
        for register in dumped(machine_check_error_boot, "iommu")
        if "value" in register
    }
    assert "CONTROL" in registers
    assert "EXTENDED_FEATURE" in registers


def test_a_disabled_iommu_is_not_read(amd_iommu_boot: str):
    # The enable bit in the capability is what says the register block is
    # mapped at all, so reading it anyway is how a dump hangs a machine.
    ((found,)) = dumped(amd_iommu_boot, "iommu")
    assert found["enabled"] is False


def test_the_firmware_names_itself(default_boot: str):
    # Which firmware built the platform is the first thing a hardware run
    # needs, since every quirk below is that build's.
    ((uefi,)) = dumped(default_boot, "uefi")
    assert uefi["vendor"]
    assert uefi["spec"]


def test_smbios_structures_are_dumped_whole(default_boot: str):
    # Type 0 is the BIOS information structure and type 127 ends the table,
    # so both present means the walk ran the length of it.
    types = {
        structure["type"]
        for structure in dumped(default_boot, "smbios")
        if "type" in structure
    }
    assert {0, 1, 127} <= types


def test_smbios_carries_its_strings(default_boot: str):
    # A structure's text sits after its formatted part, so a record holding
    # only `len` bytes would drop the vendor and version names with it.
    ((bios,)) = [
        structure
        for structure in dumped(default_boot, "smbios")
        if structure.get("type") == 0
    ]
    assert len(bios["hex"]) > 2 * bios["len"]
    assert bytes.fromhex(bios["hex"]).endswith(b"\x00\x00")


def test_tpm_registers_are_read_under_the_active_layout(tpm_tis_boot: str):
    # CRB and FIFO decode the same window differently, so reading the wrong
    # set is how a CRB TPM once reported itself as a 1.2 TIS one. Only the
    # FIFO layout is reachable here: the CRB boot carries no `drtm.conf`,
    # which is what makes it the text-mode fixture.
    registers = {
        register["name"]: int(register["value"], 16)
        for register in dumped(tpm_tis_boot, "tpm")
        if "value" in register
    }
    assert "LOC_STATE" not in registers
    assert registers["ACCESS"] & 0x80, "TPM_ACCESS never reads valid clear"


def test_tpm_registers_are_skipped_without_a_tpm(default_boot: str):
    # Nothing answers at locality 0 here, so the window holds no registers
    # to read and reading it anyway is a fault on some platforms.
    assert dumped(default_boot, "tpm") == [{"rec": "tpm", "found": False}]


def test_the_firmware_inventory_is_reported_when_absent(default_boot: str):
    # This firmware publishes no ESRT, so the branch saying so is the only
    # one reachable here. A board carrying an inventory is hardware's to
    # show, and the entries go in as bytes for exactly that run.
    assert dumped(default_boot, "esrt") == [{"rec": "esrt", "found": False}]


def test_the_psp_registers_the_table_names_are_read(mmconfig_above_4gib_boot: str):
    # Each structure in the ASPT names register offsets, and the dump reads
    # every one it knows the shape of. The mailbox is the one register here
    # that no structure names, so it comes from the feature register instead.
    registers = {
        register["name"]: register
        for register in dumped(mmconfig_above_4gib_boot, "psp")
        if "name" in register
    }
    assert registers.keys() == {
        "FEATURE (C2PMSG_63)",
        "INT_ENABLE",
        "INT_STATUS",
        "SEV_CMD_RESP (C2PMSG_32)",
        "SEV_CMD_BUF_LO (C2PMSG_56)",
        "SEV_CMD_BUF_HI (C2PMSG_57)",
        "ACPI_CMD_RESP (C2PMSG_22)",
        "DRTM_CMD_RESP (C2PMSG_72)",
    }
    for register in registers.values():
        assert "value" in register, register["name"]
    assert int(registers["FEATURE (C2PMSG_63)"]["at"], 16) == 0x9FC
    assert int(registers["DRTM_CMD_RESP (C2PMSG_72)"]["at"], 16) == 0xA20


def test_the_mailbox_numbers_come_from_the_table(mmconfig_above_4gib_boot: str):
    # Each number is the distance from the feature register the table names,
    # so a platform declaring other offsets is numbered from its own. The
    # interrupt registers sit below the mailbox block, so they carry none.
    registers = {
        register["name"]: int(register["at"], 16)
        for register in dumped(mmconfig_above_4gib_boot, "psp")
        if "name" in register
    }
    feature = registers["FEATURE (C2PMSG_63)"]
    block = feature - 63 * 4
    for name, offset in registers.items():
        number = re.search(r"C2PMSG_(\d+)\)$", name)
        if number is None:
            assert offset < block, name
        else:
            assert offset == block + int(number.group(1)) * 4, name


def test_the_drtm_argument_registers_are_left_alone(mmconfig_above_4gib_boot: str):
    # C2PMSG_93 to 95 hold a command's response, and issuing a command is a
    # write. Reading them before one would report a value that means nothing.
    offsets = {
        int(register["at"], 16)
        for register in dumped(mmconfig_above_4gib_boot, "psp")
        if "at" in register
    }
    assert not offsets & {0xA74, 0xA78, 0xA7C}


def test_the_declared_register_block_is_reported(mmconfig_above_4gib_boot: str):
    # Where the registers are and how far they run, which is what says
    # whether an offset the table names is inside them.
    ((block,)) = [
        record for record in dumped(mmconfig_above_4gib_boot, "psp") if "base" in record
    ]
    assert int(block["base"], 16) == 0xFEDA_0000
    assert block["pages"] == 1
    assert block["structures"] == 3


def test_no_psp_registers_are_read_without_a_secure_processor(default_boot: str):
    # Nothing answers with a Secure Processor ID here, so an ASPT alone would
    # have the dump reading an address with nothing behind it.
    assert dumped(default_boot, "psp") == [{"rec": "psp", "found": False}]


def test_no_psp_registers_are_read_without_the_table(machine_check_error_boot: str):
    # The Secure Processor is there and its registers answer, but only the
    # table says at which address, and this boot publishes none.
    assert dumped(machine_check_error_boot, "psp") == [{"rec": "psp", "table": False}]
