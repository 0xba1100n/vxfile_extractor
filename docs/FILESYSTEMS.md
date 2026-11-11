# 五类文件系统

本文说明 `vxfile_extractor` 目前支持的 VxWorks 文件系统结构。README 里的表格只列类型和代表型号，具体布局放在这里。

| 族 | 代表型号 | 魔数 / 入口 |
|---|---|---|
| MINIFS v1 | Mercury MW/MR 系列 | `MINIFS` + 首个 `5A 00 00 80` |
| MINIFS v2 | Mercury MAC1200RE | `MINIFS`，版本字段 = 2 |
| MINIFS v3 | TP-Link Archer C80 / C64 | `MINIFS`，版本字段 ≥ 3，带 ToN/ToF/ToC |
| OW | Mercury MR900 系列 | LZMA 流内的 32 字节 `OW` 魔数 |
| PK11 | Netgear WGR614 / WGT624 等 CHK | `PK11` |
| TROC（旧） | Linksys WRT54G v5/v6 | `TROC` |

## MINIFS v1

Mercury 早期机型（MW150R、MW300R、MR900 之前的版本）用的布局：

```
+0x00  char magic[16]      "MINIFS" + 0x00 填充
+0x10  u32  flag           观察值多为 1
+0x14  file_info[]         每个 0x58 字节
         +0x00  u32 size   (BE)
         +0x04  u32 offset (BE)
         +0x08  char name[0x50]
```

数据区从第一个 `5A 00 00 80` 开始，**每个文件是一条独立的 LZMA-alone 流**，表里的 `offset` 相对这个起点；一条流的结束位置就是下一条流的起点。

工具按 offset 排序后逐段解压，遇到尾部多余字节时保留已解出的部分。

## MINIFS v2

MAC1200RE 这一代，头部字段变成：

```
+0x00  char magic[16]      "MINIFS"
+0x10  u32  version        2
+0x14  u32  file_count
+0x18  u32  name_blob_len
+0x1C  u32  data_len
```

之后是「记录 + 流」交替排列，重复 `file_count` 次：

```
record 32 B:  hash[16] | comp_len(u32 BE) | name_off(u32 BE) | decomp_len(u32 BE) | 0
stream     :  comp_len 字节的 LZMA-alone 流
```

名字区被哈希化，镜像里没有明文文件名，因此只能按内容给出 `v2_000_xxxx.png` 这类编号名。

## MINIFS v3

Archer C80 / C64 这一代，三张表 + 分块压缩：

```
header 32 B:  magic[16] | version | file_count | ton_size
ToN : 名字池
ToF : 每条 20 B = path_off | name_off | chunk_no | offset_in_chunk | file_size
ToC : 每条 12 B = chunk_offset | chunk_size | decompressed_size
raw : 4 字节配置字 5D 00 00 80，之后是各 chunk 的 LZMA 流
```

同一 chunk 内的文件共用一次解压结果，按 `offset_in_chunk` 切分。

## OW

MR900 系列。外层是 LZMA 流，解压后是一个以 32 字节 `OW` 开头的小文件系统：

```
+0x00  "OWOW..." ×16      32 字节
+0x20  u32 version
+0x24  u32 entry_count
+0x28  record[]           每条 76 B
         +0x00  char name[60]
         +0x3C  u32 0
         +0x40  u32 size
         +0x44  u32 offset   相对本 blob 起点
         +0x48  u32 0
```

## PK11

Netgear CHK 解压层里的目录表：

```
"PK11"
[name\0][u32 LE size] ...   重复到非文件名为止
数据按记录顺序紧跟在表后
```

## TROC（旧格式）

Linksys WRT54G v5/v6 的布局，仅作兼容保留：

```
+0x00  char name[13]
+0x0D  u32  size   (LE)
+0x11  u32  offset (LE)
stride 0x18
```
