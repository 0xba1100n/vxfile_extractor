# 输出说明

跑完一个固件之后，工具会在 `vxfile_<名字>/` 下留这些东西。这份文档只讲"产物怎么读"，不讲原理。

## 跑完之后屏幕上是什么

以 Archer C80 为例，末尾会打这么一段：

```
[+] Filesystem unpacked (MINIFS, layer 0x158910): 297 files
[+] Filename table (vxtable/minifs-pool): 297 entries -> vxfile_Archer_C80_US_V1_191122\MINIFS_Table.csv
[+] SUMMARY.json: layers=45, symbol=SYMBOL_Table, named=297, valid=265
[+] Symbol name table: vxfile_Archer_C80_US_V1_191122\SYMBOL_Table
[+] Output layout
    bin/   8 file(s)  <- executable / firmware image candidates
    src/   285 file(s)  <- static resources (js 166, html 81, css 13, json 9, xml 6)
    cfg/   4 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer @0x158910 (3947396 B)
    main HTTP/CGI code lives inside this image, not as a separate file
[*] Other binary candidates:
    result_vxworks_file\fw\mtk\WIFI_RAM_CODE_iemi.bin    raw    393836 B
[*] Extracted tree: vxfile_Archer_C80_US_V1_191122\...extracted
```

## 目录结构

```
vxfile_<固件名>/
├─ SYMBOL_Table              符号名原始数据（NUL 分隔，没有地址）
├─ SYMBOL_Names.txt          一行一个符号名，方便 grep
├─ MINIFS_Table.csv          name,size,offset：文件偏移表
├─ NAMES_ONLY.csv            只有名字、没有偏移时的清单
├─ SUMMARY.json              层数、版本、恢复数量、输入 sha256
├─ result_vxworks_file/
│  ├─ bin/                   可执行体 / 固件镜像候选
│  ├─ src/                   静态资源（网页、脚本、样式、图片）
│  ├─ cfg/                   证书与配置
│  └─ ...                    按原始路径恢复的文件（原名）
└─ <固件>.extracted/
   ├─ 0x158910                binwalk 解出的各层，文件名是偏移
   └─ ...                     原样保留，方便回查
```

`bin/`、`src/`、`cfg/` 是**硬链接**（同盘），不额外占空间；跨盘时退化成复制。

## 三个目录怎么分的

| 目录 | 判定依据 | 例子 |
|---|---|---|
| `bin/` | ELF `7f454c46` / PE `MZ` 魔数、`.bin/.out/.elf/.img/.rom` 扩展名、含 `vxWorks`/`WIND version`/`symTbl` 特征串 | `WIFI_RAM_CODE_iemi.bin` |
| `src/` | `.htm .html .js .css .xml .json .png .gif .jpg .ico .svg .woff .ttf` | `web/common/Index.htm` |
| `cfg/` | `.pem .cer .crt .key .conf .cfg .ini .txt` | `conf/server-cert.pem` |

其余（`.dat` 之类）留在 `result_vxworks_file/` 根下，不强行归类。

## "主逻辑在哪个文件"

这类固件的主逻辑**通常不在独立文件里，而在内核镜像层**。所以提示优先给内核层：

```
[*] Likely main logic: kernel image layer @0x158910 (3947396 B)
```

`@0x158910` 是这一层在输入镜像里的偏移，`<固件>.extracted/158910` 就是它。

找候选的顺序：

1. ELF / PE 可执行体
2. 文件名里带 `httpd`、`goahead`、`webs`、`boa`、`cgi`
3. 体积最大的 `raw` 镜像

名字里带 `wifi`、`ram_code`、`mcu`、`patch`、`dat` 的会被降权——它们通常是 WiFi 固件或配置，不是主逻辑。

## 文件名恢复的三种情况

| 情况 | 表现 | 能不能自动改名 |
|---|---|---|
| 有名字 + 有偏移 | 按原路径落盘，例如 `web/common/Advance.htm` | ✅ 自动 |
| 只有名字、没有偏移 | 只出 `NAMES_ONLY.csv`，内容对不上 | ❌ |
| 名字被哈希/加密 | 按内容编号，例如 `v2_000_000CE420.png` | ❌ |

目前 308 个样本里，能自动改名的 236 个（77%），其中名字和内容都对上的 163 个（53%）。

## SYMBOL_Table 是什么，不是什么

它是**符号名串**：一串 NUL 分隔的函数名，没有地址。

```
intStubSelect\0excNoFpu\0excReturn\0nested\0save_critical\0...
```

所以它不能直接给出「函数在哪个地址」。要地址得另做：定位 VxWorks 的 `symTbl` 结构（名字指针 + 值指针的成对表），或者从调用点反推。这件事目前没做。

符号名的用途仍然是有的：`grep` 函数名确认某个库在不在、判断版本、给反汇编器做函数名提示。

## SUMMARY.json

| 字段 | 含义 |
|---|---|
| `input` / `size` / `source_sha256` | 输入文件和指纹 |
| `layers` | 解出的层数 |
| `layer_detail[]` | 每层偏移和 binwalk 描述 |
| `vxworks_versions[]` | 识别到的 VxWorks 版本（5.4.2 / 5.5.1 / 6.6） |
| `symbol_table` | 符号表文件名，没有就是 null |
| `symbol_names` | 符号名个数 |
| `recovered_named` | 按名字恢复的文件数 |
| `recovered_valid` | 其中内容校验通过的 |
| `batch` | 耗时、是否命中缓存、是否触到层数上限 |

## Rust 版的输出

`vxfile_extractor extract fw.bin -o out/` 的布局略有不同，因为 Rust 版不跑 binwalk：

```
out/
├─ minifs/        MINIFS v1/v3 解出的文件（保留原路径）
├─ files/         PK11（Netgear）解出的文件
├─ streams/       LZMA 流兜底解出的文件（stream_<偏移>.<类型>）
├─ SUMMARY.json
└─ <偏移>         各解压层
```

同样的英文提示会打到 **stderr**，stdout 只留 JSON，方便管道：

```bash
vxfile_extractor extract fw.bin -o out/ 2>/dev/null | jq .verdict
```

## 常见问题

**`src/` 是空的？** 这个固件没有网页资源，或者网页在加密分区里。看 `SUMMARY.json` 的 `recovered_named`。

**`bin/` 里几百个文件？** 说明把解压层也算进去了。`vxfile_extractor` 会自动排除 `00158910_lzma-alone` 这类层转储。

**符号表有，但没地址？** 正常，见上面那节。

**文件名是 `v2_000_...`？** 那是 MiniFS v2，名字在镜像里被哈希化了，只能按内容编号。

## 如果固件本身就是内核

有些升级包没有容器，文件本身就是 VxWorks 镜像（或者只套了 20 字节头）。这时不会去"解包"，而是写一个 `kernel.bin`：

```
[+] Payload unpacked (arm-kernel-with-20b-header): 1 file(s)
[+] Output layout
    bin/   1 file(s)  <- executable / firmware image candidates
[*] Likely main logic: raw kernel image (kernel.bin, header stripped)
```

| 类别 | 判定 | 输出 |
|---|---|---|
| `arm-kernel-with-20b-header` | 前 4 字节 BE 等于 `len-20`，且偏移 0x14 处是合法 ARM 指令 | 剥掉 20 字节头后的 `kernel.bin` |
| `flat-vxworks-kernel` | 含 `WIND version`，且没有已知文件系统 | 原样的 `kernel.bin` |

这两种情况本来就没有文件系统可解，所以不计入「解包率」，但状态会从 `unknown` 变成有明确结论。
