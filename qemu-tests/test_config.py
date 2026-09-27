# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""Exercises `drtm.conf`, the file beside the binary that sets the log level.

No boot exists for these alone. Two boots whose tests only read results,
which every level prints, carry the config states the others cannot:
`tpm_crb_boot` has no file, `amd_iommu_enabled_boot` a bad one.
"""

from conftest import CHECKS_COMPLETE, completed, results, verdicts


def test_without_a_config_the_scan_is_not_printed(tpm_crb_boot: str):
    # The default is `info`, so the per-device dump stays out of the way.
    assert "[SCAN]" not in tpm_crb_boot
    assert CHECKS_COMPLETE in tpm_crb_boot


def test_without_a_config_every_check_still_reports(tpm_crb_boot: str):
    assert verdicts(tpm_crb_boot, "tpm.present") == ["PASS"]
    assert verdicts(tpm_crb_boot, "amd.iommu") == ["WARN"]


def test_a_missing_config_is_not_a_warning(tpm_crb_boot: str):
    assert verdicts(tpm_crb_boot, "config") == []


def test_raising_the_level_prints_the_scan(default_boot: str):
    assert "[SCAN] PCI root bridges: 1" in default_boot


def test_an_unknown_level_is_reported_with_its_line(amd_iommu_enabled_boot: str):
    # The value and where it sits are what someone fixing the file needs.
    (tag, detail), _ = results(amd_iommu_enabled_boot, "config")
    assert tag == "WARN"
    assert "line 2" in detail
    assert "verbose" in detail


def test_launch_foreign_needs_a_build_that_embeds_one(amd_iommu_enabled_boot: str):
    _, (tag, detail) = results(amd_iommu_enabled_boot, "config")
    assert tag == "WARN"
    assert "line 3" in detail
    assert "DRTM_FOREIGN_SL" in detail


def test_an_unknown_level_falls_back_to_the_default(amd_iommu_enabled_boot: str):
    assert "[SCAN]" not in amd_iommu_enabled_boot
    assert completed(amd_iommu_enabled_boot)
