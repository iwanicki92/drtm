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
import time
from pathlib import Path

from firmware import cached_firmware_path
from qmp_client import QmpClient

# The variable store lives in the first bytes of the firmware image; zeroing
# it resets settings (including boot order) to defaults on every run, same as
# Dasharo's own scripts/ci/qemu-run.sh does.
_VARSTORE_ZERO_BYTES = 256


def _free_tcp_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def _kvm_available() -> bool:
    return os.access("/dev/kvm", os.R_OK | os.W_OK)


class QemuVm:
    """Runs one UEFI binary as the only boot entry on a scratch ESP.

    `log_dir` receives the QEMU process's own stdout/stderr (`qemu.log`) and
    the raw serial console capture (`serial.log`), written as they arrive so
    they're useful even if the test times out or crashes.
    """

    def __init__(self, efi_binary: Path, log_dir: Path):
        self.efi_binary = Path(efi_binary)
        self.log_dir = Path(log_dir)
        self._workdir = None
        self._process = None
        self._serial_sock = None
        self._serial_log_f = None
        self._qemu_log_f = None
        self.qmp = None

    def __enter__(self):
        self._workdir = tempfile.mkdtemp(prefix="drtm-qemu-")
        self._serial_log_f = open(self.log_dir / "serial.log", "wb")
        self._qemu_log_f = open(self.log_dir / "qemu.log", "wb")
        esp_dir = Path(self._workdir) / "esp" / "EFI" / "BOOT"
        esp_dir.mkdir(parents=True)
        shutil.copy(self.efi_binary, esp_dir / "BOOTX64.EFI")

        firmware = Path(self._workdir) / "firmware.rom"
        shutil.copy(cached_firmware_path(), firmware)
        with open(firmware, "r+b") as f:
            f.write(b"\x00" * _VARSTORE_ZERO_BYTES)

        serial_port = _free_tcp_port()
        # QMP is a Unix socket. AF_UNIX paths are capped at ~108 bytes, so it
        # cannot live under the (potentially long) per-test workdir. Reserve
        # a short, unique path with mkstemp, then hand QEMU the bare path
        # (it creates the actual socket there itself).
        qmp_fd, qmp_sock = tempfile.mkstemp(prefix="drtm-qmp-", suffix=".sock")
        os.close(qmp_fd)
        os.unlink(qmp_sock)

        args = [
            "qemu-system-x86_64",
            "-machine",
            "q35,smm=on",
            "-cpu",
            "Skylake-Client",
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
            f"tcp:127.0.0.1:{serial_port},server,nowait",
            "-qmp",
            f"unix:{qmp_sock},server,nowait",
            "-nographic",
        ]
        if _kvm_available():
            args.append("-enable-kvm")

        self._process = subprocess.Popen(
            args,
            stdout=self._qemu_log_f,
            stderr=subprocess.STDOUT,
        )
        self._qmp_sock_path = qmp_sock
        self.qmp = QmpClient(qmp_sock)
        self._serial_sock = self._connect_serial(serial_port)
        return self

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

    def read_serial_until(self, needle: str, timeout: float = 40.0) -> str:
        """Reads from the serial console until `needle` appears, or raises
        TimeoutError with everything read so far."""
        assert self._serial_sock is not None, (
            "read_serial_until() outside a `with` block"
        )
        assert self._serial_log_f is not None, (
            "read_serial_until() outside a `with` block"
        )
        needle_bytes = needle.encode()
        buf = b""
        self._serial_sock.settimeout(0.2)
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if needle_bytes in buf:
                return buf.decode("utf-8", "replace")
            try:
                chunk = self._serial_sock.recv(65536)
            except TimeoutError:
                continue
            if not chunk:
                break
            buf += chunk
            self._serial_log_f.write(chunk)
            self._serial_log_f.flush()
        raise TimeoutError(
            f"Timed out waiting for {needle!r} on serial console. "
            f"Read so far:\n{buf.decode('utf-8', 'replace')}"
        )

    def __exit__(self, exc_type, exc_value, traceback):
        if self._serial_sock is not None:
            self._serial_sock.close()
        if self._process is not None:
            self._process.terminate()
            try:
                self._process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self._process.kill()
                self._process.wait()
        if getattr(self, "_qmp_sock_path", None):
            Path(self._qmp_sock_path).unlink(missing_ok=True)
        if self._workdir is not None:
            shutil.rmtree(self._workdir, ignore_errors=True)
        if self._serial_log_f is not None:
            self._serial_log_f.close()
        if self._qemu_log_f is not None:
            self._qemu_log_f.close()
        return False
