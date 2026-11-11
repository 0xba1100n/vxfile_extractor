# 三项指标定义与当前结果

测试对象：**308 个已确认 VxWorks 固件**（逐个经 `tools/verdict.py` 判定），对应 `TESTED_MODELS.csv`。

## M1 正常解包（unpack）

**定义**：工具能识别封装并把内嵌内容落盘，产出 **至少 1 个非输入副本的文件**。

- 计数来源：`results.csv` 的 `files_recovered` 列，大于 1 记为通过（等于 1 表示只有输入文件本身）。
- 计入的产物：解压层（LZMA / deflate / gzip / bzip2 / xz）、carve 出的图片与 HTML、表内文件。
- 不计入：`log.txt`、小于 4KB 的碎片（防止碎片灌水）。

**当前：298 / 308 = 97%**

## M2 恢复符号表（symbol table）

**定义**：从固件里提取出 VxWorks 符号表，并且符号名是真实的（不是随机 token）。

- 判定：产出 `SYMBOL_Table` + `SYMBOL_Names.txt`，且 `symbol_names > 0`。
- 支持四种形态：压缩符号表流（props 5A / 5C / 5D / 5E / 6E）、未压缩名字表（4/8 字节 NUL 对齐）、裸 LZMA1、递归解压层。
- 防造假：评测侧要求 **至少 20 个唯一符号名 + 至少 1 个 VxWorks 标记**，随机数据凑 token 会被拦下。

**当前：116 / 308 = 38%**，符号名总计 **824,018 个**

## M3 恢复文件名（filename）

这一项分三档，**弱证据不与强证据混算**：

| 档 | 定义 | 产物 | 当前 |
|---|---|---|---|
| M3a 有表（强） | 解析出结构化文件名表：名字 + size + offset | `MINIFS_Table.csv` | 209 / 308 |
| M3b 仅名字（弱） | 没有表，但从镜像/层里收割到至少 20 个真实文件名 | `NAMES_ONLY.csv` | 27 / 308 |
| M3c 名字+内容 | 名字与内容都恢复，且内容通过魔数/可打印率校验 | `result_vxworks_file/` | 69 / 308 |

- 支持 6 种表布局：MINIFS 定长、MINIFS 名字池、fixed-stride（含 Linksys TROC 0x18、WRT54G2 0x4C）、offsets-only、nameblock、lenprefix。
- 防造假：扩展名白名单 + 按扩展名做内容校验（htm 需以 `<` 开头、png 需 `\x89PNG`、pem 需 `-----BEGIN` 等），输入副本不计。

## 分厂商

| 厂商 | 样本 | M1 解包 | M2 符号表 | M3a 有表 | M3c 名字+内容 |
|---|---|---|---|---|---|
| mercury | 143 | 140 | 38 | 130 | 44 |
| tp-link | 58 | 57 | 20 | 50 | 23 |
| tektronix | 45 | 39 | 31 | 0 | 0 |
| fast | 14 | 14 | 3 | 12 | 2 |
| anritsu | 12 | 12 | 12 | 0 | 0 |
| linksys | 10 | 10 | 7 | 10 | 0 |
| netgear | 9 | 9 | 4 | 0 | 0 |
| belkin | 7 | 7 | 0 | 0 | 0 |
| trendnet | 7 | 7 | 0 | 7 | 0 |
| intellistar | 2 | 2 | 1 | 0 | 0 |
| nortel | 1 | 1 | 0 | 0 | 0 |

## 明细

逐样本结果见 [`TESTED_MODELS.csv`](TESTED_MODELS.csv)：vendor / model / firmware_version / kernel_version / family / file / size_bytes / sha256_16 / verdict / m1_unpack / m2_symbol_table / m2_symbol_names / m3_filename_table / m3_filename_entries / m3_names_only / m3_named_content。
