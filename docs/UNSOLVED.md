# 之前"解不开"的类别，逐类重试记录

验收口径：解出文件的样本数 / 恢复的文件名条目。

| # | 类别 | 样本数 | 状态 | 手段 | 结果 |
|---|---|---|---:|---|---|
| 1 | Mercury MiniFS v1（逐文件 LZMA） | 39 | ✅ 已解 | 每文件一条 LZMA 流，offset 相对首个 `5A 00 00 80` | 每样本 ~104–123 个文件 |
| 2 | TP-Link/Mercury MiniFS v3（分块 LZMA） | 17 | ✅ 已解 | 32 字节头 + ToN/ToF/ToC + chunk LZMA，配置字 `5D 00 00 80` | C80 297/297 |
| 3 | MiniFS v2（MAC1200RE） | 1+ | ✅ 已解（无名字） | 头 32B + [hash16+压缩长度+...+解压长度+0] 与流交替；名字已哈希化 | 175/175 个文件（80 png/50 html/26 js/4 gif/2 pem） |
| 4 | MR900 系列（OW 文件系统） | 5 | ✅ 已解 | LZMA 流内 32 字节 `OW` 魔数 + 计数 + 76 字节记录（名字[60]+size+offset） | 237/239 个文件（202 htm/20 gif/8 js/4 css） |
| 5 | TP-Link 老机型裸内核 | ~34 | ❌ 无解（已证） | 只有内核 + 名字清单；逐样本确认 `<html`/`GIF8`/`PNG` 出现次数全为 0 | 文件数据不在升级包里，只能给名字 |
| 6 | Netgear PK11 文件系统 | 9 | ✅ 已解 | CHK 层内 `PK11` + `[name\0][u32 LE size]`，数据顺序排列 | 每样本 120–348 个原名文件（102 htm/7 gif/5 js/4 xml/2 css） |
| 6b | Belkin（无表） | 7 | ❌ 无表 | 层内 375–1719 个 `.htm` 全是 MIME/调试字符串，无 `PK11`/`MINIFS`/`OW`/`TROC` 魔数 | 只能给名字清单 |
| 7 | 泰克 TDS210/THS 裸内核 | 8 | ❌ 无解 | 无 FS、无符号表 | — |
| 8 | TRENDnet `*_nml.bin` | 已下载 1 个 | ⏸ 缺密钥 | 786,432 B：8 字节头 + ~590 KB 密文（熵 7.96）+ 填充；无 ECB 重复块；68 个候选密钥 AES 全败 | 需要设备或 bootloader 里的密钥 |

## 关键结论

**这类固件基本没有真加密**：熵接近 8 只是 LZMA。MW313R 之前判"加密"，实际是
`0x91B50` 处一条 LZMA-alone 流；MAC1200RE 的 v2 表也一样，只是表结构不同。

## 参考来源

- [CTF导航：VxWorks固件加载地址分析方法研究](https://www.ctfiot.com/86294.html)（MINIFS 结构）
- [H4lo/MINIFS-Decompression](https://github.com/H4lo/MINIFS-Decompression)（v1 逐文件 LZMA）
- [RedaOps/minifs-extractor](https://github.com/RedaOps/minifs-extractor)（v3 分块）
- [arXiv 2407.05064](https://arxiv.org/html/2407.05064v1)（论文）
- [binwalk issue #894](https://github.com/ReFirmLabs/binwalk/issues/894)（线索来源）

## 当前全量

- 解出文件总数：30,690
- 有效命名文件：18,014
- 文件名+内容都恢复的样本：163 / 308（53%）


## 实现

早期 Python 实现已从仓库移除，Rust 是唯一实现；`vx batch` 三项能力的验收数据在 `stats/RUST_METRICS.csv`。

## 最终指标（308 样本）

| 指标 | 值 |
|---|---|
| 正常解包 | 298（97%），另 10 个仅内核（单独计数） |
| 文件名 + 内容 | 162（53%） |
| 仅名字（弱证据） | 17 |
| 符号表 | 116（38%），824,018 个名字 |
| 解出文件总数 | 37,685 |
| 有效命名文件 | 18,999 |

## 结论

能解的四个变体全部解出并双端落地；剩下三类经证据确认不可解：
老机型升级包不含文件数据（零 HTML/图片）、Belkin 无目录表、TRENDnet 需要设备密钥。

## 10 个「解包失败」的体检结果

这 10 个样本没有一个是「里面有文件系统但解不开」，全部是「文件本身就是内核」。

### 情况一：裸内核，外面没有容器（7 个）

| 样本 | 大小 | 熵 | WIND version | 结论 |
|---|---:|---:|---:|---|
| TDS210_1.06 / 1.09 / 1.17 / 1.19 / 1.19b | 524,288 B | 6.48 | 1 | 整片 flash 镜像 |
| TDS520_v2.16e | 1,572,864 B | 6.33 | 1 | 整片 flash 镜像 |

判断依据：大小是精确整数（512 KB / 1.5 MB）；熵 6.3–6.5 说明是未压缩代码（压缩数据接近 8.0）；
含 `WIND version` 字符串；没有 MINIFS / OW / PK11 / TROC；没有符号表。

### 情况二：20 字节头 + 裸 ARM 内核（3 个）

MR804 ×2、TL-R402M v2。之前被误判成「未知容器」，其实是没剥头。

```
00 06 1d 80 | 25 7d 89 38 ... 32 bf 84 df | 02 00 A0 E3 0A 00 00 EA ...
   ↑ 长度        ↑ 16 字节（签名/随机盐）          ↑ ARM: mov r0,#2 ; b
```

- 前 4 字节 BE = 400,768，文件 = 400,788，差正好 20
- TL-R402M：401,312 vs 401,332，同样差 20
- 偏移 0x14 起是 ARM 指令，随后是明文 `Copyright`
- 载荷不是压缩数据（LZMA/zlib/gzip/bzip2 全不匹配），熵 7.99 是因为 ARM 代码本身密集
- 那 16 字节与 md5/sha1/sha256(payload) 都不匹配，判断为签名或随机盐

### 工具现在怎么处理

`fs_carve.detect_raw_kernel()` 加了两条规则：

1. `u32 BE == len - 20` 且偏移 0x14 处是合法 ARM 指令 → `arm-kernel-with-20b-header`，输出剥头后的 `kernel.bin`
2. 含 `WIND version` 且无已知文件系统 → `flat-vxworks-kernel`，输出 `kernel.bin`

屏幕上会打印：

```
[+] Payload unpacked (arm-kernel-with-20b-header): 1 file(s)
[*] Likely main logic: raw kernel image (kernel.bin, header stripped)
```

这 10 个仍然不会计入「解包率」（阈值是解出 >1 个文件），但状态从 `unknown` 变成「已知内核，无可解容器」。
