# 三项能力矩阵

| 能力 | 命中样本 | 占比 |
|---|---|---|
| 正常解包（解出 >=1 个内嵌文件） | **298** | 97% |
| 恢复符号表 | **116** | 38% |
| └ 其中符号名总数 | 824018 个 | — |
| 恢复文件名（清单） | **236** | 77% |
| └ 其中仅靠名字收割（无偏移） | 17 | 6% |
| 恢复文件名（清单+内容） | **163** | 53% |
| 三项同时满足 | **66** | 21% |
| 样本总数 | 308 | 100% |

## 分厂商

| 厂商 | 样本 | 解包 | 符号表 | 文件名清单 | 文件名+内容 |
|---|---|---|---|---|---|
| mercury | 143 | 140 | 38 | 131 | 108 |
| tp-link | 58 | 57 | 20 | 50 | 38 |
| tektronix | 45 | 39 | 31 | 0 | 0 |
| fast | 14 | 14 | 3 | 12 | 7 |
| anritsu | 12 | 12 | 12 | 0 | 0 |
| linksys | 10 | 10 | 7 | 10 | 1 |
| netgear | 9 | 9 | 4 | 9 | 9 |
| belkin | 7 | 7 | 0 | 0 | 0 |
| trendnet | 7 | 7 | 0 | 7 | 0 |
| intellistar | 2 | 2 | 1 | 0 | 0 |
| nortel | 1 | 1 | 0 | 0 | 0 |

## 按型号族

| 族 | 样本 | 解包 | 符号表 | 文件名(有表) | 文件名(仅名字) |
|---|---|---|---|---|---|
| mercury:opaque | 103 | 100 | 37 | 91 | 3 |
| tektronix:instrument | 45 | 39 | 31 | 0 | 0 |
| mercury:minifs | 40 | 40 | 1 | 39 | 1 |
| tp-link:legacy | 34 | 33 | 0 | 26 | 7 |
| tp-link:rtos | 23 | 23 | 19 | 23 | 0 |
| fast:opaque | 13 | 13 | 3 | 11 | 0 |
| anritsu:instrument | 12 | 12 | 12 | 0 | 0 |
| linksys:troc | 9 | 9 | 7 | 9 | 0 |
| netgear:chk | 9 | 9 | 4 | 0 | 9 |
| trendnet:other | 7 | 7 | 0 | 7 | 0 |
| belkin:deflate-stub | 6 | 6 | 0 | 0 | 6 |
| intellistar:other | 2 | 2 | 1 | 0 | 0 |
| belkin:other | 1 | 1 | 0 | 0 | 1 |
| fast:minifs | 1 | 1 | 0 | 1 | 0 |
| linksys:other | 1 | 1 | 0 | 1 | 0 |
| nortel:other | 1 | 1 | 0 | 0 | 0 |
| tp-link:other | 1 | 1 | 1 | 1 | 0 |

## 恢复符号表的样本（116）

- [anritsu] anritsu_MS2024A_V1.66_fw.zip — 6060 个符号名
- [anritsu] anritsu_MS2026B_V2.01_fw.zip — 6041 个符号名
- [anritsu] anritsu_MS2034A_V1.21_fw.zip — 6060 个符号名
- [anritsu] anritsu_MS2717B_V4.04_fw.zip — 6060 个符号名
- [anritsu] anritsu_MS2720T_V4.80_fw.zip — 6041 个符号名
- [anritsu] anritsu_MS2721B_V1.51_fw.zip — 6060 个符号名
- [anritsu] anritsu_MS2723B_V1.39_fw.zip — 6060 个符号名
- [anritsu] anritsu_MS272xC_V2.30_fw.zip — 6041 个符号名
- [anritsu] anritsu_MT8222A_V1.80_fw.zip — 6060 个符号名
- [anritsu] anritsu_MW82119A_V2.30_fw.zip — 6041 个符号名
- [anritsu] anritsu_MW82119B_V4.80_fw.zip — 6041 个符号名
- [anritsu] anritsu_S412E_V4.81_fw.zip — 6041 个符号名
- [tektronix] tektronix_TDS510A_1.0e_fw.bin — 4887 个符号名
- [tektronix] tektronix_TDS520A_v3.8.7e_fw.bin — 4891 个符号名
- [tektronix] tektronix_TDS520C_v5.0e_fw.bin — 6207 个符号名
- [tektronix] tektronix_TDS520C_v5.1.1e_fw.bin — 6252 个符号名
- [tektronix] tektronix_TDS520C_5.3e_fw.bin — 6286 个符号名
- [tektronix] tektronix_TDS524A_v3.8.7e_fw.bin — 4891 个符号名
- [tektronix] tektronix_TDS540A_v3.8.3e_fw.bin — 4881 个符号名
- [tektronix] tektronix_TDS540B_v4.1e_fw.bin — 5340 个符号名
- [tektronix] tektronix_TDS540C_v5.2e_fw.bin — 6310 个符号名
- [tektronix] tektronix_TDS544A_v3.8.4e_eprom_fw.bin — 4884 个符号名
- [tektronix] tektronix_TDS620A_v3.8.5e_fw.bin — 4786 个符号名
- [tektronix] tektronix_TDS620B_v4.2e_fw.bin — 5196 个符号名
- [tektronix] tektronix_TDS620B_v4.4.1e_fw.bin — 5270 个符号名
- [tektronix] tektronix_TDS644A_v3.8.4e_fw.bin — 4786 个符号名
- [tektronix] tektronix_TDS644A_v3.8.5e_fw.bin — 4786 个符号名
- [tektronix] tektronix_TDS684C_v5.1e_fw.bin — 5950 个符号名
- [tektronix] tektronix_TDS714L_v7.4e_fw.bin — 7115 个符号名
- [tektronix] tektronix_TDS714L_v7.1.1e_fw.bin — 7100 个符号名
- [tektronix] tektronix_TDS744A_1.0e_fw.bin — 5247 个符号名
- [tektronix] tektronix_TDS744A_1.1e_fw.bin — 5250 个符号名
- [tektronix] tektronix_TDS744A_4.0e_fw.bin — 5337 个符号名
- [tektronix] tektronix_TDS744A_4.2.1e_fw.bin — 5340 个符号名
- [tektronix] tektronix_TDS754D_v6.0e_fw.bin — 7019 个符号名
- [tektronix] tektronix_TDS784D_v6.6e_fw.bin — 7081 个符号名
- [tektronix] tektronix_TDS784D_v7.2e_fw.bin — 7112 个符号名
- [tektronix] tektronix_TDS794D_v8.0e_fw.bin — 7085 个符号名
- [tektronix] tektronix_TDS794D_v6.3e_fw.bin — 7051 个符号名
- [tektronix] tektronix_TDS794D_v6.4e_fw.bin — 7079 个符号名
- [tektronix] tektronix_TLS216_1.0.3e_fw.bin — 4761 个符号名
- [tektronix] tektronix_TLS216_1.0e_fw.bin — 4751 个符号名
- [fast] FW300R__2012126162529.bin — 7610 个符号名
- [fast] 2013117155732.rar — 6798 个符号名
- [fast] 2013724172437.rar — 6862 个符号名
- [intellistar] intellistar_thunderstorm_vxworks.bin — 2555 个符号名
- [linksys] linksys_WRT54Gv5v6_1.02.5.bin — 683 个符号名
- [linksys] linksys_WRT54Gv5v6_1.02.8.bin — 684 个符号名
- [linksys] linksys_WRT54Gv7.2_7.2.01.007.bin — 684 个符号名
- [linksys] linksys_WRT54G2v1_1.0.01.003.bin — 684 个符号名
- [linksys] linksys_FW_WRT54GSv5v6_1.52.8.001_US_20091005.bin — 684 个符号名
- [linksys] linksys_WRT54GS-v5v6_1.52.5.002_fw.bin — 683 个符号名
- [linksys] linksys_WRT54GSv5_v1_50_9.zip — 746 个符号名
- [mercury] MAC1200RE_V2.0.zip — 6176 个符号名
- [mercury] MR900_V3.0_20161126.zip — 17955 个符号名
- [mercury] MR900B_V3.0_150521.zip — 17933 个符号名
- [mercury] MR900B_V3.0_161126.zip — 17955 个符号名
- [mercury] MW150R_V5_V6_111219.rar — 6692 个符号名
- [mercury] mercury_MW150R_V5-V6_111231_test.bin — 6685 个符号名
- [mercury] mercury_MW150R_V5-V6_120109_special.bin — 6690 个符号名
- [mercury] mercury_MW150R_V5-V6_120217_special.bin — 6725 个符号名
- [mercury] MW150RM_V1_V2_120530.rar — 7099 个符号名
- [mercury] MW150RM_V1_V2_120809.rar — 7122 个符号名
- [mercury] MW151R_V1_100113.rar — 4516 个符号名
- [mercury] MW151R__2010628151219.bin — 4525 个符号名
- [mercury] MW151RM_V1_120425.rar — 7054 个符号名
- [mercury] MW151RM_V1_121107.rar — 7121 个符号名
- [mercury] MW151RM__201223163746.bin — 7129 个符号名
- [mercury] MW153R_V1_110909.rar — 7377 个符号名
- [mercury] MW300R_V4_121126.rar — 7610 个符号名
- [mercury] MW300R_V6.0_130705.rar — 6862 个符号名
- [mercury] MW300R_V6_130226.rar — 6836 个符号名
- [mercury] MW300R__201275172121.bin — 7609 个符号名
- [mercury] MW300R__201275173752.bin — 7610 个符号名
- [mercury] MW300R__201216162049.bin — 7623 个符号名
- [mercury] MW300R__201275174246.bin — 7609 个符号名
- [mercury] MW300R__201275174464.bin — 7610 个符号名
- [mercury] MW303R_V1_120201.rar — 7607 个符号名
- [mercury] MW303R__201275174751.bin — 7609 个符号名
- [mercury] MW303R__201275174943.bin — 7610 个符号名
- [mercury] MW305R_V1.0_130627.rar — 6862 个符号名
- [mercury] MW305R_V1_130111.rar — 6798 个符号名
- [mercury] MW310R_V1_V2_120201.rar — 7607 个符号名
- [mercury] MW310R_V1_V2_130201.rar — 7624 个符号名
- [mercury] mercury_MW310R_V1-V2_120507_special.bin — 7609 个符号名
- [mercury] mercury_MW310R_V1-V2_120508_special.bin — 7610 个符号名
- [mercury] MW450R_V2.0_140225.zip — 5215 个符号名
- [mercury] MW54R_V10_110311.rar — 7345 个符号名
- [mercury] MW54R__201181192837.bin — 7371 个符号名
- [mercury] MW54R__2011111162552.bin — 7529 个符号名
- [mercury] MW54R__2011111163235.bin — 7529 个符号名
- [netgear] a6b_3_WGT624_20and_20WGT624v2_20Firmware_20Version_204.2.10.zip — 9634 个符号名
- [netgear] WGT624v1_FW4.2.8.zip — 9634 个符号名
- [netgear] WGT624v2_FW4.2.11.zip — 9639 个符号名
- [netgear] a6_WPN824v2_2.0.16.zip — 7563 个符号名
- [tektronix] tektronix_tds754c_fw53e.zip — 6286 个符号名
- [tp-link] Archer C64(EU)_V1_210219.zip — 10385 个符号名
- [tp-link] tplink_ArcherC80_V1_200628_US.bin — 11042 个符号名
- [tp-link] tplink_ArcherC80_V1_200907_US.bin — 10936 个符号名
- [tp-link] tplink_ArcherC80_V1_201003_US.bin — 10944 个符号名
- [tp-link] tplink_ArcherC80_V1_210128_US.bin — 10373 个符号名
- [tp-link] tplink_ArcherC80_V1_210308_US.bin — 11208 个符号名
- [tp-link] tplink_ArcherC80_V1_210203_CA.bin — 10373 个符号名
- [tp-link] Archer_C80(US)_V1_200221.zip — 10959 个符号名
- [tp-link] Archer_C80_US_V1_191122.zip — 10956 个符号名
- [tp-link] Archer C64(EU)_V1_210317.zip — 10390 个符号名
- [tp-link] Archer C64(EU)_V1_210609.zip — 10418 个符号名
- [tp-link] Archer C80(EU)_V1 200907.zip — 10936 个符号名
- [tp-link] Archer C80(EU)_V1_191204.zip — 10951 个符号名
- [tp-link] Archer C80(EU)_V1_200330.zip — 11041 个符号名
- [tp-link] Archer C80(EU)_V1_200628.zip — 11042 个符号名
- [tp-link] Archer C80(EU)_V1_201003.zip — 10944 个符号名
- [tp-link] Archer C80(EU)_V1_210128.zip — 10373 个符号名
- [tp-link] Archer C80(EU)_V1_210308.zip — 11208 个符号名
- [tp-link] TL-WR820N(EU)_V2_191129.zip — 6196 个符号名
- [tp-link] TL-WR820N(EU)_V2_200731.zip — 6329 个符号名

## 能恢复文件名的样本（236，名字清单共 29400 条）

- [tp-link] tplink_ArcherC80_V1_200907_US.bin — 清单 317 条 / 有效文件 291 个
- [tp-link] tplink_ArcherC80_V1_201003_US.bin — 清单 317 条 / 有效文件 291 个
- [tp-link] Archer C80(EU)_V1 200907.zip — 清单 317 条 / 有效文件 291 个
- [tp-link] Archer C80(EU)_V1_201003.zip — 清单 317 条 / 有效文件 291 个
- [tp-link] tplink_ArcherC80_V1_200628_US.bin — 清单 314 条 / 有效文件 289 个
- [tp-link] Archer C80(EU)_V1_200330.zip — 清单 314 条 / 有效文件 289 个
- [tp-link] Archer C80(EU)_V1_200628.zip — 清单 314 条 / 有效文件 289 个
- [tp-link] Archer_C80(US)_V1_200221.zip — 清单 311 条 / 有效文件 286 个
- [tp-link] Archer C64(EU)_V1_210317.zip — 清单 310 条 / 有效文件 285 个
- [tp-link] Archer C64(EU)_V1_210609.zip — 清单 310 条 / 有效文件 285 个
- [tp-link] Archer C64(EU)_V1_210219.zip — 清单 309 条 / 有效文件 283 个
- [tp-link] tplink_ArcherC80_V1_210128_US.bin — 清单 309 条 / 有效文件 283 个
- [tp-link] tplink_ArcherC80_V1_210308_US.bin — 清单 309 条 / 有效文件 283 个
- [tp-link] tplink_ArcherC80_V1_210203_CA.bin — 清单 309 条 / 有效文件 283 个
- [tp-link] Archer C80(EU)_V1_210128.zip — 清单 309 条 / 有效文件 283 个
- [tp-link] Archer C80(EU)_V1_210308.zip — 清单 309 条 / 有效文件 283 个
- [tp-link] Archer C80(EU)_V1_191204.zip — 清单 299 条 / 有效文件 274 个
- [tp-link] Archer_C80_US_V1_191122.zip — 清单 297 条 / 有效文件 272 个
- [tp-link] TL-ER604W(EU)_V2_160825_1480562237189w.zip — 清单 265 条 / 有效文件 261 个
- [tp-link] TL-WR820N(EU)_V2_200731.zip — 清单 265 条 / 有效文件 246 个
- [tp-link] TL-ER604W_V1_130116.zip — 清单 251 条 / 有效文件 248 个
- [mercury] MR900B_V3.0_150521.zip — 清单 251 条 / 有效文件 247 个
- [mercury] MR900B_V3.0_161126.zip — 清单 251 条 / 有效文件 247 个
- [tp-link] TL-WR820N(EU)_V2_191129.zip — 清单 251 条 / 有效文件 233 个
- [mercury] MR900B_V2_130109.rar — 清单 246 条 / 有效文件 242 个
- [tp-link] TL-ER6020_V1_120807.zip — 清单 239 条 / 有效文件 237 个
- [tp-link] TL-ER6120_V1_120719.zip — 清单 239 条 / 有效文件 237 个
- [mercury] MR900_V3.0_20161126.zip — 清单 239 条 / 有效文件 235 个
- [fast] 2013125191408.rar — 清单 234 条 / 有效文件 230 个
- [mercury] MR900_V2_130109.rar — 清单 234 条 / 有效文件 230 个
- [tp-link] TL-ER5120_V2_140212.zip — 清单 226 条 / 有效文件 224 个
- [tp-link] TL-ER5120_V1_120719.zip — 清单 224 条 / 有效文件 222 个
- [mercury] MW150RM_V1_V2_120530.rar — 清单 212 条 / 有效文件 84 个
- [mercury] MW151RM_V1_120425.rar — 清单 212 条 / 有效文件 84 个
- [mercury] MW151RM__201223163746.bin — 清单 212 条 / 有效文件 84 个
- [mercury] MW150RM_V1_V2_120809.rar — 清单 212 条 / 有效文件 83 个
- [mercury] MW151RM_V1_121107.rar — 清单 212 条 / 有效文件 83 个
- [mercury] MR904B_v2_110714.rar — 清单 180 条 / 有效文件 181 个
- [mercury] MW1515R_V1_121012.rar — 清单 175 条 / 有效文件 70 个
- [mercury] MR904B_v2_110529.rar — 清单 174 条 / 有效文件 175 个
- [mercury] mercury_MW150R_V5-V6_120217_special.bin — 清单 170 条 / 有效文件 69 个
- [fast] FW300R__2012126162529.bin — 清单 170 条 / 有效文件 67 个
- [mercury] MW54R__2011111162552.bin — 清单 169 条 / 有效文件 68 个
- [mercury] MW54R__2011111163235.bin — 清单 169 条 / 有效文件 68 个
- [mercury] MW153R_V1_110909.rar — 清单 169 条 / 有效文件 67 个
- [mercury] MW310R_V1_V2_130201.rar — 清单 168 条 / 有效文件 66 个
- [fast] FW150R__20121212153667.bin — 清单 168 条 / 有效文件 61 个
- [mercury] MR904_V3_110714.rar — 清单 167 条 / 有效文件 168 个
- [mercury] MW150R_V5_V6_111219.rar — 清单 167 条 / 有效文件 66 个
- [mercury] mercury_MW150R_V5-V6_111231_test.bin — 清单 167 条 / 有效文件 66 个
- [mercury] mercury_MW150R_V5-V6_120109_special.bin — 清单 167 条 / 有效文件 66 个
- [fast] 201182201224.rar — 清单 166 条 / 有效文件 167 个
- [mercury] MW300R_V4_121126.rar — 清单 166 条 / 有效文件 68 个
- [mercury] MW300R__201275172121.bin — 清单 166 条 / 有效文件 68 个
- [mercury] MW300R__201275173752.bin — 清单 166 条 / 有效文件 68 个
- [mercury] MW303R_V1_120201.rar — 清单 166 条 / 有效文件 66 个
- [mercury] MW303R__201275174751.bin — 清单 166 条 / 有效文件 66 个
- [mercury] MW303R__201275174943.bin — 清单 166 条 / 有效文件 66 个
- [mercury] MW310R_V1_V2_120201.rar — 清单 166 条 / 有效文件 66 个
- [mercury] mercury_MW310R_V1-V2_120507_special.bin — 清单 166 条 / 有效文件 66 个
- [mercury] mercury_MW310R_V1-V2_120508_special.bin — 清单 166 条 / 有效文件 66 个
- [mercury] MW150R__201342200320.bin — 清单 166 条 / 有效文件 63 个
- [mercury] MW153R__201342200827.bin — 清单 166 条 / 有效文件 63 个
- [mercury] MW153R__201342200957.bin — 清单 166 条 / 有效文件 63 个
- [mercury] mercury_MW150R_V8_130226_test.bin — 清单 166 条 / 有效文件 62 个
- [fast] 2014314161063.zip — 清单 165 条 / 有效文件 59 个
- [mercury] MW150R_V8_121113.rar — 清单 164 条 / 有效文件 61 个
- [mercury] MW153R_V1_121127.rar — 清单 164 条 / 有效文件 61 个
- [mercury] MW155R_V1.0_140310.zip — 清单 163 条 / 有效文件 61 个
- [fast] 2014326163556.zip — 清单 163 条 / 有效文件 58 个
- [mercury] MW300R_V9.0_140210.rar — 清单 162 条 / 有效文件 59 个
- [mercury] MW305R_V3.0_140210.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW305R_V3.0_140528.zip — 清单 162 条 / 有效文件 59 个
- [mercury] 2014512165923.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW306R_V1.0_140210.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW306R_V1.0_140528.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW309R_V1.0_140219.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW309R_V1.0_140613.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW316R_V1.0_141105.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW316R_V1.0_150611.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW316R_V1.0_150817.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW320R_V1.0_141111.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW320R_V1.0_150925.zip — 清单 162 条 / 有效文件 59 个
- [mercury] MW300R_V9.0_140528.zip — 清单 162 条 / 有效文件 58 个
- [mercury] MW450R_V2.0_140225.zip — 清单 162 条 / 有效文件 58 个
- [mercury] MR904_V3_110529.rar — 清单 161 条 / 有效文件 162 个
- [mercury] MW155R_V1_V2_130427.rar — 清单 160 条 / 有效文件 60 个
- [mercury] MW150R_V10.0_140225.rar — 清单 160 条 / 有效文件 58 个
- [tp-link] TL-WA5210G_V1_130502.zip — 清单 154 条 / 有效文件 0 个
- [tp-link] TL_WA5110G_V1_2009721151054.zip — 清单 151 条 / 有效文件 2 个
- [tp-link] TL-WA5110G_V1_For_EU.zip — 清单 151 条 / 有效文件 2 个
- [tp-link] TL-WA5110G_V2_120217.zip — 清单 151 条 / 有效文件 2 个
- [trendnet] trendnet_TEW-511BRP_1.0.19_US.zip — 清单 149 条 / 有效文件 0 个
- [trendnet] trendnet_TEW-511BRP_1.0.20_EU.zip — 清单 149 条 / 有效文件 0 个
- [mercury] MW54R__201181192837.bin — 清单 143 条 / 有效文件 48 个
- [mercury] MW300R__201216162049.bin — 清单 140 条 / 有效文件 48 个
- [mercury] MW300R__201275174246.bin — 清单 140 条 / 有效文件 48 个
- [mercury] MW300R__201275174464.bin — 清单 140 条 / 有效文件 48 个
- [mercury] MW54R_V10_110311.rar — 清单 140 条 / 有效文件 48 个
- [tp-link] TL-WR340G_V1_101102.zip — 清单 136 条 / 有效文件 45 个
- [tp-link] TL_WR340G_V2_2010102011103318.zip — 清单 136 条 / 有效文件 45 个
- [tp-link] ru_2010462074711.zip — 清单 136 条 / 有效文件 45 个
- [tp-link] TL-R460_v5.0_120704_RU_UA.zip — 清单 136 条 / 有效文件 44 个
- [tp-link] TL_WR541G_V3_200911516304613.zip — 清单 135 条 / 有效文件 0 个
- [tp-link] TL_WR541G_V4_200961473666.zip — 清单 135 条 / 有效文件 0 个
- [tp-link] ru_2010461450313.zip — 清单 134 条 / 有效文件 44 个
- [fast] 201615150824.zip — 清单 132 条 / 有效文件 96 个
- [mercury] MW151R_V1_100113.rar — 清单 127 条 / 有效文件 43 个
- [mercury] MW151R__2010628151219.bin — 清单 127 条 / 有效文件 43 个
- [tp-link] TL_R860_V1_201112513445417.zip — 清单 126 条 / 有效文件 0 个
- [mercury] MW325R_V4.0.zip — 清单 125 条 / 有效文件 93 个
- [mercury] MW300R_V12.0_150610.zip — 清单 124 条 / 有效文件 94 个
- [mercury] MW300R_V12.0_160309.zip — 清单 124 条 / 有效文件 94 个
- [mercury] MW310R_V5.0_150624.zip — 清单 124 条 / 有效文件 94 个
- [mercury] MW310R_V5.0_160301.zip — 清单 124 条 / 有效文件 94 个
- [mercury] MW326R_V1.0_150626.zip — 清单 124 条 / 有效文件 94 个
- [mercury] MW300R_V15.0_160506.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW305R_V5.0_150929.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW305R_V5.0_151111.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW305R_V7.0_150706.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW305R_V7.0_160121.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW313R_V3.0_151021.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW313R_V3.0_160314.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW313R_V4.0_160602.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW315R_V1.0_160323.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW315R_V1.0_161020.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW315R_V2.0_160616.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW325R_V2.0_160607.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW325R_V3.0_160725.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW325R_V3.0_160926.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW326R_V2.0_160328.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW326R_V2.0_160803.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW351R_V1.0_160329.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW450R_V4.0_150925.zip — 清单 124 条 / 有效文件 92 个
- [mercury] MW54R_V4_V5_V6_V7_100531.rar — 清单 124 条 / 有效文件 38 个
- [mercury] mercury_MW54R_V4V5V6V7_100625_special.bin — 清单 124 条 / 有效文件 38 个
- [mercury] mercury_MW54R_V4V5V6V7_101227_special.bin — 清单 123 条 / 有效文件 38 个
- [mercury] MW548R_V1_091104.rar — 清单 121 条 / 有效文件 0 个
- [fast] 20091028152225.rar — 清单 117 条 / 有效文件 0 个
- [fast] 2009102990137.rar — 清单 117 条 / 有效文件 0 个
- [fast] 2009102990609.rar — 清单 117 条 / 有效文件 0 个
- [mercury] mercury_MR808_v1v2v3_090916_special.bin — 清单 116 条 / 有效文件 0 个
- [mercury] 2009115170928.rar — 清单 116 条 / 有效文件 0 个
- [mercury] 200911695119.rar — 清单 116 条 / 有效文件 0 个
- [mercury] 2009116100221.rar — 清单 116 条 / 有效文件 0 个
- [mercury] MR100P-AC_V1.0_171124.zip — 清单 106 条 / 有效文件 77 个
- [linksys] linksys_WRT54GS2v1_1.0.02.007.bin — 清单 104 条 / 有效文件 0 个
- [mercury] MW310RE_V2.0_170227.zip — 清单 103 条 / 有效文件 73 个
- [mercury] MR804_V9_121225.rar — 清单 103 条 / 有效文件 27 个
- [mercury] MW300RE_V2.0_20161230.zip — 清单 102 条 / 有效文件 72 个
- [linksys] linksys_FW_WRT54G2V1.5_1.5.02.005_US_20120221.bin — 清单 102 条 / 有效文件 10 个
- [linksys] linksys_FW_WRT54G2V1.3_1.3.00.028_US_20081205.bin — 清单 102 条 / 有效文件 0 个
- [mercury] MR804_V9_121023.rar — 清单 101 条 / 有效文件 26 个
- [mercury] MW301RE_V1.0_160421.zip — 清单 99 条 / 有效文件 71 个
- [mercury] MW300R_V10.0_150504.zip — 清单 96 条 / 有效文件 71 个
- [mercury] MW300R_V10.0_150902.zip — 清单 96 条 / 有效文件 71 个
- [mercury] MW305R_V4.0_150428.zip — 清单 96 条 / 有效文件 71 个
- [mercury] MW313R_V2.0_150110.zip — 清单 96 条 / 有效文件 71 个
- [mercury] MW313R_V2.0_150305.zip — 清单 96 条 / 有效文件 71 个
- [mercury] MW313R_V2.0_150428.zip — 清单 96 条 / 有效文件 71 个
- [mercury] MW323R_V1.0_150504.zip — 清单 96 条 / 有效文件 71 个
- [tp-link] TL-WR820N(RU)_V1_190422.zip — 清单 95 条 / 有效文件 4 个
- [linksys] linksys_WRT54Gv5v6_1.02.5.bin — 清单 93 条 / 有效文件 0 个
- [linksys] linksys_WRT54GS-v5v6_1.52.5.002_fw.bin — 清单 93 条 / 有效文件 0 个
- [tp-link] TL-WR820N(EU)_V1_190506.zip — 清单 92 条 / 有效文件 6 个
- [tp-link] TL-WR820N(EU)_V1_180817.zip — 清单 91 条 / 有效文件 5 个
- [mercury] MW310RE_V3.0.zip — 清单 89 条 / 有效文件 61 个
- [tp-link] TL-WR820N(EU)_V1_190905.zip — 清单 89 条 / 有效文件 6 个
- [linksys] linksys_WRT54Gv5v6_1.02.8.bin — 清单 86 条 / 有效文件 0 个
- [linksys] linksys_WRT54Gv7.2_7.2.01.007.bin — 清单 86 条 / 有效文件 0 个
- [linksys] linksys_FW_WRT54GSv5v6_1.52.8.001_US_20091005.bin — 清单 86 条 / 有效文件 0 个
- [linksys] linksys_WRT54G2v1_1.0.01.003.bin — 清单 85 条 / 有效文件 0 个
- [linksys] linksys_WRT54GSv5_v1_50_9.zip — 清单 84 条 / 有效文件 0 个
- [trendnet] trendnet_TEW-453APB_2.3.03.zip — 清单 68 条 / 有效文件 0 个
- [trendnet] trendnet_TEW-453APB_2.0.39_WW.zip — 清单 62 条 / 有效文件 0 个
- [tp-link] TL-WA501G_V1_120727.zip — 清单 57 条 / 有效文件 0 个
- [tp-link] 20104235180311.zip — 清单 57 条 / 有效文件 0 个
- [trendnet] trendnet_TEW-510APB_1.1.21_EU.zip — 清单 51 条 / 有效文件 0 个
- [trendnet] trendnet_TEW-455APBO_v1-8-9_EU.zip — 清单 48 条 / 有效文件 0 个
- [trendnet] trendnet_TEW-455APBO_v1-8-9_A.zip — 清单 48 条 / 有效文件 0 个
- [mercury] MR904_V2.0_101130.zip — 清单 23 条 / 有效文件 0 个
- [fast] 2011824152834.rar — 清单 19 条 / 有效文件 0 个
- [mercury] MR804_V7_100624.rar — 清单 18 条 / 有效文件 1 个
- [mercury] 2010628150457.rar — 清单 18 条 / 有效文件 1 个
- [fast] 2010119143518.rar — 清单 18 条 / 有效文件 0 个
- [mercury] MR804_V8_101220.rar — 清单 18 条 / 有效文件 0 个
- [mercury] MR804_V8_120312.rar — 清单 18 条 / 有效文件 0 个
- [mercury] 2011113135422.rar — 清单 18 条 / 有效文件 0 个
- [mercury] 2011921171332.rar — 清单 18 条 / 有效文件 0 个
- [mercury] 20111215182029.rar — 清单 18 条 / 有效文件 0 个
- [mercury] MR807_V2_101125.rar — 清单 18 条 / 有效文件 0 个
- [mercury] MR807_V2_120312.rar — 清单 18 条 / 有效文件 0 个
- [mercury] 2011113140253.rar — 清单 18 条 / 有效文件 0 个
- [mercury] 2011113140954.rar — 清单 18 条 / 有效文件 0 个
- [mercury] MR914B_V1_100107.rar — 清单 17 条 / 有效文件 1 个
- [mercury] 2009115163851.rar — 清单 17 条 / 有效文件 0 个
- [mercury] 2009115171255.rar — 清单 17 条 / 有效文件 0 个
- [mercury] mercury_MR814_V1_070807.bin — 清单 17 条 / 有效文件 0 个
- [mercury] 200911695340.rar — 清单 17 条 / 有效文件 0 个
- [mercury] mercury_MR824_V2_081020.bin — 清单 17 条 / 有效文件 0 个
- [mercury] MR904_V1_100107.rar — 清单 17 条 / 有效文件 0 个
- [mercury] MR904B_V1_100107.rar — 清单 17 条 / 有效文件 0 个
- [mercury] MR914_V1_100107.rar — 清单 17 条 / 有效文件 0 个
- [tp-link] TL-R410_V1_070703.zip — 清单 17 条 / 有效文件 0 个
- [tp-link] TL_WR641G_V1_200774114608.zip — 清单 17 条 / 有效文件 0 个
- [tp-link] TL_WA501G_V1_2007818085933.zip — 清单 16 条 / 有效文件 0 个
- [tp-link] TL_R402M_V3_2010122118525815.zip — 清单 13 条 / 有效文件 0 个
- [tp-link] TL-R4299G_V1_091225.zip — 清单 10 条 / 有效文件 0 个
- [tp-link] TL-R488T_V2_091225.zip — 清单 10 条 / 有效文件 0 个
- [netgear] a6_WPN824v2_2.0.16.zip — 清单 0 条 / 有效文件 201 个
- [netgear] WGR614v7_FW2.0.30.zip — 清单 0 条 / 有效文件 179 个
- [mercury] MAC1200RE_V2.0.zip — 清单 0 条 / 有效文件 161 个
- [netgear] a6b_2_WGR614v5_20Firmware_20Version_201.07.zip — 清单 0 条 / 有效文件 137 个
- [netgear] WGR614v5_FW1.09.zip — 清单 0 条 / 有效文件 137 个
- [netgear] a6b_1_WGR614v5_20Initial_20Release_20Firmware_20Version_201.03.zip — 清单 0 条 / 有效文件 135 个
- [netgear] a6b_3_WGT624_20and_20WGT624v2_20Firmware_20Version_204.2.10.zip — 清单 0 条 / 有效文件 134 个
- [netgear] WGT624v1_FW4.2.8.zip — 清单 0 条 / 有效文件 134 个
- [netgear] WGT624v2_FW4.2.11.zip — 清单 0 条 / 有效文件 134 个
- [netgear] WGR614v3_FW2.00.zip — 清单 0 条 / 有效文件 113 个
- [belkin] belkin_F5D5230-4-v1.20.001.bin — 清单 0 条 / 有效文件 0 个
- [belkin] belkin_F5D5230u4-v1.20.002-runtime.bin — 清单 0 条 / 有效文件 0 个
- [belkin] belkin_F5D5230u4-v1.20.003-runtime.bin — 清单 0 条 / 有效文件 0 个
- [belkin] belkin_F5D6230-3-v1.20.000.bin — 清单 0 条 / 有效文件 0 个
- [belkin] belkin_F5D6230-3-v1.20.004-runtime.bin — 清单 0 条 / 有效文件 0 个
- [belkin] belkin_F5D6230g3-v1.20.000-runtime.bin — 清单 0 条 / 有效文件 0 个
- [belkin] a6_belkin_F5D5231-4_v1-1.00.00.bin — 清单 0 条 / 有效文件 0 个
- [mercury] mercury_MW108R_V1_070820.bin — 清单 0 条 / 有效文件 0 个
- [mercury] 2009115105714.rar — 清单 0 条 / 有效文件 0 个
- [mercury] 2009115105451.rar — 清单 0 条 / 有效文件 0 个
- [tp-link] TL_WR340GD_V1_2007811080925.zip — 清单 0 条 / 有效文件 0 个
- [tp-link] TL-WR541G_V2_070714.zip — 清单 0 条 / 有效文件 0 个
- [tp-link] TL_WR541G_V4_20071217204439.zip — 清单 0 条 / 有效文件 0 个
- [tp-link] TL_WR543G_V1_2007811082554.zip — 清单 0 条 / 有效文件 0 个
- [tp-link] TL_WR543G_V1_200872391554.zip — 清单 0 条 / 有效文件 0 个
- [tp-link] TL-WR543G_V2_081013.zip — 清单 0 条 / 有效文件 0 个
- [tp-link] 200774112527.zip — 清单 0 条 / 有效文件 0 个

## 文件名 + 内容都恢复（163）

- [fast] 2013125191408.rar — 230 个有效文件
- [fast] 201182201224.rar — 167 个有效文件
- [fast] 2014326163556.zip — 58 个有效文件
- [fast] FW150R__20121212153667.bin — 61 个有效文件
- [fast] FW300R__2012126162529.bin — 67 个有效文件
- [fast] 2014314161063.zip — 59 个有效文件
- [fast] 201615150824.zip — 96 个有效文件
- [linksys] linksys_FW_WRT54G2V1.5_1.5.02.005_US_20120221.bin — 10 个有效文件
- [mercury] MAC1200RE_V2.0.zip — 161 个有效文件
- [mercury] MR100P-AC_V1.0_171124.zip — 77 个有效文件
- [mercury] MR804_V7_100624.rar — 1 个有效文件
- [mercury] MR804_V9_121023.rar — 26 个有效文件
- [mercury] MR804_V9_121225.rar — 27 个有效文件
- [mercury] 2010628150457.rar — 1 个有效文件
- [mercury] MR900_V2_130109.rar — 230 个有效文件
- [mercury] MR900_V3.0_20161126.zip — 235 个有效文件
- [mercury] MR900B_V2_130109.rar — 242 个有效文件
- [mercury] MR900B_V3.0_150521.zip — 247 个有效文件
- [mercury] MR900B_V3.0_161126.zip — 247 个有效文件
- [mercury] MR904_V3_110529.rar — 162 个有效文件
- [mercury] MR904_V3_110714.rar — 168 个有效文件
- [mercury] MR904B_v2_110529.rar — 175 个有效文件
- [mercury] MR904B_v2_110714.rar — 181 个有效文件
- [mercury] MR914B_V1_100107.rar — 1 个有效文件
- [mercury] MW150R_V10.0_140225.rar — 58 个有效文件
- [mercury] MW150R_V5_V6_111219.rar — 66 个有效文件
- [mercury] MW150R_V8_121113.rar — 61 个有效文件
- [mercury] MW150R__201342200320.bin — 63 个有效文件
- [mercury] mercury_MW150R_V5-V6_111231_test.bin — 66 个有效文件
- [mercury] mercury_MW150R_V5-V6_120109_special.bin — 66 个有效文件
- [mercury] mercury_MW150R_V5-V6_120217_special.bin — 69 个有效文件
- [mercury] mercury_MW150R_V8_130226_test.bin — 62 个有效文件
- [mercury] MW150RM_V1_V2_120530.rar — 84 个有效文件
- [mercury] MW150RM_V1_V2_120809.rar — 83 个有效文件
- [mercury] MW1515R_V1_121012.rar — 70 个有效文件
- [mercury] MW151R_V1_100113.rar — 43 个有效文件
- [mercury] MW151R__2010628151219.bin — 43 个有效文件
- [mercury] MW151RM_V1_120425.rar — 84 个有效文件
- [mercury] MW151RM_V1_121107.rar — 83 个有效文件
- [mercury] MW151RM__201223163746.bin — 84 个有效文件
- [mercury] MW153R_V1_110909.rar — 67 个有效文件
- [mercury] MW153R_V1_121127.rar — 61 个有效文件
- [mercury] MW153R__201342200827.bin — 63 个有效文件
- [mercury] MW153R__201342200957.bin — 63 个有效文件
- [mercury] MW155R_V1.0_140310.zip — 61 个有效文件
- [mercury] MW155R_V1_V2_130427.rar — 60 个有效文件
- [mercury] MW300R_V10.0_150504.zip — 71 个有效文件
- [mercury] MW300R_V10.0_150902.zip — 71 个有效文件
- [mercury] MW300R_V12.0_150610.zip — 94 个有效文件
- [mercury] MW300R_V12.0_160309.zip — 94 个有效文件
- [mercury] MW300R_V15.0_160506.zip — 92 个有效文件
- [mercury] MW300R_V4_121126.rar — 68 个有效文件
- [mercury] MW300R_V9.0_140210.rar — 59 个有效文件
- [mercury] MW300R_V9.0_140528.zip — 58 个有效文件
- [mercury] MW300R__201275172121.bin — 68 个有效文件
- [mercury] MW300R__201275173752.bin — 68 个有效文件
- [mercury] MW300R__201216162049.bin — 48 个有效文件
- [mercury] MW300R__201275174246.bin — 48 个有效文件
- [mercury] MW300R__201275174464.bin — 48 个有效文件
- [mercury] MW300RE_V2.0_20161230.zip — 72 个有效文件
- [mercury] MW301RE_V1.0_160421.zip — 71 个有效文件
- [mercury] MW303R_V1_120201.rar — 66 个有效文件
- [mercury] MW303R__201275174751.bin — 66 个有效文件
- [mercury] MW303R__201275174943.bin — 66 个有效文件
- [mercury] MW305R_V3.0_140210.zip — 59 个有效文件
- [mercury] MW305R_V3.0_140528.zip — 59 个有效文件
- [mercury] MW305R_V4.0_150428.zip — 71 个有效文件
- [mercury] MW305R_V5.0_150929.zip — 92 个有效文件
- [mercury] MW305R_V5.0_151111.zip — 92 个有效文件
- [mercury] MW305R_V7.0_150706.zip — 92 个有效文件
- [mercury] MW305R_V7.0_160121.zip — 92 个有效文件
- [mercury] 2014512165923.zip — 59 个有效文件
- [mercury] MW306R_V1.0_140210.zip — 59 个有效文件
- [mercury] MW306R_V1.0_140528.zip — 59 个有效文件
- [mercury] MW309R_V1.0_140219.zip — 59 个有效文件
- [mercury] MW309R_V1.0_140613.zip — 59 个有效文件
- [mercury] MW310R_V1_V2_120201.rar — 66 个有效文件
- [mercury] MW310R_V1_V2_130201.rar — 66 个有效文件
- [mercury] MW310R_V5.0_150624.zip — 94 个有效文件
- [mercury] MW310R_V5.0_160301.zip — 94 个有效文件
- [mercury] mercury_MW310R_V1-V2_120507_special.bin — 66 个有效文件
- [mercury] mercury_MW310R_V1-V2_120508_special.bin — 66 个有效文件
- [mercury] MW310RE_V2.0_170227.zip — 73 个有效文件
- [mercury] MW310RE_V3.0.zip — 61 个有效文件
- [mercury] MW313R_V2.0_150110.zip — 71 个有效文件
- [mercury] MW313R_V2.0_150305.zip — 71 个有效文件
- [mercury] MW313R_V2.0_150428.zip — 71 个有效文件
- [mercury] MW313R_V3.0_151021.zip — 92 个有效文件
- [mercury] MW313R_V3.0_160314.zip — 92 个有效文件
- [mercury] MW313R_V4.0_160602.zip — 92 个有效文件
- [mercury] MW315R_V1.0_160323.zip — 92 个有效文件
- [mercury] MW315R_V1.0_161020.zip — 92 个有效文件
- [mercury] MW315R_V2.0_160616.zip — 92 个有效文件
- [mercury] MW316R_V1.0_141105.zip — 59 个有效文件
- [mercury] MW316R_V1.0_150611.zip — 59 个有效文件
- [mercury] MW316R_V1.0_150817.zip — 59 个有效文件
- [mercury] MW320R_V1.0_141111.zip — 59 个有效文件
- [mercury] MW320R_V1.0_150925.zip — 59 个有效文件
- [mercury] MW323R_V1.0_150504.zip — 71 个有效文件
- [mercury] MW325R_V2.0_160607.zip — 92 个有效文件
- [mercury] MW325R_V3.0_160725.zip — 92 个有效文件
- [mercury] MW325R_V3.0_160926.zip — 92 个有效文件
- [mercury] MW325R_V4.0.zip — 93 个有效文件
- [mercury] MW326R_V1.0_150626.zip — 94 个有效文件
- [mercury] MW326R_V2.0_160328.zip — 92 个有效文件
- [mercury] MW326R_V2.0_160803.zip — 92 个有效文件
- [mercury] MW351R_V1.0_160329.zip — 92 个有效文件
- [mercury] MW450R_V2.0_140225.zip — 58 个有效文件
- [mercury] MW450R_V4.0_150925.zip — 92 个有效文件
- [mercury] MW54R_V10_110311.rar — 48 个有效文件
- [mercury] MW54R_V4_V5_V6_V7_100531.rar — 38 个有效文件
- [mercury] MW54R__201181192837.bin — 48 个有效文件
- [mercury] MW54R__2011111162552.bin — 68 个有效文件
- [mercury] MW54R__2011111163235.bin — 68 个有效文件
- [mercury] mercury_MW54R_V4V5V6V7_100625_special.bin — 38 个有效文件
- [mercury] mercury_MW54R_V4V5V6V7_101227_special.bin — 38 个有效文件
- [netgear] a6b_1_WGR614v5_20Initial_20Release_20Firmware_20Version_201.03.zip — 135 个有效文件
- [netgear] a6b_2_WGR614v5_20Firmware_20Version_201.07.zip — 137 个有效文件
- [netgear] a6b_3_WGT624_20and_20WGT624v2_20Firmware_20Version_204.2.10.zip — 134 个有效文件
- [netgear] WGR614v7_FW2.0.30.zip — 179 个有效文件
- [netgear] WGR614v3_FW2.00.zip — 113 个有效文件
- [netgear] WGT624v1_FW4.2.8.zip — 134 个有效文件
- [netgear] WGT624v2_FW4.2.11.zip — 134 个有效文件
- [netgear] a6_WPN824v2_2.0.16.zip — 201 个有效文件
- [netgear] WGR614v5_FW1.09.zip — 137 个有效文件
- [tp-link] Archer C64(EU)_V1_210219.zip — 283 个有效文件
- [tp-link] tplink_ArcherC80_V1_200628_US.bin — 289 个有效文件
- [tp-link] tplink_ArcherC80_V1_200907_US.bin — 291 个有效文件
- [tp-link] tplink_ArcherC80_V1_201003_US.bin — 291 个有效文件
- [tp-link] tplink_ArcherC80_V1_210128_US.bin — 283 个有效文件
- [tp-link] tplink_ArcherC80_V1_210308_US.bin — 283 个有效文件
- [tp-link] tplink_ArcherC80_V1_210203_CA.bin — 283 个有效文件
- [tp-link] Archer_C80(US)_V1_200221.zip — 286 个有效文件
- [tp-link] Archer_C80_US_V1_191122.zip — 272 个有效文件
- [tp-link] TL-ER5120_V1_120719.zip — 222 个有效文件
- [tp-link] TL-ER5120_V2_140212.zip — 224 个有效文件
- [tp-link] TL-ER6020_V1_120807.zip — 237 个有效文件
- [tp-link] TL-ER604W_V1_130116.zip — 248 个有效文件
- [tp-link] TL-ER604W(EU)_V2_160825_1480562237189w.zip — 261 个有效文件
- [tp-link] TL-ER6120_V1_120719.zip — 237 个有效文件
- [tp-link] TL-R460_v5.0_120704_RU_UA.zip — 44 个有效文件
- [tp-link] TL_WA5110G_V1_2009721151054.zip — 2 个有效文件
- [tp-link] TL-WA5110G_V1_For_EU.zip — 2 个有效文件
- [tp-link] TL-WA5110G_V2_120217.zip — 2 个有效文件
- [tp-link] TL-WR340G_V1_101102.zip — 45 个有效文件
- [tp-link] TL_WR340G_V2_2010102011103318.zip — 45 个有效文件
- [tp-link] ru_2010461450313.zip — 44 个有效文件
- [tp-link] ru_2010462074711.zip — 45 个有效文件
- [tp-link] TL-WR820N(EU)_V1_180817.zip — 5 个有效文件
- [tp-link] TL-WR820N(EU)_V1_190905.zip — 6 个有效文件
- [tp-link] Archer C64(EU)_V1_210317.zip — 285 个有效文件
- [tp-link] Archer C64(EU)_V1_210609.zip — 285 个有效文件
- [tp-link] Archer C80(EU)_V1 200907.zip — 291 个有效文件
- [tp-link] Archer C80(EU)_V1_191204.zip — 274 个有效文件
- [tp-link] Archer C80(EU)_V1_200330.zip — 289 个有效文件
- [tp-link] Archer C80(EU)_V1_200628.zip — 289 个有效文件
- [tp-link] Archer C80(EU)_V1_201003.zip — 291 个有效文件
- [tp-link] Archer C80(EU)_V1_210128.zip — 283 个有效文件
- [tp-link] Archer C80(EU)_V1_210308.zip — 283 个有效文件
- [tp-link] TL-WR820N(EU)_V1_190506.zip — 6 个有效文件
- [tp-link] TL-WR820N(RU)_V1_190422.zip — 4 个有效文件
- [tp-link] TL-WR820N(EU)_V2_191129.zip — 233 个有效文件
- [tp-link] TL-WR820N(EU)_V2_200731.zip — 246 个有效文件

## 完全无产出的样本（10）

- [tektronix] tektronix_TDS210_1.06_fw.bin
- [tektronix] tektronix_TDS210_1.09_fw.bin
- [tektronix] tektronix_TDS210_1.17_fw.bin
- [tektronix] tektronix_TDS210_1.19_fw.bin
- [tektronix] tektronix_TDS210_1.19b_fw.bin
- [tektronix] tektronix_TDS520_v2.16e_fw.bin
- [mercury] MR804_V1_V5_V6_100430.rar
- [mercury] mercury_MR804_V1V5V6_100526_special.bin
- [mercury] mercury_MR804_V1V5V6_100728_test.bin
- [tp-link] TL-R402M_V2_091224.zip
