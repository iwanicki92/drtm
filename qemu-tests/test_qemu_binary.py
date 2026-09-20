# SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>
#
# SPDX-License-Identifier: BSD-3-Clause

"""The check that the suite runs against our QEMU build, with this suite's
options. The stand-in binaries that exercise the probe itself are tested
where it lives, in drtm-tests."""

from machine import unsupported_binary


def test_the_configured_binary_is_ours():
    assert unsupported_binary() is None
