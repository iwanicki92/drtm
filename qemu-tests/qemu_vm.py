# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: Apache-2.0

"""Boots a UEFI binary in QEMU under Dasharo's firmware and exposes its serial
console for tests to read.
"""

import os
import shutil
import socket
import subprocess
import tempfile
import threading
import time
from pathlib import Path

from firmware import cached_firmware_path
from qmp_client import QmpClient

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

# CPUID family 1Ah model 02h, an EPYC 9015, whose PPR the platform dump's
# AMD MSR list is taken from. `EPYC` alone is family 17h, so without this
# nothing here ever reaches that list. Both are upstream CPU properties,
# folded into the extended fields the same way Linux reads them back, so
# this asks nothing of our build.
FAMILY_1AH = "family=26,model=2"
MMCONFIG_ABOVE_4GIB = f"{DEFAULT_CPU},mmio-conf-base=0xf8000021,{FAMILY_1AH}"


def _free_tcp_port() -> int:
    # The port is free again between here and QEMU binding it, so concurrent
    # boots could in principle pick the same one. Not seen in practice; a
    # Unix socket, as QMP uses, would remove it if it ever bites.
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def _binary() -> str:
    """Which `qemu-system-x86_64` to run, honouring `DRTM_QEMU_BINARY`.

    A path, or a name to find on `PATH`. The checks that need a patched QEMU
    are why this exists, see `docs/testing.md`.
    """
    return os.environ.get("DRTM_QEMU_BINARY", "qemu-system-x86_64")


def _use_kvm() -> bool:
    """Whether to pass `-enable-kvm`, honouring `DRTM_QEMU_ACCEL`.

    `tcg` is the default because KVM measured slower on every machine tried:
    marginally so on bare metal, and about 2x on a nested-virtualisation
    host. This guest boots with `smm=on`, and nested, every SMM transition
    traps out through both hypervisor layers, while TCG emulates it in
    userspace with no exits at all.

    `kvm` forces KVM and `auto` picks it whenever `/dev/kvm` is usable, both
    kept so the comparison can be re-run on a machine where it may not hold.
    """
    accel = os.environ.get("DRTM_QEMU_ACCEL", "tcg").lower()
    if accel == "tcg":
        return False
    if accel == "kvm":
        return True
    if accel != "auto":
        raise ValueError(
            f"DRTM_QEMU_ACCEL must be 'auto', 'kvm' or 'tcg', got {accel!r}"
        )
    return os.access("/dev/kvm", os.R_OK | os.W_OK)


def _amd_iommu(enabled: bool, ivrs: bool = True) -> str:
    device = f"amd-iommu,firmware-enabled={'on' if enabled else 'off'}"
    return device if ivrs else f"{device},ivrs=off"


# Starting paused and quitting takes well under a second.
_PROBE_TIMEOUT = 30.0

# The firmware's boot prompt, and how long to allow for it. It is the first
# thing on the serial console, so the allowance is only against a machine
# under load.
_BOOT_PROMPT = "ENTER to boot directly"
_BOOT_PROMPT_TIMEOUT = 60.0


def unsupported_binary() -> str | None:
    """Why the QEMU in use cannot run this suite, or `None` if it can.

    Starts it paused with every option only our build accepts. Upstream
    refuses the properties and the device outright but drops `skinit` with
    a warning.
    """
    args = [
        _binary(),
        "-machine",
        "q35",
        "-accel",
        "tcg",
        "-nodefaults",
        "-display",
        "none",
        "-S",
        "-monitor",
        "stdio",
        "-cpu",
        MMCONFIG_BELOW_4GIB,
        "-device",
        _amd_iommu(enabled=True, ivrs=False),
        "-device",
        "amd-psp",
    ]
    try:
        probe = subprocess.run(
            args,
            input="quit\n",
            check=False,
            capture_output=True,
            text=True,
            timeout=_PROBE_TIMEOUT,
        )
    except FileNotFoundError:
        return f"{_binary()} was not found"
    except subprocess.TimeoutExpired:
        return f"{_binary()} did not quit within {_PROBE_TIMEOUT:.0f}s"
    if probe.returncode != 0:
        lines = probe.stderr.strip().splitlines()
        return lines[-1] if lines else f"it exited with {probe.returncode}"
    if "skinit" in probe.stderr:
        return "it does not report the skinit CPUID bit"
    return None


class QemuVm:
    """Runs one UEFI binary as the only boot entry on a scratch ESP.

    `log_dir` receives the QEMU process's own stdout/stderr (`qemu.log`) and
    the raw serial console capture (`serial.log`), written as they arrive so
    they're useful even if the test times out or crashes.

    A background thread drains the serial console for the whole life of the
    `with` block, independent of what `read_serial_until` is waiting for, so
    `serial.log` captures everything the guest printed up to teardown, not
    just up to the first matched string.

    `tpm` attaches an emulated TPM 2.0 backed by `swtpm`, as either a
    `tpm-tis` (FIFO) or `tpm-crb` device. Leave it `None` for no TPM at all,
    which is what the platform looks like to a guest on a machine without
    one.

    `cpu` is passed straight to `-cpu`, so it decides the vendor string and
    feature bits the guest sees. Any name `qemu-system-x86_64 -cpu help`
    lists works.

    `iommu` attaches QEMU's `amd-iommu`, which shows up as an AMD function
    carrying the secure-device capability an IOMMU scan looks for. QEMU puts
    it at whatever slot is free, not the `0:0.2` of real silicon.
    `"disabled"` leaves the capability's enable bit clear as upstream QEMU
    does, `"enabled"` sets it the way firmware would.

    `amd_drtm` boots our QEMU's `-machine q35,amd-drtm=on`, the machine a
    launch runs on: `skinit` on every CPU, an enabled `amd-iommu` at `0:0.2`
    as function 2 of the host bridge, and AMD's northbridge at `0:18.0`.
    Not with `iommu`, since QEMU takes one IOMMU per machine.

    `ivrs=False` has our QEMU leave that IOMMU out of ACPI, so the guest sees
    no IVRS table at all.

    `extra_root_bridge` adds a second PCI host bridge (`pxb-pcie`) with a
    root port on bus 0x20, the way a server has one per NUMA node.

    `psp` attaches our QEMU's `amd-psp`, a function answering with that AMD
    device ID and its mailbox registers behind it. `psp_ready` sets the bit
    the DRTM service sets when it will take a command, `psp_aspt` leaves
    the ACPI table naming those registers' address out, and `psp_drtm_device`
    publishes the SSDT describing the DRTM ACPI device.

    `vga_and_nic` keeps QEMU's default display adapter and network card.
    They cost 1.3s of firmware initialisation each boot, so only the boot
    that wants a populated bus to scan asks for them.

    `corrected_error_bank` names a machine-check bank to hold a corrected
    error from reset on. QEMU starts paused, logs it, then runs.

    `config` is written beside the binary as `drtm.conf`. Leave it `None` for
    no file at all, which is what a stick carrying only the binary looks like.

    `firmware` is the image to boot, copied into the working directory so
    the guest's writes to its variable store land on the copy. Defaults to
    the downloaded release.

    `save_firmware_to` keeps that copy, after QEMU has exited and only if
    the block left without an exception. It is how a warmed image is made.
    """

    def __init__(
        self,
        efi_binary: Path,
        log_dir: Path,
        tpm: str | None = None,
        cpu: str = DEFAULT_CPU,
        iommu: str | None = None,
        ivrs: bool = True,
        amd_drtm: bool = False,
        config: str | None = None,
        extra_root_bridge: bool = False,
        corrected_error_bank: int | None = None,
        psp: int | None = None,
        psp_ready: bool = True,
        psp_aspt: bool = True,
        psp_drtm_device: bool = False,
        vga_and_nic: bool = False,
        firmware: Path | None = None,
        save_firmware_to: Path | None = None,
    ):
        if tpm not in (None, "tis", "crb"):
            raise ValueError(f"tpm must be None, 'tis' or 'crb', got {tpm!r}")
        if iommu not in (None, "disabled", "enabled"):
            raise ValueError(
                f"iommu must be None, 'disabled' or 'enabled', got {iommu!r}"
            )
        if amd_drtm and iommu is not None:
            raise ValueError("amd_drtm brings its own IOMMU, so iommu must be None")
        self.efi_binary = Path(efi_binary)
        self.log_dir = Path(log_dir)
        self.tpm = tpm
        self.cpu = cpu
        self.iommu = iommu
        self.ivrs = ivrs
        self.amd_drtm = amd_drtm
        self.config = config
        self.extra_root_bridge = extra_root_bridge
        self.corrected_error_bank = corrected_error_bank
        self.psp = psp
        self.psp_ready = psp_ready
        self.psp_aspt = psp_aspt
        self.psp_drtm_device = psp_drtm_device
        self.vga_and_nic = vga_and_nic
        self.firmware = Path(firmware) if firmware else cached_firmware_path()
        self.save_firmware_to = save_firmware_to
        self._firmware_copy = None
        self._workdir = None
        self._process = None
        self._swtpm_process = None
        self._swtpm_log_f = None
        self._swtpm_sock_path = None
        self._serial_sock = None
        self._serial_log_f = None
        self._qemu_log_f = None
        self._serial_buf = bytearray()
        self._serial_buf_lock = threading.Lock()
        self._serial_reader = None
        self._stop_reading = threading.Event()
        self._qmp_sock_path = None
        self.qmp = None

    def __enter__(self):
        self._workdir = tempfile.mkdtemp(prefix="drtm-qemu-")
        self._serial_log_f = open(self.log_dir / "serial.log", "wb")
        self._qemu_log_f = open(self.log_dir / "qemu.log", "wb")
        esp_dir = Path(self._workdir) / "esp" / "EFI" / "BOOT"
        esp_dir.mkdir(parents=True)
        shutil.copy(self.efi_binary, esp_dir / "BOOTX64.EFI")
        if self.config is not None:
            (esp_dir / "drtm.conf").write_text(self.config)

        # The variable store lives in the first bytes of this image. It is
        # left as it comes, so a populated store survives: the binary boots
        # as removable media regardless of what boot order it holds.
        firmware = Path(self._workdir) / "firmware.rom"
        shutil.copy(self.firmware, firmware)
        self._firmware_copy = firmware

        serial_port = _free_tcp_port()
        # QMP is a Unix socket. AF_UNIX paths are capped at ~108 bytes, so it
        # cannot live under the (potentially long) per-test workdir. Reserve
        # a short, unique path with mkstemp, then hand QEMU the bare path
        # (it creates the actual socket there itself).
        qmp_fd, qmp_sock = tempfile.mkstemp(prefix="drtm-qmp-", suffix=".sock")
        os.close(qmp_fd)
        os.unlink(qmp_sock)

        args = [
            _binary(),
            "-machine",
            "q35,smm=on" + (",amd-drtm=on" if self.amd_drtm else ""),
            "-cpu",
            self.cpu,
            "-m",
            "1G",
            "-global",
            "driver=cfi.pflash01,property=secure,value=off",
            "-drive",
            f"if=pflash,format=raw,unit=0,file={firmware}",
            "-global",
            "ICH9-LPC.disable_s3=1",
            "-drive",
            f"file=fat:rw:{self._workdir}/esp,format=raw",
            "-serial",
            f"tcp:127.0.0.1:{serial_port},server",
            "-qmp",
            f"unix:{qmp_sock},server,nowait",
            "-nographic",
        ]
        if not self.vga_and_nic:
            args += ["-vga", "none", "-nic", "none"]
        if self.tpm is not None:
            self._swtpm_log_f = open(self.log_dir / "swtpm.log", "wb")
            swtpm_sock = self._start_swtpm()
            args += [
                "-chardev",
                f"socket,id=chrtpm,path={swtpm_sock}",
                "-tpmdev",
                "emulator,id=tpm0,chardev=chrtpm",
                "-device",
                f"tpm-{self.tpm},tpmdev=tpm0",
            ]

        if self.iommu:
            args += ["-device", _amd_iommu(self.iommu == "enabled", self.ivrs)]

        if self.extra_root_bridge:
            args += [
                "-device",
                "pxb-pcie,id=pxb1,bus_nr=0x20",
                "-device",
                "pcie-root-port,id=rp1,bus=pxb1,chassis=1",
            ]

        if self.psp is not None:
            device = f"amd-psp,device-id={self.psp:#x}"
            device += f",drtm-ready={'on' if self.psp_ready else 'off'}"
            device += f",aspt={'on' if self.psp_aspt else 'off'}"
            device += f",drtm-device={'on' if self.psp_drtm_device else 'off'}"
            args += ["-device", device]

        if self.corrected_error_bank is not None:
            args.append("-S")

        if _use_kvm():
            args.append("-enable-kvm")

        # Record the exact invocation next to its output, so runs compared
        # against each other (KVM vs TCG, say) show what actually differed.
        (self.log_dir / "qemu-args.txt").write_text("\n".join(args) + "\n")

        self._process = subprocess.Popen(
            args,
            stdout=self._qemu_log_f,
            stderr=subprocess.STDOUT,
        )
        self._qmp_sock_path = qmp_sock
        self.qmp = QmpClient(qmp_sock)
        # Before anything lets the guest run. QEMU's serial server waits for
        # this connection rather than dropping what it would have printed
        # first, and the boot prompt is the first thing out.
        self._serial_sock = self._connect_serial(serial_port)
        self._serial_sock.settimeout(0.2)
        self._serial_reader = threading.Thread(
            target=self._read_serial_loop, daemon=True
        )
        self._serial_reader.start()
        if self.corrected_error_bank is not None:
            self._wait_for_qmp(qmp_sock)
            # mce <cpu> <bank> <status> <mcg_status> <addr> <misc>
            self.qmp.human_monitor_command(
                f"mce 0 {self.corrected_error_bank} {CORRECTED_ERROR:#x} 0 0 0"
            )
            self.qmp.cont()
        self._skip_boot_menu()
        return self

    def _skip_boot_menu(self) -> None:
        """Answers the boot prompt with ENTER, which boots the default entry
        at once instead of after the boot manager's 3s timeout.

        Not reaching the prompt is an error rather than something to boot
        through: the firmware release is pinned, so it either prints this or
        something changed that is worth seeing.
        """
        assert self._serial_sock is not None
        self.read_serial_until(_BOOT_PROMPT, timeout=_BOOT_PROMPT_TIMEOUT)
        self._serial_sock.sendall(b"\r")

    def _start_swtpm(self) -> str:
        """Starts `swtpm` on a Unix socket and returns that socket's path.

        QEMU's `emulator` TPM backend drives swtpm over its control channel,
        so the same socket serves both. Follows Dasharo's own recipe in
        `Dasharo/open-source-firmware-validation:scripts/ci/qemu-run.sh`.
        """
        assert self._workdir is not None
        state_dir = Path(self._workdir) / "swtpm-state"
        state_dir.mkdir()
        # Same AF_UNIX path length limit as QMP below, so reserve a short
        # unique path instead of one under the long per-test workdir.
        fd, sock = tempfile.mkstemp(prefix="drtm-swtpm-", suffix=".sock")
        os.close(fd)
        os.unlink(sock)

        swtpm_args = [
            "swtpm",
            "socket",
            "--tpm2",
            "--tpmstate",
            f"dir={state_dir}",
            "--ctrl",
            f"type=unixio,path={sock}",
        ]
        # Errors go to stderr regardless of --log, so an empty swtpm.log
        # means a clean run. --log adds debug tracing only: level 5+ enables
        # libtpms logging, 20 dumps every command (~38K per boot).
        level = os.environ.get("DRTM_SWTPM_LOG_LEVEL")
        if level:
            swtpm_args += ["--log", f"fd=1,level={level}"]

        assert self._swtpm_log_f is not None
        self._swtpm_process = subprocess.Popen(
            swtpm_args,
            stdout=self._swtpm_log_f,
            stderr=subprocess.STDOUT,
        )
        self._swtpm_sock_path = sock

        deadline = time.monotonic() + 5.0
        while time.monotonic() < deadline:
            if Path(sock).exists():
                return sock
            if self._swtpm_process.poll() is not None:
                raise RuntimeError(
                    f"swtpm exited with {self._swtpm_process.returncode} before "
                    f"creating its socket, see {self.log_dir / 'swtpm.log'}"
                )
            time.sleep(0.05)
        raise TimeoutError(f"swtpm never created its control socket at {sock}")

    def _wait_for_qmp(self, path, timeout=5.0):
        deadline = time.monotonic() + timeout
        while not os.path.exists(path):
            if time.monotonic() >= deadline:
                raise TimeoutError("QEMU never created its QMP socket")
            time.sleep(0.05)

    def _connect_serial(self, port, timeout=5.0):
        deadline = time.monotonic() + timeout
        last_error = None
        while time.monotonic() < deadline:
            try:
                return socket.create_connection(("127.0.0.1", port), timeout=1)
            except ConnectionRefusedError as e:
                last_error = e
                time.sleep(0.1)
        raise TimeoutError(f"QEMU never opened its serial port: {last_error}")

    def _read_serial_loop(self):
        """Runs in a background thread for the life of the `with` block,
        continuously draining the serial socket into `serial.log` and the
        in-memory buffer `read_serial_until` polls."""
        assert self._serial_sock is not None
        assert self._serial_log_f is not None
        while not self._stop_reading.is_set():
            try:
                chunk = self._serial_sock.recv(65536)
            except TimeoutError:
                continue
            except OSError:
                # Socket closed from __exit__ while recv() was blocked.
                break
            if not chunk:
                break
            with self._serial_buf_lock:
                # __exit__ closes the log file under this same lock, so
                # re-check the stop flag here: if the join below it timed out,
                # this thread must not write to an already-closed file.
                if self._stop_reading.is_set():
                    break
                self._serial_log_f.write(chunk)
                self._serial_log_f.flush()
                self._serial_buf.extend(chunk)

    def read_serial_until(self, needle: str, timeout: float = 40.0) -> str:
        """Waits until `needle` appears anywhere in the serial output seen so
        far, or raises TimeoutError with everything read so far.

        The default suits a wait for something the guest prints soon after it
        is already running. Waiting for a full boot needs longer: that path
        passes `conftest.BOOT_TIMEOUT`.
        """
        assert self._serial_reader is not None, (
            "read_serial_until() outside a `with` block"
        )
        needle_bytes = needle.encode()
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            with self._serial_buf_lock:
                if needle_bytes in self._serial_buf:
                    return bytes(self._serial_buf).decode("utf-8", "replace")
            time.sleep(0.05)
        with self._serial_buf_lock:
            buf = bytes(self._serial_buf)
        raise TimeoutError(
            f"Timed out waiting for {needle!r} on serial console. "
            f"Read so far:\n{buf.decode('utf-8', 'replace')}"
        )

    def __exit__(self, exc_type, exc_value, traceback):
        self._stop_reading.set()
        if self._serial_sock is not None:
            self._serial_sock.close()
        if self._serial_reader is not None:
            self._serial_reader.join(timeout=5)
        if self._process is not None:
            self._process.terminate()
            try:
                self._process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._process.kill()
                self._process.wait()
        # swtpm goes down after QEMU, so QEMU never sees its backend vanish
        # mid-teardown and log an error that looks like a real failure.
        if self._swtpm_process is not None:
            self._swtpm_process.terminate()
            try:
                self._swtpm_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._swtpm_process.kill()
                self._swtpm_process.wait()
        if self._swtpm_sock_path is not None:
            Path(self._swtpm_sock_path).unlink(missing_ok=True)
        if self._swtpm_log_f is not None:
            self._swtpm_log_f.close()
        if self._qmp_sock_path is not None:
            Path(self._qmp_sock_path).unlink(missing_ok=True)
        # After QEMU is down, so pflash writes have all landed, and only on
        # a clean exit: a boot that failed is not one to cache.
        if (
            self.save_firmware_to is not None
            and exc_type is None
            and self._firmware_copy is not None
        ):
            shutil.copy(self._firmware_copy, self.save_firmware_to)
        if self._workdir is not None:
            shutil.rmtree(self._workdir, ignore_errors=True)
        with self._serial_buf_lock:
            if self._serial_log_f is not None:
                self._serial_log_f.close()
        if self._qemu_log_f is not None:
            self._qemu_log_f.close()
        return False
