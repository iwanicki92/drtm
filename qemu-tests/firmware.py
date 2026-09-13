"""Fetches and caches the Dasharo QEMU Q35 firmware image used for tests.

We use Dasharo's coreboot+UEFI build instead of stock OVMF: it is a more
feature-complete, open-source UEFI implementation that Dasharo itself already
uses to test TrenchBoot, which leaves room to grow into TPM/DRTM-adjacent
QEMU testing later without switching firmware. See CLAUDE.md.
"""

import urllib.request
from pathlib import Path

# Pinned rather than "latest" so test runs are reproducible. Bump deliberately.
FIRMWARE_RELEASE_TAG = "qemu_q35_v0.2.1"
FIRMWARE_ASSET = "qemu_q35_all_menus.rom"
FIRMWARE_URL = (
    f"https://github.com/Dasharo/coreboot/releases/download/"
    f"{FIRMWARE_RELEASE_TAG}/{FIRMWARE_ASSET}"
)

CACHE_DIR = Path(__file__).parent / "dl-cache"


def cached_firmware_path() -> Path:
    """Returns the path to the cached firmware image, downloading it if needed."""
    CACHE_DIR.mkdir(exist_ok=True)
    path = CACHE_DIR / f"{FIRMWARE_RELEASE_TAG}-{FIRMWARE_ASSET}"
    if not path.exists():
        tmp_path = path.with_suffix(".tmp")
        urllib.request.urlretrieve(FIRMWARE_URL, tmp_path)
        tmp_path.rename(path)
    return path
