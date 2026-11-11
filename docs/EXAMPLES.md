# 输出案例

vxfile_extractor extract 的 stderr 提示：20 个解开、20 个解不开。

- [+] Output layout：产物分类
- [*] Likely main logic：主逻辑所在（内核层 / 裸内核 / 独立可执行体）
- [!] high-entropy 0x起-0x止 (大小, 熵)：高熵区，判决为 compressed (data was recovered) 或 likely encrypted / unknown container

## 解开的 20 个

### samples/expand-e/anritsu_MS2024A_V1.66_fw.zip

厂商 anritsu，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-e/anritsu_MS2024A_V1.66_fw.zip
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

### samples/expand-e/anritsu_MS2026B_V2.01_fw.zip

厂商 anritsu，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-e/anritsu_MS2026B_V2.01_fw.zip
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

### samples/expand-e/anritsu_MS2034A_V1.21_fw.zip

厂商 anritsu，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-e/anritsu_MS2034A_V1.21_fw.zip
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

### samples/expand-b/belkin_F5D5230-4-v1.20.001.bin

厂商 belkin，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-b/belkin_F5D5230-4-v1.20.001.bin
[!] high-entropy 0x00004000-0x0009277C (569 KiB, 98.8%) -> compressed (data was recovered)
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AC5 (1848212 B)
```

### samples/expand-b/belkin_F5D5230u4-v1.20.002-runtime.bin

厂商 belkin，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-b/belkin_F5D5230u4-v1.20.002-runtime.bin
[!] high-entropy 0x00004000-0x000973B0 (588 KiB, 98.8%) -> compressed (data was recovered)
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AC5 (1938104 B)
```

### samples/expand-b/belkin_F5D5230u4-v1.20.003-runtime.bin

厂商 belkin，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-b/belkin_F5D5230u4-v1.20.003-runtime.bin
[!] high-entropy 0x00004000-0x00099000 (596 KiB, 98.8%) -> compressed (data was recovered)
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AC5 (1962248 B)
```

### samples/expand-g/trendnet_TEW-453APB_2.0.39_WW.zip

厂商 trendnet，覆盖率 93%

```
$ vxfile_extractor extract samples/expand-g/trendnet_TEW-453APB_2.0.39_WW.zip
[!] high-entropy 0x00005000-0x00110000 (1068 KiB, 98.8%) -> compressed (data was recovered)
[+] Output layout
[*] Likely main logic: kernel image layer 0x4D05 (2741508 B)
```

### samples/expand-g/trendnet_TEW-453APB_2.3.03.zip

厂商 trendnet，覆盖率 91.3%

```
$ vxfile_extractor extract samples/expand-g/trendnet_TEW-453APB_2.3.03.zip
[!] high-entropy 0x00005000-0x00113000 (1080 KiB, 98.9%) -> compressed (data was recovered)
[+] Output layout
[*] Likely main logic: kernel image layer 0x4D05 (2868184 B)
```

### samples/expand-g/trendnet_TEW-510APB_1.1.21_EU.zip

厂商 trendnet，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-g/trendnet_TEW-510APB_1.1.21_EU.zip
[!] high-entropy 0x00005000-0x000D7748 (841 KiB, 98.8%) -> compressed (data was recovered)
[+] Output layout
[*] Likely main logic: kernel image layer 0x4CA5 (2041528 B)
```

### samples/expand-d/tektronix_TDS210_1.06_fw.bin

厂商 tektronix，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-d/tektronix_TDS210_1.06_fw.bin
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

### samples/expand-d/tektronix_TDS210_1.09_fw.bin

厂商 tektronix，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-d/tektronix_TDS210_1.09_fw.bin
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

### samples/expand-d/tektronix_TDS210_1.17_fw.bin

厂商 tektronix，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-d/tektronix_TDS210_1.17_fw.bin
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

### samples/fast/2011824152834.rar

厂商 fast，覆盖率 100%

```
$ vxfile_extractor extract samples/fast/2011824152834.rar
[!] high-entropy 0x00001000-0x00057DD4 (347 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
[*] Likely main logic: kernel image layer 0x165C (902576 B)
```

### samples/expand-a/20091028152225.rar

厂商 fast，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-a/20091028152225.rar
[!] high-entropy 0x00004000-0x000C7214 (780 KiB, 98.9%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
```

### samples/expand-a/2009102990137.rar

厂商 fast，覆盖率 100%

```
$ vxfile_extractor extract samples/expand-a/2009102990137.rar
[!] high-entropy 0x00001000-0x00092954 (582 KiB, 99.4%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
```

### samples/other/intellistar_thunderstorm_vxworks.bin

厂商 intellistar，覆盖率 100%

```
$ vxfile_extractor extract samples/other/intellistar_thunderstorm_vxworks.bin
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

### samples/jp/linksys_WRT54Gv5v6_1.02.5.bin

厂商 linksys，覆盖率 100%

```
$ vxfile_extractor extract samples/jp/linksys_WRT54Gv5v6_1.02.5.bin
[!] high-entropy 0x0001A000-0x0010D000 (972 KiB, 99.4%) -> compressed (data was recovered)
[!] high-entropy 0x00119000-0x00198724 (509 KiB, 99.1%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

### samples/jp/linksys_WRT54Gv5v6_1.02.8.bin

厂商 linksys，覆盖率 100%

```
$ vxfile_extractor extract samples/jp/linksys_WRT54Gv5v6_1.02.8.bin
[!] high-entropy 0x0001A000-0x00116000 (1008 KiB, 99.4%) -> compressed (data was recovered)
[!] high-entropy 0x00153000-0x001AFDC8 (371 KiB, 99.2%) -> compressed (data was recovered)
[!] high-entropy 0x00121000-0x0014D000 (176 KiB, 98.3%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

### samples/jp/linksys_WRT54Gv7.2_7.2.01.007.bin

厂商 linksys，覆盖率 100%

```
$ vxfile_extractor extract samples/jp/linksys_WRT54Gv7.2_7.2.01.007.bin
[!] high-entropy 0x0001A000-0x00115000 (1004 KiB, 99.4%) -> compressed (data was recovered)
[!] high-entropy 0x00153000-0x001AFAB4 (370 KiB, 99.2%) -> compressed (data was recovered)
[!] high-entropy 0x00121000-0x00142000 (132 KiB, 99.0%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
```

## 解不开的 20 个

### samples/expand-b/belkin_F5D6230-3-v1.20.000.bin

厂商 belkin，覆盖率 5.8%

```
$ vxfile_extractor extract samples/expand-b/belkin_F5D6230-3-v1.20.000.bin
[!] high-entropy 0x00004000-0x000A5000 (644 KiB, 98.8%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AD1 (2324184 B)
```

### samples/expand-g/trendnet_TEW-455APBO_v1-8-9_EU.zip

厂商 trendnet，覆盖率 0.7%

```
$ vxfile_extractor extract samples/expand-g/trendnet_TEW-455APBO_v1-8-9_EU.zip
[!] high-entropy 0x00005000-0x000FE000 (996 KiB, 98.8%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x4D71 (2474968 B)
```

### samples/expand-g/trendnet_TEW-455APBO_v1-8-9_A.zip

厂商 trendnet，覆盖率 0.7%

```
$ vxfile_extractor extract samples/expand-g/trendnet_TEW-455APBO_v1-8-9_A.zip
[!] high-entropy 0x00005000-0x000FE000 (996 KiB, 98.8%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x4D71 (2474968 B)
```

### samples/expand-a/2010119143518.rar

厂商 fast，覆盖率 5.9%

```
$ vxfile_extractor extract samples/expand-a/2010119143518.rar
[!] high-entropy 0x00004000-0x0013CC74 (1251 KiB, 98.9%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x3809 (3528608 B)
```

### samples/expand-a/2014326163556.zip

厂商 fast，覆盖率 0.9%

```
$ vxfile_extractor extract samples/expand-a/2014326163556.zip
[!] high-entropy 0x00012000-0x000D3904 (774 KiB, 99.4%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x0 (866564 B)
```

### samples/expand-a/inner/FW150R__20121212153667.bin

厂商 fast，覆盖率 12.9%

```
$ vxfile_extractor extract samples/expand-a/inner/FW150R__20121212153667.bin
[!] high-entropy 0x00042000-0x00106D54 (787 KiB, 99.3%) -> likely encrypted / unknown container
[!] high-entropy 0x00006000-0x0002C000 (152 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x0 (1076564 B)
```

### samples/expand-a/inner/FW300R__2012126162529.bin

厂商 fast，覆盖率 0.2%

```
$ vxfile_extractor extract samples/expand-a/inner/FW300R__2012126162529.bin
[!] high-entropy 0x00040000-0x00141000 (1028 KiB, 99.4%) -> likely encrypted / unknown container
[!] high-entropy 0x00143000-0x001807E4 (245 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x40114 (3968960 B)
```

### samples/expand-a/2014314161063.zip

厂商 fast，覆盖率 0.8%

```
$ vxfile_extractor extract samples/expand-a/2014314161063.zip
[!] high-entropy 0x00011000-0x000D0A24 (766 KiB, 99.4%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x0 (854564 B)
```

### samples/other/intellistar_thunderstorm_bootrom.bin

厂商 intellistar，覆盖率 4.7%

```
$ vxfile_extractor extract samples/other/intellistar_thunderstorm_bootrom.bin
[!] high-entropy 0x00004000-0x0002AB00 (154 KiB, 98.9%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x417D (378880 B)
```

### samples/mercury/MR804_V9_121023.rar

厂商 mercury，覆盖率 26.2%

```
$ vxfile_extractor extract samples/mercury/MR804_V9_121023.rar
[!] high-entropy 0x00020000-0x00078000 (352 KiB, 99.4%) -> likely encrypted / unknown container
[!] high-entropy 0x00001000-0x0001F000 (120 KiB, 98.8%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x0 (493508 B)
```

### samples/mercury/MR804_V9_121225.rar

厂商 mercury，覆盖率 14.9%

```
$ vxfile_extractor extract samples/mercury/MR804_V9_121225.rar
[!] high-entropy 0x00020000-0x00077000 (348 KiB, 99.4%) -> likely encrypted / unknown container
[!] high-entropy 0x00001000-0x0001F000 (120 KiB, 99.0%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x0 (491076 B)
```

### samples/mercury/MR904_V2.0_101130.zip

厂商 mercury，覆盖率 6.3%

```
$ vxfile_extractor extract samples/mercury/MR904_V2.0_101130.zip
[!] high-entropy 0x00003000-0x00131934 (1210 KiB, 99.1%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x3361 (3059072 B)
```

### samples/mercury/MW150R_V10.0_140225.rar

厂商 mercury，覆盖率 0.4%

```
$ vxfile_extractor extract samples/mercury/MW150R_V10.0_140225.rar
[!] high-entropy 0x00012000-0x000D2000 (768 KiB, 99.4%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x0 (860948 B)
```

### samples/mercury/MW150R_V5_V6_111219.rar

厂商 mercury，覆盖率 0.2%

```
$ vxfile_extractor extract samples/mercury/MW150R_V5_V6_111219.rar
[!] high-entropy 0x00040000-0x00135000 (980 KiB, 99.4%) -> likely encrypted / unknown container
[!] high-entropy 0x00137000-0x00173CF4 (243 KiB, 99.3%) -> likely encrypted / unknown container
[!] high-entropy 0x00007000-0x00037000 (192 KiB, 99.4%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x40114 (3138464 B)
```

### samples/mercury/MW150R_V8_121113.rar

厂商 mercury，覆盖率 12.1%

```
$ vxfile_extractor extract samples/mercury/MW150R_V8_121113.rar
[!] high-entropy 0x00042000-0x00104F74 (779 KiB, 99.4%) -> likely encrypted / unknown container
[!] high-entropy 0x00006000-0x0002B000 (148 KiB, 99.4%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x0 (1068916 B)
```

### samples/tp-link/TL-R460_v5.0_120704_RU_UA.zip

厂商 tp-link，覆盖率 2.2%

```
$ vxfile_extractor extract samples/tp-link/TL-R460_v5.0_120704_RU_UA.zip
[!] high-entropy 0x00001000-0x0007C000 (492 KiB, 99.3%) -> likely encrypted / unknown container
[!] high-entropy 0x0007E000-0x000A8C28 (171 KiB, 99.1%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x16C4 (1788992 B)
```

### samples/tp-link/TL-WA5110G_V1_For_EU.zip

厂商 tp-link，覆盖率 3%

```
$ vxfile_extractor extract samples/tp-link/TL-WA5110G_V1_For_EU.zip
[!] high-entropy 0x00005000-0x00102000 (1012 KiB, 98.9%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x54B5 (2895216 B)
```

### samples/tp-link/TL-WA5210G_V1_130502.zip

厂商 tp-link，覆盖率 3.2%

```
$ vxfile_extractor extract samples/tp-link/TL-WA5210G_V1_130502.zip
[!] high-entropy 0x00005000-0x0010A000 (1044 KiB, 98.9%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x54B5 (2996192 B)
```

### samples/tp-link/TL-WR340G_V1_101102.zip

厂商 tp-link，覆盖率 2.3%

```
$ vxfile_extractor extract samples/tp-link/TL-WR340G_V1_101102.zip
[!] high-entropy 0x00004000-0x000AD000 (676 KiB, 99.4%) -> likely encrypted / unknown container
[!] high-entropy 0x000B0000-0x000DE4FC (185 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
[*] Likely main logic: kernel image layer 0x3A14 (2337200 B)
```

