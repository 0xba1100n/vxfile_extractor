# 输出案例

解开的 100 个、解不开的全部 75 个，逐条附 extract 实际输出和一句解读。

## 解开的 100 个

### anritsu_MS2024A_V1.66_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### belkin_F5D5230-4-v1.20.001.bin（belkin）

覆盖率 100.0%，高熵占比 96.4%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AC5 (1848212 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### trendnet_TEW-453APB_2.0.39_WW.zip（trendnet）

覆盖率 93.0%，高熵占比 96.5%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x4D05 (2741508 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 93.0%），无独立文件系统。

### tektronix_TDS210_1.06_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.0%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 2011824152834.rar（fast）

覆盖率 100.0%，高熵占比 98.3%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x165C (902576 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### intellistar_thunderstorm_vxworks.bin（intellistar）

覆盖率 100.0%，高熵占比 0.3%。

```
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### linksys_WRT54Gv5v6_1.02.5.bin（linksys）

覆盖率 100.0%，高熵占比 90.0%。

```
[!] high-entropy 0x00119000-0x00198724 (509 KiB, 99.1%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### MAC1200RE_V2.0.zip（mercury）

覆盖率 95.5%，高熵占比 99.7%。

```
[+] Output layout
    bin/   14 file(s)  <- executable / firmware image candidates
    src/   160 file(s)  <- static resources (png 80, html 50, js 26, gif 4)
    cfg/   2 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x400 (2872752 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 95.5%），无独立文件系统。

### a6b_1_WGR614v5_20Initial_20Release_20Firmware_20Version_201.03.zip（netgear）

覆盖率 100.0%，高熵占比 100.0%。

```
[+] Output layout
    src/   144 file(s)  <- static resources (htm 124, gif 9, js 5, xml 4, css 2)
[*] Likely main logic: kernel image layer 0x56 (3332312 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：PK11 解出 144 个文件，覆盖率 100.0%。

### nortel_contivity1500_partial.img（nortel）

覆盖率 100.0%，高熵占比 0.0%。

```
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 3 条流，覆盖率 100.0%。

### Archer C64(EU)_V1_210219.zip（tp-link）

覆盖率 100.0%，高熵占比 99.5%。

```
[!] high-entropy 0x00000000-0x0015C000 (1392 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   40 file(s)  <- executable / firmware image candidates
    src/   296 file(s)  <- static resources (js 172, html 77, css 20, json 8, xml 8)
    cfg/   3 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x15FF94 (5603340 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：MINIFS 解出 338 个文件，覆盖率 100.0%。

### anritsu_MS2026B_V2.01_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### belkin_F5D5230u4-v1.20.002-runtime.bin（belkin）

覆盖率 100.0%，高熵占比 96.2%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AC5 (1938104 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### trendnet_TEW-453APB_2.3.03.zip（trendnet）

覆盖率 91.3%，高熵占比 96.3%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x4D05 (2868184 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 91.3%），无独立文件系统。

### tektronix_TDS210_1.09_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.0%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 20091028152225.rar（fast）

覆盖率 100.0%，高熵占比 97.9%。

```
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### linksys_WRT54Gv5v6_1.02.8.bin（linksys）

覆盖率 100.0%，高熵占比 88.5%。

```
[!] high-entropy 0x00153000-0x001AFDC8 (371 KiB, 99.2%) -> compressed (data was recovered)
[!] high-entropy 0x00121000-0x0014D000 (176 KiB, 98.3%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### MR804_V1_V5_V6_100430.rar（mercury）

覆盖率 100.0%，高熵占比 99.2%。

```
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### a6b_2_WGR614v5_20Firmware_20Version_201.07.zip（netgear）

覆盖率 100.0%，高熵占比 100.0%。

```
[+] Output layout
    src/   146 file(s)  <- static resources (htm 126, gif 9, js 5, xml 4, css 2)
[*] Likely main logic: kernel image layer 0x56 (3367373 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：PK11 解出 146 个文件，覆盖率 100.0%。

### tplink_ArcherC80_V1_200628_US.bin（tp-link）

覆盖率 100.0%，高熵占比 99.4%。

```
[!] high-entropy 0x00000000-0x00158000 (1376 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   30 file(s)  <- executable / firmware image candidates
    src/   301 file(s)  <- static resources (js 178, html 84, css 15, json 8, xml 6)
    cfg/   3 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x15BE78 (5172851 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：MINIFS 解出 333 个文件，覆盖率 100.0%。

### anritsu_MS2034A_V1.21_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### belkin_F5D5230u4-v1.20.003-runtime.bin（belkin）

覆盖率 100.0%，高熵占比 96.4%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AC5 (1962248 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### trendnet_TEW-510APB_1.1.21_EU.zip（trendnet）

覆盖率 100.0%，高熵占比 96.4%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x4CA5 (2041528 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS210_1.17_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.2%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 2009102990137.rar（fast）

覆盖率 100.0%，高熵占比 99.3%。

```
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### linksys_WRT54Gv7.2_7.2.01.007.bin（linksys）

覆盖率 100.0%，高熵占比 88.6%。

```
[!] high-entropy 0x00153000-0x001AFAB4 (370 KiB, 99.2%) -> compressed (data was recovered)
[!] high-entropy 0x00121000-0x00142000 (132 KiB, 99.0%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### MR804_V7_100624.rar（mercury）

覆盖率 100.0%，高熵占比 96.5%。

```
[+] Output layout
    src/   1 file(s)  <- static resources (html 1)
[*] Likely main logic: kernel image layer 0x2E0C (930528 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### a6b_3_WGT624_20and_20WGT624v2_20Firmware_20Version_204.2.10.zip（netgear）

覆盖率 100.0%，高熵占比 100.0%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   143 file(s)  <- static resources (htm 123, gif 8, js 5, xml 4, css 2)
[*] Likely main logic: kernel image layer 0x54 (4733068 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：PK11 解出 143 个文件，覆盖率 100.0%。

### tplink_ArcherC80_V1_200907_US.bin（tp-link）

覆盖率 100.0%，高熵占比 99.3%。

```
[!] high-entropy 0x00000000-0x00155000 (1364 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   30 file(s)  <- executable / firmware image candidates
    src/   304 file(s)  <- static resources (js 178, html 84, css 16, json 8, xml 8)
    cfg/   3 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x158EC8 (5197580 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：MINIFS 解出 336 个文件，覆盖率 100.0%。

### anritsu_MS2717B_V4.04_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### belkin_F5D6230-3-v1.20.004-runtime.bin（belkin）

覆盖率 100.0%，高熵占比 94.4%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AD1 (2426476 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### trendnet_TEW-511BRP_1.0.19_US.zip（trendnet）

覆盖率 100.0%，高熵占比 97.7%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x4CE5 (2783700 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS210_1.19_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.0%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 2009102990609.rar（fast）

覆盖率 100.0%，高熵占比 99.3%。

```
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### linksys_WRT54G2v1_1.0.01.003.bin（linksys）

覆盖率 100.0%，高熵占比 88.6%。

```
[!] high-entropy 0x00156000-0x001B25B0 (369 KiB, 99.2%) -> compressed (data was recovered)
[!] high-entropy 0x00126000-0x0013E000 (96 KiB, 99.3%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### MR804_V8_101220.rar（mercury）

覆盖率 100.0%，高熵占比 98.3%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x165C (902576 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### WGR614v7_FW2.0.30.zip（netgear）

覆盖率 100.0%，高熵占比 100.0%。

```
[+] Output layout
    src/   190 file(s)  <- static resources (htm 167, gif 10, js 5, xml 4, css 3)
[*] Likely main logic: kernel image layer 0x54 (3634832 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：PK11 解出 190 个文件，覆盖率 100.0%。

### tplink_ArcherC80_V1_201003_US.bin（tp-link）

覆盖率 100.0%，高熵占比 99.3%。

```
[!] high-entropy 0x00000000-0x00155000 (1364 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   30 file(s)  <- executable / firmware image candidates
    src/   304 file(s)  <- static resources (js 178, html 84, css 16, json 8, xml 8)
    cfg/   3 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x158EC8 (5197628 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：MINIFS 解出 336 个文件，覆盖率 100.0%。

### anritsu_MS2720T_V4.80_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### belkin_F5D6230g3-v1.20.000-runtime.bin（belkin）

覆盖率 100.0%，高熵占比 94.6%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AD1 (2507604 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### trendnet_TEW-511BRP_1.0.20_EU.zip（trendnet）

覆盖率 100.0%，高熵占比 97.8%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x4CE5 (2956000 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS210_1.19b_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.0%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 2013117155732.rar（fast）

覆盖率 100.0%，高熵占比 71.5%。

```
[!] high-entropy 0x00160000-0x001A7000 (284 KiB, 99.3%) -> compressed (data was recovered)
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x33E0 (4303616 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### linksys_FW_WRT54G2V1.3_1.3.00.028_US_20081205.bin（linksys）

覆盖率 100.0%，高熵占比 96.1%。

```
[!] high-entropy 0x0014D000-0x001A4BF8 (350 KiB, 99.2%) -> compressed (data was recovered)
[!] high-entropy 0x00118000-0x00147000 (188 KiB, 98.8%) -> compressed (data was recovered)
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x138A5C (93406 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### MR804_V8_120312.rar（mercury）

覆盖率 100.0%，高熵占比 98.4%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x165C (993136 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### WGR614v3_FW2.00.zip（netgear）

覆盖率 100.0%，高熵占比 100.0%。

```
[+] Output layout
    src/   120 file(s)  <- static resources (htm 102, gif 7, js 5, xml 4, css 2)
[*] Likely main logic: kernel image layer 0x54 (3898976 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：PK11 解出 120 个文件，覆盖率 100.0%。

### tplink_ArcherC80_V1_210128_US.bin（tp-link）

覆盖率 100.0%，高熵占比 99.4%。

```
[!] high-entropy 0x00000000-0x0015C000 (1392 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   37 file(s)  <- executable / firmware image candidates
    src/   296 file(s)  <- static resources (js 172, html 77, css 20, json 8, xml 8)
    cfg/   3 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x15FF1C (5569906 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：MINIFS 解出 335 个文件，覆盖率 100.0%。

### anritsu_MS2721B_V1.51_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### a6_belkin_F5D5231-4_v1-1.00.00.bin（belkin）

覆盖率 100.0%，高熵占比 99.8%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x51 (2968344 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS510A_1.0e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 1.1%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 2013724172437.rar（fast）

覆盖率 100.0%，高熵占比 71.8%。

```
[!] high-entropy 0x00160000-0x001A5000 (276 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x33E0 (4335216 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### linksys_FW_WRT54G2V1.5_1.5.02.005_US_20120221.bin（linksys）

覆盖率 100.0%，高熵占比 96.6%。

```
[!] high-entropy 0x00156000-0x0019EFF8 (291 KiB, 99.4%) -> compressed (data was recovered)
[!] high-entropy 0x00121000-0x0014C000 (172 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   10 file(s)  <- static resources (js 10)
[*] Likely main logic: kernel image layer 0x142334 (93308 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 111 条流，覆盖率 100.0%。

### 2010628150457.rar（mercury）

覆盖率 100.0%，高熵占比 96.6%。

```
[+] Output layout
    src/   1 file(s)  <- static resources (html 1)
[*] Likely main logic: kernel image layer 0x2E0C (945104 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### WGT624v1_FW4.2.8.zip（netgear）

覆盖率 100.0%，高熵占比 100.0%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   143 file(s)  <- static resources (htm 123, gif 8, js 5, xml 4, css 2)
[*] Likely main logic: kernel image layer 0x54 (4734908 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：PK11 解出 143 个文件，覆盖率 100.0%。

### tplink_ArcherC80_V1_210308_US.bin（tp-link）

覆盖率 100.0%，高熵占比 99.4%。

```
[!] high-entropy 0x00174000-0x002DD000 (1444 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   37 file(s)  <- executable / firmware image candidates
    src/   296 file(s)  <- static resources (js 172, html 77, css 20, json 8, xml 8)
    cfg/   3 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x173F1C (5572834 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：MINIFS 解出 335 个文件，覆盖率 100.0%。

### anritsu_MS2723B_V1.39_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS520_v2.16e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 2.5%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 201615150824.zip（fast）

覆盖率 100.0%，高熵占比 95.0%。

```
[!] high-entropy 0x00092000-0x000D8000 (280 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   5 file(s)  <- executable / firmware image candidates
    src/   127 file(s)  <- static resources (htm 74, png 25, js 13, xml 6, css 4)
    cfg/   2 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0xF114 (1620336 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：MINIFS 解出 131 个文件，覆盖率 100.0%。

### linksys_FW_WRT54GSv5v6_1.52.8.001_US_20091005.bin（linksys）

覆盖率 100.0%，高熵占比 88.3%。

```
[!] high-entropy 0x00153000-0x001AFFA8 (371 KiB, 99.2%) -> compressed (data was recovered)
[!] high-entropy 0x00121000-0x0013A000 (100 KiB, 99.0%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 2011113135422.rar（mercury）

覆盖率 100.0%，高熵占比 96.6%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x2DEC (953088 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### WGT624v2_FW4.2.11.zip（netgear）

覆盖率 100.0%，高熵占比 100.0%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   143 file(s)  <- static resources (htm 123, gif 8, js 5, xml 4, css 2)
[*] Likely main logic: kernel image layer 0x54 (4736412 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：PK11 解出 143 个文件，覆盖率 100.0%。

### tplink_ArcherC80_V1_210203_CA.bin（tp-link）

覆盖率 100.0%，高熵占比 99.4%。

```
[!] high-entropy 0x00000000-0x0015C000 (1392 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   37 file(s)  <- executable / firmware image candidates
    src/   296 file(s)  <- static resources (js 172, html 77, css 20, json 8, xml 8)
    cfg/   3 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x15FF1C (5569606 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：MINIFS 解出 335 个文件，覆盖率 100.0%。

### anritsu_MS272xC_V2.30_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS520A_v3.8.7e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 1.0%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### linksys_WRT54GS-v5v6_1.52.5.002_fw.bin（linksys）

覆盖率 100.0%，高熵占比 89.8%。

```
[!] high-entropy 0x00119000-0x001987DC (509 KiB, 99.1%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 2011921171332.rar（mercury）

覆盖率 100.0%，高熵占比 98.3%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x165C (903904 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### a6_WPN824v2_2.0.16.zip（netgear）

覆盖率 100.0%，高熵占比 100.0%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   252 file(s)  <- static resources (htm 182, gif 51, js 10, css 5, xml 4)
[*] Likely main logic: kernel image layer 0x54 (4253292 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：PK11 解出 252 个文件，覆盖率 100.0%。

### Archer_C80(US)_V1_200221.zip（tp-link）

覆盖率 100.0%，高熵占比 99.3%。

```
```

解读：MINIFS 解出 328 个文件，覆盖率 100.0%。

### anritsu_MT8222A_V1.80_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS520C_v5.0e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.1%。

```
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 2 条流，覆盖率 100.0%。

### linksys_WRT54GSv5_v1_50_9.zip（linksys）

覆盖率 100.0%，高熵占比 87.9%。

```
[!] high-entropy 0x0012B000-0x0017C164 (324 KiB, 98.9%) -> compressed (data was recovered)
[!] high-entropy 0x00113000-0x0012A000 (92 KiB, 99.1%) -> compressed (data was recovered)
[*] Likely main logic: raw kernel image (kernel.bin, flat-vxworks-kernel)
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 20111215182029.rar（mercury）

覆盖率 100.0%，高熵占比 98.2%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x165C (920304 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### WGR614v5_FW1.09.zip（netgear）

覆盖率 100.0%，高熵占比 100.0%。

```
[+] Output layout
    src/   146 file(s)  <- static resources (htm 126, gif 9, js 5, xml 4, css 2)
[*] Likely main logic: kernel image layer 0x56 (3367772 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：PK11 解出 146 个文件，覆盖率 100.0%。

### Archer_C80_US_V1_191122.zip（tp-link）

覆盖率 100.0%，高熵占比 99.3%。

```
[!] high-entropy 0x00159000-0x0027E000 (1172 KiB, 99.4%) -> compressed (data was recovered)
[+] Output layout
    bin/   11 file(s)  <- executable / firmware image candidates
    src/   284 file(s)  <- static resources (js 166, html 81, css 13, json 8, xml 6)
    cfg/   3 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x158910 (3947396 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：MINIFS 解出 297 个文件，覆盖率 100.0%。

### anritsu_MW82119A_V2.30_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS520C_v5.1.1e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.1%。

```
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 2 条流，覆盖率 100.0%。

### linksys_WRT54GS2v1_1.0.02.007.bin（linksys）

覆盖率 100.0%，高熵占比 95.9%。

```
[!] high-entropy 0x00156000-0x001AE3C0 (352 KiB, 99.2%) -> compressed (data was recovered)
[!] high-entropy 0x0011E000-0x0014B000 (180 KiB, 99.3%) -> compressed (data was recovered)
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x1329FC (52946 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### mercury_MR804_V1V5V6_100526_special.bin（mercury）

覆盖率 100.0%，高熵占比 99.1%。

```
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### TL-R402M_V2_091224.zip（tp-link）

覆盖率 100.0%，高熵占比 99.2%。

```
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### anritsu_MW82119B_V4.80_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS520C_5.3e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.1%。

```
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 2 条流，覆盖率 100.0%。

### mercury_MR804_V1V5V6_100728_test.bin（mercury）

覆盖率 100.0%，高熵占比 99.2%。

```
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### TL_R402M_V3_2010122118525815.zip（tp-link）

覆盖率 100.0%，高熵占比 97.1%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x2BFC (959584 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### anritsu_S412E_V4.81_fw.zip（anritsu）

覆盖率 100.0%，高熵占比 0.0%。

```
    bin/   1 file(s)  <- executable / firmware image candidates
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS524A_v3.8.7e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 1.3%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### MR807_V2_101125.rar（mercury）

覆盖率 100.0%，高熵占比 98.3%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x16EC (886192 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### TL-R410_V1_070703.zip（tp-link）

覆盖率 100.0%，高熵占比 97.5%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3611 (1894992 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS540_v2.16e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 2.5%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### MR807_V2_120312.rar（mercury）

覆盖率 100.0%，高熵占比 98.2%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x16EC (993136 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### TL-R4299G_V1_091225.zip（tp-link）

覆盖率 100.0%，高熵占比 97.5%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3809 (3717824 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS540A_v3.8.3e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.9%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 2011113140253.rar（mercury）

覆盖率 100.0%，高熵占比 98.6%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x13EC (936160 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### TL-R488T_V2_091225.zip（tp-link）

覆盖率 100.0%，高熵占比 97.6%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3809 (3705968 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### tektronix_TDS540B_v4.1e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 1.3%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### 2011113140954.rar（mercury）

覆盖率 100.0%，高熵占比 98.2%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x16EC (920288 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### TL_R860_V1_201112513445417.zip（tp-link）

覆盖率 100.0%，高熵占比 99.3%。

```
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### tektronix_TDS540C_v5.2e_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 0.0%。

```
    header stripped, no container left to unpack
```

解读：LZMA 流兜底解出 2 条流，覆盖率 100.0%。

### 2009115163851.rar（mercury）

覆盖率 100.0%，高熵占比 97.7%。

```
[*] Likely main logic: raw kernel image (kernel.bin, arm-kernel-with-20b-header)
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

### TL_WA501G_V1_2007818085933.zip（tp-link）

覆盖率 100.0%，高熵占比 97.0%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x54B5 (2432960 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：LZMA 流兜底解出 1 条流，覆盖率 100.0%。

### tektronix_TDS544A_v3.8.4e_eprom_fw.bin（tektronix）

覆盖率 100.0%，高熵占比 1.2%。

```
    header stripped, no container left to unpack
```

解读：内核镜像整片解出（覆盖率 100.0%），无独立文件系统。

## 解不开的全部 75 个

### belkin_F5D6230-3-v1.20.000.bin（belkin）

覆盖率 5.8%，高熵占比 94.6%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3AD1 (2324184 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（94.6%），覆盖率只有 5.8%，判定为加密或未识别容器。

### trendnet_TEW-455APBO_v1-8-9_EU.zip（trendnet）

覆盖率 0.7%，高熵占比 96.0%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x4D71 (2474968 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.0%），覆盖率只有 0.7%，判定为加密或未识别容器。

### trendnet_TEW-455APBO_v1-8-9_A.zip（trendnet）

覆盖率 0.7%，高熵占比 96.0%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x4D71 (2474968 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.0%），覆盖率只有 0.7%，判定为加密或未识别容器。

### 2010119143518.rar（fast）

覆盖率 5.9%，高熵占比 97.3%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3809 (3528608 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.3%），覆盖率只有 5.9%，判定为加密或未识别容器。

### 2014326163556.zip（fast）

覆盖率 0.9%，高熵占比 95.5%。

```
[+] Output layout
    bin/   3 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (866564 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（95.5%），覆盖率只有 0.9%，判定为加密或未识别容器。

### FW150R__20121212153667.bin（fast）

覆盖率 12.9%，高熵占比 89.1%。

```
[!] high-entropy 0x00006000-0x0002C000 (152 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   4 file(s)  <- executable / firmware image candidates
    src/   62 file(s)  <- static resources (html 59, js 3)
[*] Likely main logic: kernel image layer 0x0 (1076564 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（89.1%），覆盖率只有 12.9%，判定为加密或未识别容器。

### FW300R__2012126162529.bin（fast）

覆盖率 0.2%，高熵占比 85.2%。

```
[!] high-entropy 0x00143000-0x001807E4 (245 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   67 file(s)  <- static resources (html 64, js 3)
[*] Likely main logic: kernel image layer 0x40114 (3968960 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（85.2%），覆盖率只有 0.2%，判定为加密或未识别容器。

### 2014314161063.zip（fast）

覆盖率 0.8%，高熵占比 96.2%。

```
[+] Output layout
    bin/   4 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (854564 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.2%），覆盖率只有 0.8%，判定为加密或未识别容器。

### intellistar_thunderstorm_bootrom.bin（intellistar）

覆盖率 4.7%，高熵占比 90.0%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: kernel image layer 0x417D (378880 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（90.0%），覆盖率只有 4.7%，判定为加密或未识别容器。

### MR804_V9_121023.rar（mercury）

覆盖率 26.2%，高熵占比 97.7%。

```
[!] high-entropy 0x00001000-0x0001F000 (120 KiB, 98.8%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   27 file(s)  <- static resources (html 24, js 3)
[*] Likely main logic: kernel image layer 0x0 (493508 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.7%），覆盖率只有 26.2%，判定为加密或未识别容器。

### MR804_V9_121225.rar（mercury）

覆盖率 14.9%，高熵占比 97.6%。

```
[!] high-entropy 0x00001000-0x0001F000 (120 KiB, 99.0%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   27 file(s)  <- static resources (html 24, js 3)
[*] Likely main logic: kernel image layer 0x0 (491076 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.6%），覆盖率只有 14.9%，判定为加密或未识别容器。

### MR904_V2.0_101130.zip（mercury）

覆盖率 6.3%，高熵占比 97.8%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x3361 (3059072 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.8%），覆盖率只有 6.3%，判定为加密或未识别容器。

### MW150R_V10.0_140225.rar（mercury）

覆盖率 0.4%，高熵占比 95.3%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (860948 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（95.3%），覆盖率只有 0.4%，判定为加密或未识别容器。

### MW150R_V5_V6_111219.rar（mercury）

覆盖率 0.2%，高熵占比 95.1%。

```
[!] high-entropy 0x00137000-0x00173CF4 (243 KiB, 99.3%) -> likely encrypted / unknown container
[!] high-entropy 0x00007000-0x00037000 (192 KiB, 99.4%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x40114 (3138464 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（95.1%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW150R_V8_121113.rar（mercury）

覆盖率 12.1%，高熵占比 88.9%。

```
[!] high-entropy 0x00006000-0x0002B000 (148 KiB, 99.4%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   62 file(s)  <- static resources (html 59, js 3)
[*] Likely main logic: kernel image layer 0x0 (1068916 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（88.9%），覆盖率只有 12.1%，判定为加密或未识别容器。

### MW150R__201342200320.bin（mercury）

覆盖率 1.7%，高熵占比 89.2%。

```
[!] high-entropy 0x00006000-0x0002C000 (152 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   62 file(s)  <- static resources (html 59, js 3)
[*] Likely main logic: kernel image layer 0x0 (1074164 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（89.2%），覆盖率只有 1.7%，判定为加密或未识别容器。

### mercury_MW150R_V5-V6_111231_test.bin（mercury）

覆盖率 0.4%，高熵占比 99.2%。

```
[!] high-entropy 0x000F7000-0x00133D34 (243 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3138384 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.4%，判定为加密或未识别容器。

### mercury_MW150R_V5-V6_120109_special.bin（mercury）

覆盖率 0.1%，高熵占比 99.2%。

```
[!] high-entropy 0x000F7000-0x00133D14 (243 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3144592 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.1%，判定为加密或未识别容器。

### mercury_MW150R_V5-V6_120217_special.bin（mercury）

覆盖率 0.1%，高熵占比 99.3%。

```
[!] high-entropy 0x000F8000-0x001362F4 (248 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   68 file(s)  <- static resources (html 66, js 2)
[*] Likely main logic: kernel image layer 0x94 (3152304 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.3%），覆盖率只有 0.1%，判定为加密或未识别容器。

### mercury_MW150R_V8_130226_test.bin（mercury）

覆盖率 1.7%，高熵占比 89.2%。

```
[!] high-entropy 0x00006000-0x0002C000 (152 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   62 file(s)  <- static resources (html 59, js 3)
[*] Likely main logic: kernel image layer 0x0 (1074148 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（89.2%），覆盖率只有 1.7%，判定为加密或未识别容器。

### MW150RM_V1_V2_120530.rar（mercury）

覆盖率 0.1%，高熵占比 99.1%。

```
[!] high-entropy 0x000EF000-0x0013C844 (310 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    src/   82 file(s)  <- static resources (html 79, js 3)
[*] Likely main logic: kernel image layer 0x94 (3579264 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.1%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW150RM_V1_V2_120809.rar（mercury）

覆盖率 0.1%，高熵占比 94.9%。

```
[!] high-entropy 0x00130000-0x0017D4F4 (309 KiB, 99.2%) -> likely encrypted / unknown container
[!] high-entropy 0x00007000-0x00037000 (192 KiB, 99.4%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   82 file(s)  <- static resources (html 79, js 3)
[*] Likely main logic: kernel image layer 0x40114 (3556240 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（94.9%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW1515R_V1_121012.rar（mercury）

覆盖率 5.8%，高熵占比 98.2%。

```
[!] high-entropy 0x00002000-0x00044000 (264 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   70 file(s)  <- static resources (html 67, js 3)
[*] Likely main logic: kernel image layer 0x0 (980564 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（98.2%），覆盖率只有 5.8%，判定为加密或未识别容器。

### MW151R_V1_100113.rar（mercury）

覆盖率 0.1%，高熵占比 94.8%。

```
[!] high-entropy 0x0011A000-0x00143000 (164 KiB, 99.2%) -> likely encrypted / unknown container
[!] high-entropy 0x00008000-0x0001B000 (76 KiB, 99.0%) -> likely encrypted / unknown container
[+] Output layout
    bin/   3 file(s)  <- executable / firmware image candidates
    src/   43 file(s)  <- static resources (html 41, js 2)
[*] Likely main logic: kernel image layer 0x20038 (2833824 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（94.8%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW151R__2010628151219.bin（mercury）

覆盖率 0.4%，高熵占比 94.7%。

```
[!] high-entropy 0x0011B000-0x00144000 (164 KiB, 99.3%) -> likely encrypted / unknown container
[!] high-entropy 0x00008000-0x0001B000 (76 KiB, 99.0%) -> likely encrypted / unknown container
[+] Output layout
    bin/   3 file(s)  <- executable / firmware image candidates
    src/   43 file(s)  <- static resources (html 41, js 2)
[*] Likely main logic: kernel image layer 0x20038 (2841040 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（94.7%），覆盖率只有 0.4%，判定为加密或未识别容器。

### MW151RM_V1_120425.rar（mercury）

覆盖率 0.1%，高熵占比 99.0%。

```
[!] high-entropy 0x000ED000-0x0013AF94 (311 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    src/   82 file(s)  <- static resources (html 79, js 3)
[*] Likely main logic: kernel image layer 0x94 (3557616 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.0%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW151RM_V1_121107.rar（mercury）

覆盖率 0.1%，高熵占比 94.9%。

```
[!] high-entropy 0x00130000-0x0017D894 (310 KiB, 99.3%) -> likely encrypted / unknown container
[!] high-entropy 0x00007000-0x00037000 (192 KiB, 99.4%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   82 file(s)  <- static resources (html 79, js 3)
[*] Likely main logic: kernel image layer 0x40114 (3559504 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（94.9%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW151RM__201223163746.bin（mercury）

覆盖率 0.2%，高熵占比 99.1%。

```
[!] high-entropy 0x000F8000-0x00145AE4 (310 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    src/   82 file(s)  <- static resources (html 79, js 3)
[*] Likely main logic: kernel image layer 0x94 (3713232 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.1%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW153R_V1_110909.rar（mercury）

覆盖率 0.1%，高熵占比 99.1%。

```
[!] high-entropy 0x000FB000-0x00138364 (244 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   67 file(s)  <- static resources (html 65, js 2)
[*] Likely main logic: kernel image layer 0x94 (3720944 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.1%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW153R_V1_121127.rar（mercury）

覆盖率 12.2%，高熵占比 88.9%。

```
[!] high-entropy 0x00006000-0x0002C000 (152 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   62 file(s)  <- static resources (html 59, js 3)
[*] Likely main logic: kernel image layer 0x0 (1069828 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（88.9%），覆盖率只有 12.2%，判定为加密或未识别容器。

### MW153R__201342200827.bin（mercury）

覆盖率 1.8%，高熵占比 89.2%。

```
[!] high-entropy 0x00006000-0x0002C000 (152 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   62 file(s)  <- static resources (html 59, js 3)
[*] Likely main logic: kernel image layer 0x0 (1074340 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（89.2%），覆盖率只有 1.8%，判定为加密或未识别容器。

### MW153R__201342200957.bin（mercury）

覆盖率 1.8%，高熵占比 89.2%。

```
[!] high-entropy 0x00006000-0x0002C000 (152 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   62 file(s)  <- static resources (html 59, js 3)
[*] Likely main logic: kernel image layer 0x0 (1074244 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（89.2%），覆盖率只有 1.8%，判定为加密或未识别容器。

### MW155R_V1.0_140310.zip（mercury）

覆盖率 0.2%，高熵占比 95.3%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   61 file(s)  <- static resources (html 59, js 2)
[*] Likely main logic: kernel image layer 0x0 (866036 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（95.3%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW155R_V1_V2_130427.rar（mercury）

覆盖率 0.8%，高熵占比 95.2%。

```
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   60 file(s)  <- static resources (html 57, js 3)
[*] Likely main logic: kernel image layer 0x0 (846276 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（95.2%），覆盖率只有 0.8%，判定为加密或未识别容器。

### MW300R_V4_121126.rar（mercury）

覆盖率 0.1%，高熵占比 85.2%。

```
[!] high-entropy 0x00143000-0x0017F9D4 (242 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x40114 (3968960 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（85.2%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW300R_V9.0_140210.rar（mercury）

覆盖率 0.2%，高熵占比 96.0%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (848964 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.0%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW300R_V9.0_140528.zip（mercury）

覆盖率 0.2%，高熵占比 96.1%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (849748 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.1%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW300R__201275172121.bin（mercury）

覆盖率 0.1%，高熵占比 99.2%。

```
[!] high-entropy 0x00103000-0x0013F9C4 (242 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3967968 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW300R__201275173752.bin（mercury）

覆盖率 0.3%，高熵占比 99.2%。

```
[!] high-entropy 0x00103000-0x0013F854 (242 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3967936 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.3%，判定为加密或未识别容器。

### MW300R__201216162049.bin（mercury）

覆盖率 0.1%，高熵占比 99.2%。

```
[!] high-entropy 0x00103000-0x001339F4 (194 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   48 file(s)  <- static resources (html 46, js 2)
[*] Likely main logic: kernel image layer 0x94 (3975728 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW300R__201275174246.bin（mercury）

覆盖率 0.1%，高熵占比 99.3%。

```
[!] high-entropy 0x00103000-0x001333E4 (192 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   48 file(s)  <- static resources (html 46, js 2)
[*] Likely main logic: kernel image layer 0x94 (3967968 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.3%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW300R__201275174464.bin（mercury）

覆盖率 0.1%，高熵占比 99.3%。

```
[!] high-entropy 0x00103000-0x00133274 (192 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   48 file(s)  <- static resources (html 46, js 2)
[*] Likely main logic: kernel image layer 0x94 (3967936 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.3%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW303R_V1_120201.rar（mercury）

覆盖率 0.1%，高熵占比 99.2%。

```
[!] high-entropy 0x00103000-0x0013F404 (241 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3962544 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW303R__201275174751.bin（mercury）

覆盖率 0.1%，高熵占比 99.2%。

```
[!] high-entropy 0x00103000-0x0013FA14 (242 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3967968 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW303R__201275174943.bin（mercury）

覆盖率 0.1%，高熵占比 98.8%。

```
[!] high-entropy 0x00103000-0x0013F8A4 (242 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3967936 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（98.8%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW305R_V3.0_140210.zip（mercury）

覆盖率 0.2%，高熵占比 96.1%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (848964 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.1%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW305R_V3.0_140528.zip（mercury）

覆盖率 0.2%，高熵占比 96.0%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (849764 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.0%），覆盖率只有 0.2%，判定为加密或未识别容器。

### 2014512165923.zip（mercury）

覆盖率 0.2%，高熵占比 96.1%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (848964 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.1%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW306R_V1.0_140210.zip（mercury）

覆盖率 0.2%，高熵占比 96.1%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (848964 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.1%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW306R_V1.0_140528.zip（mercury）

覆盖率 0.2%，高熵占比 96.0%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (849764 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.0%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW309R_V1.0_140219.zip（mercury）

覆盖率 0.2%，高熵占比 96.1%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (849508 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.1%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW309R_V1.0_140613.zip（mercury）

覆盖率 0.2%，高熵占比 96.1%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (850276 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.1%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW310R_V1_V2_120201.rar（mercury）

覆盖率 0.1%，高熵占比 99.2%。

```
[!] high-entropy 0x00103000-0x0013F414 (241 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3962544 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW310R_V1_V2_130201.rar（mercury）

覆盖率 0.1%，高熵占比 85.3%。

```
[!] high-entropy 0x00144000-0x001808D4 (242 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x40114 (3915808 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（85.3%），覆盖率只有 0.1%，判定为加密或未识别容器。

### mercury_MW310R_V1-V2_120507_special.bin（mercury）

覆盖率 0.1%，高熵占比 99.2%。

```
[!] high-entropy 0x00103000-0x0013FA14 (242 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3967968 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.1%，判定为加密或未识别容器。

### mercury_MW310R_V1-V2_120508_special.bin（mercury）

覆盖率 0.1%，高熵占比 98.8%。

```
[!] high-entropy 0x00103000-0x0013F8A4 (242 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   66 file(s)  <- static resources (html 64, js 2)
[*] Likely main logic: kernel image layer 0x94 (3967936 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（98.8%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW316R_V1.0_141105.zip（mercury）

覆盖率 0.2%，高熵占比 96.2%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (851860 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.2%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW320R_V1.0_141111.zip（mercury）

覆盖率 0.2%，高熵占比 96.1%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x0 (852052 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（96.1%），覆盖率只有 0.2%，判定为加密或未识别容器。

### MW450R_V2.0_140225.zip（mercury）

覆盖率 0.1%，高熵占比 97.1%。

```
[+] Output layout
    bin/   2 file(s)  <- executable / firmware image candidates
    src/   58 file(s)  <- static resources (html 56, js 2)
[*] Likely main logic: kernel image layer 0x15822 (587109 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.1%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW54R_V10_110311.rar（mercury）

覆盖率 0.1%，高熵占比 99.1%。

```
[!] high-entropy 0x000F9000-0x00129000 (192 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   47 file(s)  <- static resources (html 45, js 2)
[*] Likely main logic: kernel image layer 0x94 (3692592 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.1%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW54R_V4_V5_V6_V7_100531.rar（mercury）

覆盖率 0.7%，高熵占比 97.1%。

```
[!] high-entropy 0x000AE000-0x000D5A5C (158 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   3 file(s)  <- executable / firmware image candidates
    src/   38 file(s)  <- static resources (html 36, js 2)
[*] Likely main logic: kernel image layer 0x3A14 (2313872 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.1%），覆盖率只有 0.7%，判定为加密或未识别容器。

### MW54R__201181192837.bin（mercury）

覆盖率 0.1%，高熵占比 99.2%。

```
[!] high-entropy 0x000FA000-0x0012BBE4 (198 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   48 file(s)  <- static resources (html 46, js 2)
[*] Likely main logic: kernel image layer 0x94 (3721024 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.2%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW54R__2011111162552.bin（mercury）

覆盖率 0.1%，高熵占比 99.0%。

```
[!] high-entropy 0x000FF000-0x0013D000 (248 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   67 file(s)  <- static resources (html 65, js 2)
[*] Likely main logic: kernel image layer 0x94 (3800848 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.0%），覆盖率只有 0.1%，判定为加密或未识别容器。

### MW54R__2011111163235.bin（mercury）

覆盖率 0.1%，高熵占比 99.0%。

```
[!] high-entropy 0x000FF000-0x0013D000 (248 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
    src/   67 file(s)  <- static resources (html 65, js 2)
[*] Likely main logic: kernel image layer 0x94 (3800848 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（99.0%），覆盖率只有 0.1%，判定为加密或未识别容器。

### mercury_MW54R_V4V5V6V7_100625_special.bin（mercury）

覆盖率 0.6%，高熵占比 97.1%。

```
[!] high-entropy 0x000AF000-0x000D641C (157 KiB, 99.1%) -> likely encrypted / unknown container
[+] Output layout
    bin/   3 file(s)  <- executable / firmware image candidates
    src/   38 file(s)  <- static resources (html 36, js 2)
[*] Likely main logic: kernel image layer 0x3A14 (2320528 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.1%），覆盖率只有 0.6%，判定为加密或未识别容器。

### mercury_MW54R_V4V5V6V7_101227_special.bin（mercury）

覆盖率 0.6%，高熵占比 97.2%。

```
[!] high-entropy 0x000B1000-0x000D8000 (156 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   3 file(s)  <- executable / firmware image candidates
    src/   38 file(s)  <- static resources (html 36, js 2)
[*] Likely main logic: kernel image layer 0x3A14 (2346480 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.2%），覆盖率只有 0.6%，判定为加密或未识别容器。

### TL-R460_v5.0_120704_RU_UA.zip（tp-link）

覆盖率 2.2%，高熵占比 97.6%。

```
[!] high-entropy 0x0007E000-0x000A8C28 (171 KiB, 99.1%) -> likely encrypted / unknown container
[+] Output layout
    bin/   4 file(s)  <- executable / firmware image candidates
    src/   43 file(s)  <- static resources (html 39, js 4)
[*] Likely main logic: kernel image layer 0x16C4 (1788992 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.6%），覆盖率只有 2.2%，判定为加密或未识别容器。

### TL-WA5110G_V1_For_EU.zip（tp-link）

覆盖率 3.0%，高熵占比 97.2%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x54B5 (2895216 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.2%），覆盖率只有 3.0%，判定为加密或未识别容器。

### TL-WA5210G_V1_130502.zip（tp-link）

覆盖率 3.2%，高熵占比 97.2%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x54B5 (2996192 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.2%），覆盖率只有 3.2%，判定为加密或未识别容器。

### TL-WR340G_V1_101102.zip（tp-link）

覆盖率 2.3%，高熵占比 97.0%。

```
[!] high-entropy 0x000B0000-0x000DE4FC (185 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   4 file(s)  <- executable / firmware image candidates
    src/   44 file(s)  <- static resources (html 40, js 4)
[*] Likely main logic: kernel image layer 0x3A14 (2337200 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.0%），覆盖率只有 2.3%，判定为加密或未识别容器。

### TL_WR340G_V2_2010102011103318.zip（tp-link）

覆盖率 2.6%，高熵占比 97.2%。

```
[!] high-entropy 0x000B0000-0x000DE3EC (184 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   4 file(s)  <- executable / firmware image candidates
    src/   44 file(s)  <- static resources (html 40, js 4)
[*] Likely main logic: kernel image layer 0x3A14 (2337040 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.2%），覆盖率只有 2.6%，判定为加密或未识别容器。

### TL_WR340GD_V1_2007811080925.zip（tp-link）

覆盖率 7.2%，高熵占比 97.0%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x54B5 (2562000 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.0%），覆盖率只有 7.2%，判定为加密或未识别容器。

### ru_2010461450313.zip（tp-link）

覆盖率 2.3%，高熵占比 97.1%。

```
[!] high-entropy 0x000AE000-0x000DAD9C (179 KiB, 99.3%) -> likely encrypted / unknown container
[+] Output layout
    bin/   4 file(s)  <- executable / firmware image candidates
    src/   42 file(s)  <- static resources (html 38, js 4)
[*] Likely main logic: kernel image layer 0x39C4 (2300384 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.1%），覆盖率只有 2.3%，判定为加密或未识别容器。

### TL_WR541G_V3_200911516304613.zip（tp-link）

覆盖率 2.5%，高熵占比 97.2%。

```
[+] Output layout
[*] Likely main logic: kernel image layer 0x54B5 (3007632 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.2%），覆盖率只有 2.5%，判定为加密或未识别容器。

### ru_2010462074711.zip（tp-link）

覆盖率 2.2%，高熵占比 97.1%。

```
[!] high-entropy 0x000B3000-0x000E1530 (185 KiB, 99.2%) -> likely encrypted / unknown container
[+] Output layout
    bin/   4 file(s)  <- executable / firmware image candidates
    src/   43 file(s)  <- static resources (html 39, js 4)
[*] Likely main logic: kernel image layer 0x39C4 (2379184 B)
    main HTTP/CGI code lives inside this image, not as a separate file
```

解读：整块高熵（97.1%），覆盖率只有 2.2%，判定为加密或未识别容器。
