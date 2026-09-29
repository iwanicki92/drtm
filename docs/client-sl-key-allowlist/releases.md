<!--
SPDX-FileCopyrightText: 2026 iwanicki92 <iwanicki92@gmail.com>

SPDX-License-Identifier: BSD-3-Clause
-->

# Client firmware releases by SL key allowlist

Generated from a survey of public client BIOS releases on 2026-09-29: Lenovo, HP
and Dell download catalogs, LVFS, ASUS and a few MSI boards. Each release is
listed once, under the verdict of its DRTM TA. A model count such as (+2) means
the release also covers two other models. See the
[overview](../client-sl-key-allowlist.md) for what the verdicts mean.

- [Inconclusive](#inconclusive): 8 releases
- [Not scanned](#not-scanned): 15 releases
- [No allowlist](#no-allowlist): 68 releases
- [No DRTM TA](#no-drtm-ta): 136 releases
- [Allowlist](#allowlist): 593 releases

## Inconclusive

fwscan found no hardcoded digest in these DRTM TAs at all, so it cannot tell a
TA without an allowlist from one whose compare it does not recognise. Read by
hand, the TAs hash the SL key and store the digest without comparing it, so they
quite likely have no allowlist.

| Vendor | Model                       | BIOS      | Date       | DRTM TA   | Source      |
| ------ | --------------------------- | --------- | ---------- | --------- | ----------- |
| ASUS   | PRIME B550-PLUS             | [0320][1] | 2020-05-28 | 0.7.0.1   | vendor site |
| ASUS   | PRIME B550-PLUS             | [0603][2] | 2020-06-11 | 0.7.0.1   | vendor site |
| ASUS   | PRIME B550-PLUS             | [0608][3] | 2020-06-16 | 0.7.0.1   | vendor site |
| ASUS   | PRIME B550-PLUS             | [0805][4] | 2020-07-03 | 0.7.0.1   | vendor site |
| ASUS   | PRIME B550-PLUS             | [1004][5] | 2020-08-17 | 4.11.0.14 | vendor site |
| Lenovo | ThinkCentre M75q-2          | [1.51][6] | 2022-07-20 | 4.11.0.14 | LVFS        |
| Lenovo | ThinkCentre M75q-2          | [1.57][7] | 2023-08-31 | 4.11.0.14 | LVFS        |
| MSI    | MAG B550 TOMAHAWK (MS-7C91) | [10][8]   | 2020-06-04 | 0.7.0.1   | vendor site |

## Not scanned

No PSP firmware image was found in the package.

| Vendor | Model                      | BIOS         | Date       | DRTM TA | Source |
| ------ | -------------------------- | ------------ | ---------- | ------- | ------ |
| Dell   | Latitude 5495              | [1.18.0][9]  | 2024-12-09 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series     | [1.1.2][10]  | 2017-12-01 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series     | [1.2.10][11] | 2019-08-20 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series     | [1.2.12][12] | 2021-05-03 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU    | [1.0.10][13] | 2018-04-27 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU    | [1.2.7][14]  | 2019-05-31 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU    | [1.2.11][15] | 2021-05-03 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen CPU    | [1.0.1][16]  | 2017-08-23 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen CPU    | [1.0.7][17]  | 2017-09-18 | none    | LVFS   |
| Lenovo | ThinkCentre M75n-1         | [0.1.18][18] | 2024-03-27 | none    | LVFS   |
| Lenovo | ThinkPad L13 Yoga Gen4 AMD | [0.1.06][19] | 2023-06-08 | none    | LVFS   |
| Lenovo | ThinkPad L13 Yoga Gen4 AMD | [0.1.07][20] | 2023-09-21 | none    | LVFS   |
| Lenovo | ThinkPad L13 Yoga Gen4 AMD | [0.1.09][21] | 2024-07-01 | none    | LVFS   |
| Lenovo | ThinkPad L13 Yoga Gen4 AMD | [0.1.15][22] | 2024-12-23 | none    | LVFS   |
| Lenovo | ThinkPad L13 Yoga Gen4 AMD | [0.1.16][23] | 2025-02-03 | none    | LVFS   |

## No allowlist

The DRTM TA hashes the SL key and compares it with nothing, so `drtm`'s own SL
is quite likely accepted. Most of these are older releases of models whose later
releases carry the allowlist. Only the Lenovo ThinkCentre M75q-2CZN and Lenovo
ThinkCentre M75s gen2 have no allowlist in the newest release checked.
Anti-rollback may block a downgrade to one of these releases (not checked).

| Vendor | Model                            | BIOS                   | Date       | DRTM TA   | Source      |
| ------ | -------------------------------- | ---------------------- | ---------- | --------- | ----------- |
| ASUS   | PRIME B550-PLUS                  | [1202][24]             | 2020-10-27 | 4.11.0.19 | vendor site |
| ASUS   | PRIME B550-PLUS                  | [1212][25]             | 2020-11-05 | 4.11.0.19 | vendor site |
| ASUS   | PRIME B550-PLUS                  | [1216][26]             | 2020-11-25 | 4.11.0.19 | vendor site |
| ASUS   | PRIME B550-PLUS                  | [1602][27]             | 2021-01-08 | 4.11.0.1D | vendor site |
| ASUS   | PRIME B550-PLUS                  | [1804][28]             | 2021-02-09 | 4.11.0.1E | vendor site |
| ASUS   | PRIME B550-PLUS                  | [2003][29]             | 2021-03-26 | 4.11.0.1E | vendor site |
| ASUS   | PRIME B550-PLUS                  | [2006][30]             | 2021-04-08 | 4.11.0.1E | vendor site |
| ASUS   | PRIME B550-PLUS                  | [2403][31]             | 2021-06-21 | 4.11.0.21 | vendor site |
| ASUS   | PRIME B550-PLUS                  | [2407][32]             | 2021-08-04 | 4.11.0.21 | vendor site |
| ASUS   | PRIME B550-PLUS                  | [2423][33]             | 2021-08-26 | 4.11.0.21 | vendor site |
| ASUS   | PRIME B550-PLUS                  | [2604][34]             | 2022-03-07 | 4.11.0.2E | vendor site |
| ASUS   | PRIME B550-PLUS                  | [2803][35]             | 2022-04-29 | 4.11.0.2F | vendor site |
| ASUS   | PRIME B550-PLUS                  | [2806][36]             | 2022-12-12 | 4.11.0.2F | vendor site |
| ASUS   | PRIME B550-PLUS                  | [3002][37]             | 2023-03-14 | 4.11.0.30 | vendor site |
| Dell   | Inspiron 15 3525 (+1)            | [1.3.0][38]            | 2022-04-13 | 4.11.0.27 | LVFS        |
| Dell   | Inspiron 15 3525 (+1)            | [1.9.0][39]            | 2022-03-16 | 4.11.0.27 | LVFS        |
| Dell   | Inspiron 15 3525 (+1)            | [1.10.0][40]           | 2023-03-15 | 4.11.0.27 | LVFS        |
| Dell   | Inspiron 15 3525 (+1)            | [1.11.0][41]           | 2023-05-18 | 4.11.0.30 | LVFS        |
| Dell   | Inspiron 15 3525 (+1)            | [1.12.0][42]           | 2023-07-31 | 4.11.0.30 | LVFS        |
| Dell   | Inspiron 15 3525 (+2)            | [1.8.0][43]            | 2023-03-02 | 4.11.0.27 | LVFS        |
| Dell   | Inspiron 15 3525 (+2)            | [1.9.0][44]            | 2022-03-16 | 4.11.0.27 | LVFS        |
| Dell   | Inspiron 15 3525 (+2)            | [1.10.0][45]           | 2023-03-15 | 4.11.0.27 | LVFS        |
| Dell   | Inspiron 15 3525 (+2)            | [1.11.0][46]           | 2023-05-18 | 4.11.0.30 | LVFS        |
| Dell   | Inspiron 15 3525 (+2)            | [1.12.0][47]           | 2023-07-31 | 4.11.0.30 | LVFS        |
| HP     | EliteBook 645 14 inch G10 (+1)   | [V77 01.01.00][48]     |            | 4.11.0.30 | vendor site |
| HP     | EliteBook 645 14 inch G9 (+1)    | [U77/U87 01.10.00][49] |            | 4.11.0.30 | vendor site |
| HP     | EliteBook 8x5 G9 Notebook PC U82 | [0x1010c00][50]        | 2022-07-01 | 4.11.0.2F | LVFS        |
| HP     | ProBook 445 14 inch G10 (+1)     | [V78 01.02.03][51]     |            | 4.11.0.30 | vendor site |
| HP     | ProBook 445 14 inch G9 (+1)      | [U78/U88 01.10.00][52] |            | 4.11.0.30 | vendor site |
| HP     | ProBook 445 G8 (+1)              | [T78/T96 01.13.01][53] |            | 4.11.0.30 | vendor site |
| HP     | ZHAN 66 Pro A 14 G4              | [T79/T98 01.13.01][54] |            | 4.11.0.30 | vendor site |
| HP     | ZHAN 66 Pro A 14 G5              | [U79 01.10.00][55]     |            | 4.11.0.30 | vendor site |
| Lenovo | ThinkCentre M75q-2CZN            | [1.40][56]             | 2023-02-06 | 4.11.0.21 | LVFS        |
| Lenovo | ThinkCentre M75q-2CZN            | [1.41][57]             | 2023-07-12 | 4.11.0.21 | LVFS        |
| Lenovo | ThinkCentre M75s gen2            | [0.1.82][58]           | 2024-07-29 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad L13 Gen 4 AMD           | [0.1.10][59]           | 2022-06-08 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad L14 AMD / L15 AMD       | [0.1.20][60]           | 2021-07-12 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad L14 AMD / L15 AMD       | [0.1.25][61]           | 2022-03-04 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad L14 AMD / L15 AMD       | [0.1.27][62]           | 2022-06-27 | 4.11.0.2E | LVFS        |
| Lenovo | ThinkPad L14 AMD / L15 AMD       | [0.1.28][63]           | 2023-01-06 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad L14 AMD / L15 AMD       | [0.1.29][64]           | 2023-03-28 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad P15v Gen 3 AMD          | [0.1.10][65]           | 2022-10-28 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 1 AMD      | [0.1.34][66]           | 2021-07-13 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 1 AMD      | [0.1.35][67]           | 2021-07-13 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 1 AMD      | [0.1.36][68]           | 2021-07-13 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 1 AMD      | [0.1.37][69]           | 2021-07-13 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 1 AMD      | [0.1.40][70]           | 2021-07-13 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 1 AMD      | [0.1.41][71]           | 2021-07-13 | 4.11.0.2E | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 1 AMD      | [0.1.42][72]           | 2021-07-13 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 1 AMD      | [0.1.43][73]           | 2021-07-13 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 2 AMD      | [0.1.19][74]           | 2020-07-01 | 4.11.0.2F | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 2 AMD      | [0.1.21][75]           | 2020-07-01 | 4.11.0.2F | LVFS        |
| Lenovo | ThinkPad T14/P14s Gen 2 AMD      | [0.1.23][76]           | 2020-07-01 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD      | [0.1.34][77]           | 2021-07-13 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD      | [0.1.35][78]           | 2021-07-13 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD      | [0.1.36][79]           | 2021-07-13 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD      | [0.1.40][80]           | 2021-07-13 | 4.11.0.1D | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD      | [0.1.41][81]           | 2021-07-13 | 4.11.0.2E | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD      | [0.1.42][82]           | 2021-07-13 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD      | [0.1.43][83]           | 2021-07-13 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 2 AMD      | [0.1.26][84]           | 2020-07-01 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 3 AMD      | [0.1.30][85]           | 2021-07-13 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 3 AMD      | [0.1.31][86]           | 2021-07-13 | 4.11.0.30 | LVFS        |
| Lenovo | ThinkPad X13/T14s Gen 3 AMD      | [0.1.35][87]           | 2021-07-13 | 4.11.0.30 | LVFS        |
| MSI    | MAG B550 TOMAHAWK (MS-7C91)      | [16][88]               | 2021-05-25 | 4.11.0.1E | vendor site |
| MSI    | MAG B550 TOMAHAWK (MS-7C91)      | [19][89]               | 2022-04-13 | 4.11.0.2E | vendor site |
| MSI    | MAG B550 TOMAHAWK (MS-7C91)      | [1C][90]               | 2023-03-14 | 4.11.0.30 | vendor site |
| MSI    | MAG B650 TOMAHAWK WIFI (MS-7D75) | [10][91]               | 2022-09-28 | 4.11.0.2B | vendor site |

## No DRTM TA

No DRTM TA was found in the PSP firmware, so the platform quite likely does not
offer DRTM.

| Vendor | Model                       | BIOS             | Date       | DRTM TA | Source |
| ------ | --------------------------- | ---------------- | ---------- | ------- | ------ |
| Dell   | Inspiron 15 3505 (+1)       | [1.16.0][92]     | 2023-12-18 | none    | LVFS   |
| Dell   | Inspiron 15 3505 (+1)       | [1.17.0][93]     | 2023-12-18 | none    | LVFS   |
| Dell   | Inspiron 15 3505 (+1)       | [1.18.0][94]     | 2023-12-18 | none    | LVFS   |
| Dell   | Inspiron 15 3505 (+1)       | [1.25.0][95]     | 2024-04-16 | none    | LVFS   |
| Dell   | Inspiron 15 3505 (+1)       | [1.26.0][96]     | 2024-07-17 | none    | LVFS   |
| Dell   | Inspiron 15 3505 (+1)       | [1.27.0][97]     | 2024-09-13 | none    | LVFS   |
| Dell   | Inspiron 15 3505 (+1)       | [1.29.0][98]     | 2025-02-14 | none    | LVFS   |
| Dell   | Inspiron 15 3505 (+1)       | [1.30.0][99]     | 2025-03-11 | none    | LVFS   |
| Dell   | Inspiron 15 3505 (+1)       | [1.34.0][100]    | 2025-07-11 | none    | LVFS   |
| Dell   | Inspiron 15 3505 (+1)       | [1.36.0][101]    | 2026-01-15 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.16.0][102]    | 2023-10-04 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.17.0][103]    | 2023-12-18 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.18.0][104]    | 2023-12-30 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.25.0][105]    | 2024-03-07 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.26.1][106]    | 2024-04-16 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.27.0][107]    | 2024-07-17 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.28.0][108]    | 2024-09-13 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.30.0][109]    | 2025-02-14 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.31.0][110]    | 2025-03-10 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.35.0][111]    | 2025-07-11 | none    | LVFS   |
| Dell   | Inspiron 15 3515 (+1)       | [1.37.0][112]    | 2026-01-15 | none    | LVFS   |
| Dell   | Inspiron 15 3525 (+1)       | [1.8.0][113]     | 2023-03-02 | none    | LVFS   |
| Dell   | Inspiron 15 3595            | [1.5.0][114]     | 2022-03-16 | none    | LVFS   |
| Dell   | Inspiron 3585 3785          | [1.9.0][115]     | 2022-10-25 | none    | LVFS   |
| Dell   | Inspiron 3585 3785          | [1.13.0][116]    | 2023-04-21 | none    | LVFS   |
| Dell   | Inspiron 3585 3785          | [1.16.0][117]    | 2022-03-16 | none    | LVFS   |
| Dell   | Latitude 5495               | [1.8.1][118]     | 2022-06-20 | none    | LVFS   |
| Dell   | Latitude 5495               | [1.9.0][119]     | 2022-07-07 | none    | LVFS   |
| Dell   | Latitude 5495               | [1.10.0][120]    | 2022-12-23 | none    | LVFS   |
| Dell   | Latitude 5495               | [1.11.0][121]    | 2023-06-07 | none    | LVFS   |
| Dell   | Latitude 5495               | [1.13.0][122]    | 2023-08-10 | none    | LVFS   |
| Dell   | Latitude 5495               | [1.14.1][123]    | 2024-01-16 | none    | LVFS   |
| Dell   | Latitude 5495               | [1.15.0][124]    | 2024-01-29 | none    | LVFS   |
| Dell   | Latitude 5495               | [1.16.0][125]    | 2024-07-04 | none    | LVFS   |
| Dell   | Latitude 5495               | [1.17.0][126]    | 2024-11-07 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series      | [1.2.50][127]    | 2021-07-28 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series      | [1.6.2][128]     | 2022-07-01 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series      | [1.7.0][129]     | 2022-08-03 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series      | [1.8.0][130]     | 2022-10-07 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series      | [1.10.0][131]    | 2023-08-09 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series      | [1.11.0][132]    | 2023-12-20 | none    | LVFS   |
| Dell   | OptiPlex 5055 A-Series      | [1.12.0][133]    | 2024-03-04 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU     | [1.2.50][134]    | 2021-07-28 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU     | [1.6.1][135]     | 2022-06-11 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU     | [1.7.0][136]     | 2022-08-03 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU     | [1.8.0][137]     | 2022-10-12 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU     | [1.10.0][138]    | 2023-08-10 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU     | [1.11.0][139]    | 2023-12-20 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen APU     | [1.12.0][140]    | 2024-03-04 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen CPU     | [1.1.50][141]    | 2021-07-28 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen CPU     | [1.6.1][142]     | 2022-06-11 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen CPU     | [1.7.0][143]     | 2022-08-03 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen CPU     | [1.8.0][144]     | 2022-10-12 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen CPU     | [1.9.0][145]     | 2022-12-23 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen CPU     | [1.10.0][146]    | 2023-08-10 | none    | LVFS   |
| Dell   | OptiPlex 5055 Ryzen CPU     | [1.11.0][147]    | 2023-12-19 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.15.1][148]    | 2025-01-03 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.16.0][149]    | 2025-02-03 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.17.0][150]    | 2025-02-26 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.20.0][151]    | 2025-06-25 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.24.0][152]    | 2026-01-23 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.25.0][153]    | 2026-03-31 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.26.0][154]    | 2026-04-23 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.27.0][155]    | 2026-05-27 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.28.0][156]    | 2026-06-25 | none    | LVFS   |
| Dell   | Precision 7865 Tower        | [1.29.0][157]    | 2026-07-22 | none    | LVFS   |
| Dell   | PV15255                     | [1.7.0][158]     | 2026-02-10 | none    | LVFS   |
| HP     | Dev One                     | [251920384][159] | 2022-04-03 | none    | LVFS   |
| HP     | Dev One                     | [251985920][160] | 2022-06-15 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.31][161]    | 2020-11-19 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.32][162]    | 2020-11-19 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.34][163]    | 2021-07-19 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.35][164]    | 2021-08-02 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.36][165]    | 2021-12-24 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.37][166]    | 2022-04-26 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.38][167]    | 2022-07-18 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.39][168]    | 2022-12-30 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.40][169]    | 2023-04-14 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.41][170]    | 2023-06-21 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.43][171]    | 2023-12-26 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.44][172]    | 2024-03-27 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.45][173]    | 2024-06-19 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.46][174]    | 2024-10-08 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.47][175]    | 2025-01-07 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.48][176]    | 2025-03-03 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.49][177]    | 2025-05-22 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.50][178]    | 2025-11-13 | none    | LVFS   |
| Lenovo | ThinkCentre M75n            | [0.1.51][179]    | 2026-05-20 | none    | LVFS   |
| Lenovo | ThinkPad T14/P14s Gen 1 AMD | [0.1.25][180]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad T495               | [0.1.22][181]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad T495               | [0.1.25][182]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad T495               | [0.1.26][183]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad T495               | [0.1.33][184]    | 2021-03-02 | none    | LVFS   |
| Lenovo | ThinkPad T495               | [0.1.37][185]    | 2021-08-10 | none    | LVFS   |
| Lenovo | ThinkPad T495               | [0.1.38][186]    | 2021-08-10 | none    | LVFS   |
| Lenovo | ThinkPad T495               | [0.1.39][187]    | 2021-08-10 | none    | LVFS   |
| Lenovo | ThinkPad T495               | [0.1.40][188]    | 2021-08-10 | none    | LVFS   |
| Lenovo | ThinkPad T495               | [0.1.54][189]    | 2026-06-08 | none    | LVFS   |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD | [0.1.25][190]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD | [0.1.29][191]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad X13/T14s Gen 1 AMD | [0.1.30][192]    | 2021-01-04 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.20][193]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.21][194]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.22][195]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.28][196]    | 2016-07-08 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.29][197]    | 2020-12-30 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.30][198]    | 2021-08-04 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.33][199]    | 2021-08-04 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.34][200]    | 2021-08-04 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.35][201]    | 2021-08-04 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.36][202]    | 2021-08-04 | none    | LVFS   |
| Lenovo | ThinkPad X395 T495s         | [0.1.50][203]    | 2026-06-12 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.30][204]      | 2021-04-23 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.31][205]      | 2021-05-26 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.35][206]      | 2021-09-17 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.36][207]      | 2021-10-22 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.41][208]      | 2023-08-23 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.79][209]      | 2023-01-17 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.81][210]      | 2023-05-04 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.89][211]      | 2023-11-12 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.90][212]      | 2023-11-21 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.91][213]      | 2024-02-02 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.92][214]      | 2024-04-10 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.93][215]      | 2024-04-19 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.94][216]      | 2024-05-17 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.95][217]      | 2024-06-07 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.96][218]      | 2024-07-11 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.97][219]      | 2024-07-18 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.98][220]      | 2024-07-18 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.102][221]     | 2025-04-18 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.103][222]     | 2025-06-23 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.105][223]     | 2025-08-07 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.107][224]     | 2025-09-26 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.108][225]     | 2026-03-30 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.110][226]     | 2026-07-16 | none    | LVFS   |
| Lenovo | ThinkStation P620           | [1.111][227]     | 2026-08-13 | none    | LVFS   |

## Allowlist

The DRTM TA most likely has an SL key allowlist: its code compares the SL key
token's digest with the Windows AMD SL key's. So `drtm`'s own SL is expected to
be refused. This has not been tried on hardware.

| Vendor    | Model                                                           | BIOS                     | Date       | DRTM TA                         | Source      |
| --------- | --------------------------------------------------------------- | ------------------------ | ---------- | ------------------------------- | ----------- |
| AMD       | Desktop Ryzen AI MAX 300                                        | [03.03][228]             | 2026-06-10 | 4.2D.0.3A                       | LVFS        |
| AMD       | Desktop Ryzen AI MAX 300                                        | [03.04][229]             | 2026-07-27 | 4.2D.0.3A                       | LVFS        |
| ASUS      | PRIME B550-PLUS                                                 | [3202][230]              | 2023-08-02 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3205][231]              | 2023-09-04 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3404][232]              | 2023-10-24 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3405][233]              | 2024-01-08 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3607][234]              | 2024-04-03 | 4.11.0.33, 4.11.0.38            | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3611][235]              | 2024-11-04 | 4.11.0.33, 4.11.0.38            | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3621][236]              | 2025-04-01 | 4.11.0.33, 4.11.0.38            | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3631][237]              | 2025-08-14 | 4.11.0.39                       | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3634][238]              | 2025-09-22 | 4.11.0.39                       | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3636][239]              | 2026-01-27 | 4.11.0.39                       | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3644][240]              | 2026-08-27 | 4.11.0.39                       | vendor site |
| ASUS      | PRIME B550-PLUS                                                 | [3645][241]              | 2026-09-15 | 4.11.0.39                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1409][242]              | 2023-04-21 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1410][243]              | 2023-04-14 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1413][244]              | 2023-04-28 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1414][245]              | 2023-04-28 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1601][246]              | 2023-05-04 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1616][247]              | 2023-05-16 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1636][248]              | 2023-08-04 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1654][249]              | 2023-08-29 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1656][250]              | 2023-09-21 | 4.11.0.33                       | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1807][251]              | 2023-10-02 | 4.11.0.34, 4.2D.0.33            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [1811][252]              | 2023-10-16 | 4.11.0.34, 4.2D.0.33            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [2214][253]              | 2024-01-04 | 4.11.0.34, 4.2D.0.36            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [2412][254]              | 2024-01-30 | 4.11.0.38, 4.2D.0.36            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [2413][255]              | 2024-02-07 | 4.11.0.38, 4.2D.0.36            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [2613][256]              | 2024-04-18 | 4.11.0.38, 4.2D.0.37            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3014][257]              | 2024-07-23 | 4.11.0.39, 4.2D.0.39            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3024][258]              | 2024-08-07 | 4.11.0.39, 4.2D.0.39            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3035][259]              | 2024-09-12 | 4.11.0.39, 4.2D.0.39            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3042][260]              | 2024-10-22 | 4.11.0.39, 4.2D.0.39            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3057][261]              | 2024-11-06 | 4.11.0.39, 4.2D.0.39            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3067][262]              | 2024-12-18 | 4.11.0.39, 4.2D.0.39            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3201][263]              | 2025-01-16 | 4.11.0.39, 4.2D.0.39            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3208][264]              | 2025-02-27 | 4.11.0.39, 4.2D.0.39            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3222][265]              | 2025-03-07 | 4.11.0.39, 4.2D.0.39            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3263][266]              | 2025-06-18 | 4.11.0.39, 4.2D.0.3B            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3279][267]              | 2025-08-05 | 4.11.0.39, 4.2D.0.3B            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3283][268]              | 2025-09-19 | 4.11.0.39, 4.2D.0.3B            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3287][269]              | 2025-10-22 | 4.11.0.39, 4.2D.0.3B            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3602][270]              | 2025-12-02 | 4.11.0.39, 4.2D.0.3B            | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3827][271]              | 2026-02-13 | 4.11.0.39, 4.2D.0.3B, 4.2D.0.41 | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3842][272]              | 2026-03-27 | 4.11.0.39, 4.2D.0.3B, 4.2D.0.41 | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3854][273]              | 2026-04-23 | 4.11.0.39, 4.2D.0.3B, 4.2D.0.41 | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3881][274]              | 2026-06-29 | 4.11.0.39, 4.2D.0.3B, 4.2D.0.41 | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [3886][275]              | 2026-07-02 | 4.11.0.39, 4.2D.0.3B, 4.2D.0.41 | vendor site |
| ASUS      | PRIME B650-PLUS                                                 | [4004][276]              | 2026-09-21 | 4.11.0.39, 4.2D.0.3B, 4.2D.0.41 | vendor site |
| Dell      | 14 DC14255                                                      | [1.13.1][277]            | 2026-05-13 | 4.2D.0.3B                       | LVFS        |
| Dell      | 14 DC14255                                                      | [1.14.0][278]            | 2026-07-08 | 4.2D.0.3B                       | LVFS        |
| Dell      | 14 DC14255                                                      | [1.15.0][279]            | 2026-08-04 | 4.2D.0.3B                       | LVFS        |
| Dell      | DC16255_DC16256                                                 | [1.0.1][280]             | 2025-07-08 | 4.2D.0.3B                       | LVFS        |
| Dell      | DC16255_DC16256                                                 | [1.1.2][281]             | 2025-07-23 | 4.2D.0.3B                       | LVFS        |
| Dell      | DC16255_DC16256                                                 | [1.2.0][282]             | 2025-08-21 | 4.2D.0.3B                       | LVFS        |
| Dell      | DC16255_DC16256                                                 | [1.3.0][283]             | 2025-09-17 | 4.2D.0.3B                       | LVFS        |
| Dell      | DC16255_DC16256                                                 | [1.5.1][284]             | 2026-01-05 | 4.2D.0.3B                       | LVFS        |
| Dell      | DC16255_DC16256                                                 | [1.7.0][285]             | 2026-04-07 | 4.2D.0.3B                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.13.0][286]            | 2023-10-04 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.14.0][287]            | 2023-10-20 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.15.1][288]            | 2023-10-20 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.16.0][289]            | 2023-10-20 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.17.0][290]            | 2023-10-20 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.18.0][291]            | 2023-12-15 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.19.0][292]            | 2024-05-09 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.22.0][293]            | 2024-09-18 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.25.0][294]            | 2025-02-14 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.26.0][295]            | 2025-03-11 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.30.0][296]            | 2025-07-11 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.34.0][297]            | 2026-02-10 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3525 (+1)                                           | [1.36.0][298]            | 2026-08-04 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.13.0][299]            | 2023-05-18 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.14.0][300]            | 2023-10-20 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.15.1][301]            | 2023-12-15 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.16.0][302]            | 2023-12-15 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.17.0][303]            | 2023-12-15 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.18.0][304]            | 2023-12-15 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.19.0][305]            | 2024-05-09 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.22.0][306]            | 2024-09-18 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.25.0][307]            | 2025-02-14 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.26.0][308]            | 2025-03-11 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.30.0][309]            | 2025-07-11 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3525 (+2)                                           | [1.36.0][310]            | 2026-08-04 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.9.0][311]             | 2023-12-16 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.12.0][312]            | 2024-03-19 | 4.11.0.33                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.13.1][313]            | 2024-04-16 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.15.0][314]            | 2024-07-17 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.16.0][315]            | 2024-08-16 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.17.0][316]            | 2024-09-18 | 4.11.0.38                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.20.0][317]            | 2025-02-14 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.21.0][318]            | 2025-03-11 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.23.0][319]            | 2025-06-10 | 4.11.0.39                       | LVFS        |
| Dell      | Inspiron 15 3535 (+1)                                           | [1.28.0][320]            | 2026-02-10 | 4.11.0.39                       | LVFS        |
| Dell      | Precision 7875 Tower                                            | [1.13.0][321]            | 2025-04-01 | 4.11.0.38                       | LVFS        |
| Dell      | Precision 7875 Tower                                            | [2.1.1][322]             | 2025-06-24 | 4.11.0.38, 4.2D.0.3D            | LVFS        |
| Dell      | Precision 7875 Tower                                            | [2.2.1][323]             | 2025-07-18 | 4.11.0.38, 4.2D.0.3D            | LVFS        |
| Dell      | Precision 7875 Tower                                            | [2.5.0][324]             | 2025-10-13 | 4.11.0.38, 4.2D.0.3D            | LVFS        |
| Dell      | Precision 7875 Tower                                            | [2.6.1][325]             | 2025-11-03 | 4.11.0.38, 4.2D.0.3D            | LVFS        |
| Dell      | Precision 7875 Tower                                            | [2.7.3][326]             | 2026-01-08 | 4.11.0.38, 4.2D.0.3D            | LVFS        |
| Dell      | Precision 7875 Tower                                            | [2.9.1][327]             | 2026-03-24 | 4.11.0.38, 4.2D.0.3D            | LVFS        |
| Dell      | Precision 7875 Tower                                            | [2.10.1][328]            | 2026-05-13 | 4.11.0.38, 4.2D.0.3D            | LVFS        |
| Dell      | Precision 7875 Tower                                            | [2.11.0][329]            | 2026-06-29 | 4.11.0.38, 4.2D.0.3D            | LVFS        |
| Dell      | Precision 7875 Tower                                            | [2.12.0][330]            | 2026-07-24 | 4.11.0.38, 4.2D.0.3D            | LVFS        |
| Dell      | Pro 13 Plus PB13255 (+2)                                        | [1.4.3][331]             | 2025-05-28 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 13 Plus PB13255 (+2)                                        | [1.5.1][332]             | 2025-06-23 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 13 Plus PB13255 (+2)                                        | [1.7.0][333]             | 2025-08-13 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 13 Plus PB13255 (+2)                                        | [1.11.0][334]            | 2026-02-03 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 13 Plus PB13255 (+2)                                        | [1.12.0][335]            | 2026-03-17 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 13 Plus PB13255 (+2)                                        | [1.14.0][336]            |            | 4.2D.0.3A, 4.2D.0.3B            | vendor site |
| Dell      | Pro 13 Plus PB13255 (+2)                                        | [1.15.0][337]            |            | 4.2D.0.3A, 4.2D.0.3B            | vendor site |
| Dell      | Pro 13 Plus PB13255 (+2)                                        | [1.16.0][338]            | 2026-05-27 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 14 Essential PV14255                                        | [1.12.0][339]            |            | 4.2D.0.3B                       | vendor site |
| Dell      | Pro 14 Essential PV14255                                        | [1.13.1][340]            |            | 4.2D.0.3B                       | vendor site |
| Dell      | Pro 14 Essential PV14255                                        | [1.14.0][341]            | 2026-07-08 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 14 Essential PV14255                                        | [1.15.0][342]            | 2026-08-04 | 4.2D.0.41                       | LVFS        |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.2.0][343]             | 2025-03-24 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.3.1][344]             | 2025-04-18 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.5.1][345]             | 2025-06-21 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.7.0][346]             | 2025-08-11 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.8.1][347]             | 2025-11-10 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.11.1][348]            | 2026-01-13 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.12.0][349]            | 2026-03-31 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.13.0][350]            |            | 4.2D.0.3A, 4.2D.0.3B            | vendor site |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.14.1][351]            |            | 4.2D.0.3A, 4.2D.0.3B            | vendor site |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.15.0][352]            | 2026-06-05 | 4.2D.0.3B                       | LVFS        |
| Dell      | Pro 14 PC14255 (+1)                                             | [1.17.1][353]            | 2026-07-31 | 4.2D.0.41                       | LVFS        |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [1.2.1][354]             | 2025-07-24 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [1.3.0][355]             | 2025-08-07 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [1.4.2][356]             | 2025-09-18 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [1.5.0][357]             | 2025-10-15 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [1.6.2][358]             | 2025-11-18 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [1.8.0][359]             | 2025-12-25 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [2.1.2][360]             | 2026-03-13 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [2.2.2][361]             |            | 4.2D.0.3A, 4.2D.0.41            | vendor site |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [2.4.0][362]             |            | 4.2D.0.3A, 4.2D.0.41            | vendor site |
| Dell      | Pro Max 14 MC14255 (+1)                                         | [2.5.1][363]             | 2026-06-17 | 4.2D.0.3A                       | LVFS        |
| Dell      | Pro Max 16 MC16255                                              | [2.1.2][364]             | 2026-03-16 | 4.2D.0.41                       | LVFS        |
| Dell      | Pro Max 16 MC16255                                              | [2.2.2][365]             | 2026-04-01 | 4.2D.0.41                       | LVFS        |
| Dell      | Pro Max 16 MC16255                                              | [2.4.0][366]             | 2026-05-13 | 4.2D.0.41                       | LVFS        |
| Dell      | Pro Max 16 MC16255                                              | [2.5.1][367]             | 2026-06-17 | 4.2D.0.41                       | LVFS        |
| Dell      | Pro Micro QCM1255 (+2)                                          | [1.5.0][368]             |            | 4.2D.0.3B                       | vendor site |
| Dell      | Pro Micro QCM1255 (+2)                                          | [1.13.0][369]            |            | 4.2D.0.3B                       | vendor site |
| Framework | Desktop Ryzen AI MAX 300                                        | [0.0.3.2][370]           | 2025-07-22 | 4.2D.0.3A                       | LVFS        |
| Framework | Desktop Ryzen AI MAX 300                                        | [0.0.3.3][371]           | 2025-09-16 | 4.2D.0.3A                       | LVFS        |
| Framework | Desktop Ryzen AI MAX 300                                        | [0.0.3.4][372]           | 2025-11-19 | 4.2D.0.3A                       | LVFS        |
| Framework | Desktop Ryzen AI MAX 300                                        | [0.0.3.5][373]           | 2026-01-21 | 4.2D.0.3A                       | LVFS        |
| Framework | Desktop Ryzen AI MAX 300                                        | [0.0.3.6][374]           | 2026-07-13 | 4.2D.0.3A                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen 7040                                        | [0.0.3.3][375]           | 2023-10-17 | 4.2D.0.35                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen 7040                                        | [0.0.3.5][376]           | 2024-03-28 | 4.2D.0.37                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen 7040                                        | [0.0.3.7][377]           | 2025-02-19 | 4.2D.0.39                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen 7040                                        | [0.0.3.9][378]           | 2025-04-22 | 4.2D.0.39                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen 7040                                        | [0.0.3.16][379]          | 2025-07-25 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen 7040                                        | [0.0.3.17][380]          | 2025-11-14 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen 7040                                        | [0.0.3.18][381]          | 2026-01-08 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen 7040                                        | [0.0.3.20][382]          | 2026-06-23 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen AI 300                                      | [0.0.3.3][383]           | 2025-03-10 | 4.2D.0.3A                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen AI 300                                      | [0.0.3.4][384]           | 2025-07-02 | 4.2D.0.3A                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen AI 300                                      | [0.0.3.5][385]           | 2025-10-30 | 4.2D.0.3A                       | LVFS        |
| Framework | Laptop 13 AMD Ryzen AI 300                                      | [0.0.4.2][386]           | 2026-06-08 | 4.2D.0.3A                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen 7040                                        | [0.0.3.3][387]           | 2024-03-27 | 4.2D.0.37                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen 7040                                        | [0.0.3.5][388]           | 2024-11-13 | 4.2D.0.39                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen 7040                                        | [0.0.3.6][389]           | 2025-07-28 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen 7040                                        | [0.0.3.7][390]           | 2025-08-27 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen 7040                                        | [0.0.4.2][391]           | 2025-11-13 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen 7040                                        | [0.0.4.3][392]           | 2025-12-22 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen 7040                                        | [0.0.4.4][393]           | 2026-03-23 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen 7040                                        | [0.0.4.5][394]           | 2026-06-30 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen AI 300                                      | [0.0.3.3][395]           | 2025-10-15 | 4.2D.0.3A                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen AI 300                                      | [0.0.3.4][396]           | 2025-11-06 | 4.2D.0.3A                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen AI 300                                      | [0.0.3.5][397]           | 2026-03-13 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen AI 300                                      | [0.0.3.6][398]           | 2026-04-24 | 4.2D.0.3B                       | LVFS        |
| Framework | Laptop 16 AMD Ryzen AI 300                                      | [0.0.4.1][399]           | 2026-06-11 | 4.2D.0.3B                       | LVFS        |
| HP        | Commercial Desktop W06 Family                                   | [0x2080000][400]         | 2024-08-14 | 4.2D.0.37                       | LVFS        |
| HP        | Commercial Desktop W06 Family                                   | [0x20a0000][401]         | 2024-10-30 | 4.2D.0.37                       | LVFS        |
| HP        | Commercial Desktop W06 Family                                   | [0x20b0000][402]         | 2024-12-30 | 4.2D.0.39                       | LVFS        |
| HP        | Commercial Desktop W06 Family                                   | [0x20d0000][403]         | 2025-07-08 | 4.2D.0.3B                       | LVFS        |
| HP        | Commercial Desktop W06 Family                                   | [0x20e0000][404]         | 2025-11-17 | 4.2D.0.3B                       | LVFS        |
| HP        | Elite mt645 G8 Mobile Thin Client W71                           | [0x1020300][405]         | 2024-07-09 | 4.11.0.38                       | LVFS        |
| HP        | Elite mt645 G8 Mobile Thin Client W71                           | [0x1050000][406]         | 2025-04-15 | 4.11.0.39                       | LVFS        |
| HP        | Elite mt645 G8 Mobile Thin Client W71                           | [0x1060000][407]         | 2025-08-25 | 4.11.0.39                       | LVFS        |
| HP        | Elite mt645 G8 Mobile Thin Client W71                           | [0x1080200][408]         | 2026-05-04 | 4.11.0.39                       | LVFS        |
| HP        | Elite SFF 805 G9 Desktop PC                                     | [W06 02.14.00][409]      |            | 4.2D.0.3B                       | vendor site |
| HP        | Elite SFF 805 G9 Desktop PC                                     | [W06 02.16.00][410]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 6 G1a 14 inch Notebook AI PC (+3)                     | [X77 01.01.02][411]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 6 G1a 14 inch Notebook AI PC (+3)                     | [X77 01.05.04][412]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 6 G2a 14 inch Notebook AI PC (+4)                     | [Y77, Y98 01.01.06][413] |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 6 G2a 14 inch Notebook AI PC (+4)                     | [Y77/Y98 01.02.00][414]  |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 6 G2a 14 inch Notebook AI PC (+4)                     | [Y77/Y98 01.02.03][415]  |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 6 G2a 14 inch Notebook Next Gen AI PC (+1)            | [Y76 01.01.04][416]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteBook 6 G2a 14 inch Notebook Next Gen AI PC (+1)            | [Y76 01.02.02][417]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteBook 645 14 inch G10 (+1)                                  | [V77 01.14.00][418]      |            | 4.11.0.39                       | vendor site |
| HP        | EliteBook 645 14 inch G11 (+1)                                  | [W77 01.01.01][419]      |            | 4.11.0.38                       | vendor site |
| HP        | EliteBook 645 14 inch G11 (+1)                                  | [W77 01.09.01][420]      |            | 4.11.0.39                       | vendor site |
| HP        | EliteBook 645 14 inch G9 (+1)                                   | [U77/U87 01.23.00][421]  |            | 4.11.0.39                       | vendor site |
| HP        | Elitebook 645 G1a Notebook PC X76                               | [0x1010200][422]         | 2025-04-14 | 4.2D.0.3A                       | LVFS        |
| HP        | EliteBook 6x5 G11 Notebook PC W77                               | [0x1080100][423]         | 2026-03-19 | 4.11.0.39                       | LVFS        |
| HP        | EliteBook 6x5 G11 Notebook PC W77                               | [0x1080300][424]         | 2026-05-19 | 4.11.0.39                       | LVFS        |
| HP        | EliteBook 6x5 G11 Notebook PC W77                               | [0x1090100][425]         | 2026-07-08 | 4.11.0.39                       | LVFS        |
| HP        | EliteBook 6x5 G1a Notebook PC X77                               | [0x1010200][426]         | 2025-04-14 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 6x5 G1a Notebook PC X77                               | [0x1030100][427]         | 2025-12-12 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 6x5 G1a Notebook PC X77                               | [0x1040400][428]         | 2026-03-26 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 6x5 G1a Notebook PC X77                               | [0x1040500][429]         | 2026-05-06 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 6x5 G1a Notebook PC X77                               | [0x1050300][430]         | 2026-07-14 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8 G1a 13 inch Notebook AI PC                          | [X81 01.02.02][431]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 8 G1a 13 inch Notebook AI PC                          | [X81 01.05.04][432]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 8 G1a 13 inch Notebook AI PC X81                      | [0x1020200][433]         | 2025-06-05 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8 G1a 13 inch Notebook AI PC X81                      | [0x1040100][434]         | 2025-12-16 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8 G1a 13 inch Notebook AI PC X81                      | [0x1050400][435]         | 2026-05-08 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8 G1a 13 inch Notebook AI PC X83                      | [0x1020100][436]         | 2025-05-23 | 4.2D.0.3A                       | LVFS        |
| HP        | EliteBook 8 G1a 13 inch Notebook AI PC X83                      | [0x1040100][437]         | 2025-11-17 | 4.2D.0.3A                       | LVFS        |
| HP        | EliteBook 8 G1a 13 inch Notebook AI PC X83                      | [0x1050400][438]         | 2026-05-19 | 4.2D.0.41                       | LVFS        |
| HP        | EliteBook 8 G1a 13 inch Notebook AI PC X83                      | [0x1060100][439]         | 2026-07-17 | 4.2D.0.41                       | LVFS        |
| HP        | EliteBook 8 G1a 13 inch Notebook Next Gen AI PC                 | [X83 01.02.01][440]      |            | 4.2D.0.3A                       | vendor site |
| HP        | EliteBook 8 G1a 13 inch Notebook Next Gen AI PC                 | [X83 01.06.01][441]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteBook 8 G1a 14 inch Notebook AI PC (+2)                     | [X82 01.02.02][442]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 8 G1a 14 inch Notebook AI PC (+2)                     | [X82 01.05.04][443]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 8 G1a 14 inch Notebook Next Gen AI PC (+3)            | [X84 01.02.01][444]      |            | 4.2D.0.3A                       | vendor site |
| HP        | EliteBook 8 G1a 14 inch Notebook Next Gen AI PC (+3)            | [X84 01.06.01][445]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteBook 8 G2a 14 inch Notebook Next Gen AI PC (+3)            | [Y81/Y82 01.01.03][446]  |            | 4.2D.0.41                       | vendor site |
| HP        | EliteBook 8 G2a 14 inch Notebook Next Gen AI PC (+3)            | [Y81/Y82 01.02.00][447]  |            | 4.2D.0.41                       | vendor site |
| HP        | EliteBook 835 13 inch G10                                       | [V83 01.03.02][448]      |            | 4.2D.0.33                       | vendor site |
| HP        | EliteBook 835 13 inch G10                                       | [V83 01.13.00][449]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 835 13 inch G11                                       | [W83 01.04.01][450]      |            | 4.2D.0.39                       | vendor site |
| HP        | EliteBook 835 13 inch G11                                       | [W83 01.09.01][451]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 835 13 inch G11 Notebook PC W83                       | [0x1080200][452]         | 2026-05-04 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 835 13 inch G11 Notebook PC W83                       | [0x1090200][453]         | 2026-08-21 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 835 13.3 inch G9 (+2)                                 | [U82 01.07.00][454]      |            | 4.11.0.34                       | vendor site |
| HP        | EliteBook 835 13.3 inch G9 (+2)                                 | [U82 01.17.00][455]      |            | 4.11.0.39                       | vendor site |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x1040a00][456]         | 2024-03-01 | 4.2D.0.35                       | LVFS        |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x1050b00][457]         | 2024-06-18 | 4.2D.0.37                       | LVFS        |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x1060200][458]         | 2024-08-23 | 4.2D.0.39                       | LVFS        |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x1080000][459]         | 2025-02-20 | 4.2D.0.39                       | LVFS        |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x1090000][460]         | 2025-04-14 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x10a0300][461]         | 2025-10-10 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x10b0000][462]         | 2025-11-05 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x10c0000][463]         | 2026-02-10 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x10c0200][464]         | 2026-05-18 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 835 G10 Notebook PC V83                               | [0x10d0000][465]         | 2026-06-15 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 835 G8 (+2)                                           | [T82 01.15.00][466]      |            | 4.11.0.33                       | vendor site |
| HP        | EliteBook 835 G8 (+2)                                           | [T82 01.25.00][467]      |            | 4.11.0.39                       | vendor site |
| HP        | EliteBook 845 14 inch G10 (+2)                                  | [V82 01.03.02][468]      |            | 4.2D.0.33                       | vendor site |
| HP        | EliteBook 845 14 inch G10 (+2)                                  | [V82 01.13.00][469]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 845 14 inch G11 (+2)                                  | [W82 01.04.01][470]      |            | 4.2D.0.39                       | vendor site |
| HP        | EliteBook 845 14 inch G11 (+2)                                  | [W82 01.09.01][471]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x1030900][472]         | 2023-12-11 | 4.2D.0.33                       | LVFS        |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x1040a00][473]         | 2024-03-01 | 4.2D.0.35                       | LVFS        |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x1050b00][474]         | 2024-06-18 | 4.2D.0.37                       | LVFS        |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x1060200][475]         | 2024-08-23 | 4.2D.0.39                       | LVFS        |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x1080000][476]         | 2025-02-20 | 4.2D.0.39                       | LVFS        |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x10a0300][477]         | 2025-10-10 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x10b0000][478]         | 2025-11-05 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x10c0000][479]         | 2026-02-10 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x10c0300][480]         | 2026-05-18 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8x5 G10 Notebook PC V82                               | [0x10d0000][481]         | 2026-06-15 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8x5 G11 Notebook PC W82                               | [0x1090200][482]         | 2026-08-21 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook 8x5 G9 Notebook PC U82                                | [0x1060000][483]         | 2023-07-03 | 4.11.0.33                       | LVFS        |
| HP        | EliteBook 8x5 G9 Notebook PC U82                                | [0x1070000][484]         | 2023-10-05 | 4.11.0.34                       | LVFS        |
| HP        | EliteBook 8x5 G9 Notebook PC U82                                | [0x1080000][485]         | 2024-01-19 | 4.11.0.38                       | LVFS        |
| HP        | EliteBook 8x5 G9 Notebook PC U82                                | [0x10f0000][486]         | 2025-11-03 | 4.11.0.39                       | LVFS        |
| HP        | EliteBook 8x5 G9 Notebook PC U82                                | [0x1100000][487]         | 2026-01-28 | 4.11.0.39                       | LVFS        |
| HP        | EliteBook 8x5 G9 Notebook PC U82                                | [0x1100100][488]         | 2026-05-01 | 4.11.0.39                       | LVFS        |
| HP        | EliteBook 8x5 G9 Notebook PC U82                                | [0x1110000][489]         | 2026-06-09 | 4.11.0.39                       | LVFS        |
| HP        | EliteBook X G1a 14 inch Notebook Next Gen AI PC                 | [X88 01.01.04][490]      |            | 4.2D.0.3A                       | vendor site |
| HP        | EliteBook X G1a 14 inch Notebook Next Gen AI PC                 | [X88 01.07.02][491]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteBook X G1a 14 inch Notebook Next Gen AI PC X88             | [0x1060100][492]         | 2026-03-12 | 4.2D.0.41                       | LVFS        |
| HP        | EliteBook X G2a 14 inch Notebook Next Gen AI PC                 | [Y88 01.01.12][493]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteBook X G2a 14 inch Notebook Next Gen AI PC                 | [Y88 01.02.04][494]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteBook/ZBook 8 G1a/ah 14/16 inch Notebook/Workstation PC X82 | [0x1020200][495]         | 2025-06-05 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook/ZBook 8 G1a/ah 14/16 inch Notebook/Workstation PC X82 | [0x1040100][496]         | 2025-12-16 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteBook/ZBook 8 G1a/ah 14/16 inch Notebook/Workstation PC X82 | [0x1050400][497]         | 2026-05-08 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop AI PC                              | [0x2010200][498]         | 2025-06-04 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop AI PC                              | [0x2030000][499]         | 2025-08-27 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop AI PC                              | [0x2040100][500]         | 2025-11-14 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop AI PC                              | [0x2050000][501]         | 2026-01-15 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop AI PC                              | [0x2060100][502]         | 2026-05-18 | 4.2D.0.3B                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop Next Gen AI PC                     | [0x2010200][503]         | 2025-06-04 | 4.2D.0.3A                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop Next Gen AI PC                     | [0x2030100][504]         | 2025-09-05 | 4.2D.0.3A                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop Next Gen AI PC                     | [0x2040100][505]         | 2025-11-14 | 4.2D.0.3A                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop Next Gen AI PC                     | [0x2050000][506]         | 2026-01-09 | 4.2D.0.3A                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop Next Gen AI PC                     | [0x2060100][507]         | 2026-05-18 | 4.2D.0.3A                       | LVFS        |
| HP        | EliteDesk 8 Mini G1a Desktop Next Gen AI PC (+1)                | [X26 02.04.01][508]      |            | 4.2D.0.3A                       | vendor site |
| HP        | EliteDesk 8 Mini G1a Desktop Next Gen AI PC (+1)                | [X27 02.04.01][509]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteDesk 8 Mini G1a Desktop Next Gen AI PC (+1)                | [X27 02.06.01][510]      |            | 4.2D.0.3B                       | vendor site |
| HP        | EliteDesk 8 Mini G1a Desktop Next Gen AI PC (+1)                | [X26 02.07.00][511]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteDesk 8 Mini G2a Desktop Next Gen AI PC                     | [Y26 02.01.02][512]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteDesk 8 Mini G2a Desktop Next Gen AI PC                     | [Y26 02.02.02][513]      |            | 4.2D.0.41                       | vendor site |
| HP        | EliteDesk 8 Mini G2a Desktop Next Gen AI PC                     | [0x2010100][514]         | 2026-02-26 | 4.2D.0.41                       | LVFS        |
| HP        | Pro x360 435 13.3 inch G10                                      | [V80 01.05.00][515]      |            | 4.11.0.33                       | vendor site |
| HP        | Pro x360 435 13.3 inch G10                                      | [V80 01.14.01][516]      |            | 4.11.0.39                       | vendor site |
| HP        | Pro x360 435 13.3 inch G9                                       | [U80 01.13.00][517]      |            | 4.11.0.33                       | vendor site |
| HP        | Pro x360 435 13.3 inch G9                                       | [U80 01.23.00][518]      |            | 4.11.0.39                       | vendor site |
| HP        | ProBook 4 G1a 14 inch Notebook AI PC (+3)                       | [X78 01.01.02][519]      |            | 4.2D.0.3B                       | vendor site |
| HP        | ProBook 4 G1a 14 inch Notebook AI PC (+3)                       | [X78 01.05.05][520]      |            | 4.2D.0.3B                       | vendor site |
| HP        | ProBook 4 G2a 14 inch Notebook AI PC (+3)                       | [Y78, Y99 01.01.05][521] |            | 4.2D.0.3B                       | vendor site |
| HP        | ProBook 4 G2a 14 inch Notebook AI PC (+3)                       | [Y78/Y99 01.02.00][522]  |            | 4.2D.0.3B                       | vendor site |
| HP        | ProBook 4 G2a 14 inch Notebook AI PC (+3)                       | [Y78/Y99 01.02.03][523]  |            | 4.2D.0.3B                       | vendor site |
| HP        | ProBook 4 G2a 14 inch Notebook Next Gen AI PC (+1)              | [Y75 01.01.04][524]      |            | 4.2D.0.41                       | vendor site |
| HP        | ProBook 4 G2a 14 inch Notebook Next Gen AI PC (+1)              | [Y75 01.02.02][525]      |            | 4.2D.0.41                       | vendor site |
| HP        | ProBook 445 14 inch G10 (+1)                                    | [V78 01.14.01][526]      |            | 4.11.0.39                       | vendor site |
| HP        | ProBook 445 14 inch G11 (+1)                                    | [W78 01.01.01][527]      |            | 4.11.0.38                       | vendor site |
| HP        | ProBook 445 14 inch G11 (+1)                                    | [W78 01.09.00][528]      |            | 4.11.0.39                       | vendor site |
| HP        | ProBook 445 14 inch G9 (+1)                                     | [U78/U88 01.23.00][529]  |            | 4.11.0.39                       | vendor site |
| HP        | ProBook 445 G8 (+1)                                             | [T78/T96 01.25.00][530]  |            | 4.11.0.39                       | vendor site |
| HP        | ProBook 4x5 G11 Notebook PC W78                                 | [0x1080100][531]         | 2026-03-11 | 4.11.0.39                       | LVFS        |
| HP        | ProBook 4x5 G11 Notebook PC W78                                 | [0x1080300][532]         | 2026-05-19 | 4.11.0.39                       | LVFS        |
| HP        | ProBook 4x5 G11 Notebook PC W78                                 | [0x1090000][533]         | 2026-06-10 | 4.11.0.39                       | LVFS        |
| HP        | ProBook 4x5 G1a Notebook PC X78                                 | [0x1010200][534]         | 2025-04-14 | 4.2D.0.3B                       | LVFS        |
| HP        | ProBook 4x5 G1a Notebook PC X78                                 | [0x1030100][535]         | 2025-12-12 | 4.2D.0.3B                       | LVFS        |
| HP        | ProBook 4x5 G1a Notebook PC X78                                 | [0x1040400][536]         | 2026-03-26 | 4.2D.0.3B                       | LVFS        |
| HP        | ProBook 4x5 G1a Notebook PC X78                                 | [0x1040500][537]         | 2026-05-06 | 4.2D.0.3B                       | LVFS        |
| HP        | ProBook 4x5 G1a Notebook PC X78                                 | [0x1040800][538]         | 2026-05-21 | 4.2D.0.3B                       | LVFS        |
| HP        | ProBook 4x5 G1a Notebook PC X78                                 | [0x1050400][539]         | 2026-07-14 | 4.2D.0.3B                       | LVFS        |
| HP        | ProBook 635 Aero G8                                             | [T83 01.15.00][540]      |            | 4.11.0.33                       | vendor site |
| HP        | ProBook 635 Aero G8                                             | [T83 01.25.00][541]      |            | 4.11.0.39                       | vendor site |
| HP        | ProBook x360 435 G8                                             | [T80 01.15.00][542]      |            | 4.11.0.33                       | vendor site |
| HP        | ProBook x360 435 G8                                             | [T80 01.25.00][543]      |            | 4.11.0.39                       | vendor site |
| HP        | ZBook 8 G1as/ak 14 inch Mobile Workstation PC X84               | [0x1040100][544]         | 2025-11-17 | 4.2D.0.3A                       | LVFS        |
| HP        | ZBook 8 G1as/ak 14 inch Mobile Workstation PC X84               | [0x1050400][545]         | 2026-05-18 | 4.2D.0.41                       | LVFS        |
| HP        | ZBook 8 G1as/ak 14 inch Mobile Workstation PC X84               | [0x1060100][546]         | 2026-07-17 | 4.2D.0.41                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation PC               | [V85 01.04.00][547]      |            | 4.2D.0.35                       | vendor site |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation PC               | [V85 01.13.00][548]      |            | 4.2D.0.3B                       | vendor site |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x1030000][549]         | 2023-09-11 | 4.2D.0.33                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x1030900][550]         | 2023-12-11 | 4.2D.0.33                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x1040000][551]         | 2024-01-18 | 4.2D.0.35                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x1080000][552]         | 2025-02-11 | 4.2D.0.39                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x1090000][553]         | 2025-04-14 | 4.2D.0.3B                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x10a0000][554]         | 2025-08-19 | 4.2D.0.3B                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x10b0000][555]         | 2025-11-03 | 4.2D.0.3B                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x10c0000][556]         | 2026-02-10 | 4.2D.0.3B                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x10c0100][557]         | 2026-05-04 | 4.2D.0.3B                       | LVFS        |
| HP        | ZBook Power 15.6 inch G10 A Mobile Workstation V85              | [0x10d0000][558]         | 2026-06-10 | 4.2D.0.3B                       | LVFS        |
| HP        | ZBook Power 16 inch G11 A Mobile Workstation PC                 | [W85 01.06.01][559]      |            | 4.2D.0.39                       | vendor site |
| HP        | ZBook Power 16 inch G11 A Mobile Workstation PC                 | [W85 01.10.05][560]      |            | 4.2D.0.3B                       | vendor site |
| HP        | ZBook Power 16 inch G11 A Mobile Workstation PC W85             | [0x10a0400][561]         | 2026-03-09 | 4.2D.0.3B                       | LVFS        |
| HP        | ZBook Power 16 inch G11 A Mobile Workstation PC W85             | [0x10a0500][562]         | 2026-05-04 | 4.2D.0.3B                       | LVFS        |
| HP        | ZBook Power 16 inch G11 A Mobile Workstation PC W85             | [0x10b0100][563]         | 2026-07-06 | 4.2D.0.3B                       | LVFS        |
| HP        | ZBook Ultra G1a 14 inch Mobile Workstation PC                   | [X89 01.01.02][564]      |            | 4.2D.0.3A                       | vendor site |
| HP        | ZBook Ultra G1a 14 inch Mobile Workstation PC                   | [X89 01.06.02][565]      |            | 4.2D.0.3A                       | vendor site |
| HP        | ZBook Ultra G1a 14 inch Mobile Workstation PC X89               | [0x1020100][566]         | 2025-03-05 | 4.2D.0.3A                       | LVFS        |
| HP        | ZBook Ultra G1a 14 inch Mobile Workstation PC X89               | [0x1030000][567]         | 2025-04-25 | 4.2D.0.3A                       | LVFS        |
| HP        | ZBook Ultra G1a 14 inch Mobile Workstation PC X89               | [0x1050700][568]         | 2026-05-05 | 4.2D.0.3A                       | LVFS        |
| HP        | ZHAN 66 Pro A 14 G4                                             | [T79/T98 01.25.00][569]  |            | 4.11.0.39                       | vendor site |
| HP        | ZHAN 66 Pro A 14 G5                                             | [U79 01.23.00][570]      |            | 4.11.0.39                       | vendor site |
| Lenovo    | ThinkCentre M75Q Gen 5                                          | [M55KT2AA_1.42][571]     |            | 4.2D.0.3B                       | vendor site |
| Lenovo    | ThinkCentre M75Q Gen 6                                          | [M6CKT1BA][572]          |            | 4.2D.0.3B, 4.2D.0.41            | vendor site |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.7][573]             | 2025-04-17 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.9][574]             | 2025-08-12 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.13][575]            | 2025-12-16 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.15][576]            | 2026-03-12 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.16][577]            | 2026-05-15 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.23][578]            | 2024-10-08 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.25][579]            | 2024-11-04 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.26][580]            | 2025-02-06 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.38][581]            | 2026-04-08 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.39][582]            | 2026-05-07 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen5                                           | [0.1.42][583]            | 2026-07-27 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkCentre M75q Gen6                                           | [0.1.27][584]            | 2026-09-10 | 4.2D.0.3B, 4.2D.0.41            | LVFS        |
| Lenovo    | ThinkCentre M75S Gen 5 (+1)                                     | [M4RKT3BA_1.59][585]     |            | 4.11.0.39, 4.2D.0.3B            | vendor site |
| Lenovo    | ThinkCentre M75S Gen 6 (+1)                                     | [M6JKT1FA][586]          |            | 4.2D.0.3B, 4.2D.0.41            | vendor site |
| Lenovo    | ThinkCentre M75s Gen5                                           | [0.1.16][587]            | 2026-05-09 | 4.11.0.39, 4.2D.0.3B            | LVFS        |
| Lenovo    | ThinkCentre M75ts Gen5                                          | [0.1.42][588]            | 2024-10-30 | 4.11.0.38, 4.2D.0.37            | LVFS        |
| Lenovo    | ThinkCentre M75ts Gen5                                          | [0.1.48][589]            | 2025-06-19 | 4.11.0.38, 4.2D.0.37            | LVFS        |
| Lenovo    | ThinkCentre M75ts Gen5                                          | [0.1.50][590]            | 2025-09-26 | 4.11.0.38, 4.2D.0.37            | LVFS        |
| Lenovo    | ThinkCentre M75ts Gen5                                          | [0.1.58][591]            | 2026-05-08 | 4.11.0.39, 4.2D.0.3B            | LVFS        |
| Lenovo    | ThinkPad E14 Gen 7 Type 21T0 21T1 (+1)                          | [1.27][592]              |            | 4.2D.0.3B                       | vendor site |
| Lenovo    | ThinkPad E14 Gen 8 Type 21Y2 21Y3 (+1)                          | [1.05][593]              |            | 4.2D.0.41                       | vendor site |
| Lenovo    | ThinkPad E14/E16 Gen 8/Gen 4 AMD                                | [0.1.02][594]            | 2026-03-26 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad E14/E16 Gen 8/Gen 4 AMD                                | [0.1.03][595]            | 2026-05-20 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad E14/E16 Gen 8/Gen 4 AMD                                | [0.1.04][596]            | 2026-06-01 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad E14/E16 Gen 8/Gen 4 AMD                                | [0.1.05][597]            | 2026-08-27 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad L13 Gen 2 Type 21AB 21AC (+1)                          | [1.40][598]              |            | 4.11.0.39                       | vendor site |
| Lenovo    | ThinkPad L13 Gen 3 Type 21B9 21BA (+1)                          | [1.39][599]              |            | 4.11.0.39                       | vendor site |
| Lenovo    | ThinkPad L13 Gen 4 AMD                                          | [0.1.11][600]            | 2023-09-21 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad L13 Gen 4 AMD                                          | [0.1.14][601]            | 2024-01-18 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad L13 Gen 4 AMD                                          | [0.1.17][602]            | 2024-07-01 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad L13 Gen 4 AMD                                          | [0.1.22][603]            | 2024-12-23 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad L13 Gen 4 AMD                                          | [0.1.23][604]            | 2017-12-15 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L13 Gen 4 AMD                                          | [0.1.28][605]            | 2017-12-15 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L13 Gen 4 AMD                                          | [0.1.31][606]            | 2017-12-15 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L13 Gen 4 AMD                                          | [0.1.32][607]            | 2017-12-15 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L13 Gen 4 Type 21FN 21FQ (+1)                          | [1.32][608]              |            | 4.11.0.39                       | vendor site |
| Lenovo    | ThinkPad L13 Gen 6 Type 21RB 21RC (+1)                          | [1.19][609]              |            | 4.2D.0.3B                       | vendor site |
| Lenovo    | ThinkPad L13 Gen3/L13 Yoga Gen3 AMD                             | [0.1.36][610]            | 2025-10-01 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14 AMD / L15 AMD                                      | [0.1.32][611]            | 2023-10-31 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad L14 AMD / L15 AMD                                      | [0.1.34][612]            | 2024-08-02 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad L14 AMD / L15 AMD                                      | [0.1.35][613]            | 2024-11-08 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad L14 AMD / L15 AMD                                      | [0.1.37][614]            | 2025-02-18 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14 AMD / L15 AMD                                      | [0.1.40][615]            | 2026-05-26 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14 AMD Gen2 / L15 AMD Gen2                            | [0.1.30][616]            | 2023-11-17 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad L14 AMD Gen2 / L15 AMD Gen2                            | [0.1.32][617]            | 2024-09-20 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad L14 AMD Gen2 / L15 AMD Gen2                            | [0.1.33][618]            | 2025-02-18 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14 AMD Gen2 / L15 AMD Gen2                            | [0.1.36][619]            | 2026-04-16 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14 Gen 2 Type 20X5 20X6 (+1)                          | [1.36][620]              |            | 4.11.0.39                       | vendor site |
| Lenovo    | ThinkPad L14 Gen 3 Type 21C5 21C6 (+1)                          | [1.39][621]              |            | 4.11.0.39                       | vendor site |
| Lenovo    | ThinkPad L14 Gen 5 Type 21L5 21L6 (+1)                          | [1.44][622]              |            | 4.11.0.39                       | vendor site |
| Lenovo    | ThinkPad L14 Gen 6 Type 21S8 21S9 (+1)                          | [1.19][623]              |            | 4.2D.0.3B                       | vendor site |
| Lenovo    | ThinkPad L14 Gen 6 Type 21SE 21SF (+1)                          | [1.26][624]              |            | 4.2D.0.41                       | vendor site |
| Lenovo    | ThinkPad L14 Gen 6/ L16 Gen 2 AMD                               | [0.1.12][625]            | 2025-08-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad L14 Gen 6/ L16 Gen 2 AMD                               | [0.1.14][626]            | 2025-11-28 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad L14 Gen 6/ L16 Gen 2 AMD                               | [0.1.18][627]            | 2026-03-31 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad L14 Gen 6/ L16 Gen 2 AMD                               | [0.1.19][628]            | 2026-05-28 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad L14 Gen 6/ L16 Gen 2 AMD                               | [0.1.25][629]            | 2026-07-16 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad L14 Gen 6/ L16 Gen 2 AMD                               | [0.1.26][630]            | 2026-08-13 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad L14 Gen 7 Type 21X6 21X7 (+1)                          | [1.21][631]              |            | 4.2D.0.41                       | vendor site |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.20][632]            | 2024-07-26 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.21][633]            | 2017-12-15 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.23][634]            | 2025-02-06 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.24][635]            | 2017-12-15 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.25][636]            | 2025-07-23 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.26][637]            | 2025-09-24 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.27][638]            | 2025-11-25 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.28][639]            | 2026-01-21 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.29][640]            | 2026-03-20 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L1415 Gen 4 AMD                                        | [0.1.30][641]            | 2026-05-21 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.29][642]            | 2023-12-08 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.30][643]            | 2024-03-01 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.31][644]            | 2024-06-07 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.32][645]            | 2024-09-06 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.33][646]            | 2024-11-15 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.34][647]            | 2025-02-12 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.35][648]            | 2025-05-27 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.36][649]            | 2025-12-10 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.38][650]            | 2026-04-27 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L14L15 Gen3 AMD                                        | [0.1.39][651]            | 2026-05-27 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad L15 Gen 4 Type 21H7 21H8 (+1)                          | [1.31][652]              |            | 4.11.0.39                       | vendor site |
| Lenovo    | ThinkPad P14S Gen 7 Type 21X0 21X1 (+2)                         | [1.07][653]              |            | 4.2D.0.41                       | vendor site |
| Lenovo    | ThinkPad P15v Gen 3 AMD                                         | [0.1.14][654]            | 2023-06-06 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad P15v Gen 3 AMD                                         | [0.1.20][655]            | 2024-03-07 | 4.11.0.34                       | LVFS        |
| Lenovo    | ThinkPad P15v Gen 3 AMD                                         | [0.1.21][656]            | 2024-05-27 | 4.11.0.34                       | LVFS        |
| Lenovo    | ThinkPad P15v Gen 3 AMD                                         | [0.1.23][657]            | 2024-08-26 | 4.11.0.34                       | LVFS        |
| Lenovo    | ThinkPad P15v Gen 3 AMD                                         | [0.1.25][658]            | 2024-11-08 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad P15v Gen 3 AMD                                         | [0.1.27][659]            | 2025-07-22 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad P15v Gen 3 AMD                                         | [0.1.28][660]            | 2025-12-04 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad P15v Gen 3 AMD                                         | [0.1.31][661]            | 2026-06-22 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad P15V Gen 3 Type 21EN 21EM                              | [BIOS1.31_ECFW1.17][662] |            | 4.11.0.39                       | vendor site |
| Lenovo    | ThinkPad P16S Gen 4 Type 21QR 21QS (+5)                         | [1.22][663]              |            | 4.2D.0.41                       | vendor site |
| Lenovo    | ThinkPad P16s Gen5 AMD                                          | [0.1.07][664]            | 2026-06-05 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad P16V Gen 1 Type 21FE 21FF                              | [1.65_1.13][665]         |            | 4.2D.0.3B                       | vendor site |
| Lenovo    | ThinkPad T14 Gen 7 T16 Gen 5 AMD                                | [0.1.04][666]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14 Gen 7 T16 Gen 5 AMD                                | [0.1.06][667]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14 Gen 7 T16 Gen 5 AMD                                | [0.1.07][668]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14 Gen 7 T16 Gen 5 AMD                                | [0.1.09][669]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14 Gen 7 Type 22AL 22AM (+1)                          | [1.09][670]              |            | 4.2D.0.3B                       | vendor site |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.44][671]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.45][672]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.46][673]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.47][674]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.49][675]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.50][676]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.51][677]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.52][678]            | 2025-04-30 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.53][679]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.54][680]            | 2026-04-30 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 1 AMD                                     | [0.1.56][681]            | 2026-06-04 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.24][682]            | 2020-07-01 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.25][683]            | 2020-07-01 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.26][684]            | 2020-07-01 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.28][685]            | 2020-07-01 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.29][686]            | 2020-07-01 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.30][687]            | 2020-07-01 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.31][688]            | 2020-07-01 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.32][689]            | 2020-07-01 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.33][690]            | 2020-07-01 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.34][691]            | 2026-05-20 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 2 AMD                                     | [0.1.35][692]            | 2020-07-01 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.52][693]            | 2021-07-13 | 4.11.0.34                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.53][694]            | 2021-07-13 | 4.11.0.34                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.54][695]            | 2021-07-13 | 4.11.0.34                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.56][696]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.57][697]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.58][698]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.60][699]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.61][700]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.62][701]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 3 T16/P16s Gen 1 AMD                      | [0.1.64][702]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.37][703]            | 2021-07-13 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.38][704]            | 2021-07-13 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.39][705]            | 2021-07-13 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.41][706]            | 2021-07-13 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.45][707]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.46][708]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.47][709]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.48][710]            | 2026-01-30 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.49][711]            | 2026-03-24 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.50][712]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 4 T16/P16s Gen 2 AMD                      | [0.1.51][713]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.07][714]            | 2024-06-18 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.08][715]            | 2024-08-07 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.09][716]            | 2024-08-23 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.10][717]            | 2024-09-26 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.11][718]            | 2024-11-15 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.17][719]            | 2025-10-02 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.19][720]            | 2026-01-30 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.21][721]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.22][722]            | 2026-05-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 5 AMD                                     | [0.1.23][723]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 6 T16/P16s Gen 4 AMD                      | [0.1.2][724]             | 2021-07-13 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 6 T16/P16s Gen 4 AMD                      | [0.1.15][725]            | 2021-07-13 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 6 T16/P16s Gen 4 AMD                      | [0.1.17][726]            | 2025-12-13 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 6 T16/P16s Gen 4 AMD                      | [0.1.19][727]            | 2021-07-13 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 6 T16/P16s Gen 4 AMD                      | [0.1.21][728]            | 2021-07-13 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 6 T16/P16s Gen 4 AMD                      | [0.1.22][729]            | 2026-09-18 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 7 AMD / T16 Gen 5 AMD                     | [0.1.01][730]            | 2026-02-05 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 7 AMD / T16 Gen 5 AMD                     | [0.1.04][731]            | 2026-03-20 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 7 AMD / T16 Gen 5 AMD                     | [0.1.05][732]            | 2026-04-09 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 7 AMD / T16 Gen 5 AMD                     | [0.1.06][733]            | 2026-06-04 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen 7 AMD / T16 Gen 5 AMD                     | [0.1.07][734]            | 2021-07-13 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen6 AMD                                      | [0.1.05][735]            | 2025-04-05 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14/P14s Gen6 AMD                                      | [0.1.10][736]            | 2025-04-16 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.19][737]            | 2021-07-13 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.20][738]            | 2021-07-13 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.21][739]            | 2021-07-13 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.22][740]            | 2024-09-12 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.23][741]            | 2024-12-23 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.24][742]            | 2025-05-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.25][743]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.27][744]            | 2025-10-02 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.28][745]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.29][746]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.30][747]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 4 AMD                                         | [0.1.32][748]            | 2021-07-13 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 6 AMD                                         | [0.1.09][749]            | 2024-11-25 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 6 AMD                                         | [0.1.10][750]            | 2024-12-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 6 AMD                                         | [0.1.13][751]            | 2024-12-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 6 AMD                                         | [0.1.15][752]            | 2025-11-11 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 6 AMD                                         | [0.1.16][753]            | 2025-12-19 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 6 AMD                                         | [0.1.18][754]            | 2024-12-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 6 AMD                                         | [0.1.19][755]            | 2024-12-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad T14s Gen 6 AMD                                         | [0.1.20][756]            | 2024-12-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad X13 Gen6 AMD                                           | [0.1.05][757]            | 2024-12-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad X13 Gen6 AMD                                           | [0.1.08][758]            | 2024-12-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad X13 Gen6 AMD                                           | [0.1.10][759]            | 2024-12-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad X13 Gen6 AMD                                           | [0.1.11][760]            | 2024-12-18 | 4.2D.0.3A                       | LVFS        |
| Lenovo    | ThinkPad X13 Gen7 AMD                                           | [0.1.09][761]            | 2024-12-18 | 4.2D.0.41                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 1 AMD                                     | [0.1.44][762]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 1 AMD                                     | [0.1.45][763]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 1 AMD                                     | [0.1.46][764]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 1 AMD                                     | [0.1.48][765]            | 2024-08-07 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 1 AMD                                     | [0.1.49][766]            | 2021-07-13 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 1 AMD                                     | [0.1.51][767]            | 2024-12-03 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 1 AMD                                     | [0.1.52][768]            | 2025-04-30 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 1 AMD                                     | [0.1.53][769]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 1 AMD                                     | [0.1.54][770]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 2 AMD                                     | [0.1.27][771]            | 2020-07-01 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 2 AMD                                     | [0.1.28][772]            | 2020-07-01 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 2 AMD                                     | [0.1.29][773]            | 2024-06-05 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 2 AMD                                     | [0.1.31][774]            | 2020-07-01 | 4.11.0.33                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 2 AMD                                     | [0.1.32][775]            | 2020-07-01 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 2 AMD                                     | [0.1.33][776]            | 2020-07-01 | 4.11.0.38                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 2 AMD                                     | [0.1.34][777]            | 2020-07-01 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 2 AMD                                     | [0.1.36][778]            | 2020-07-01 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 2 AMD                                     | [0.1.37][779]            | 2020-07-01 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 3 AMD                                     | [0.1.40][780]            | 2024-05-20 | 4.11.0.34                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 3 AMD                                     | [0.1.42][781]            | 2021-07-13 | 4.11.0.34                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 3 AMD                                     | [0.1.44][782]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 3 AMD                                     | [0.1.45][783]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 3 AMD                                     | [0.1.46][784]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 3 AMD                                     | [0.1.50][785]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad X13/T14s Gen 3 AMD                                     | [0.1.51][786]            | 2021-07-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.69][787]            | 2024-08-14 | 4.11.0.34                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.70][788]            | 2024-09-05 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.71][789]            | 2024-12-30 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.72][790]            | 2025-05-21 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.73][791]            | 2025-06-10 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.74][792]            | 2025-08-20 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.76][793]            | 2025-12-01 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.78][794]            | 2026-03-20 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.79][795]            | 2026-04-28 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 1 / Z16 Gen 1                                  | [0.1.81][796]            | 2026-08-13 | 4.11.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.14][797]            | 2024-01-10 | 4.2D.0.33                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.15][798]            | 2024-03-04 | 4.2D.0.33                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.30][799]            | 2024-06-20 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.31][800]            | 2024-07-26 | 4.2D.0.37                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.32][801]            | 2024-09-10 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.33][802]            | 2024-12-23 | 4.2D.0.39                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.36][803]            | 2025-07-26 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.37][804]            | 2025-11-11 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.39][805]            | 2026-03-26 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.40][806]            | 2026-05-22 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 / Z16 Gen 2                                  | [0.1.42][807]            | 2026-08-11 | 4.2D.0.3B                       | LVFS        |
| Lenovo    | ThinkPad Z13 Gen 2 Type 21JV 21JW (+1)                          | [1.40][808]              |            | 4.2D.0.3B                       | vendor site |
| Lenovo    | ThinkPad Z13 Type 21D2 21D3 (+1)                                | [1.81][809]              |            | 4.11.0.39                       | vendor site |
| Lenovo    | Yoga Pro 7 15ASH11 AMD                                          | [97.67.25][810]          | 2026-08-07 | 4.2D.0.3A                       | LVFS        |
| MSI       | B850 GAMING PLUS WIFI (MS-7E56)                                 | [1A1][811]               | 2024-11-20 | 4.11.0.39, 4.2D.0.39            | vendor site |
| MSI       | B850 GAMING PLUS WIFI (MS-7E56)                                 | [2A6][812]               | 2025-05-13 | 4.11.0.39, 4.2D.0.39            | vendor site |
| MSI       | B850 GAMING PLUS WIFI (MS-7E56)                                 | [4A43][813]              | 2026-09-03 | 4.11.0.39, 4.2D.0.3B, 4.2D.0.41 | vendor site |
| MSI       | MAG B550 TOMAHAWK (MS-7C91)                                     | [1E1][814]               | 2023-06-09 | 4.11.0.33                       | vendor site |
| MSI       | MAG B550 TOMAHAWK (MS-7C91)                                     | [1F][815]                | 2023-10-19 | 4.11.0.33                       | vendor site |
| MSI       | MAG B550 TOMAHAWK (MS-7C91)                                     | [2C2][816]               | 2026-08-21 | 4.11.0.39                       | vendor site |
| MSI       | MAG B650 TOMAHAWK WIFI (MS-7D75)                                | [1L1][817]               | 2024-10-21 | 4.11.0.39, 4.2D.0.39            | vendor site |
| MSI       | MAG B650 TOMAHAWK WIFI (MS-7D75)                                | [1Q][818]                | 2026-05-22 | 4.11.0.39, 4.2D.0.3B, 4.2D.0.41 | vendor site |
| Star Labs | StarBook MkVI-AMD coreboot                                      | [26.06][819]             | 2026-06-16 | 4.11.0.33                       | LVFS        |
| Star Labs | StarBook MkVI-AMD coreboot                                      | [26.07][820]             | 2026-07-16 | 4.11.0.33                       | LVFS        |

[1]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-0320.zip
[2]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-0603.zip
[3]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-0608.zip
[4]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-0805.zip
[5]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-1004.zip
[6]: https://fwupd.org/downloads/46d29bbc34fd53ade25cca00ebd818a1bd1d4bff9f6b8d8b72efe8bc8ebe0462-Lenovo-Think-M75q-2-SystemFirmware-M3CKT33A.cab
[7]: https://fwupd.org/downloads/f5fe451d066b6cf6226955aad16356ee176c3d4fdf8ed1c0fb18138288a3e805-Lenovo-Think-M75q-2-SystemFirmware-M3CKT39A.cab
[8]: https://download.msi.com/bos_exe/mb/7C91v10.zip
[9]: https://fwupd.org/downloads/7f6ef6ed8ef3f2141e207cefbacccb4ea42cf997214e15bf64a0bc2e94065c5a-Signed_1152921505698702526.cab
[10]: https://fwupd.org/downloads/8fbdd9c25174b627ed8e4bdac4ee210bfd7aa3f8-firmware_OptiPlex_5055_A-Series_1.1.2.wu.cab
[11]: https://fwupd.org/downloads/cfce38e8be2ac3b41780893eef9da35641689bf5-firmware_OptiPlex_5055_A-Series_01.02.10.Wu.cab
[12]: https://fwupd.org/downloads/8019adccf63651affd31647fcab5f489c9d8d2ab0d54ac8fe726d5fe094658ab-Kona_Bristol_01.02.12.Wu.cab
[13]: https://fwupd.org/downloads/446754c8a454ba64a0eb6cc41e74024dff4b489e-OptiPlex_5055_APU_1.0.10.wu.cab
[14]: https://fwupd.org/downloads/9bad93ac4a2e41882938134c9c55d3d9bf0d5495-firmware_OptiPlex_5055_APU_01.02.07.wu.cab
[15]: https://fwupd.org/downloads/0884c207c49f35e0dfb4e511a23553364f0ef624d79fe6475993c35df62745f4-Kona_Raven_01.02.11.Wu.cab
[16]: https://fwupd.org/downloads/3aa624c6c7d880b0c19605da96e427316437b051-firmware_OptiPlex_5055_Ryzen_CPU_1.0.1.wu.cab
[17]: https://fwupd.org/downloads/71fb8b3e9dc51eec61c942d39f8eed037223d66a-firmware_OptiPlex_5055_Ryzen_CPU_1.0.7.cab
[18]: https://fwupd.org/downloads/cd14786819e898c6a5f4f8f495fc87afb3cba861a5a596f451ed084ec235f4df-Lenovo-ThinkCentre-M33CT12A-eSIOFirmware.cab
[19]: https://fwupd.org/downloads/5c5f4f340c8070e34c7d0c2502ab43b85ca7001b92548e4c831a00102b6289ae-Lenovo-ThinkPadL13YogaGen4-SystemFirmware-0.1.06.cab
[20]: https://fwupd.org/downloads/43cc163ab94abbceecca6514173714a65e6152eb6b967d9d4906023ac5f3719f-R26HT23W.cab
[21]: https://fwupd.org/downloads/f14a271c4a955162451254e106bf8e8e6568c32a8ab864e9462d17aad5231077-R26HT25W_LVFS.cab
[22]: https://fwupd.org/downloads/42c8a55907f2a2c5eb64321b537e09a494f6f26997f5ec1e256beb9635843b65-Lenovo-ThinkPadL13YogaGen4-SystemFirmware-0.1.15.cab
[23]: https://fwupd.org/downloads/c1d2a3a3df96209579ac6bd5aa7b7c8560f1e55a6efc7ea5a669f28146415ebc-R26HT32W_LVFS.cab
[24]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-1202.ZIP
[25]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-1212.ZIP
[26]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-1216.ZIP
[27]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-1602.ZIP
[28]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-1804.ZIP
[29]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-2003.ZIP
[30]: https://dlcdnets.asus.com/pub/ASUS/mb/SocketAM4/PRIME_B550-PLUS/PRIME-B550-PLUS-ASUS-2006.ZIP
[31]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-2403.ZIP
[32]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-2407.ZIP
[33]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-2423.ZIP
[34]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-2604.ZIP
[35]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-2803.zip
[36]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-2806.zip
[37]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3002.zip
[38]: https://fwupd.org/downloads/4d79d939d2c5303155db83510acc65713082f9279a721032afcec2fdf372374b-14091_new_lu_1.3.0.cab
[39]: https://fwupd.org/downloads/06a9c01d670f52ffa361670861d4f180f575e9114090481629791852d7efead9-19092_Inspiron_3525_1.9.0_0B9Fs.cab
[40]: https://fwupd.org/downloads/4d4db3d3eeafe023ddf403a01b4eeb3b42f8676c3d3b71cb5e0bbda337ead64f-19381_Inspiron_3525_1.10.0_LCN.cab
[41]: https://fwupd.org/downloads/6ce44cd19e3c2b2df46c4b0c837fe03a11eb6e60730f391790a908ddffce608a-20111_Inspiron_3525_1.11.0_LCN.cab
[42]: https://fwupd.org/downloads/57b026e688c29335fa503d57df2cb18bc46ae5357c78cde45791b53aaeb194a9-20997_Inspiron_3525_1.12.0_LCN2.cab
[43]: https://fwupd.org/downloads/6035e9186d0893a5056c155b0864261366c9d3b36ed7ce31fdf2d35a8cfef1d6-18738_Inspiron_3525_1.8.0_BAR_32.cab
[44]: https://fwupd.org/downloads/383f30da09b2c9a3255dc3e9f9a814bc0197808a72abc9f11b10b93a9f786b16-18987_Inspiron_3525_1.9.0_0B6B.cab
[45]: https://fwupd.org/downloads/87f88b6db07b5b3fab01fb3bfa00c6a4e05fde353f4956396d3c7c10f0ce4ea7-19379_Inspiron_3525_1.10.0_BAR.cab
[46]: https://fwupd.org/downloads/0adeb59d3e3ca0ce818b108d74e7587d830a41b7cf1f26c5904ee14b3048ebd5-20109_Inspiron_3525_1.11.0_BAR.cab
[47]: https://fwupd.org/downloads/0f454eb6847962f80d9553c0478e393f3ff17c113a36f7577121688a86b59da0-20995_Inspiron_3525_1.12.0_BAR2.cab
[48]: https://ftp.hp.com/pub/softpaq/sp144001-144500/sp144428.exe
[49]: https://ftp.hp.com/pub/softpaq/sp144501-145000/sp144556.exe
[50]: https://fwupd.org/downloads/73120509453a46a8cdbdde761168b94553cab6821f481966eed308cb9d6ccaae-U82_01011200.cab
[51]: https://ftp.hp.com/pub/softpaq/sp147501-148000/sp147511.exe
[52]: https://ftp.hp.com/pub/softpaq/sp144501-145000/sp144570.exe
[53]: https://ftp.hp.com/pub/softpaq/sp146001-146500/sp146050.exe
[54]: https://ftp.hp.com/pub/softpaq/sp146001-146500/sp146071.exe
[55]: https://ftp.hp.com/pub/softpaq/sp144501-145000/sp144566.exe
[56]: https://fwupd.org/downloads/103d0b1c7b1a590addc988fc0e75e5c6bfd407f0baa98238dad8f36c9318f9f9-Lenovo-Think-M75q-2CZN-SystemFirmware-M47KT28A.cab
[57]: https://fwupd.org/downloads/9698b99476de8b9871c0f7bdea91206edb084abb86264088eda2035f5209ad9e-Lenovo-Think-M75q-2CZN-SystemFirmware-M47KT29A.cab
[58]: https://fwupd.org/downloads/ed07eb9658abf0a4fb6041224562cac2c65af77252e16408daa72161e3e4a9d4-Lenovo-ThinkCentre-M75s-gen2-System-Firmware-M46KT52A.cab
[59]: https://fwupd.org/downloads/22d398ea3fefecfb98b4c980528a79f55025309b9b0087cff9d49021e312617e-R26ET32W.cab
[60]: https://fwupd.org/downloads/f54b1f144174a25580d92f69cfa30447be9e229cf937e6098c00dcdec9c2ecac-R19ET36W.cab
[61]: https://fwupd.org/downloads/2581b637db47e5d6f41838c62a293d0541355f84582582cafe41e6b8a8652768-R19ET41W.cab
[62]: https://fwupd.org/downloads/ab37068111cc0712aa1b6d421040eb73ac9f308c650a51d52bcd6e8c0fe6067d-R19ET43W.cab
[63]: https://fwupd.org/downloads/3d41a16a15a882cd8b4eb1855716a35911b84958b083b90d6e0bc0b9d1191a82-R19ET44W.cab
[64]: https://fwupd.org/downloads/4e0c630c9b5d7cdabcea512683a3b4414b9eeabb31e4ab54209b19bcef3b6119-R19ET45W.cab
[65]: https://fwupd.org/downloads/e5411986e1a36e11691c3343843e4ad7ef7bbc6bc403af8e12542ecc1b627c29-Lenovo-ThinkPad-P15vGen3-AMD-SystemFirmware-1.10.cab
[66]: https://fwupd.org/downloads/79f97157d337c279fafb129f6a2e986bb430d89f019c66cc6b1ad8d09fa1e1a2-Lenovo-ThinkPad-T14-P14s-AMD-SystemFirmware-1.34.cab
[67]: https://fwupd.org/downloads/5a163089afa64ca6a2ba379c8a6a66ffce1026941a0a8a73dc7a7d528be35b1c-Lenovo-ThinkPad-T14-P14s-AMD-SystemFirmware-1.35.cab
[68]: https://fwupd.org/downloads/83a832855c7187f6951f8e9e8b0620a9a70cf1d836a63ceb5e6b9d278d5c6396-Lenovo-ThinkPad-T14-P14s-AMD-SystemFirmware-1.36.cab
[69]: https://fwupd.org/downloads/3181bd6e9dbe86ee80c2cc28e8a3e32d5e1caf0d05f2bf629e5b12818f284dca-r1bul68w.cab
[70]: https://fwupd.org/downloads/2f25c9a9a62861d8f20ffb1b57fa00019a5c360a06f1eab883eb3129467bb9a6-r1bul71w.cab
[71]: https://fwupd.org/downloads/c1ad6495c718d8d13365b98f2057d11a4423d4ef617f5aeca83d618e1d6386fd-r1bul72w.cab
[72]: https://fwupd.org/downloads/6b9395807b51b62fcb62492b2c09f8e66d4a12f3760ef887fdc15a566cb8fac3-r1bul73w.cab
[73]: https://fwupd.org/downloads/7d75287704a454afef9808855fd78cc57d625515c3979fa6c46800490c00a7e7-r1bul74w.cab
[74]: https://fwupd.org/downloads/2845a4d34bf94b29b5e3459cd68aee5b3f1601a6d4811a2d187b53cb8310c4d5-r1mul49w.cab
[75]: https://fwupd.org/downloads/03f20bb8621cc11b5023974018d89809cceaeb83fe5b038e578f06941688f071-r1mul51w.cab
[76]: https://fwupd.org/downloads/782ac622fe69e5779b3987d35f8fc292623cb013eb8d6234581d3fc3d5915714-r1mul53w.cab
[77]: https://fwupd.org/downloads/08d448215307b8e5f8017e216a50e82439cab6a77721c7113e8f06a7f4cf6b8a-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.34.cab
[78]: https://fwupd.org/downloads/091dbeeb56656a632d315209a3fcd9d2d4408285cb185650d1ce11f00dfe6a40-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.35.cab
[79]: https://fwupd.org/downloads/b6b6eca6aa303b79e7637be727c3dc0876074c266a454d3c581ae8072ac02f36-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.36.cab
[80]: https://fwupd.org/downloads/0b6872fd950e1d82c0b498bd8b24c5ce660909160838d8fa74cd82696c897292-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.40.cab
[81]: https://fwupd.org/downloads/122d2dca1de34ee3635bc5657ea45b0cf8df57eb89aec338ff87eca182a3ce6b-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.41.cab
[82]: https://fwupd.org/downloads/def7140be125322a9ba5c4183c3863eaa1bb533f2746c8eac3879d68187cf2ff-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.42.cab
[83]: https://fwupd.org/downloads/c3a3324390aedac3a9f2964b66045b612b09ec67671baeb5edef8af38158ca80-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.43.cab
[84]: https://fwupd.org/downloads/f3b732d77cee87d9b6f0d7c55ef7a6cba80ddad2ea534add6f327b48ed6b9195-r1nul56w.cab
[85]: https://fwupd.org/downloads/e990348e54a60fa0387dca55ede0705ad49d9f5e7078521bfabcc11fb7b4cd6e-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.30.cab
[86]: https://fwupd.org/downloads/2a0ea63c2b23ad35350d5a11e1586dd18d955899a0d49def104a8ecaa2e74469-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.31.cab
[87]: https://fwupd.org/downloads/8954258f2285aa17f1cade983bb6315b80645573bd07342024e4d84511218114-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.35.cab
[88]: https://download.msi.com/bos_exe/mb/7C91v16.zip
[89]: https://download.msi.com/bos_exe/mb/7C91v19.zip
[90]: https://download.msi.com/bos_exe/mb/7C91v1C.zip
[91]: https://download.msi.com/bos_exe/mb/7D75v10.zip
[92]: https://fwupd.org/downloads/4634cbde7a64a186d6af29065e3bd7b361df9cee3d3f5b72b0fd99911f8bf2d2-23344_Inspiron_3505_0A11_1.16.0_2.cab
[93]: https://fwupd.org/downloads/57769e08500ec07404e0d64e0c5b1079faeb0a6829407adaaae944dad0e33ee1-23762_Inspiron_3505_0A11_1.17.0.cab
[94]: https://fwupd.org/downloads/82c8aed0aa0b5026e276d44533700765b59aaf0fda19d7c4b56315a461fa7074-24173_Inspiron_3505_0A11_1.18.0.cab
[95]: https://fwupd.org/downloads/b1361f3d4a95e874f2cadb81b894f995d8cf9c209a057d39f531bd3a1c62973c-Inspiron_3505_1.25.0.cab
[96]: https://fwupd.org/downloads/60ea62298043c275b67ad1b6f1092ce51ac5f841182e9e39a6ddd401d45cd927-28732_Inspiron_3505_1.26.0.cab
[97]: https://fwupd.org/downloads/d6ac9463f1d325e39c15cf6d3e83a253de8bf0f0a0f752c9a0f92a817e1d1010-Inspiron_3505_1.27.0.cab
[98]: https://fwupd.org/downloads/73caad9faff3bcbec458fc5915cd1a11a9f87c39598b4c01587fd8044f078206-34800_Inspiron_3505_1.29.0_1.cab
[99]: https://fwupd.org/downloads/c5df6f2ecd1d616a2d774895fcdff2dc4fd973dfeaa71b10fbfebb8494337066-35468_Inspiron_3505_1.30.0.cab
[100]: https://fwupd.org/downloads/276c5d3bc69b6c7f77c12d2ffb2319158a33709fe6b6c2eab620e5b5085c593d-38545_Inspiron_3505_1.34.0.cab
[101]: https://fwupd.org/downloads/09491d6971096fe51e8664f111334eb8d7552f358aa1c16de0e19ec20990b529-44323_BU1360.cab
[102]: https://fwupd.org/downloads/22f8c860ea5af4ae4ab0393982f4d89a094f0e05bb9683ce6a3b6d48bca1d3b7-23296_Inspiron_3515_1.16.0_2.cab
[103]: https://fwupd.org/downloads/d54705e7f6fa90b4c698fb13bd06017f60606780b3d01ee3175d5cd332276099-23345_Inspiron_3515_1.17.0.cab
[104]: https://fwupd.org/downloads/a30e7a7c3af305f4c6a1f590168b9c7633c19f1500b13bea734e6afcccdb1582-24174_Inspiron_3515_1.18.0.cab
[105]: https://fwupd.org/downloads/9a255ff1cf9bbef2d46808c790d29beb5610be8cefeb5ffc1c5eaa6132f90713-Inspiron_3515_1.25.0.cab
[106]: https://fwupd.org/downloads/284d611dd4f29b0fb281bff62c3602baa210b952f2391c7fc742abc2b4e851da-26493_Inspiron_3515_1.26.1.cab
[107]: https://fwupd.org/downloads/35c56b9faba0c947f604d1bd18d025a27c4be5d63b54c063d1ec0793753b3e1a-28733_Inspiron_3515_1.27.0.cab
[108]: https://fwupd.org/downloads/4bdbc41dce5c4a01b56a835952dba48fb8a3ff5be9616fca87dc3a7c439daff1-Inspiron_3515_1.28.0.cab
[109]: https://fwupd.org/downloads/534a874ca13528cfa68936ca4e40af78d4424a3a0828e26b2e7bdb9f3d982b67-34770_Inspiron_3515_1.30.0.cab
[110]: https://fwupd.org/downloads/145f4516a522b97428ba92339e227c50a7cae49e3e8fe074f15c4fcad9047e90-Inspiron_3515_1.31.0_3.cab
[111]: https://fwupd.org/downloads/610fbd9f3ba77e2a8e5784b53393f3883e0354b647d64868fda6aa3c176e2e47-38546_Inspiron_3515_1.35.0.cab
[112]: https://fwupd.org/downloads/aea5c4bc3526a92d322732cf7858ce41b5fb42f69d7d629b5870ef252e43200e-44324_WAR_1370.cab
[113]: https://fwupd.org/downloads/78db127abfd75a0471f549b5dd1489567283ec3f46a93e09c2190e895e85100f-18739_Inspiron_3525_1.8.0_DALI_32.cab
[114]: https://fwupd.org/downloads/0f96e75de7e95017cf25a2e48cf64441a8d0a284f9b2a324457698c30c616f47-Inspiron_3495_1.5.0.cab
[115]: https://fwupd.org/downloads/5fa8b2268c1eecf91803217ef62f756dd818900d878513edca95d54f2954ae76-16900_Warlock_15_PIC_DAL_1.9.0.cab
[116]: https://fwupd.org/downloads/a3dbfb8f7f7939cbffd54c1a73b0a5c529287e37bb7c126192eb9710c2c2c8ef-19718_LOKIN3_AMD_1.13.0.cab
[117]: https://fwupd.org/downloads/ab226e098197c53533dc04f6447e7d8cbb1650f3f0850f24e709fab49fa8241f-Inspiron_3585_1.10.0.cab
[118]: https://fwupd.org/downloads/2ba42fa0b6e701c39b034e4ecbbe38ea2c04ab8f5a5e157d6503357b2a445b43-Latitude_5495_1.8.1.cab
[119]: https://fwupd.org/downloads/8c3421db28fdb7522b1d1bd4f38d205e5bea48c0f9041490e2e430beb77058b9-Latitude_5495_1.9.0.cab
[120]: https://fwupd.org/downloads/45f6a8c1caf791c19c24c7c3eb5d4cd46e1684cb9acfb66d9d9d484bbeceffba-Latitude_5495_1.10.0.cab
[121]: https://fwupd.org/downloads/699106cb4b7402e992ac9c0e48b043995b0827eb7dffbd45819a688ffeef95e2-Latitude_5495_1.11.0.cab
[122]: https://fwupd.org/downloads/edf84b2fa70d728a1daa59b837222161359528c0887996a7e7c65d534f1ec39c-Latitude_5495_1.13.0.cab
[123]: https://fwupd.org/downloads/2884c29a5394abf0a6b346c457c7cc07c2da26c741cc9956f9c570255a44df36-Signed_1152921505697273397_WT_1.14.1.cab
[124]: https://fwupd.org/downloads/a5c47016fb732c8c627708903c59d1bd1c16961d9d7e7fe0a1d645b3d2220148-Signed_1152921505697331338_WT_1.15.0.cab
[125]: https://fwupd.org/downloads/e6458c604a657ccb3d48aff1bc98d5c2eeb2f16fe57b0dc3478e2bf79cff58b3-Signed_1152921505697989245_WT_1.16.0.cab
[126]: https://fwupd.org/downloads/10066c4fd61b2ac93f18f2ae713d6c2773f2440eb515fc086e369c575995884a-Signed_1152921505698533787_WT_1.17.0.cab
[127]: https://fwupd.org/downloads/c4e53a6e67356d5ec53e847f82f46ab8ef3f2372ae8fc225ec0a73a1742105c1-Signed_1152921505694741907.cab
[128]: https://fwupd.org/downloads/5e1b9d39ba77bb7bc8f87516f5061b707d96a4ded1ff0a10a372f4ff58358a35-Signed_1152921505695135256.cab
[129]: https://fwupd.org/downloads/51b292f507333a648e0fd08e4ccc701f528e1aec6452fde19ff5b1abde805133-Signed_1152921505695258710.cab
[130]: https://fwupd.org/downloads/a6e3a80b4b47273eed621fff19693f8d985ecde26a3ca0e011b90a7888878441-Signed_1152921505695539491.cab
[131]: https://fwupd.org/downloads/fe1f299db1b0999ce7b9184ba603ce7914ca956d145fc672e23ce4a8cd3f6e5e-Signed_1152921505696768547-BR.cab
[132]: https://fwupd.org/downloads/5d7742c25145a535c2a9cb8cf2662e6786c1f45a3230f10ca5ec0ccaa4f0a496-Kona_Bristol_firmware.cab
[133]: https://fwupd.org/downloads/fbcef9183886dd9f2df90dcf4fb2b60ddec15bec4a2b95e1ecd6b956e7691b77-Kona_Bristol_firmware.cab
[134]: https://fwupd.org/downloads/4e1dd9de4a7f3ecafb9008a76d9b793e69e05aadc35c70f89da94652a3dcad7c-Signed_1152921505694741618.cab
[135]: https://fwupd.org/downloads/1eb2dbb4dfb179d237f10638beed524a4d9a6d05ea8a5e9bb92d269e77e0d6e1-Signed_1152921505695134870.cab
[136]: https://fwupd.org/downloads/404196ba0ba118033ea3522341676bd20edbdb29a7164f78308c31153b5bb5ac-Signed_1152921505695258524.cab
[137]: https://fwupd.org/downloads/58267869d70ae96e14ef21cfbde5aaef4cd7342151565d28c6fbcf390ac80dfd-Signed_1152921505695539403.cab
[138]: https://fwupd.org/downloads/77cf2c5ffe27b5c6cb7a5dd2bf9edf47c467e003c7ec86e93bbf9f2e99645e0e-Signed_1152921505696768424RR.cab
[139]: https://fwupd.org/downloads/1ce3cecf42408ac65659b1ae8ddc090fbcae960de8952546d0c285145e4dcbd1-Kona_Raven_firmware.cab
[140]: https://fwupd.org/downloads/a331d3d1d7bbb7bdc12e7d41a1aa8d8fa979ac44e4bbe0e4d54ac9bac1e271be-Kona_Raven_firmware.cab
[141]: https://fwupd.org/downloads/2c5a3618b506a3cb148216e341806afae1ff5143f22a7886b735dd5bd1288d5d-Signed_1152921505694742203.cab
[142]: https://fwupd.org/downloads/bee61a0213046e316bf43f5374a8f6c95964c9849629216f5bb12fcb5c7f0f20-Signed_1152921505695135259.cab
[143]: https://fwupd.org/downloads/9a9ae5030bc7b22742fb89ae2385d28f61dcc327d98467e3214a929d26febc8f-Signed_1152921505695258711.cab
[144]: https://fwupd.org/downloads/779adf9cb9274c9686bd0e015ebff58cb0fd4111ef50a19708afe29d4917d01f-Signed_1152921505695539507.cab
[145]: https://fwupd.org/downloads/23f83831c140ec929823663bf12681710b05f29e23bb9980289b8db8a3699615-Signed_1152921505695927272.cab
[146]: https://fwupd.org/downloads/298e43140935d0c9984396f7a42ffe3f4e0844315132ae3ff78c2b8c5d8757c7-Signed_1152921505696768084-SR.cab
[147]: https://fwupd.org/downloads/e55cd969e521d027ece32d3785f33b646cf67272c0b4e89cab0ada6a27a74471-Kona_Summit_firmware.cab
[148]: https://fwupd.org/downloads/5fcb4670d24c246423dfaa4b9a21490b4c5daea56887a35ac7a788fae8e0d3fd-PantherCglEvt_firmware.cab
[149]: https://fwupd.org/downloads/0250b26a6ba28567740844a563527bd68bc16a99832d632b24975648268d5d70-PantherCglEvt_firmware.cab
[150]: https://fwupd.org/downloads/e18342615b21cd16f069d8fa8f3c7496da37e62d5e6b4143e4bee4c58e092031-PantherCglEvt_firmware.cab
[151]: https://fwupd.org/downloads/0aa9b3d7fbe1a0b384e825457eabb763469506d721f3002dcc33031e73efde29-PantherCglEvt_firmware.cab
[152]: https://fwupd.org/downloads/18a0fc32a28afed8650ae5c0d259d9f5fd2887664133bcae20502f4fe6aead93-PantherCglEvt_firmware.cab
[153]: https://fwupd.org/downloads/35b57ca2038c1fe78e0e8ec3cd98fcf1057cc68c7b4de29d8f4c10f31d6b5331-PantherCglEvt_firmware.cab
[154]: https://fwupd.org/downloads/0348450fb6a32fe31c9f139ce289733f197f879970e63c888946c2c9ca0540fe-Precision_7865_1.26.0.cab
[155]: https://fwupd.org/downloads/8ed0a6f7ca04d7da1ab3fe709b6badd1f617e154f9a6e0ec95e701542953dce9-Precision_7865_1.27.0.cab
[156]: https://fwupd.org/downloads/01134b65cb6d273321083942eb31241901152f0f8edbfa6e28ce52f0249d5baa-PantherCglEvt_firmware.cab
[157]: https://fwupd.org/downloads/ae3f416e32bc64f01961a3c7512d1163c91ceb396322ad040f392a17f45a6061-PantherCglEvt_firmware.lvfs.cab
[158]: https://fwupd.org/downloads/7ec0406d41aed4a2645e7aa48010b587bc3e9b5946bfe4a5dfbb65026da5f668-45322_War_mlk2_PV_170.cab
[159]: https://fwupd.org/downloads/dd917aba6d2a9fdcda6f0205d26ebdafcb6447766dee40a88f7a3d97e5aa1482-firmware-f04.cab
[160]: https://fwupd.org/downloads/cb39cf2db123b11657b8aed144860269282916b7da9e539f47e93d95a04505ae-firmware-f05.cab
[161]: https://fwupd.org/downloads/e0a0793923ea987764bd1f5674012b3dfebf641d2158a63773e03617833e01c5-Lenovo-ThinkCentre-M33-SystemFirmware-M33KT1FA.cab
[162]: https://fwupd.org/downloads/06b28816f697123091c8f9b7810c66efc4261aa9291c37e432944fb064f1c2e4-Lenovo-ThinkCentre-M33-SystemFirmware-M2AKT20A.cab
[163]: https://fwupd.org/downloads/4f102b169a25d4f7af5a0e2707f7c26a58192e6cbf32ac730d352e1c8168bb37-Lenovo-M75n-M33-SystemFirmware-M33KT22A.cab
[164]: https://fwupd.org/downloads/5d1d5061d755f98e4ad5f50eb0bf93efcfd80bae342ce189ced6690729d9a9fb-Lenovo-M75n-M33-SystemFirmware-M33KT23A.cab
[165]: https://fwupd.org/downloads/340c9092a4ded17ec511d1f920bbaa438264b48fde112da66e476d947c0a51bd-Lenovo-M75n-M33-SystemFirmware-M33KT24A.cab
[166]: https://fwupd.org/downloads/ed2ecfe3e2c309b717e70ecb513e572204ffcd3789d88b44e6cf276376a2de54-Lenovo-M75n-M33-SystemFirmware-M33KT25A_0509.cab
[167]: https://fwupd.org/downloads/8c9feac912edf29508bcc970bade81e4d5f67e2dfe2689a2e7cc18f24007c43d-Lenovo-M75n-M33-SystemFirmware-M33KT26A.cab
[168]: https://fwupd.org/downloads/fe3824d8620c5ab2a9364643e52119de3f1e23b10c4bd1fdd014d7ec738e2fcf-Lenovo-M75n-M33-SystemFirmware-M33KT27A.cab
[169]: https://fwupd.org/downloads/3a5028716920a5c5c98e30da4db187f113597714ab5198267b1fd18a677754bc-Lenovo-M75n-M33-SystemFirmware-M33KT28A.cab
[170]: https://fwupd.org/downloads/3cd9435578cd4b8982e2038116e4049290da9fb5c09f392d295673e5a6dc42e4-Lenovo-M75n-M33-SystemFirmware-M33KT29A.cab
[171]: https://fwupd.org/downloads/590ea11966cc10cac31c2ef62a6115f64967a3cba422f358f07e1637e5639f3d-Lenovo-M75n-M33-SystemFirmware-M33KT2BA.cab
[172]: https://fwupd.org/downloads/3afa17bc2d201676c4af42d0932c46afa99754cb8c807845e8c56c26117d0188-Lenovo-M75n-M33-SystemFirmware-M33KT2CA.cab
[173]: https://fwupd.org/downloads/564e9e55b026c992f5062b43dfd60a9b52ecd4bc1f3d1f62332a1fe10b18013c-Lenovo-M75n-M33-SystemFirmware-M33KT2DA.cab
[174]: https://fwupd.org/downloads/35e2768c0a8859bfcdb2492b958f5cdb578180346b06b805973e1ce2cff10252-Lenovo-M75n-M33-SystemFirmware-M33KT2EA.cab
[175]: https://fwupd.org/downloads/32003fa13c063eb5e78ba7a17f1a30da4093ff2af6a4de8689bd9a99edc3c885-Lenovo-M75n-M33-SystemFirmware-M33KT2FA.cab
[176]: https://fwupd.org/downloads/904ffd9e8cbf0f4e734e3e01f8ae05942244cfc740583fef6f97843506b82065-Lenovo-M75n-M33-SystemFirmware-M33KT30A.cab
[177]: https://fwupd.org/downloads/aa238904112204078aac1b4a50ff87fdaafae1f56bab1d9078a08a92dabd26b5-Lenovo-M75n-M33-SystemFirmware-M33KT31A.cab
[178]: https://fwupd.org/downloads/d052919ecd30180a84ca5880fde9f32fd1d5920a40b8f2bbaa656ea8ae46c809-Lenovo-M75n-M33-SystemFirmware-M33KT32A.cab
[179]: https://fwupd.org/downloads/01c2c5759e6a7c8c432612888df9707ae77fbf461b0c682d40bf6774f5ea6154-Lenovo-M75n-M33-SystemFirmware-M33KT33A.cab
[180]: https://fwupd.org/downloads/1148215aad841f73651c93efa1ad8379ef412ffbc203613e1184337eb47cef8c-Lenovo-ThinkPad-T14-P14s-AMD-SystemFirmware-1.25.cab
[181]: https://fwupd.org/downloads/5b96fea64f3f3478a27ffab786ffea3a6d9ccbbf-Lenovo-ThinkPad-T495-SystemFirmware-1.16.cab
[182]: https://fwupd.org/downloads/5284633f364fe7096f2c887bcc56138909038c0b19f48c2345209cee8315f4e8-Lenovo-ThinkPad-T495-SystemFirmware-1.19P.cab
[183]: https://fwupd.org/downloads/6752fcf5c616db695b18d7f5f18ecdd79f68a7f3b78fb655195e4a5ca38d07c4-Lenovo-ThinkPad-T495-SystemFirmware-1.20P.cab
[184]: https://fwupd.org/downloads/6c942b5d2c9da158750c7c62ea893d569fa6772f389dc5421d4a10bb8c4eb318-Lenovo-ThinkPad-T495-SystemFirmware-1.27P.cab
[185]: https://fwupd.org/downloads/813b8f7fa4d29f76da2372c3ab9b3c580d98b82a95133c93e6a43006b992cef1-Lenovo-ThinkPad-T495-AMD-SystemFirmware-1.37P.cab
[186]: https://fwupd.org/downloads/4e0a62709f5960b1c4c32f0f0abbdce6a64b9944802289ced8766b7ee2e1dc74-Lenovo-ThinkPad-T495-AMD-SystemFirmware-1.38P.cab
[187]: https://fwupd.org/downloads/4b846bb9a2e9841f439b109ceab8aca97c75e504e6a4a2dfaee6d0c1ccb22012-Lenovo-ThinkPad-T495-AMD-SystemFirmware-1.39W.cab
[188]: https://fwupd.org/downloads/6d08e2bee0f8ad7d75dd2090e41eac232480af58fb1791512603bed70ea7fc7b-Lenovo-ThinkPad-T495-AMD-SystemFirmware-1.40W.cab
[189]: https://fwupd.org/downloads/03a5c53998840ca4b28139a38f39a5fc5be69a271ba4a6aaa79e2ea2fb4212e2-Lenovo-ThinkPad-T495-AMD-SystemFirmware-1.42W.cab
[190]: https://fwupd.org/downloads/4e86901643441fe7bf4082090a4c6fc59e87c4545f859d001bbdce097e895b80-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.25.cab
[191]: https://fwupd.org/downloads/38a49743c685c911cdaa04a1e19ac603fceb8ab48b972023add330dd829d2100-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.29.cab
[192]: https://fwupd.org/downloads/637e6f0de80dcfd8eacdc228e8ee4c5a90e4b4b5cd8e4e40eb1e96a43f8e39a9-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.30.cab
[193]: https://fwupd.org/downloads/bb60ba4757461c0bb8fcd57fa54558affbf4edfd-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.14P.cab
[194]: https://fwupd.org/downloads/63fa9f4ba349f2dc22272182f7d70151647b766435e46c42c9e2bb0acb6f338a-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.15.cab
[195]: https://fwupd.org/downloads/1b1978c0f45977a037078db52094f7ffd7da7be6d366eb0c6eee4fb4cbf74540-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.16P.cab
[196]: https://fwupd.org/downloads/99319f7326854a0a6a4d68145ccc33200a2abcb1750e076ff754caccfdf8757b-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.22P.cab
[197]: https://fwupd.org/downloads/5e49b82f571ead8f159f00c0fb20de28c150995529d6efedfb1ee2f089a55891-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.23P.cab
[198]: https://fwupd.org/downloads/0c07144e2a7d47509d78b6127791e03175db5f6072d985fe3852cbd3fca914de-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.24.cab
[199]: https://fwupd.org/downloads/31a233518dcb2c6c520015584f1c68d025dfd41647d8329faf3b8a0951131dc5-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.27.cab
[200]: https://fwupd.org/downloads/6e22da27f0335f175c945528e94f8b2c06c6b242bdd5ba34af41414537796d9e-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.28P.cab
[201]: https://fwupd.org/downloads/ccb4d72559e39e22b7d020e7920b02f9b75fe96579b5b2894b02b3f737ad9a94-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.29P.cab
[202]: https://fwupd.org/downloads/b8e555bb4a4592d49720142bbdad1c289c968d4e6fc3301e3279f53f5703223b-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.30P.cab
[203]: https://fwupd.org/downloads/1ca9f33b77e40ec9546414ed3d6dd50cbcb19b59ba9ee2bff21d24eae4ba61d6-Lenovo-ThinkPad-X395-T495s-SystemFirmware-1.32P.cab
[204]: https://fwupd.org/downloads/98d66431f51b6710b7b45dcb4e0a00b6a8d3d126cb6c149b8e4c72a556d2ad30-Lenovo-Think-P620-System-Firmware-S07KT1EA.cab
[205]: https://fwupd.org/downloads/b3789e1cdfb2a3ee5a7ffd4d477a28c4324abddde07ff507a6d633a030c9a3a8-Lenovo-Think-P620-System-Firmware-S07KT1FA.cab
[206]: https://fwupd.org/downloads/21a400304d9fe064d3a0cdad154d5a5bfc00aa70348461775f9792b23d3d9d32-Lenovo-Think-P620-System-Firmware-S07KT23A.cab
[207]: https://fwupd.org/downloads/ad1c58bfc59b20b16340b00fb6dd82cc11dfd039a9f7777616e187a3f4f52ebf-Lenovo-Think-P620-System-Firmware-S07KT24A.cab
[208]: https://fwupd.org/downloads/e624a9679f0c0086d44b1431a24f27166c9dad17e9305f66fcb24104ea1aefa9-Lenovo-Think-P620-System-Firmware-S07KT29A.cab
[209]: https://fwupd.org/downloads/f3d7fab5819698b82c0c6a57a7c39b9a4e4976564c0c22b1090daf99af5ac675-Lenovo-Think-P620-System-Firmware-S07KT4FA.cab
[210]: https://fwupd.org/downloads/4b2714a46b7cc714740e19ba12366092c108a8170d2b51d46f2523acb37db65e-Lenovo-Think-P620-System-Firmware-S07KT51A.cab
[211]: https://fwupd.org/downloads/f5177248d7a4c1254ce57d04bab60e90ee57369caba8301d2faf4f7ca1baf230-Lenovo-Think-P620-System-Firmware-S07KT59A.cab
[212]: https://fwupd.org/downloads/8cfbadf76040ec9e3061deda1435b9e59213ba4d9e22ddbf17368046ee115335-Lenovo-Think-P620-System-Firmware-S07KT5AA.cab
[213]: https://fwupd.org/downloads/9fd84896d4d0f50521f713ba682bdac689dd8c1d65a9b6dbd88ed39bec7372fe-Lenovo-Think-P620-System-Firmware-S07KT5BA.cab
[214]: https://fwupd.org/downloads/41eac58ad323276fd7ef242a082e7d7e3e7dff1450449f53447495b035fea3d6-Lenovo-Think-P620-System-Firmware-S07KT5CA.cab
[215]: https://fwupd.org/downloads/d60b0f463c2dbd1e23122b25fe5bbc78c328b01ac851de193faabafbcab01fd5-Lenovo-Think-P620-System-Firmware-S07KT5DA.cab
[216]: https://fwupd.org/downloads/e457333abf27983c019128aa4526b893dd5e048aad34df073dda2924f6b7e757-Lenovo-Think-P620-System-Firmware-S07KT5EA.cab
[217]: https://fwupd.org/downloads/d739d02b148175912d6856e133ffdfc97036bc4138836789d867646c06fd1d37-Lenovo-Think-P620-System-Firmware-S07KT5FA.cab
[218]: https://fwupd.org/downloads/367b1b4e4dd67131a18a6c338817847b42345a32d6d874c28fb5722bf5a71583-Lenovo-Think-P620-System-Firmware-S07KT60A.cab
[219]: https://fwupd.org/downloads/7f4f00386e3e32356b8d4356ec6573499fa2cb81534b43d4929bc44247039e31-Lenovo-Think-P620-System-Firmware-S07KT61A.cab
[220]: https://fwupd.org/downloads/cbd6c9e6885f09eddb08cd2fff0dc9a734db3b0dbcd6829d1d72c22422056969-Lenovo-Think-P620-System-Firmware-S07KT62A.cab
[221]: https://fwupd.org/downloads/45a51c9785aee221512e10720d93befdb1f9a631f4e71def983a20a836d04972-Lenovo-ThinkStation-P620-System-Firmware-S07KT66A.cab
[222]: https://fwupd.org/downloads/b823508c3420108e358c12d9242b8d78cdfbdc9c95814be94a8e77d03120ef9d-Lenovo-Think-P620-System-Firmware-S07KT67A.cab
[223]: https://fwupd.org/downloads/6b0d70e709457680b3bc54a52c3bed77f7f9ae7e768d1a2937a22f058a7a42f6-Lenovo-Think-P620-System-Firmware-S07KT69A.cab
[224]: https://fwupd.org/downloads/9e594920c6848eb3cfe91570b544dadcc93fd729b6a009020ec7ff9ea5522355-Lenovo-Think-P620-System-Firmware-S07KT6BA.cab
[225]: https://fwupd.org/downloads/f204e30014495d8e7f0732f13b0c6ea9c09960fc1ee5117c992a38f4a64095c0-Lenovo-Think-P620-System-Firmware-S07KT6CA.cab
[226]: https://fwupd.org/downloads/4736f5ad331dcf41d8d4180b3b114eaa8f67af60255e79fe796ef3b87331aecd-Lenovo-Think-P620-System-Firmware-S07KT6EA.cab
[227]: https://fwupd.org/downloads/638325580f64449d508ad4906da27b71c9bb1d099c8cdea42e69200cacf7e37b-Lenovo-Think-P620-System-Firmware-S07KT6FA.cab
[228]: https://fwupd.org/downloads/5f3dd40729cf837dda90eadb5fe40c8cd03d49f2883f16ca7260a938bb6a6187-LFGB0_303.cab
[229]: https://fwupd.org/downloads/8705fb9b1f8803d8e92b5279f9ad4a2df2ae5775bae4a8ce622fe7acaa58d507-LFGB0_304.cab
[230]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3202.zip
[231]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3205.zip
[232]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3404.zip
[233]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3405.zip
[234]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3607.zip
[235]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3611.zip
[236]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3621.zip
[237]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3631.zip
[238]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3634.zip
[239]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3636.zip
[240]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3644.zip
[241]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B550-PLUS-ASUS-3645.zip
[242]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1409.zip
[243]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1410.zip
[244]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1413.zip
[245]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1414.zip
[246]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1601.zip
[247]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1616.zip
[248]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1636.zip
[249]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1654.zip
[250]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1656.zip
[251]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1807.zip
[252]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-1811.zip
[253]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-2214.zip
[254]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-2412.zip
[255]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-2413.zip
[256]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-2613.zip
[257]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3014.zip
[258]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3024.zip
[259]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3035.zip
[260]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3042.zip
[261]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3057.zip
[262]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3067.zip
[263]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3201.zip
[264]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3208.zip
[265]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3222.zip
[266]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3263.zip
[267]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3279.zip
[268]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3283.zip
[269]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3287.zip
[270]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3602.zip
[271]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3827.ZIP
[272]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3842.ZIP
[273]: https://dlcdnets.asus.com/pub/ASUS/MB/BIOS/PRIME-B650-PLUS-ASUS-3854.ZIP
[274]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3881.ZIP
[275]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-3886.ZIP
[276]: https://dlcdnets.asus.com/pub/ASUS/mb/BIOS/PRIME-B650-PLUS-ASUS-4004.ZIP
[277]: https://fwupd.org/downloads/0ae26584f1e3a796e6edbe3a6d1d8c50e590f60350a4b64473bb449c313cd119-firmware_hwk_1.13.1.cab
[278]: https://fwupd.org/downloads/55636b54e5884824f8a58aad43f926c2600cf39dadedcdbd8f13c7976e37f530-firmware_hwk_amd1.14.0.cab
[279]: https://fwupd.org/downloads/5d12fddeeb301c236bb007500601202215aec201fc340ec59ca445b9dded4102-HPT_1.15.0.cab
[280]: https://fwupd.org/downloads/cd49c9fa940e8ed3fe65cb2d3257d2d2d48fd95f3bcf6427b14c435a04a4a311-38435_Sentry_MLK_101.cab
[281]: https://fwupd.org/downloads/b7fc2bc6b51416ac4043a143c56c469cc4aab438ff83f91790790b0175583f8f-38788_Sentry_MLK_112.cab
[282]: https://fwupd.org/downloads/a93b17963e763a1c1371fc2288ad613461e6b20e9a08f351bc6b27c024456553-39606_Sentry_MLK_120.cab
[283]: https://fwupd.org/downloads/48c1d0a9a51552ea6974ce3c8c64a08e11d3903742e369d4855d080259f438b9-40387_Sentry_MLK_130.cab
[284]: https://fwupd.org/downloads/e9bdb265688e9013911bb9fc088d38dacf2484096f77918d3f53eebd75e8e2cb-Sentry_MLK_151.cab
[285]: https://fwupd.org/downloads/5a66f34ff60fcb282351213dc54837ae6eb3fa0540351a8f6ecad75fbd9ddf0d-SEN_MLK_170_2.cab
[286]: https://fwupd.org/downloads/0f2a353bfba364bcc1f207803e6a0100cb78a37e2ede50ee7a5e256b4fcfdf51-22454_Inspiron_3525_1.13.0_DALI.cab
[287]: https://fwupd.org/downloads/0bd7ac99f0b3b57a591a583147c674a2669bb77ca6fb2ef88cf567a2b076e6d0-23284_Inspiron_3525_1.14.0_LCN.cab
[288]: https://fwupd.org/downloads/4bdd9a0711a7c1cb849d8391c321934ec92feb225fac1882c638cb643e9272d5-23348_Inspiron_3525_1.15.1_LCN.cab
[289]: https://fwupd.org/downloads/b493b0b965cfea9f41e4cd86b9ebebfbebd8a6504088964695462ea447f1212c-23765_Inspiron_3525_1.16.0_LCN.cab
[290]: https://fwupd.org/downloads/7ff4b4f1b232f047dc822253ed664efba86b8afb05afd820d717eabe82f39007-24177_Inspiron_3525_1.17.0_L.cab
[291]: https://fwupd.org/downloads/6d973dd97d365cedc647ee5ad60828fcd903abb15dfd3e3f5f18272182a697f8-Inspiron_3525_1.18.0_D.cab
[292]: https://fwupd.org/downloads/130e3e236b49b86565f11f3ee9d6d6b79fefa9d86e72f89c5255d02eb49e499b-26492_Inspiron_3525_1.19.0_Dali.cab
[293]: https://fwupd.org/downloads/03be209a80049989559b278f0c1f7f596f2738d9e465a2c88d3c0a3c0f9e1e3f-Inspiron_3525_1.22.0_DALI.cab
[294]: https://fwupd.org/downloads/3e78bb8f0ae115561bc32be5e2a06bb0ac7f09f3a7591862e230129fb1ca2910-34772_Inspiron_3525_1.25.0_DALI.cab
[295]: https://fwupd.org/downloads/64fb52303aa91dcc4716a64300823d184e5106de65bc5a4bcd5b24303a80fb69-35471_Inspiron_3525_1.26.0_DALI.cab
[296]: https://fwupd.org/downloads/e052abea1b06189abc6581938c9e3490b42ca6e76ca834a058e1c4d592957abb-38548_Inspiron_3525_1.30.0_D.cab
[297]: https://fwupd.org/downloads/3e293f31fb91222748c565c9fa61b51f2e4cc1e1f3209612dda776e603b9c108-45318_War_M_D_1340.cab
[298]: https://fwupd.org/downloads/f7f0f78300bd3815af4c26380776fe456decfdacbb081355f65555d8696dc8de-51838_Inspiron_3525_1.36.0_Dali.cab
[299]: https://fwupd.org/downloads/6186fd0c4e71fc532c839a2594fd0d56fca5fafdc6cdf3e3d85bbf5e4528f5c4-22453_Inspiron_3525_1.13.0_BAR.cab
[300]: https://fwupd.org/downloads/8c3e8761e5ed53c496e3bf469293f8cf77f8688320b2b482792926734ed8c140-23282_Inspiron_3525_1.14.0_BAR.cab
[301]: https://fwupd.org/downloads/614fa8bb6a34b2a288580496ef8dbe7a8df7714cf0773ff498205324846a6f76-23346_Inspiron_3525_1.15.1_BAR.cab
[302]: https://fwupd.org/downloads/4705f6f6d4d05b39e867b529730af5cd6918e6ba6df19440e8f0342323420a79-23763_Inspiron_3525_1.16.0_BAR.cab
[303]: https://fwupd.org/downloads/28b498708be0375c5b7a6454be4f460c53cdb9c75964dbdf374453b4f8b4f6ba-24175_Inspiron_3525_1.17.0_B.cab
[304]: https://fwupd.org/downloads/1fdfcceb80b7d7ed445443f699a5636029d44331f452a9cd20a15755719df459-Inspiron_3525_1.18.0_B.cab
[305]: https://fwupd.org/downloads/834723d23ed57ad02b2b35642ce4e355b98904ee9e3273af906d324f3054aa85-26494_Inspiron_3525_1.19.0_BAR.cab
[306]: https://fwupd.org/downloads/1b74bfd3e38a01647ebe56f6caf588ce19ac805530ce490ef7febbd5974e67af-Inspiron_3525_1.22.0_BAR.cab
[307]: https://fwupd.org/downloads/1ba0e8a974bbcf194973a7922ff32cfd53fb256b338c3e34aba728f869df32b5-34771_Inspiron_3525_1.25.0_BAR.cab
[308]: https://fwupd.org/downloads/8608fb36e440bd0e06889fb4c197932d78bfffd6b86ec6c633a751538baf37f8-35470_Inspiron_3525_1.26.0_BAR.cab
[309]: https://fwupd.org/downloads/9dcf6d980753124dcafc702613b687f944fdf832e36b2eb2c056e0cb9dc96c72-38547_Inspiron_3525_1.30.0_B.cab
[310]: https://fwupd.org/downloads/a44cd24732b8dd84f955df0aea4974c4519154824fba029f066652f5ada2a739-Inspiron_3525_1.36.0_Barcelo.cab
[311]: https://fwupd.org/downloads/dd199e3e15ea7070569b8d3902acc049f0b48eb7c888701789fae670e88ff6b7-23766_Inspiron_3535_1.9.0.cab
[312]: https://fwupd.org/downloads/ca1b4b64f679f94cfcae9c8a2de47d75a42b27bb98ef2c3fb8cbaf490daa642f-24815_Inspiron_3535_1.12.0.cab
[313]: https://fwupd.org/downloads/8839260f6df189c4c405ba41b5375ebf2b47e37098a04ef57c8cf6789a8dfa52-Inspiron_3535_1.13.1.cab
[314]: https://fwupd.org/downloads/deea102c176d87ec07cde4a8b4df678450e32a108cf33ad2da9da5af504d574a-28737_Inspiron_3535_1.15.0.cab
[315]: https://fwupd.org/downloads/479f409d5e1e75ff6c5c2c12ab457978ce1961bf81e028809a575a14087f213e-29953_Inspiron_3535_1.16.0.cab
[316]: https://fwupd.org/downloads/ed9af2421e231a49514e09aa3f542619447ca663da889e5835844ce8cdf74685-Inspiron_3535_1.17.0.cab
[317]: https://fwupd.org/downloads/f49686d0747744950256b1ce7c94035ec3c0dccc8647f6183bc101648e7103ed-34774_Inspiron_3535_1.20.0.cab
[318]: https://fwupd.org/downloads/67cc1a7bed69c6f9f712dd69dc79e9e28827578b5e519aa765ffd6eeba32f9d9-35473_Inspiron_3535_1.21.0.cab
[319]: https://fwupd.org/downloads/550cae698c2047246214851dd7d981451197dd2eac75584076b6f7c32ee236dd-38085_Inspiron_3535_1.23.0.cab
[320]: https://fwupd.org/downloads/746a28ab01701a00a3909f1161b9e11aa3291f5c72a7278746181043de8929be-45320_War_mlk2_1280.cab
[321]: https://fwupd.org/downloads/cdaa7127fa20f8637cdfac71bff7af0c92ee8d3bf0ec7086747e3ae942b14129-TramoreStp_firmware.cab
[322]: https://fwupd.org/downloads/5b384822fca5f4f06c90f54fbc36868689621eb74aa2312b54dc2c3c226332c9-TramoreStp_firmware.cab
[323]: https://fwupd.org/downloads/4a93e4c4220da548188b595e009c9ab8d3db205d721347bbac7e6d0abe3a0607-TramoreStp_firmware.cab
[324]: https://fwupd.org/downloads/3c0a3467a2cd7216f275a433976fdc16c186c6649a69ddb78413ca0d502b5558-TramoreStp_firmware.cab
[325]: https://fwupd.org/downloads/e982dd42eee4e3f6f4ef3f37ca11ff538492d00a2bc62126d0e429b18e8c18dc-TramoreStp_firmware.cab
[326]: https://fwupd.org/downloads/bbdf8c778a83285f1f5fb908eb8fa93701672d40f65d6642d6757c5925132c64-TramoreStp_firmware.cab
[327]: https://fwupd.org/downloads/10c9058b37a1e97f58da6adb14dda394bb192bcc624a9a48911dc820be85c4bf-TramoreStp_firmware.cab
[328]: https://fwupd.org/downloads/c329bd0f3c2451469f376f7933a4e27e49cf2d8a10c2ae0a3adf67dad143d039-TramoreStp_firmware.cab
[329]: https://fwupd.org/downloads/266f9c412b253e04391aa96a3cd7248b29071dc13f84c0179e87de5462e2dbf5-TramoreStp_firmware.cab
[330]: https://fwupd.org/downloads/a426cd6370ab1568fbe45bb51a96483ff9af93d33aa82db9936f42b446565e6a-TramoreStp_firmware.lvfs.cab
[331]: https://fwupd.org/downloads/76b8b11b04046f4daa8227e6482cc15e1759fc376e08a0a701e3d3b9c85951df-TarokoHptOne_firmware.cab
[332]: https://fwupd.org/downloads/20057f5af5b354dda8755643b4d6fce2c1ab26d878076302fbdf6d28f8443905-TarokoHptOne_firmware.cab
[333]: https://fwupd.org/downloads/ba4bb85f7d8bd64411025bea3df4f20873a7d9a58d69e0ac2b2238ea62bf5556-TarokoHptOne_firmware.cab
[334]: https://fwupd.org/downloads/0aafcfcfc0a8d68ad75cea35da05bb9d9a1a441bbe2e2d21c61fc5b5b19d62b7-TarokoHptOne_firmware.cab
[335]: https://fwupd.org/downloads/69fac99ba140e1225dd2cf50b8e28680d2be9a79af20ccc7f39124ce9046cfd1-TarokoHptOne_firmware.cab
[336]: https://downloads.dell.com/FOLDER14412490M/1/Dell_Pro_Plus_PB13255_PB14255_PB16255_1.14.0.exe
[337]: https://downloads.dell.com/FOLDER14485826M/1/Dell_Pro_Plus_PB13255_PB14255_PB16255_1.15.0.exe
[338]: https://fwupd.org/downloads/a24d305a140b235cfa8b2d9e837a5829f9eee4c07c99d4a7b2803d1553cfb51d-TarokoHptOne_firmware.cab
[339]: https://downloads.dell.com/FOLDER14421236M/1/Dell_Pro_14_Essential_PV14255_1.12.0.exe
[340]: https://downloads.dell.com/FOLDER14545628M/1/Dell_Pro_14_Essential_PV14255_1.13.1.exe
[341]: https://fwupd.org/downloads/4fd0e2eba74b87d547c530c2a878128d6f61c1b05d87be9b48c0c972d815d999-firmware_hwk_ess1.14.0.cab
[342]: https://fwupd.org/downloads/3e39d694d225f940d716335f782ca5f431ac3fb5e411fac43e4bac4aee33f5df-firmware_krk_1.15.0.cab
[343]: https://fwupd.org/downloads/6b96e128fa851783f92de3515667f6495563706528d88cb5345083938f48affd-JadeHpt_firmware.cab
[344]: https://fwupd.org/downloads/8567a6d383c45e294a9cd26582faafa77894769938bde48c04c405ea956263b8-JadeStxKrk_firmware.cab
[345]: https://fwupd.org/downloads/56ce0b82d8d9d61ca488effe00d650d22a6ad093f30d42b12944598bb4f2aa7f-JadeHpt_firmware.cab
[346]: https://fwupd.org/downloads/8a9a68c64aae34774b9ee072d1bf8bce0e772fcbcaf14b29cd39f372be937ee9-JadeHpt_firmware.cab
[347]: https://fwupd.org/downloads/70b64237c29750e2a090c00b330d2d89c3aa041f4a22152066ad6da4f8118910-JadeStxKrk_firmware.cab
[348]: https://fwupd.org/downloads/8c4ea3f5bcfe7ebb3fb929398e4da14b4201674e4ab746e0350b9fdf9768931e-JadeHpt_firmware.cab
[349]: https://fwupd.org/downloads/0f61fb9147e38b6f26806c28db5ba681c14a517bf02ababe213351bf3b0d7c12-JadeHpt_firmware.cab
[350]: https://downloads.dell.com/FOLDER14382720M/1/Dell_Pro_PC14255_PC16255_1.13.0.exe
[351]: https://downloads.dell.com/FOLDER14545840M/1/Dell_Pro_PC14255_PC16255_1.14.1.exe
[352]: https://fwupd.org/downloads/9f8154fec6cd733274ade851a3928a0290b24793ccc71107b8bbc46aa0212e0a-JadeHpt_firmware.cab
[353]: https://fwupd.org/downloads/495cc6d3a9db4aa7c7cb5014b2c5af5586340fc5b351306a4f675d9f3ded8ec0-JadeStxKrk_firmware.cab
[354]: https://fwupd.org/downloads/19eb6f04dae2b4de029496ef344f88c5382bb039b071210ae3fcc48e9ccea2bf-GhostriderStxOne_firmware.cab
[355]: https://fwupd.org/downloads/5315b1df1657e9affd1e9d2184eb42c553a61d2fd04a6cc7f537c9ca7ced4bea-GhostriderStxOne_firmware.cab
[356]: https://fwupd.org/downloads/89b109195f9933a58614ef28bfd32199e78746720533ec8a29b08931642605f4-GhostriderStxOne_firmware.cab
[357]: https://fwupd.org/downloads/32f56043202a0e6baf0581b05b0f4e33ac7565cb47dc2551a53dfc3e62359121-GhostriderStxOne_firmware.cab
[358]: https://fwupd.org/downloads/4ea2a58abae24ccceab43cb69fb5d3ca6b943e561fe003e6a33a14d326b2231e-GhostriderStxOne_firmware.cab
[359]: https://fwupd.org/downloads/424af13097d5506f69e13719024283664c2352df9d3a7a66726e9124d2360415-GhostriderStxOne_firmware.cab
[360]: https://fwupd.org/downloads/662926e67994e00a2249ce77e84120416a2204f6aba8a2d3172cb5f8ab2212a4-GhostriderStxOne_firmware.cab
[361]: https://downloads.dell.com/FOLDER14378628M/1/Dell_Pro_Max_MC14255_MC16255_2.2.2.exe
[362]: https://downloads.dell.com/FOLDER14546803M/1/Dell_Pro_Max_MC14255_MC16255_2.4.0.exe
[363]: https://fwupd.org/downloads/7735b49d6ecca46326fce32cc1247857e1576588c613a9e1abfd2cd0f55789af-GhostriderStxOne_firmware.cab
[364]: https://fwupd.org/downloads/c21048d3781c2742b56b0f022fa2ee1aff4b5f25dfb6a7dc25f4c5526ec62482-GhostriderGptOne_firmware.cab
[365]: https://fwupd.org/downloads/4eb8d6c4f0024defa960756f4af56ee0be6581f3e764257ffc73a26e8821eb74-GhostriderGptOne_firmware.cab
[366]: https://fwupd.org/downloads/4e3692ce771aa4429e4dc5b9a1fcfb36e1cef2b1dd8c3972df2eb079179bb526-GhostriderGptOne_firmware.cab
[367]: https://fwupd.org/downloads/7d8d77a4085ac02019ddb82bfdf5868403b5d0cad2bb379e90ac6ea8de157ee1-GhostriderGptOne_firmware.cab
[368]: https://downloads.dell.com/FOLDER13421801M/1/Dell_Pro_QCT1255_QCS1255_QCM1255_1.5.0.exe
[369]: https://downloads.dell.com/FOLDER14508768M/1/Dell_Pro_QCT1255_QCS1255_QCM1255_1.13.0.exe
[370]: https://fwupd.org/downloads/b848493f99d2d56bdb4718580e232f2a009fab207c3ff9a6818e2123b47e3140-Framework_Desktop_Ryzen_AI_MAX_300_fwupd_3.02.cab
[371]: https://fwupd.org/downloads/ecd20e5f294bbd2f5742cc3afcf393d2bc88dcad575ac61cd2a56d13e6c50c2c-Framework_Desktop_Ryzen_AI_MAX_300_fwupd_3.03.cab
[372]: https://fwupd.org/downloads/a20024b975a82a6d8166c161d6fdfa9859fe6d47542932816c1bb6e3f4ccf3be-Framework_Desktop_Ryzen_AI_MAX_300_fwupd_3.04.cab
[373]: https://fwupd.org/downloads/7d4ffbcb2b386aaa37734d9767cf6f5ecadb20778f849c25640a8df015439f8c-Framework_Desktop_Ryzen_AI_MAX_300_fwupd_3.05.cab
[374]: https://fwupd.org/downloads/2742d7595114df4276bb11e1ae042ee414a24ac5628a75c6333e75d3b01b1dee-Framework_Desktop_Ryzen_AI_MAX_300_fwupd_3.06.cab
[375]: https://fwupd.org/downloads/b4fd942e87802acf33fdd7dd91d40fc6ded63ab34c8391c3fe1d1276ddc5b28c-Framework_Laptop_13_Amd_Ryzen7040_fwupd_3.03.cab
[376]: https://fwupd.org/downloads/250f80d66b0f13574f803c98d4423accee3857c38389deeb184ac942560abc67-Framework_Laptop_13_Amd_Ryzen7040_fwupd_3.05.cab
[377]: https://fwupd.org/downloads/9f4ae4d3c7094d34c87170568311d5abdc070ee0ab8759cd92d955ff610ef67e-Framework_Laptop_13_Amd_Ryzen7040_fwupd_3.07.cab
[378]: https://fwupd.org/downloads/9a9b6b23623227b1a2b5906030de4a1c08990ef3b57a65df101dab6ea4169b47-Framework_Laptop_13_Amd_Ryzen7040_fwupd_3.09.cab
[379]: https://fwupd.org/downloads/b5d46a940caceb9e74387048ef8964a84c48ebefd4537fb08711b686bd8fbbc9-Framework_Laptop_13_Amd_Ryzen7040_fwupd_3.16.cab
[380]: https://fwupd.org/downloads/7bc494f4062a69f1942f88f64fbfb8cebb53d5eb16117df909cfe9f5cf914272-Framework_Laptop_13_Amd_Ryzen7040_fwupd_3.17.cab
[381]: https://fwupd.org/downloads/3367d678c1d2a099fe1a520ea7ba2780884d9a93dc7d314d4926f73f8bb048fd-Framework_Laptop_13_Amd_Ryzen7040_fwupd_3.18.cab
[382]: https://fwupd.org/downloads/97a993f271b4ace12000ae044162bbb448b102bae16382fdfdea575f5511a08c-Framework_Laptop_13_Amd_Ryzen7040_fwupd_3.20.cab
[383]: https://fwupd.org/downloads/a561496d903823345906a6d224e7139eaad148f00dda8b67dd54e1ae71895cb0-Framework_Laptop_13_RyzenAI300_fwupd_3.03.cab
[384]: https://fwupd.org/downloads/8230ba6e582721681d23ad16e3b5ca41ce78189d7ed0fcc73d7d8ac534310a15-Framework_Laptop_13_RyzenAI300_fwupd_3.04.cab
[385]: https://fwupd.org/downloads/63be267761be4b1af6e2baf5e2c0fd60ed3d4f2a4404c9a99c08dbf6f78c88e2-Framework_Laptop_13_RyzenAI300_fwupd_3.05.cab
[386]: https://fwupd.org/downloads/9c085d03d6ea515101a8519f7855bf951e7693ff4f392585bbb692275a8b27f5-Framework_Laptop_13_RyzenAI300_fwupd_4.02.cab
[387]: https://fwupd.org/downloads/a2fbe97a46e064b9d872ba381dd19af8e268acfa9d17d4eec91fa4d55c475cc0-Framework_Laptop_16_Amd_Ryzen7040_fwupd_3.03.cab
[388]: https://fwupd.org/downloads/f3e30b2d8cb46b5a19bdc1552c6d9291f0751d0c8d74db938d1272fb40bc4db4-Framework_Laptop_16_Amd_Ryzen7040_fwupd_3.05.cab
[389]: https://fwupd.org/downloads/2e983ce287988168f816536d641a4f33d0733892f02964e819cccebdbf30e8a4-Framework_Laptop_16_Amd_Ryzen7040_fwupd_3.06.cab
[390]: https://fwupd.org/downloads/2afca825eb878496139e28185810b7bd7282efc1b6a8514ee8e182a57e53f5dd-Framework_Laptop_16_Amd_Ryzen7040_fwupd_3.07.cab
[391]: https://fwupd.org/downloads/62fe165328af6bdbfbec5ee637c5b7d1adeb6b860a23f0d8d84b3bd5b1b8cdf6-Framework_Laptop_16_Amd_Ryzen7040_fwupd_4.02.cab
[392]: https://fwupd.org/downloads/5100b9cabdc0f16fc7b44b3bc043a738af33195912e6a6766fc30dad126fbc19-Framework_Laptop_16_Amd_Ryzen7040_fwupd_4.03.cab
[393]: https://fwupd.org/downloads/7589298debed352d372cb01ed1ea9930bad3064a1665f2995ed861cf820792d0-Framework_Laptop_16_Amd_Ryzen7040_fwupd_4.04.cab
[394]: https://fwupd.org/downloads/2dcb3d734e21c939c496d45be45ec4e1cbe3332ef112f98ddde98ff53fae63d1-Framework_Laptop_16_Amd_Ryzen7040_fwupd_4.05.cab
[395]: https://fwupd.org/downloads/3f3c0749579aff88fdd2745702b627e6243f689955fa46f69d534d95466ef2ff-Framework_Laptop_16_Amd_RyzenAI300_3.03.cab
[396]: https://fwupd.org/downloads/8d74a01b65d874bfd12d75fed3c2cf64e8870a694e236547bed408fe8a92201f-Framework_Laptop_16_Amd_RyzenAI300_3.04.cab
[397]: https://fwupd.org/downloads/1eb1f46aa9412bd1e98959b9a9fa4a1a0b5f9db3ae978a63cd8221f8d80c6084-Framework_Laptop_16_Amd_RyzenAI300_3.05.cab
[398]: https://fwupd.org/downloads/fc24fe88fb96ec56ab995a174a1f86829fbc4505eee90c509b653d54b717d894-Framework_Laptop_16_Amd_RyzenAI300_3.06.cab
[399]: https://fwupd.org/downloads/698d25c19fef326fd3742059761571e8a46d7bd5e493feb845a3f1851cab600e-Framework_Laptop_16_Amd_RyzenAI300_4.01.cab
[400]: https://fwupd.org/downloads/a453b7cf9aca966b2ce455ddbee98be9337f8a4624f0e987bc41132357cf98c0-W06_02080000.cab
[401]: https://fwupd.org/downloads/c1398e0c68a4203773269e535f9b965d75525c175dceb890e3d115b0bdd0cb7a-W06_02100000.cab
[402]: https://fwupd.org/downloads/d555712a96a95118b0edb70358d27bc18d95e98ee38d1bcc1a4a9e705e2247b6-W06_02110000.cab
[403]: https://fwupd.org/downloads/699d889d144e36e6e1ec4c90a74853b4c068da72fef601f1cda9ebfb560001ed-W06_02130000.cab
[404]: https://fwupd.org/downloads/eedea3ad22f0e39f19ce5fc154c43680f67cbac7aa52b4724f4b751220cca874-W06_02140000.cab
[405]: https://fwupd.org/downloads/b2e78a9904c3fcb55a4e072484dea947d8bc8cf583b0d9e4f5881d4a4cef4a1c-W71_01020300.cab
[406]: https://fwupd.org/downloads/34323026068d6b204970dfbde81691a56f359609451894b80912331d31a1d997-W71_01050000.cab
[407]: https://fwupd.org/downloads/aff3ccc9b64850fb7292bf286c9d6f7bc318502d2c555417868598ee90507d2b-W71_01060000.cab
[408]: https://fwupd.org/downloads/097d4ed5dc98cdb479c83786a513ae3e3bdb093cf0a77821be7c576c28ba9e7f-W71_01080200.cab
[409]: https://ftp.hp.com/pub/softpaq/sp166001-166500/sp166153.exe
[410]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174302.exe
[411]: https://ftp.hp.com/pub/softpaq/sp158001-158500/sp158198.exe
[412]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174390.exe
[413]: https://ftp.hp.com/pub/softpaq/sp172001-172500/sp172127.exe
[414]: https://ftp.hp.com/pub/softpaq/sp173001-173500/sp173260.exe
[415]: https://ftp.hp.com/pub/softpaq/sp174501-175000/sp174614.exe
[416]: https://ftp.hp.com/pub/softpaq/sp172001-172500/sp172124.exe
[417]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174376.exe
[418]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174065.exe
[419]: https://ftp.hp.com/pub/softpaq/sp151501-152000/sp151996.exe
[420]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174070.exe
[421]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174053.exe
[422]: https://fwupd.org/downloads/8ae47a7660cfcd3085d28a5c080d0d425358ec148b3ff79d9ab6f445582716d6-X76_01010200.cab
[423]: https://fwupd.org/downloads/8adb3dc48127cd66e2eb8ce2699d55c51ebbc1343ac97d21ddaa816d1924f33c-W77_01080100.cab
[424]: https://fwupd.org/downloads/7241c7bd11b1a676a4bee01d867bf3c1e233f3520c0504a0306ad2310d067903-W77_01080300.cab
[425]: https://fwupd.org/downloads/030800a8bfb7c861ed13504f41682b8f5bb4200be187633bc5d491eb58b7679b-W77_01090100.cab
[426]: https://fwupd.org/downloads/b39fbeeb61858e1e8203db6819e1db69699d97ad0d2aa539b2d5c8ff0bab7321-X77_01010200.cab
[427]: https://fwupd.org/downloads/1cac4d17fbf721130abc4946a5391f1df7afacb593d19f38ba985cc8046e7369-X77_01030100.cab
[428]: https://fwupd.org/downloads/6f907b381f4089d921f50faeec0b145d3a34dbe1bf054e4b76e75f9caa715c96-X77_01040400.cab
[429]: https://fwupd.org/downloads/654caee9c7ecc2e4d5d8a5e66cc0ee30e643a1e184dab245dee9a1577280e797-X77_01040500.cab
[430]: https://fwupd.org/downloads/61cffabb29cde7d66317567ff30ce6f8b51dcad5aa52624a2ee4c71597954d53-X77_01050300.cab
[431]: https://ftp.hp.com/pub/softpaq/sp160001-160500/sp160173.exe
[432]: https://ftp.hp.com/pub/softpaq/sp173001-173500/sp173138.exe
[433]: https://fwupd.org/downloads/88c9779fd55752a53cb1098cc1ed608540d04ed15e7973077fe5a6a470ae2a07-X81_01020200.cab
[434]: https://fwupd.org/downloads/773f72d320a93c1c8991b90e7f5a8b58b2ee168980514ff9e2ff0adb8562e9c9-X81_01040100.cab
[435]: https://fwupd.org/downloads/e0ffa4f11995f9d1afa71cb373b60f2f1b04e0f76b0e01e1d9744417eb47d9c5-X81_01050400.cab
[436]: https://fwupd.org/downloads/c034dcec56393de35364ffab3833a85c41fe04a23ac7611433336f338287df1f-X83_01020100.cab
[437]: https://fwupd.org/downloads/178a12b78bd5a3bbd05842833e8f0169db34a6c47256c0a32a762f8b9c835e61-X83_01040100.cab
[438]: https://fwupd.org/downloads/06fbe87728bffbf311239c3d7e506b4e2a66ae8ad17170800cde2e04eb11d8d0-X83_01050400.cab
[439]: https://fwupd.org/downloads/187f74ffc32608238ceb27b77e0048b933882e6840efdd15d266de0d118456a7-X83_01060100.cab
[440]: https://ftp.hp.com/pub/softpaq/sp160001-160500/sp160119.exe
[441]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174335.exe
[442]: https://ftp.hp.com/pub/softpaq/sp160001-160500/sp160174.exe
[443]: https://ftp.hp.com/pub/softpaq/sp173001-173500/sp173139.exe
[444]: https://ftp.hp.com/pub/softpaq/sp160001-160500/sp160120.exe
[445]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174336.exe
[446]: https://ftp.hp.com/pub/softpaq/sp171501-172000/sp171730.exe
[447]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174377.exe
[448]: https://ftp.hp.com/pub/softpaq/sp149001-149500/sp149490.exe
[449]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174003.exe
[450]: https://ftp.hp.com/pub/softpaq/sp157501-158000/sp157824.exe
[451]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174123.exe
[452]: https://fwupd.org/downloads/7546585446ade574d03d02d12487a197583798e0dc84240c947b5fe5cfe8c2d8-W83_01080200.cab
[453]: https://fwupd.org/downloads/6ba1601f3f986ed94bbaad274c4de14bc9f976820f09d763fc71b7df767cfdee-W83_01090200.cab
[454]: https://ftp.hp.com/pub/softpaq/sp149501-150000/sp149780.exe
[455]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173915.exe
[456]: https://fwupd.org/downloads/1a52e28df5271c16489fef578020761d1bc6ec78c77439cda919fa59c160a0d1-V83_01041000.cab
[457]: https://fwupd.org/downloads/e36490b9f4418dea6a22be7b3e99401100a8c8b00ff00c60470c7d98b71c70ba-V83_01051100.cab
[458]: https://fwupd.org/downloads/63f662dcf34a286727f66895b41a807d7ff09a3f28c6eaef178b043e3ccf61e9-V83_01060200.cab
[459]: https://fwupd.org/downloads/7faeec8584889279438f6a980a2a30dc7479bcac5e345d9d587ddd6cc4227140-V83_01080000.cab
[460]: https://fwupd.org/downloads/2410be26845f699ec4ba4b606de84cf16a0212d5c0bf859ed3a89283afdc2bf1-V83_01090000.cab
[461]: https://fwupd.org/downloads/8c4417eef4ec11b7c879104a80e1b78b74d276929390939afb4c5d489d4b4e07-V83_01100300.cab
[462]: https://fwupd.org/downloads/f038809a20a7b001efe6a776bfc5dc2c23e3e94fff4875298436f7c1524cf016-V83_01110000.cab
[463]: https://fwupd.org/downloads/e7aeb3ab6a3dcaa7dbe2bd1dd331fcf1bce57121149ef1e0d02ac1a375be0a32-V83_01120000.cab
[464]: https://fwupd.org/downloads/049f6fe1460213dd055128906fb6e8576be8ef51925ac030542a0f33b0d25c3f-V83_01120200.cab
[465]: https://fwupd.org/downloads/f2c86c616d01bd20c30139473fead3ae58cd26b6bd42e9837a0f4c2abf87b6de-V83_01130000.cab
[466]: https://ftp.hp.com/pub/softpaq/sp149501-150000/sp149827.exe
[467]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173996.exe
[468]: https://ftp.hp.com/pub/softpaq/sp149001-149500/sp149489.exe
[469]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174001.exe
[470]: https://ftp.hp.com/pub/softpaq/sp157501-158000/sp157797.exe
[471]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174122.exe
[472]: https://fwupd.org/downloads/f3fb5a7377bf553436856190b9238e608209f5c549cc82ad416039abd541dadf-V82_01030900.cab
[473]: https://fwupd.org/downloads/51fd079072369738a0a1d6215ac5bb96352fb846d54e7bf8e3fae7fe52d5ff3c-V82_01041000.cab
[474]: https://fwupd.org/downloads/6cd0c075cbe05eec98991dffae71dd96c881094dd3b16c2902fa5ce63947942c-V82_01051100.cab
[475]: https://fwupd.org/downloads/828ca6ae73e2df7a793c1ae02c47f4b43465173ffea017e5cfa41803be6a30fa-V82_01060200.cab
[476]: https://fwupd.org/downloads/578f6b870cf3a816fbf37f1e039301e37312da28c098ea39c38f257846246a79-V82_01080000.cab
[477]: https://fwupd.org/downloads/2aa62dff7296ec419ac538ed5f392f62ba5451e3e5ac55792bf76c5d4712c5d4-V82_01100300.cab
[478]: https://fwupd.org/downloads/8de576b14ea83284eef792dfdfaec994d257e8eb64d96413fa9198eeaed18626-V82_01110000.cab
[479]: https://fwupd.org/downloads/d6c3464f0ed47686252d0d7d79ef3f2041cfeabdcee49980673e39dfb8dcd317-V82_01120000.cab
[480]: https://fwupd.org/downloads/3af43b36052f3c38e6b9f49ca3a971ad183299136e4a91ff95770f1ea68df436-V82_01120300.cab
[481]: https://fwupd.org/downloads/ba05b2d5b9b6de5f5213707451608f9586c739e55301aa370d635ecf2fab49a5-V82_01130000.cab
[482]: https://fwupd.org/downloads/4c44ac807844db68c046dc6a3ff09e817da05245ba3d82b24ce9ec0f1b6a25ae-W82_01090200.cab
[483]: https://fwupd.org/downloads/d60136c6c528875ecbe20c45abafb56046f66d24f462cfea42a7cb984c3298ae-U82_01060000.cab
[484]: https://fwupd.org/downloads/f544ff66cbc3351a20ca9c454a4abc42439d908b6f05dd44fe92c7ea0577e7c0-U82_01070000.cab
[485]: https://fwupd.org/downloads/dddf495dfacb2a1221d4c9b292af6ea5c8fe70cdfe37d37c065ed9a001408700-U82_01080000.cab
[486]: https://fwupd.org/downloads/ea226ed0db8af4872205b39d0a650b1a9e1b40ab1a9153fa2ff2033fb7ba0c26-U82_01150000.cab
[487]: https://fwupd.org/downloads/ae27142168a1880afbdcb7f8ea0ad9326498cadc141c8292fd2247f376aae2af-U82_01160000.cab
[488]: https://fwupd.org/downloads/8143ddbe2fa69614f94f054ad0a2b80e358683e8709999baca8b6d87a00d5b81-U82_01160100.cab
[489]: https://fwupd.org/downloads/8f89d8390d4b550464cf5006a8dd93fb97144730ae076d5e12b2a9af573b7765-U82_01170000.cab
[490]: https://ftp.hp.com/pub/softpaq/sp157001-157500/sp157272.exe
[491]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174124.exe
[492]: https://fwupd.org/downloads/2ac1d4a6c3d43a1aeed0d76cd5b9e761a3ed6bb7c555f7b8b41415a202381012-X88_01060100.cab
[493]: https://ftp.hp.com/pub/softpaq/sp171501-172000/sp171778.exe
[494]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174080.exe
[495]: https://fwupd.org/downloads/338176f1ed3312c40465fec8b063caec0df6aa95d250578a2e77f1ae454e92f8-X82_01020200.cab
[496]: https://fwupd.org/downloads/1efd1b62b0675437ca0e687405914cd7e925e9c0b0b593c6091f31eab585a9fe-X82_01040100.cab
[497]: https://fwupd.org/downloads/8f0e1341a82b813c29ec8cca012424ff77545c5261e116732b069af1919d529d-X82_01050400.cab
[498]: https://fwupd.org/downloads/22cb4456c92043bc18af28d6e049fde90d6b1285056a22e1c372aaaa4d499923-X27_02010200.cab
[499]: https://fwupd.org/downloads/16522c6bb8b0b0677c5040f9519b0d1a569d85bc7e868aabc8b1f23ab5f76284-X27_02030000.cab
[500]: https://fwupd.org/downloads/99686aed089a146fad081b8167445821538ae8e83683b0fb911793942413371f-X27_02040100.cab
[501]: https://fwupd.org/downloads/b248537a786c7c5157f892d5ad4babbe8a4575303694110bbef917230d303a40-X27_02050000.cab
[502]: https://fwupd.org/downloads/c39428c07af39fb909271fd09ca7d63929d5354f23ca2bafabda279c31d5a621-X27_02060100.cab
[503]: https://fwupd.org/downloads/c17b61d9d91be0100c65dc82a8645024104fabc71786a3c44aebb017ab367672-X26_02010200.cab
[504]: https://fwupd.org/downloads/6930b5c5741cd5c3efc377e323c8c720167c6670061cdf86a31b7890133d0b90-X26_02030100.cab
[505]: https://fwupd.org/downloads/c90a8abd7bb653b88a5a37dd66b785bbcb7b6cd778c08919966d7e084dbb04f9-X26_02040100.cab
[506]: https://fwupd.org/downloads/66161ea3f20b6c0fc21925e2ccfb812d529618ca01b687e44148ac5a38f3a055-X26_02050000.cab
[507]: https://fwupd.org/downloads/2a0b1f70aa71deeb48d8a9483afab346a0ed17f243f2edaba8ae4f8dcef5fc73-X26_02060100.cab
[508]: https://ftp.hp.com/pub/softpaq/sp165501-166000/sp165897.exe
[509]: https://ftp.hp.com/pub/softpaq/sp165501-166000/sp165898.exe
[510]: https://ftp.hp.com/pub/softpaq/sp173001-173500/sp173408.exe
[511]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174277.exe
[512]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173607.exe
[513]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174438.exe
[514]: https://fwupd.org/downloads/5bd3c0c9c2fe038d8c77ad6dbeaca9d7272151af8d3f1e9490f075f01c13bd56-Y26_02010100.cab
[515]: https://ftp.hp.com/pub/softpaq/sp151001-151500/sp151379.exe
[516]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174121.exe
[517]: https://ftp.hp.com/pub/softpaq/sp149501-150000/sp149784.exe
[518]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173917.exe
[519]: https://ftp.hp.com/pub/softpaq/sp158001-158500/sp158201.exe
[520]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174393.exe
[521]: https://ftp.hp.com/pub/softpaq/sp172001-172500/sp172126.exe
[522]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173541.exe
[523]: https://ftp.hp.com/pub/softpaq/sp174501-175000/sp174616.exe
[524]: https://ftp.hp.com/pub/softpaq/sp172001-172500/sp172122.exe
[525]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174375.exe
[526]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173972.exe
[527]: https://ftp.hp.com/pub/softpaq/sp151501-152000/sp151973.exe
[528]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174281.exe
[529]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173969.exe
[530]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173867.exe
[531]: https://fwupd.org/downloads/7d87d71593053ae686e2137e38668c48476d8fa0a16c42232da57fd9b8c4e2c9-W78_01080100.cab
[532]: https://fwupd.org/downloads/b6525bcb321f9f99ca86a33460a576f177c39b787550cc608abc2b65156abcfb-W78_01080300.cab
[533]: https://fwupd.org/downloads/00697c3e78e0f8e377a5cc49e1a8c304af65ab21b61e051917b080640ef182f2-W78_01090000.cab
[534]: https://fwupd.org/downloads/442b88d7f6ef982ef7129f6065ac81f4ba5d1e116044492d4666cfdd91d1844c-X78_01010200.cab
[535]: https://fwupd.org/downloads/89752dad3e124245fec5a2138b42307514c9a7fe158cbbb285d14f5461c26137-X78_01030100.cab
[536]: https://fwupd.org/downloads/59b1c6f988fc5ea539a42ede6b0cd98cafe8c311087e9c695fe757d3f1219f8d-X78_01040400.cab
[537]: https://fwupd.org/downloads/84a49ca75869d8050ee2d088a7dc5c587d37096798cec1640276baca4418b4f2-X78_01040500.cab
[538]: https://fwupd.org/downloads/74e9cff587b0c8a204a181e037240beabcfe7116bdfbdec961a041d5cc979d05-X78_01040800.cab
[539]: https://fwupd.org/downloads/faae684b0dd5e9dfac7e7c1790f07a0e4f1e70d674fff1c3b41b6c73e476ee7c-X78_01050400.cab
[540]: https://ftp.hp.com/pub/softpaq/sp149501-150000/sp149785.exe
[541]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173998.exe
[542]: https://ftp.hp.com/pub/softpaq/sp149501-150000/sp149800.exe
[543]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173995.exe
[544]: https://fwupd.org/downloads/d45928b051dafef0cd678482c188327b3a80e2cf5740cb80b39dfa9a638a1d98-X84_01040100.cab
[545]: https://fwupd.org/downloads/0110a7a9b33a8680de2c1eda8c10748d31929b42656c5ad3934a55109ffec458-X84_01050400.cab
[546]: https://fwupd.org/downloads/c6b6c987fb3014ce8d619926d1612216517496d72039699f99849c2d07f8d902-X84_01060100.cab
[547]: https://ftp.hp.com/pub/softpaq/sp151001-151500/sp151047.exe
[548]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174064.exe
[549]: https://fwupd.org/downloads/46a422405af7ba525dcd9ea5ce75a5144af1c35d3b0ad28819d759e30de5d9c9-V85_01030000.cab
[550]: https://fwupd.org/downloads/0108dbc7d94d8d9744bde38080d5f5458a5ed6e0e1f7700d8f618931c66d73dc-V85_01030900.cab
[551]: https://fwupd.org/downloads/4cc2d94e68f5e1766db18c484950ee5a08bc8bc1be1e5fdd0bca145ab58060c5-V85_01040000.cab
[552]: https://fwupd.org/downloads/35887c5cd838077bdb5d91ad34525626e8ae8a40a1de7f8568623ab21dc98064-V85_01080000.cab
[553]: https://fwupd.org/downloads/631695864d9c1ddb2b53058b4d1701ffa8ba0bdff20d9093107a4cb35a79e56f-V85_01090000.cab
[554]: https://fwupd.org/downloads/e735d3544ff4a769d2b45a864da07fdde824821de43cdb9f832c0ca94e166a46-V85_01100000.cab
[555]: https://fwupd.org/downloads/a6052a2ab1ed84710627c4639a22bbc5264f1d19372757fc788ba8a12c4bf280-V85_01110000.cab
[556]: https://fwupd.org/downloads/122314bfbc1d3e5b4c7a94d4d7f21067d04de8ff2a80ba18aac4b0a7a3ced566-V85_01120000.cab
[557]: https://fwupd.org/downloads/2977588c473ca6fc5bfe886da08ec65b1d0f5d7d780d4a8b917032488590b934-V85_01120100.cab
[558]: https://fwupd.org/downloads/4cde7241bf2366649760088b435214abb44638eb9ff27cfe60b008f2499b47e9-V85_01130000.cab
[559]: https://ftp.hp.com/pub/softpaq/sp157501-158000/sp157905.exe
[560]: https://ftp.hp.com/pub/softpaq/sp173001-173500/sp173070.exe
[561]: https://fwupd.org/downloads/ea1c86420dd7dcc90198d018610f9d8742f9e7090eedcb789d441183163a427e-W85_01100400.cab
[562]: https://fwupd.org/downloads/5d139235b75650943574e53bd51822845e55fd2ec1e8520f1bd8da8ea4956986-W85_01100500.cab
[563]: https://fwupd.org/downloads/d7b63030a40f6c8f6309b03727068e2f992fa73da497fc18e39c2cf55f34c047-W85_01110100.cab
[564]: https://ftp.hp.com/pub/softpaq/sp157001-157500/sp157022.exe
[565]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174483.exe
[566]: https://fwupd.org/downloads/2e82aa748dc88aa1cb631acd27929ebdaabcac60c4a3040ed1175971a0daf8e3-X89_01020100.cab
[567]: https://fwupd.org/downloads/42b8e84d00c2213f6c864f605814406b2689ce83bab908bfb49b5263454ed19d-X89_01030000.cab
[568]: https://fwupd.org/downloads/8797a5311524a3396cf773dfdeed0de604f0e3225043daf13d7d843dd18fef8b-X89_01050700.cab
[569]: https://ftp.hp.com/pub/softpaq/sp173501-174000/sp173868.exe
[570]: https://ftp.hp.com/pub/softpaq/sp174001-174500/sp174057.exe
[571]: https://download.lenovo.com/pccbbs/thinkcentre_bios/m55jy2ausa.exe
[572]: https://download.lenovo.com/pccbbs/thinkcentre_bios/m6cjy1busa.exe
[573]: https://fwupd.org/downloads/f918b2496ecee09ad915bed61fcbbac427b47a5a724294e2ac6585f38856c92d-ThinkCentre-TinyAMD-SystemFirmware-M63KT07A.cab
[574]: https://fwupd.org/downloads/5cc74ce35320a78e60e217b0cdca73f641472c4d61d171cd3913364d261d8d76-ThinkCentre-TinyAMD-SystemFirmware-M63KT09A.cab
[575]: https://fwupd.org/downloads/c4b0f25942487ce410fdc58aeaaad85b7378412edaff861efee1c012174c4886-ThinkCentre-TinyAMD-SystemFirmware-M63KT0DA.cab
[576]: https://fwupd.org/downloads/e19b0b02d4110ed5b6e7d4f823b8f9a093259c5a72ea02b67c5f7d1f9cd78042-Lenovo-ThinkCentre-TinyAMD-SystemFirmware-M63KT0FA.cab
[577]: https://fwupd.org/downloads/d117c9f12e1696bd9dcf1799ef1609493f7b9a04e767f3e376314fbca4c71177-Lenovo-ThinkCentre-TinyAMD-SystemFirmware-M63KT10A.cab
[578]: https://fwupd.org/downloads/00e3592116f91d06bc212638869151237ef3d31c1f06e863e32388fcc4cae10c-ThinkCentre-TinyAMD-SystemFirmware-M55KT17A.cab
[579]: https://fwupd.org/downloads/079071cd5ca6c17cea6845e5ebb247c6b66eb5db08e268e050597ec10e639166-ThinkCentre-TinyAMD-SystemFirmware-M55KT19A.cab
[580]: https://fwupd.org/downloads/5362b87b6ad2420c360b1fa986413c32dc411dd080b2e146edcb2861835271f0-ThinkCentre-TinyAMD-SystemFirmware-M55KT1AA.cab
[581]: https://fwupd.org/downloads/25e2086f9e092aafa2c1bb9fb95a96029b17f372c2b526c128fa112aa1164749-Lenovo-ThinkCentre-TinyAMD-SystemFirmware-M55KT26A.cab
[582]: https://fwupd.org/downloads/d4f8174f5c73664414f2de155b8a5952d3324bb593b91d901aa10c41b3a815ee-Lenovo-ThinkCentre-TinyAMD-SystemFirmware-M55KT27A.cab
[583]: https://fwupd.org/downloads/6a9a3494308e6de18f379f0b5b8f1abf9fb8f22e64dd62ee73510df954f905dc-Lenovo-ThinkCentre-TinyAMD-SystemFirmware-M55KT2AA.cab
[584]: https://fwupd.org/downloads/bd42c993b0378a2e3a4150a1a49a93516f456667bb46ab9a9782c51cdaaaaf27-Lenovo-ThinkCentre-M6C-SystemFirmware-M6CKT1BA.cab
[585]: https://download.lenovo.com/pccbbs/thinkcentre_bios/m4rjy3busa.exe
[586]: https://download.lenovo.com/pccbbs/thinkcentre_bios/m6jjy1fusa.exe
[587]: https://fwupd.org/downloads/189076fe902c0b52685f8420e1a8891343010774dabb4fc33884c4e6274792f7-ThinkCentre-M75s-5-SystemFirmware-M62KT10A.cab
[588]: https://fwupd.org/downloads/8e9a9f6b8a1ee193f2bb22b6832dbd3a44dfd0146d895a4e67393cd6874b772d-ThinkCentre-M75ts-5-SystemFirmware-M4RKT2AA.cab
[589]: https://fwupd.org/downloads/8c00b47ca04ab656cedcb81a16a4be0f1f3dee329235cdf93bf0b7a9f74d8430-ThinkCentre-M75ts-5-SystemFirmware-M4RKT30A.cab
[590]: https://fwupd.org/downloads/12042baf88096cc5ed90adeeb2243a6a698f08670970e09cacaaa7ae2f6eb142-ThinkCentre-M75ts-5-SystemFirmware-M4RKT32A.cab
[591]: https://fwupd.org/downloads/bfc5e519e98e3d4bcf0e834e83ebc2a83196a296a82f57123f3a4ce5d89198ad-ThinkCentre-M75ts-5-SystemFirmware-M4RKT3AA.cab
[592]: https://download.lenovo.com/pccbbs/mobiles/r2zuj38w.exe
[593]: https://download.lenovo.com/pccbbs/mobiles/r37uj22w.exe
[594]: https://fwupd.org/downloads/10982a128fc6e6cecdf6eeff2651431b601e21594b51271cab9ba59a501d564b-r37ul19w.cab
[595]: https://fwupd.org/downloads/5f4482256de34580a2404e141557960ef47b065ca78cd00d5afbf26458fce8ac-r37ul20w.cab
[596]: https://fwupd.org/downloads/ade7ad742f4024e2b2455f71360775a2fa440a5f0b88255c1b7e5d41b577c320-r37ul21w.cab
[597]: https://fwupd.org/downloads/fe436ef3475c95e6677486ab90b091f43a535e0f4271c65ff7b05d8bd3ccd672-r37ul22w.cab
[598]: https://download.lenovo.com/pccbbs/mobiles/r1quj19w.exe
[599]: https://download.lenovo.com/pccbbs/mobiles/r1tuj23w.exe
[600]: https://fwupd.org/downloads/2cda9b3e40703297ed41d0b4962083696e5c5ee898b6b3d1e08fc1964c3cebb2-R26ET33W.cab
[601]: https://fwupd.org/downloads/bd77a59f0a0cb639dddcc2e4858d59a9662be27a99b99360271982813b722c8b-R26ET36W.cab
[602]: https://fwupd.org/downloads/fdb69728cb57b3ba1829bb222b412a76334a496d8919485894aed243813c744a-R26ET39W.cab
[603]: https://fwupd.org/downloads/b904f8d9ef54ca59ab61b5ee2ba6785de1afa90a7526b543a2edd02a91297870-R26ET44W.cab
[604]: https://fwupd.org/downloads/064c7efdb7c1698304ef79a2c8cf96bf792218604d7288d2303d8d9496de8779-R26ET45W.cab
[605]: https://fwupd.org/downloads/f81080a0ad16766bf454526ead27425e015d52c9e63e8372d905931da21e2a70-R26ET50W.cab
[606]: https://fwupd.org/downloads/be588333910ba4151e39514291a90cccd46a1bfec48347c01479c293afb93791-R26ET53W.cab
[607]: https://fwupd.org/downloads/c2389c743eff56077f2e949858eb977452968d020fac071acbad3d80f8f37a77-R26ET54W.cab
[608]: https://download.lenovo.com/pccbbs/mobiles/r26uj19w.exe
[609]: https://download.lenovo.com/pccbbs/mobiles/r2tuj10w.exe
[610]: https://fwupd.org/downloads/e29e3665b816cc88f0cbfb80dfcbfc83b35312996fcca81a2687f6c8f59d2472-Lenovo-ThinkPad-L13YogaGen3-SystemFirmware-0.1.36.cab
[611]: https://fwupd.org/downloads/ffe297871ebbc10f1aa32e6194550f37fae6b8d3d6c152aa2707b7e065843178-R19ET48W.cab
[612]: https://fwupd.org/downloads/aba3df586b6d4010503232724fe977589bdad50b559c175760413ebc49eaad50-R19ET50W.cab
[613]: https://fwupd.org/downloads/5e8a1ad99c8ab48c89867517929f5c8c4ce9bcff659f697dcf6ae93456aa9ef5-R19ET51W.cab
[614]: https://fwupd.org/downloads/bda39efa723777bd03087b5277fa8f364a5c38cf03f6fb8812e543a89eead56e-R19ET53W.cab
[615]: https://fwupd.org/downloads/5b12f079d26a3a82164717f92c2d647ecbb6ecf398362741eac252f6cc586a1f-R19ET56W.cab
[616]: https://fwupd.org/downloads/d6f17339b164fb6d31c19c6cc26082d9b2a547711ea40a6d7003c8061cee6b30-R1KET45W.cab
[617]: https://fwupd.org/downloads/3971368c74cc2b7c7cc9b71d37040b2fae2dd8ffd3bccf406bea819a5ae1b750-R1KET47W.cab
[618]: https://fwupd.org/downloads/6b2e24860ff2be4d94157252cb908b10522db7081e512115d84e66431bc98ad5-R1KET48W.cab
[619]: https://fwupd.org/downloads/df5091f1df03c8539febd02e5fdbc5efe48c0bbb2d10acad4c780440bcba12b7-R1KET51W.cab
[620]: https://download.lenovo.com/pccbbs/mobiles/r1kuj22w.exe
[621]: https://download.lenovo.com/pccbbs/mobiles/r1yuj28w_v2.exe
[622]: https://download.lenovo.com/pccbbs/mobiles/r2iuj16w.exe
[623]: https://download.lenovo.com/pccbbs/mobiles/r2uuj09w.exe
[624]: https://download.lenovo.com/pccbbs/mobiles/r2vuj09w.exe
[625]: https://fwupd.org/downloads/9c2ea4697c1e1a96cd086dd18d4f7ab7728be66fdac32482f261513cd5d46c28-R2VSG12W.cab
[626]: https://fwupd.org/downloads/7506741f250616c70f8cf3d3c6de2b84490d7d1f69ef4ee002027a76962cfaf4-R2VSG14W.cab
[627]: https://fwupd.org/downloads/e57fa33cda07b7df6f4d288b8e0df7bd0b9cd88ac7f2b268144eb392462c78ac-R2VSG18W.cab
[628]: https://fwupd.org/downloads/1725d8e6a5d7f2fa6f08e2b0182d620dc56a1847524e44942a512106a83c0441-R2VSG19W.cab
[629]: https://fwupd.org/downloads/e67d3819188ea0977154a7c6098b64c1c215f583ed5a1ddd8c63f167d021e119-R2VSG25W.cab
[630]: https://fwupd.org/downloads/537c40e380fedf164bcfc65de4436db5f6ecf19d0775026538853952495e0bdd-R2VSG26W.cab
[631]: https://download.lenovo.com/pccbbs/mobiles/r3guj04w.exe
[632]: https://fwupd.org/downloads/914665309962983bc7bf3ae8283a53418a43ef604a4dc92252278056ac02e02b-R25ET39W.cab
[633]: https://fwupd.org/downloads/2680339d8cf770863c7e7f2640a8f77053c458f1a8b45b2d69265138f25b6ba5-R25ET40W.cab
[634]: https://fwupd.org/downloads/94d68bb45ed93b6359627103d3bf37d4d4ba4a8a30f0f836dde0ab323a3b516f-R25ET42W.cab
[635]: https://fwupd.org/downloads/1c2c9f27c7af00e432613f33fca269d219d845993f8f8b4d88f7b732f75e612d-R25ET43W.cab
[636]: https://fwupd.org/downloads/0406b060b4342bacc8667fde54d177b5d5519680994b0fe3cf6521094feb1bad-R25ET44W.cab
[637]: https://fwupd.org/downloads/628e9de2d23c3806ca861d686066c3026fabca65b9dcd6baeded3931d182f0c8-R25ET45W.cab
[638]: https://fwupd.org/downloads/04d9acc766fe7d1d24ba26d4dbc2b57248c78fc1bb3dae01a02e5d22c2b06a62-R25ET46W.cab
[639]: https://fwupd.org/downloads/feb91441041c575348698190ba0405cbf44a8d232c7861bbf265ef1a0dba242d-R25ET47W.cab
[640]: https://fwupd.org/downloads/91774c23ec2463c64e2601b337a7be2580c9b1eb755403c945ac9cd3400cd6e8-R25ET48W.cab
[641]: https://fwupd.org/downloads/a6edde03ed4eb640c636dd688d4e65d8f8471049b60f13835b3538d23cecf6aa-R25ET49W.cab
[642]: https://fwupd.org/downloads/63dcae80b25012a9d454bf02dea4e9b9244b11bbb9a27d6b14f23aedaac77f78-R1YET52W.cab
[643]: https://fwupd.org/downloads/b93549143e50199342f20e380a1a4b25063f5ca1ecbef3fc72f6ff8790bb7b62-R1YET53W.cab
[644]: https://fwupd.org/downloads/f3cc05c1e35cde3ebbf1cb8e8ee92f6319de9b8ca606319b9f5a610e17249095-R1YET54W.cab
[645]: https://fwupd.org/downloads/d049f09f829216c4df762c9dd76d4d5b8444f3fea80a495bafe9a34e1c4dafda-R1YET55W.cab
[646]: https://fwupd.org/downloads/02656becdd9d94c11649d81431eda84b80c3d66c4b65b445517016a4eb146802-R1YET56W.cab
[647]: https://fwupd.org/downloads/0ac3f890fd41a7ca5b75c83f3bf54471539b98543645b2280a68bcf949c106cb-Lenovo-ThinkPadL14L15Gen3-SystemFirmware-0.1.34.cab
[648]: https://fwupd.org/downloads/fa6b162a151c8d089484903b4223f48d184d269a1c79d041b22ce50e52bba4cf-Lenovo-ThinkPadL14L15Gen3-SystemFirmware-0.1.35.cab
[649]: https://fwupd.org/downloads/2bc24148acb646c317068e49994c9e97ec795dfc95273a99168e3971eecb46ce-Lenovo-ThinkPadL14L15Gen3-SystemFirmware-0.1.36.cab
[650]: https://fwupd.org/downloads/3cc69c2b89c448d6ad0770db82926d83bf9e76ea7f1512a3fb2d63da74f2b23f-Lenovo-ThinkPadL14L15Gen3-SystemFirmware-0.1.38.cab
[651]: https://fwupd.org/downloads/dc700e98719128813d104ce9ad4ef0caf61cf434604e59898e395e8ea4def4c8-Lenovo-ThinkPadL14L15Gen3-SystemFirmware-0.1.39.cab
[652]: https://download.lenovo.com/pccbbs/mobiles/r25uj23w.exe
[653]: https://download.lenovo.com/pccbbs/mobiles/r38uj27w.exe
[654]: https://fwupd.org/downloads/76c26dbb5070dc34ca2141b784ac2edb459c589ed646b40aef8f398e6fd6abf7-Lenovo-Thinkpad-P15vAMDGen3-SystemFirmware-1.14.cab
[655]: https://fwupd.org/downloads/d444831faab39d28083b4b8504307064531f461c649dd8295fb0d013439e4301-Lenovo-ThinkPad-P15vGen3AMD-SystemFirmware-1.20.cab
[656]: https://fwupd.org/downloads/2cd224f083f63d9466a447c7f8196836243d1cfb8775465f4f0ea5f981fd360d-Lenovo-ThinkPad-P15vGen3AMD-SystemFirmware-1.21.cab
[657]: https://fwupd.org/downloads/63cac19151f626828c0b74ccbaf11c9e87955cf4c1dbe55ade1f71fae331adf4-Lenovo-ThinkPad-P15vGen3AMD-SystemFirmware-1.23.cab
[658]: https://fwupd.org/downloads/81236b068b62e22da83ad84c2222434317d1526987184328a1cd8710c62efa37-Lenovo-ThinkPad-P15vGen3AMD-SystemFirmware-1.25.cab
[659]: https://fwupd.org/downloads/d846e9f2628065307f34c3d341e19077937d198c8067cdc0306682ab08a85068-Lenovo-ThinkPad-P15vGen3AMD-SystemFirmware-1.27.cab
[660]: https://fwupd.org/downloads/16f0cc16ae36b074f4b472c4089054939acda8360b916152a3b0ecb13caf84e6-Lenovo-ThinkPad-P15vGen3AMD-SystemFirmware-1.28.cab
[661]: https://fwupd.org/downloads/5be2ff1903e4752f743e5c66c06b2672c2bd4735fff42c59cf7b0cf14a085095-Lenovo-ThinkPad-P15vGen3AMD-SystemFirmware-1.31.cab
[662]: https://download.lenovo.com/pccbbs/mobiles/n3kuj17w.exe
[663]: https://download.lenovo.com/pccbbs/mobiles/r2xuj42w.exe
[664]: https://fwupd.org/downloads/c1cbf94eb0e45fdfb16af1df47c5ff14e17e97124f71a954378b6af150a56456-Lenovo-ThinkPad-P16sGen5-SystemFirmware-1.07.cab
[665]: https://download.lenovo.com/pccbbs/mobiles/n3vuj17w_v2.exe
[666]: https://fwupd.org/downloads/d3f883ebfcc6080e71d5291fc2cc5d19505b1e21a2fd8ef982900fa6328b69d4-r3kul20w.cab
[667]: https://fwupd.org/downloads/69c7c14c2d1baf46f90260135435009f343e204d71329e70e210407102336cf7-r3kul22w.cab
[668]: https://fwupd.org/downloads/a0f939b94ab797a7f9fa7a730943cde9afe21c2fe96eb636a77c515a22d16ce4-r3kul23w.cab
[669]: https://fwupd.org/downloads/15331115c5ea86f55088efa374917c977d6f33379eb87b396819a7d903e15b23-r3kul25w.cab
[670]: https://download.lenovo.com/pccbbs/mobiles/r3kuj25w.exe
[671]: https://fwupd.org/downloads/4eb19efe014de76ed52255e53997d2ae2c39a08ac7cdbbdc38b2effe4eeea993-r1bul75w.cab
[672]: https://fwupd.org/downloads/9bee8eba647302c96ff5c8e3206680e46a3d7bcad1fe37219b69b88fb3c6ad74-r1bul76w.cab
[673]: https://fwupd.org/downloads/a5260d13187135218ec16069585cf3015025bbe73a149b5b41513b038f530add-r1bul77w.cab
[674]: https://fwupd.org/downloads/1e9490ee801dc67d89227ea3b28e76ac62e103c03ed00d5f4ba60403d580501d-r1bul78w.cab
[675]: https://fwupd.org/downloads/201a2bcce5bb131df583b54fd4295184167d5e0b3bd7ec2ff28e3579e5e96746-r1bul80w.cab
[676]: https://fwupd.org/downloads/3b134eadfabeaf6c9d439784b2ce6314ffe9dbafb5b502cc401337c66ddd513e-r1bul81w.cab
[677]: https://fwupd.org/downloads/b97e3deb19e82cc2064801ce249e4870db7e1f501411c02147c32e8b6b502cd1-r1bul82w.cab
[678]: https://fwupd.org/downloads/0d96d719573d141e77671d382ae0650842402fc0bbf03809fb9bbb8f6aa95b1d-r1bul83w.cab
[679]: https://fwupd.org/downloads/d78cf004835576d516a0ddabcb9f8bf4f5fee4d88acaa7faf47455cecab0040b-r1bul84w.cab
[680]: https://fwupd.org/downloads/8e9552e95df79587ee83935a39bf54fa4eeac83d72d4d084fe512b1378a64129-r1bul85w.cab
[681]: https://fwupd.org/downloads/c50cbac0ba4f9e071e34eb25875f3da0972cc61fa64265c44ee9b18853acba96-r1bul87w.cab
[682]: https://fwupd.org/downloads/5c25a580a96b2aa1bd4ef18302dd3baedbeee4eb8b187b49e87299c15c84cdeb-r1mul54w.cab
[683]: https://fwupd.org/downloads/fa58301e04a984c6bfdbb0617f92058c27b6cf76c32d28ac451060969a0fe72d-r1mul55w.cab
[684]: https://fwupd.org/downloads/7cfc0a7d7b4d5c636441cd9aad7c41c69a380caf1a1e1ec7f9b086f2cbd374fd-r1mul56w.cab
[685]: https://fwupd.org/downloads/0af615c6325203ca891e96d6171112cb5a0b7e63673d2ace0c8db104ed4cccf7-r1mul58w.cab
[686]: https://fwupd.org/downloads/a1a4b526c2a7c179773a1f06bacaa027f12e2a686a72f804880e0e4069c42354-r1mul59w.cab
[687]: https://fwupd.org/downloads/b5705aa8796fe84e31f22e953012db973c9d4ed00ec0607fbfef8b20e6a7503d-r1mul60w.cab
[688]: https://fwupd.org/downloads/75450b2ebea2d680aedd61bfa4310db436f850505c69e0968dac2fe00bd00e64-r1mul61w.cab
[689]: https://fwupd.org/downloads/0f2c492108b9e02af4b767934965fdf441ddfa12a6ab56c783bb35050668897e-r1mul62w.cab
[690]: https://fwupd.org/downloads/5d7e099b2f9e170798f32d6a4cd9a9a6e8019eb73410b5d5bad2fe46bc5dd553-r1mul63w.cab
[691]: https://fwupd.org/downloads/dd4414c74bc50506def711cf62a8ed730b689132d8bcbdd3530563f5882a218d-r1mul64w.cab
[692]: https://fwupd.org/downloads/c4744fd78eefeb6f062faea328efbb3dd4e691223074b85bf4be921d7c4b5627-r1mul65w.cab
[693]: https://fwupd.org/downloads/1beb43364862e051f2e34115049f5b5abc0bfee9348cdbf4b75bc27681479c1e-r23ul76w.cab
[694]: https://fwupd.org/downloads/610ee9a058ea1b3a8b61e48ae8d092c626feea03d58685908569306899fb0172-r23ul77w.cab
[695]: https://fwupd.org/downloads/db48af2881538babe89b11e6524adf0f89d81cd2008caad565cf1bf03103735d-r23ul78w.cab
[696]: https://fwupd.org/downloads/74b9471f44072a50276db5e80f62bd83c72f70af407a447f3958397e6d8a31ff-r23ul80w.cab
[697]: https://fwupd.org/downloads/e8d033e810a1eea26f03a76f72b46312226e64b5946403839a592a6221d0a8fc-r23ul81w.cab
[698]: https://fwupd.org/downloads/688809a99bb081436a03e66035c380335ca490db9ce67c8c0f49863c8e58457a-r23ul82w.cab
[699]: https://fwupd.org/downloads/561b2c63bf82519cdddea7458d07f7de20ff456a1cd2205b7dec11f1009a8b3b-r23ul84w.cab
[700]: https://fwupd.org/downloads/68f4255e86308d492a0054f9aefb10acf5a7bda842a6518ac86a044575873e8c-r23ul85w.cab
[701]: https://fwupd.org/downloads/5f08342f09f3cdfa28f8581ca41d76bb7ef154d000ff25e295cc631d322272fb-r23ul86w.cab
[702]: https://fwupd.org/downloads/b8cf793d62c90723bc8f1d73db6f7658cd352b43cfaf7b9c4a5322dc9f2e5b31-r23ul88w.cab
[703]: https://fwupd.org/downloads/fdff083bc3b165e765174907a97fe27578352017c7553010661d6504315e71b4-r2ful57w.cab
[704]: https://fwupd.org/downloads/57b2f04151e23f36720defa24b7782e7c24e8006f286a0d8c0d3d00bcadb8189-r2ful58w.cab
[705]: https://fwupd.org/downloads/40bfb547bfc4c836d96ec300b4bc197df812ab32f6b0d2ed197b843854927df1-r2ful59w.cab
[706]: https://fwupd.org/downloads/e82b49e8a77cb4caa9ebc4d26ce6f81b8a098a8937d354d640fcc9360f87125f-r2ful61w.cab
[707]: https://fwupd.org/downloads/513b43bacacc8cccaa9b4bd2780cdf6900d1d862d6f79c8c51d1efeb95f8bc16-r2ful65w.cab
[708]: https://fwupd.org/downloads/210fccb67970f4208d7aea2b90a03f94ddcd2e6c455ff64334e53befa6f4601a-r2ful66w.cab
[709]: https://fwupd.org/downloads/f079ef43d86c11c503c00a70d638650aa63dd642a8781213471427f757704d7a-r2ful67w.cab
[710]: https://fwupd.org/downloads/683d6c89e90bd57e1c177579b308b3ddb7c07665c6940ab6b0d2c062c364d18d-r2ful68w.cab
[711]: https://fwupd.org/downloads/9cad382b3be48b98df852af3bfc90658e23a97e972f40f526b5cf6b434b3a6a3-r2ful69w.cab
[712]: https://fwupd.org/downloads/93cc4c458e30891bc6fedefe5eecfac98ffe84821f6df721a8382ecff3c35976-r2ful70w.cab
[713]: https://fwupd.org/downloads/c7b135d19d960ad996a541f77ce4c6f53eb7b646a3c7d44c0e1233f4c99958c8-r2ful71w.cab
[714]: https://fwupd.org/downloads/f050ce9825d06c17382ea5ac2d780588c13f7c31d38b65bb2e650b8798eb0f70-Lenovo-Thinkpad-T14-P14s-AMD-SystemFirmware-1.07.cab
[715]: https://fwupd.org/downloads/a73ad71522874fdfbcb892c57bdc64d0c896d16e45ee2287d2b21f73ad6fe0aa-R2LET27W.cab
[716]: https://fwupd.org/downloads/8a733d4de7b3e76a97848c343198057b1f0705282145e5494084a123fc1aa552-R2LET28W.cab
[717]: https://fwupd.org/downloads/5a706762278a978c3d9b7c01a831a4bb8b6c86d42c0f91a1b8437c320f94b227-R2LET29W.cab
[718]: https://fwupd.org/downloads/0297a55ab2fbc421dc7fc26d89d74cb913e7c33c2dd4bceb63c6c412262f5a36-R2LET30W.cab
[719]: https://fwupd.org/downloads/f6e715b6ce5f2caa88e05f82c2d24b67e0fce07eac86b28fd0e0928bcd5c5a46-R2LET36W.cab
[720]: https://fwupd.org/downloads/0b5bbe13231bad15cfbaba452eaf6ea89e2e2b86f64decf521c9e3970946c6ad-R2LET38W.cab
[721]: https://fwupd.org/downloads/035880132e8465cfea62147900b70a33a07c56007f7cb7d1b4184b042462a364-R2LET40W.cab
[722]: https://fwupd.org/downloads/ed78bdbd9083c9a843a39f67f0aca749ce299707873d4a9d91cc071caa9fa566-R2LET41W.cab
[723]: https://fwupd.org/downloads/b0477d3f050ef0bdf52be6c770b888841cfe269d34dc83addc712ed7207ce857-R2LET42W.cab
[724]: https://fwupd.org/downloads/23b51cd23e68ed82c6d8c0eff3430ab913de0224b0f1ef1fa66ea6491c98c74d-r2xul34w.cab
[725]: https://fwupd.org/downloads/527b3c52ec747b3054bcb31869a89549238f4e6b929dfb882c6eb2d3481d2059-r2xul35w.cab
[726]: https://fwupd.org/downloads/9423db4eae99e5d55b375badb2129730d11a955196fe0289c701ece13e587d94-r2xul37w.cab
[727]: https://fwupd.org/downloads/2ba516850209298c7b80e7dffffa0bfb159aa1f8788fb59d1c2932bf5f626be3-r2xul39w.cab
[728]: https://fwupd.org/downloads/52d646acf9414a67199235e8a0c2e202c5023efdf599b1ec95fd88c492ba4726-r2xul41w.cab
[729]: https://fwupd.org/downloads/527ac28b37f8b7d1edbb5dda243adb96708566f3ca62eaea0cf10ef604412d4a-r2xul42w.cab
[730]: https://fwupd.org/downloads/cff67e7c32720f49295f83160d126207ddfbe3bb9e93ef83c22a9355867a5628-r38ul21w.cab
[731]: https://fwupd.org/downloads/71d506c391d2d2657c47efe5995edbe59948357d41bb70ecbe3ee0cd036eef23-r38ul24w.cab
[732]: https://fwupd.org/downloads/a2907e70fb085860094530853d57b3b7f9a751afcdd1c6538e2463d9beb645ad-r38ul25w.cab
[733]: https://fwupd.org/downloads/34592770db3a926295eef3e301a5e966deb79839640a6d3a88d777e5badf97b1-r38ul26w.cab
[734]: https://fwupd.org/downloads/5fc7be040d56dbda2ce2ecf09028fbef9a870e6a88d9dedda85a73ccfdae6438-r38ul27w.cab
[735]: https://fwupd.org/downloads/ef5747697f532ba704222dab57d59d34b3980a8317ac6c771f9bf0ce24f33322-Lenovo-Thinkpad-T14-P14s-AMD-SystemFirmware-1.05.cab
[736]: https://fwupd.org/downloads/78a529c403678acbfebe6a376e33d84b7620b0bcee583bc769edfd02dc413cb8-Lenovo-Thinkpad-T14-P14s-AMD-SystemFirmware-1.10.cab
[737]: https://fwupd.org/downloads/6066957c39e0a946693f1b67c2662dc67eff66e5a03dc669e8b8eaa3ae0b5f6c-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.19.cab
[738]: https://fwupd.org/downloads/b72f2e8a72a52e9012c4cbc11821af86a6f6d265141b70b9af8189b48d48c8d0-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.20.cab
[739]: https://fwupd.org/downloads/9ea8a589fbfb2c641b79de04419f5f9d269fef7af11d20bbbb2788e0ae060400-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.21.cab
[740]: https://fwupd.org/downloads/e9b09e88dfbf8b65b431a460d07d26130d7eacdb56c2b3a79a0ed9b95baabf26-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.22.cab
[741]: https://fwupd.org/downloads/955d650cdcb855dd2e9ceac1ba7d14e063096dc5c3ad4165a15210be5e23fa68-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.23.cab
[742]: https://fwupd.org/downloads/8a9304d36fd0599413ee02aa817c182ebed29a2ec02b252c02c265299bfc5063-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.24.cab
[743]: https://fwupd.org/downloads/800de1bb2caa8262bd3e20af59c2be0993f40f0b08016eadf7f87d6f1fbe54d6-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.25.cab
[744]: https://fwupd.org/downloads/c77aa277166d134d6a967af5fdc2c84277977e96986836c1e9b38fc0125acba8-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.27.cab
[745]: https://fwupd.org/downloads/fa8f6e8863ae86f29cdff283051939092390b94f5cf193cac077c86d67bbd6ab-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.28.cab
[746]: https://fwupd.org/downloads/f57668b6777bbc03390c6f2f829c0182129704b7085ef297c0231315b6a5e06e-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.29.cab
[747]: https://fwupd.org/downloads/c994e5b8156a1216cbba0b818f4380124379024e8840dea5b5a3ec12b5186ae7-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.30.cab
[748]: https://fwupd.org/downloads/3966d528350daadf90e2c39c6ffc0d1c2ca450bb14b68d2dfabdb90a9e35b93a-Lenovo-Thinkpad-T14s-AMD-SystemFirmware-1.32.cab
[749]: https://fwupd.org/downloads/df6cec0f58644a131ad4e03e94a81e0a69a95bd36baad2ecb2cfd2fa6cf9e413-Lenovo-Thinkpad-T14sGen6-SystemFirmware-1.09.cab
[750]: https://fwupd.org/downloads/9dc827551a16e8aa06cff7900ea235d8c9e7baf2c3fd3d7c11371f9c74a5a25b-Lenovo-Thinkpad-T14sGen6-SystemFirmware-1.10.cab
[751]: https://fwupd.org/downloads/63e9b3a90554e961d1461605bcfc9c6ad4b5793e5cbb58b0945473e3e88154b0-r2nul06w.cab
[752]: https://fwupd.org/downloads/82be1b03942a4a5c0db6c1becc2c5cf736eeaef3df327ba8952847ddd18555f7-r2nul08w.cab
[753]: https://fwupd.org/downloads/0df71fbdf4d985f288f7ae4e6079405d9a170fc60dc5cb1abef5daba2cdf0977-r2nul09w.cab
[754]: https://fwupd.org/downloads/25167619b801d421b15dd495252b21053803a5dda07a0caec4944fd4ba9436d0-r2nul11w.cab
[755]: https://fwupd.org/downloads/81416d3643e4834934d5afb79997ebd6e4c56f9f419b702309c145fe5e7e578d-r2nul12w.cab
[756]: https://fwupd.org/downloads/0be10518faa994738f06ff9474420ef5721eab9fc24e057767102114e6ccc1e2-r2nul13w.cab
[757]: https://fwupd.org/downloads/804af64662ae8898b27e06bf41551655c67d29b9c9d04a1f089483ca989492b4-Lenovo-ThinkPad-X13Gen6-SystemFirmware_v1-1.05.cab
[758]: https://fwupd.org/downloads/45a592c6492726505334be6b08178f00d75f8c33c43fcf3fd4518da75b87ef8d-Lenovo-ThinkPad-X13Gen6-SystemFirmware_v1-1.08.cab
[759]: https://fwupd.org/downloads/8872fdba30b337f83ff428cbf9dc093dec07143a2cbc7e4e28aab5c0d4a55ba3-Lenovo-ThinkPad-X13Gen6-SystemFirmware_v1-1.10.cab
[760]: https://fwupd.org/downloads/8afd51a9d6ef56f75905234bed96eb027f12f53e7b156dea6622846f3d6ffb62-Lenovo-ThinkPad-X13Gen6-SystemFirmware_v1-1.11.cab
[761]: https://fwupd.org/downloads/374c16ec1675ff1b9ee93aca9565af252b183e79d5144f86bf090dcf56e2ed7f-Lenovo-ThinkPad-X13Gen7-SystemFirmware-1.09.cab
[762]: https://fwupd.org/downloads/4006e81f6d7d06565b86dcdb0b8f75c50fe1774e08d0ec8f7343c3156092059e-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.44.cab
[763]: https://fwupd.org/downloads/a429ddb9236a059347ee331cb5a7e9791f2179d345fec7a63b8ccfa9c0042181-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.45.cab
[764]: https://fwupd.org/downloads/ff91b66bf9fc8a8eef5f5812e7d56102cb6350787541e93d1f5946b5e1877692-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.46.cab
[765]: https://fwupd.org/downloads/8f16eeb97f048bc18b9219ff7df5eb785226df4604ff3af00f3d1a24b85ab8e4-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.48.cab
[766]: https://fwupd.org/downloads/d00806dbda64ca211c3b597c891ec092c0982cb93a7b474723ef07edf63b454b-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.49.cab
[767]: https://fwupd.org/downloads/85ff31861320d93ac477641df467e014491658c8954dfd69ba490f6758e3a036-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.51.cab
[768]: https://fwupd.org/downloads/8c12ddf6d3f79f137efd16496d6166a098d248527c024870721bce5dee6681e8-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.52.cab
[769]: https://fwupd.org/downloads/9fb76d8d59c41254cf643bbd617d7bb65c751bdfca0fe253848c97069fb4faac-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.53.cab
[770]: https://fwupd.org/downloads/d0c1f8e59a4fa5b5b05457d9fddddf4fefaeb04046ba6677f13124bc900a9169-Lenovo-ThinkPad-X13-T14s-AMD-SystemFirmware-1.54.cab
[771]: https://fwupd.org/downloads/2598fc86dce2669ef6c8f3fbe00783c197d700cc37d1ec403eb84d3da95ccaa5-r1nul57w.cab
[772]: https://fwupd.org/downloads/8c5e97f8060da018409d2331fe2f4adf1512f8621ef5364faefce679c5d1b3ab-r1nul58w.cab
[773]: https://fwupd.org/downloads/cf0d157df30be203f7ece3888468041e73fd3714564ed358e5da99f119544941-r1nul59w.cab
[774]: https://fwupd.org/downloads/e0a24c4df411e7b1c5c941a455b495d3600c550816b535279c1a6472df9bba45-r1nul61w.cab
[775]: https://fwupd.org/downloads/43730b2e5e48f564253bf88616e66b1eb15a5d8cafafc27930cdb6443d0664c9-r1nul62w.cab
[776]: https://fwupd.org/downloads/269a18ed41aab1c9055f9344fe8bbf2b116bd0bfc0e1ce966c0c3fe3c36d4129-r1nul63w.cab
[777]: https://fwupd.org/downloads/120aa0431048905b09d17482db980b2aa3db07535092bd296a9c86ce15c0c4be-r1nul64w.cab
[778]: https://fwupd.org/downloads/d5ee8251318b5027432e66f8ff6d3e12e819fd6fd2f14c26368ac74680baff63-r1nul66w.cab
[779]: https://fwupd.org/downloads/49d66a0e350fbbf207f6a9715087884770bbd0e97e285d4e676ecf98075cb8c4-r1nul67w.cab
[780]: https://fwupd.org/downloads/4ebe31743644d2ae20519e0e650a42ffba7cd2eb17ac456052fc395cdb8ed231-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.40.cab
[781]: https://fwupd.org/downloads/b7a3f73b284479cb98e241827dc6786d266462f3158b072a830661dcedb01f60-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.42.cab
[782]: https://fwupd.org/downloads/b40f3f9e50b41cd7ae057bd3573f3aa7e8274a75efed57c06eb45a1a6fd08e30-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.44.cab
[783]: https://fwupd.org/downloads/d60c3d696c80050443ed21e6476f800dc998594169c5bb3efcbcb0938150ec5e-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.45.cab
[784]: https://fwupd.org/downloads/b528ca0ff9e7d8b18672f729672eb95c7e67d765f5c91edb6ed8bb92fb405920-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.46.cab
[785]: https://fwupd.org/downloads/6fa23d103a690c9f9be82f74e072f7fb5209739c3683cfa62c65f9ad3e05577c-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.50.cab
[786]: https://fwupd.org/downloads/39a198cfd63f6dd80353515bce28c5b642770cfff91db88d8592ad5e50791aae-Lenovo-Thinkpad-X13-T14s-AMD-SystemFirmware-1.51.cab
[787]: https://fwupd.org/downloads/f5dea3d5b5ed5442d34fad4ae04ef9c64695b59aaf2b127857d5054e79d1b953-N3GET69W.cab
[788]: https://fwupd.org/downloads/da85c5322e5127438b2a7241f7669456c772e6e2a3cc9a877f0c158106999142-N3GET70W.cab
[789]: https://fwupd.org/downloads/b737c2691897866d487f0885a83f25719d1e9849fa7189d8f9ee4920e5ba43f2-N3GET71W.cab
[790]: https://fwupd.org/downloads/5126e9720e10d611635acd56bf6fcc9fdef229b768aa08311102c2c2f31de421-N3GET72W.cab
[791]: https://fwupd.org/downloads/37562b8f56a215447256b6a12ea6f35f51115042a25b1530069c15122ebaac20-N3GET73W.cab
[792]: https://fwupd.org/downloads/075fc80dee2bf074f26e8adeff2169098db02a794fcba02b4cb2e16e36edbbdd-N3GET74W.cab
[793]: https://fwupd.org/downloads/345d6a1a5f0df8c5e076aa79da563f66bdd0f2cfbddd155eae62f49ded24dd63-N3GET76W.cab
[794]: https://fwupd.org/downloads/41521603b274b757c0319780a7a527c643fc6e237bf32cfe597c36efb5641ab7-N3GET78W.cab
[795]: https://fwupd.org/downloads/504647cf799c1bfe92718e143435f130fef53a17e8dc6619af9900037527e066-N3GET79W.cab
[796]: https://fwupd.org/downloads/ce7336eecbc84f0afa3d4d5d4abe0dc7dcc863e5e5e8caac4b36ba72120aa14e-N3GET81W.cab
[797]: https://fwupd.org/downloads/43bbfc569bd4c975a3c2af5d4023139ef2e7d848f848b3e2fde74199da2b4406-Lenovo_ThinkPad_Z13Gen2_Z16Gen_2_SystemFirmware_1.14.cab
[798]: https://fwupd.org/downloads/7ded6163808113bda6f8a067935490755f37108ce37d1670df14e99d724052a3-Lenovo_ThinkPad_Z13Gen2_Z16Gen_2_SystemFirmware_1.15.cab
[799]: https://fwupd.org/downloads/6d1bdf9c93d5cdcf2e5cdb3979ee64a4d6e319a5fa55a07f6ced7568f2971a82-N41ET53W.cab
[800]: https://fwupd.org/downloads/6fad6e52fb631cbdfbdb3282b1b9ea2d241db03b788ab63feaf5a114c9ac075e-N41ET54W.cab
[801]: https://fwupd.org/downloads/0aea5c762d1dd5f8a3b1eefeffa169c2afcf04bc724758457b9fd51fcf115248-N41ET55W.cab
[802]: https://fwupd.org/downloads/5025d8ff1161f32ca53fdeda5e5f711170599c41ea6e5228b0abb3ae2add911e-N41ET56W.cab
[803]: https://fwupd.org/downloads/d70aa4819b51f4e5104cc68a59cc4e65332710263e8f4581440f6dc1dfe3cd45-N41ET59W.cab
[804]: https://fwupd.org/downloads/61c156578c6e5aed5577ece29f837b4269ed6568e8e714b2f7ec2e2e90c2284f-N41ET60W.cab
[805]: https://fwupd.org/downloads/a9736cafa4cf6df1722d458b860193c9103945a444da42b7dcf1759b25a35bb3-N41ET62W.cab
[806]: https://fwupd.org/downloads/99da7609eb87755849804d5d9c07894943672c9e2415a1b0f0d18fc738fed1ab-N41ET63W.cab
[807]: https://fwupd.org/downloads/3265bfdcebabb1525dda7f5a35826bb8d81a0cb0e6e8ca58c3927faed8fb5037-N41ET65W.cab
[808]: https://download.lenovo.com/pccbbs/mobiles/n41uj17w.exe
[809]: https://download.lenovo.com/pccbbs/mobiles/n3guj32w.exe
[810]: https://fwupd.org/downloads/9ffe523e8cf4c68f1c30de4a127ee9b20f796562b6f33b2e3fad8662cf0d6f99-Lenovo-Yoga-Pro-7-15ASH11-AMD-SystemFirmware-1.19.cab
[811]: https://download.msi.com/bos_exe/mb/7E56v1A1.zip
[812]: https://download.msi.com/bos_exe/mb/7E56v2A6.zip
[813]: https://download.msi.com/bos_exe/mb/7E56v4A43.zip
[814]: https://download.msi.com/bos_exe/mb/7C91v1E1.zip
[815]: https://download.msi.com/bos_exe/mb/7C91v1F.zip
[816]: https://download.msi.com/bos_exe/mb/7C91v2C2.zip
[817]: https://download.msi.com/bos_exe/mb/7D75v1L1.zip
[818]: https://download.msi.com/bos_exe/mb/7D75v1Q.zip
[819]: https://fwupd.org/downloads/51efbb2cdc4d8db30b719bce69c4d2cc66568d08008a2df939a565f4c3cb4313-starbook_cezanne.cab
[820]: https://fwupd.org/downloads/1d27d457da5b203082e08662e67f0f9f84978809ef815524bfd0dfb7c6600184-starbook_cezanne.cab
