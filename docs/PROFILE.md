# 逐偏移画像（vx profile）

仿 binwalk 的 `-E`（熵图）和签名扫描，但把结果落成「每块的熵 + 类型 + 合并区间」。

## 用法

```
vx profile <firmware> [--block 1024] [-o blocks.csv] [--ascii]
```

- `--block`：窗口大小，默认 1024 字节
- `-o`：把每块的 `offset,size,entropy,kind` 写成 CSV
- `--ascii`：额外打一行 80 列熵图

## 输出

```
$ vx profile samples/expand-a/inner/MW300R__201275172121.bin --block 1024 --ascii
{"file": "...", "size": 1309124, "block": 1024, "regions": 7}
0x00000000-0x00000400       1024 B  ent  7.43  data
0x00000400-0x00101000    1051648 B  ent  7.81  high-entropy
0x00101000-0x00103000       8192 B  ent  3.94  data
0x00103000-0x0012C000     167936 B  ent  7.79  high-entropy
0x0012C000-0x0012C400       1024 B  ent  7.82  lzma
0x0012C400-0x0013F800      78848 B  ent  7.80  high-entropy
0x0013F800-0x0013F9C4        452 B  ent  7.46  data
kind                  bytes  regions   share
high-entropy        1298432        3   99.2%
data                   9668        3    0.7%
lzma                   1024        1    0.1%
[entropy 0..8] @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@#@@@@@@@@@@@@@@@
```

## 类型判定

每个窗口只看**窗口起点和窗口内容**，按以下顺序命中的第一个：

| kind | 判据 |
|---|---|
| `zero` | 整窗全 0 |
| `uimage` | `27 05 19 56` |
| `trx` | `HDR0` |
| `minifs` / `pk11` / `ow` | 对应魔数 |
| `lzma` | 首字节 0x5D/0x5A/0x6E/0x5C/0x5E 且字典是 2 的幂且在 4 KiB–256 MiB |
| `gzip` / `zlib` / `bzip2` / `xz` / `7z` / `rar` / `zip` | 对应魔数 |
| `elf` / `pe` | `7f454c46` / `MZ` |
| `png` / `jpeg` / `gif` | 对应魔数 |
| `vxworks` | 窗口内含 `WIND version` 或 `vxWorks` |
| `high-entropy` | H ≥ 7.5 |
| `text` | 可打印字符占比 > 85% |
| `padding` | H ≤ 1.0 |
| `data` | 其余 |

## 两个例子

C80（TP-Link，MINIFS）：

```
0x00156000-0x00156400       1024 B  ent  4.95  minifs
0x00156400-0x00157000       3072 B  ent  4.69  text
0x00158C00-0x0027E800    1203200 B  ent  7.81  high-entropy
```

能清楚看到 MINIFS 表在 0x156000，后面 1.2 MB 是压缩的文件数据。

anritsu（SH4 内核，无文件系统）：

```
0x00000000-0x0044E800    4515840 B  ent  5.79  data
0x00451800-0x00451C00       1024 B  ent  5.20  vxworks
0x00453800-0x00453C00       1024 B  ent  4.90  vxworks
```

整片是代码（熵 4–6），`WIND version` 字符串落在 0x451800 附近——和 `raw_kernel` 判定对得上。

## 和 binwalk 的差别

| | binwalk `-E` | `vx profile` |
|---|---|---|
| 熵 | 有，出图 | 有，CSV + ASCII |
| 逐偏移定性 | 只有签名命中 | 每块都有 kind |
| 区间合并 | 无 | 相邻同类合并成区间 |
| 类型汇总 | 无 | 按 kind 统计字节和占比 |