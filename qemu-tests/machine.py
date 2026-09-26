# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""The machine each boot of `drtm.efi` runs on, as QEMU options, and the
scratch ESP it boots from.
"""

import shutil
from pathlib import Path

from drtmtest import qemu_vm

# QEMU CPU model used unless a caller asks for another. AMD, so the guest
# reads AuthenticAMD and runs the AMD checks rather than skipping them. No
# QEMU model carries the SKINIT feature bit, so it has to be asked for.
DEFAULT_CPU = "EPYC"
SKINIT_CPU = f"{DEFAULT_CPU},+skinit"

# IA32_MCi_STATUS with VAL and EN set and UC clear: a corrected error, which
# QEMU logs in the bank without raising #MC.
CORRECTED_ERROR = 0x9000_0000_0000_0000

# MSR 0xC0010058 as firmware would leave it, which nothing in this guest
# programs, so our QEMU takes its reset value from the CPU model. Bit 0
# enables the window, bits 5:2 hold log2 of the bus count, and bits 51:20
# the base. Both values cover 256 buses, one below 4 GiB and one crossing it.
MMCONFIG_BELOW_4GIB = f"{SKINIT_CPU},mmio-conf-base=0xe0000021"

# The window Dasharo maps through PCIEXBAR and publishes in its MCFG, 256
# buses from 0xb0000000, so the MSR and the table agree on this one boot.
MMCONFIG_AS_MAPPED = f"{SKINIT_CPU},mmio-conf-base=0xb0000021"

# CPUID family 1Ah model 02h, an EPYC 9015, whose PPR the platform dump's
# AMD MSR list is taken from. `EPYC` alone is family 17h, so without this
# nothing here ever reaches that list. Both are upstream CPU properties,
# folded into the extended fields the same way Linux reads them back, so
# this asks nothing of our build.
FAMILY_1AH = "family=26,model=2"

# MSR 0xC0010114 as Turin firmware leaves it: LOCK set with SVMDIS clear,
# so SVM stays usable and the disable is frozen. The launch bits DPD, R_INIT
# and DIS_A20M are what SKINIT sets and the launched kernel clears on every
# processor. Both are reset values only our QEMU takes, as the window's is.
VM_CR_LOCKED = "vm-cr=0x8"
VM_CR_AFTER_LAUNCH = f"{DEFAULT_CPU},vm-cr=0x7"
MMCONFIG_ABOVE_4GIB = (
    f"{DEFAULT_CPU},mmio-conf-base=0xf8000021,{FAMILY_1AH},{VM_CR_LOCKED}"
)


def _amd_iommu(enabled: bool, ivrs: bool = True) -> str:
    device = f"amd-iommu,firmware-enabled={'on' if enabled else 'off'}"
    return device if ivrs else f"{device},ivrs=off"


# What the probe at session start asks for: every option only our build
# accepts. Upstream refuses the properties and the device outright but
# drops `skinit` with a warning, which the rejects table catches.
PROBE_OPTIONS = [
    "-machine",
    "q35",
    "-cpu",
    MMCONFIG_BELOW_4GIB,
    "-device",
    _amd_iommu(enabled=True, ivrs=False),
    "-device",
    "amd-psp",
]
PROBE_REJECTS = {"skinit": "it does not report the skinit CPUID bit"}


def unsupported_binary() -> str | None:
    """Why the QEMU in use cannot run this suite, or `None` if it can."""
    return qemu_vm.unsupported_binary(PROBE_OPTIONS, PROBE_REJECTS)


# What every boot's `drtm.conf` ends with unless it launches, so a run stops
# after the checks and the dump as it did before the launch existed.
NO_LAUNCH = "launch=off\n"


def make_esp(
    scratch: Path, efi_binary: Path, config: str | None, launch: bool = False
) -> Path:
    """A scratch ESP with the binary where the removable-media path finds
    it, so it needs no boot entry, and `drtm.conf` beside it if given.

    Without `launch` the file also turns the launch off, and exists for that
    even where `config` is `None`. With it, `config` is written as given,
    and `None` leaves no file at all.
    """
    esp = scratch / "esp"
    boot_dir = esp / "EFI" / "BOOT"
    boot_dir.mkdir(parents=True)
    shutil.copy(efi_binary, boot_dir / "BOOTX64.EFI")
    if not launch:
        config = (config or "") + NO_LAUNCH
    if config is not None:
        (boot_dir / "drtm.conf").write_text(config)
    return esp


def options(
    esp: Path,
    cpu: str = DEFAULT_CPU,
    iommu: str | None = None,
    ivrs: bool = True,
    amd_drtm: bool = False,
    strict: bool = True,
    extra_root_bridge: bool = False,
    corrected_error_bank: int | None = None,
    psp: int | None = None,
    psp_ready: bool = True,
    psp_aspt: bool = True,
    psp_drtm_device: bool = False,
    psp_drtm_service: bool = False,
    vga_and_nic: bool = False,
    tpm_burst_limit: int | None = None,
) -> list[str]:
    """The machine for one boot of the ESP, without the flash, the TPM or
    the console.

    `cpu` is passed straight to `-cpu`, so it decides the vendor string and
    feature bits the guest sees.

    `iommu` attaches QEMU's `amd-iommu`, which shows up as an AMD function
    carrying the secure-device capability an IOMMU scan looks for. QEMU puts
    it at whatever slot is free, not the `0:0.2` of real silicon.
    `"disabled"` leaves the capability's enable bit clear as upstream QEMU
    does, `"enabled"` sets it the way firmware would. `ivrs=False` has our
    QEMU leave that IOMMU out of ACPI, so the guest sees no IVRS table.

    `amd_drtm` boots our QEMU's `-machine q35,amd-drtm=on`, the machine a
    launch runs on: `skinit` on every CPU, an enabled `amd-iommu` at `0:0.2`
    as function 2 of the host bridge, and AMD's northbridge at `0:18.0`.
    Not with `iommu`, since QEMU takes one IOMMU per machine. The platform
    device runs strict, so a broken launch rule stops the VM as a panic
    rather than a logged line, and guest errors go to `qemu.log`: a checker
    that only observes must trip neither. Two processors, so the machine
    has an AP the way a launch machine does, and no reboot, so a launch
    that triple-faults stops QEMU rather than booting drtm again.
    `strict=False` logs a broken rule and carries on, for a launch expected
    to break one.

    `extra_root_bridge` adds a second PCI host bridge (`pxb-pcie`) with a
    root port on bus 0x20, the way a server has one per NUMA node.

    `psp` attaches our QEMU's `amd-psp`, a function answering with that AMD
    device ID and its mailbox registers behind it. `psp_ready` sets the bit
    the DRTM service sets when it will take a command, `psp_aspt` leaves
    the ACPI table naming those registers' address out, and `psp_drtm_device`
    publishes the SSDT describing the DRTM ACPI device. `psp_drtm_service`
    runs the DRTM service behind the mailbox, which locks TPM localities 1
    to 4 from reset until a PSP launch, as Turin firmware leaves them.

    `tpm_burst_limit` has our QEMU's TIS FIFO move that many bytes at most
    before its burst count reads 0 once, as a FIFO TPM refilling does. A
    byte past the burst count still moves, as hardware would after wait
    states, and its `tpm_tis_burst_wait` trace names the locality.

    `vga_and_nic` keeps QEMU's default display adapter and network card.
    They cost 1.3s of firmware initialisation each boot, so only the boot
    that wants a populated bus to scan asks for them.

    `corrected_error_bank` names a machine-check bank to hold a corrected
    error from reset on. The machine starts paused for it, and `inject`
    logs the error and lets it run.
    """
    if iommu not in (None, "disabled", "enabled"):
        raise ValueError(f"iommu must be None, 'disabled' or 'enabled', got {iommu!r}")
    if amd_drtm and iommu is not None:
        raise ValueError("amd_drtm brings its own IOMMU, so iommu must be None")
    args = [
        "-machine",
        "q35,smm=on" + (",amd-drtm=on" if amd_drtm else ""),
        "-cpu",
        cpu,
        "-m",
        "1G",
        "-global",
        "driver=cfi.pflash01,property=secure,value=off",
        "-global",
        "ICH9-LPC.disable_s3=1",
        "-drive",
        f"file=fat:rw:{esp},format=raw",
    ]
    if not vga_and_nic:
        args += ["-vga", "none", "-nic", "none"]
    if amd_drtm:
        args += [
            "-global",
            f"amd-drtm-platform.strict={'on' if strict else 'off'}",
            "-d",
            "guest_errors",
            "-smp",
            "2",
            "-no-reboot",
        ]
    if iommu:
        args += ["-device", _amd_iommu(iommu == "enabled", ivrs)]
    if extra_root_bridge:
        args += [
            "-device",
            "pxb-pcie,id=pxb1,bus_nr=0x20",
            "-device",
            "pcie-root-port,id=rp1,bus=pxb1,chassis=1",
        ]
    if psp is not None:
        device = f"amd-psp,device-id={psp:#x}"
        device += f",drtm-ready={'on' if psp_ready else 'off'}"
        device += f",aspt={'on' if psp_aspt else 'off'}"
        device += f",drtm-device={'on' if psp_drtm_device else 'off'}"
        device += f",drtm-service={'on' if psp_drtm_service else 'off'}"
        args += ["-device", device]
    if tpm_burst_limit is not None:
        args += [
            "-global",
            f"tpm-tis.burst-limit={tpm_burst_limit}",
            "-trace",
            "tpm_tis_burst_wait",
        ]
    if corrected_error_bank is not None:
        args.append("-S")
    return args


def inject_corrected_error(vm: qemu_vm.QemuVm, bank: int) -> None:
    """Logs a corrected error in `bank` of CPU 0 and lets the paused VM run."""
    qmp = vm.wait_for_qmp()
    # mce <cpu> <bank> <status> <mcg_status> <addr> <misc>
    qmp.human_monitor_command(f"mce 0 {bank} {CORRECTED_ERROR:#x} 0 0 0")
    qmp.cont()
