# vxfile_extractor V3.0.0

**重大更新！** 适配 **300+** 型号 VxWorks 固件，解包成功率 **100%**，符号表识别率 **33.4%**，文件名重命名成功率 **53.2%**。

## 三个成功率

| 指标 | 结果 | 口径 |
|---|---:|---|
| 解包成功率 | 308 / 308 = **100%** | 每个固件至少解出产物（文件 / 内核镜像 / 解压层） |
| └ 整层全解出 | 209 / 308 = 67.9% | 所解容器的字节覆盖率 ≥ 90%，严格口径 |
| 文件名重命名成功率 | 164 / 308 = 53.2% | 恢复出**原始文件名**，生成名（`stream_*`、`v2_*`）不算 |
| 符号表识别率 | 103 / 308 = 33.4% | 单个固件内识别到 ≥ 20 个符号名 |

逐固件数据在 [`stats/RUST_METRICS.csv`](stats/RUST_METRICS.csv)，类型画像在 [`stats/PROFILE.csv`](stats/PROFILE.csv)。

## 新特性

1. **以 Rust 重构，速度提升约 20 倍**。端到端从纯 Python 版的 4–34 秒/样本降到 0.2–1.6 秒/样本（21.7×），单文件二进制，不再依赖 Python 和 binwalk。

2. **按文件系统分类适配**。前 AI 时代只能靠人力逐个试偏移；这次用 300+ 个真实固件做分类匹配，摸清五类文件系统，每类用不同的分析方案和解包方法：

   | 族 | 代表 | 结构 |
   |---|---|---|
   | MINIFS v1 | Mercury MW/MR | 每文件一条 LZMA 流，offset 相对首个 `5A 00 00 80` |
   | MINIFS v3 | Archer C80/C64 | 32 字节头 + ToN/ToF/ToC + chunk 级 LZMA |
   | MINIFS v2 | MAC1200RE | `[hash16][压缩长][name_off][解压长][0]` 与流交替 |
   | OW | MR900 | LZMA 流内 32 字节 `OW` 魔数 + 76 字节记录 |
   | PK11 | Netgear CHK | `PK11` + `[name\0][u32 LE size]` |

3. **新的 UI 能力（命令行）**。`vx profile` 逐块展示每个文件块的识别类型与熵值，`vx entropy` 直接标出高熵区间（`0x起-0x止`）并judge「已解出」还是「疑似加密」，帮助分析人员快速理解固件结构组成。

4. **固件案例可复现**。308 个样本的厂商、型号、版本、sha256、来源 URL、文件系统族都在 [`stats/CORPUS.csv`](stats/CORPUS.csv)。

## 编译

```bash
git clone https://github.com/0xba1100n/vxfile_extractor.git
cd vxfile_extractor
git checkout v3.0.0

cd rust/vxcore
cargo build --release
./target/release/vx --version     # vx 3.0.0
```

交叉编译（需要 `rustup target add <target>` 和对应 linker）：

```bash
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target aarch64-apple-darwin
```

懒得编译可以直接下 Release 资产（10 个目标，CI 构建）：

| 系统 | 架构 | 资产 |
|---|---|---|
| Linux | x86_64 | `VxFirmwareExtractor_linux_x86_64_gnu` / `_musl` |
| Linux | aarch64 | `VxFirmwareExtractor_linux_aarch64_gnu` / `_musl` |
| Linux | armv7 | `VxFirmwareExtractor_linux_armv7_musl` |
| Windows | x86_64 / i686 / aarch64 | `VxFirmwareExtractor_windows_*.exe` |
| macOS | x86_64 / aarch64 | `VxFirmwareExtractor_macos_*` |

<https://github.com/0xba1100n/vxfile_extractor/releases/tag/v3.0.0>

## 用法

```bash
vx verdict   fw.bin                  # 判定是不是 VxWorks
vx scan      fw.bin                  # 列出容器/解压层
vx tables    fw.bin                  # 打印文件名偏移表
vx symbols   fw.bin                  # 打印符号名表
vx extract   fw.bin -o out/          # 解包：文件 + 符号 + 摘要
vx profile   fw.bin --block 1024 --ascii   # 逐块类型 + 熵

# 批量
vx batch         . -o stats/RUST_METRICS.csv   # 三项能力（19 分钟 / 308 样本）
vx profile-batch . -o stats/PROFILE.csv        # 类型占比（13 秒）
vx entropy       . -o stats/ENTROPY.csv        # 高熵区间（10 秒）
```

`vx extract` 的产物：

```
out/
├─ minifs/      MINIFS v1/v3 解出的文件（保留原路径）
├─ files/       PK11（Netgear）解出的文件
├─ streams/     LZMA 流兜底解出的文件
├─ kernel.bin   裸内核（整片镜像，或剥掉 20 字节头）
└─ SUMMARY.json 层数 / 版本 / 计数
```

## 运行示例

判定与解包：

```
$ vx extract samples/tp-link/Archer_C80_US_V1_191122.zip -o out/
[+] Output layout
    bin/   8 file(s)  <- executable / firmware image candidates
    src/   285 file(s)  <- static resources (js 166, html 81, css 13, json 9, xml 6)
    cfg/   4 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x158910 (3947396 B)
[!] high-entropy 0x00158C00-0x0027E800 (1203 KiB, 99.4%) -> compressed (data was recovered)
```

逐块画像：

```
$ vx profile samples/expand-a/inner/MW300R__201275172121.bin --block 1024
0x00000000-0x00000400       1024 B  ent  7.43  data
0x00000400-0x00101000    1051648 B  ent  7.81  high-entropy
0x0012C000-0x0012C400       1024 B  ent  7.82  lzma
kind                  bytes  regions   share
high-entropy        1298432        3   99.2%
[entropy 0..8] @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@#@@@@@@@@@@@@@@@
```

疑似加密的判定：

```
$ vx extract samples/expand-a/inner/FW300R__2012126162529.bin -o out/
[!] high-entropy 0x00040000-0x00141000 (1028 KiB, 99.4%) -> likely encrypted / unknown container
```

更多案例（20 个解开 + 20 个解不开）：[`docs/EXAMPLES.md`](docs/EXAMPLES.md)。

## v2.1（原文）

> 一键提取 vxwork 固件内部文件，并依据 binwalk 分析结果和 uboot 镜像内文件名偏移表，正确恢复 vxwork 文件名。
>
> binwalk 尚未支持对 vxworks 系统固件的正确解包，而且已经停止更新很久了。直接用 binwalk 一把梭会遇到这种情况：全是莫名其妙的文件名。
>
> 附上博客文章作为补充说明：<https://ba1100n.tech/iot_security/%e6%8e%a2%e7%a9%b6vxworks%e6%96%87%e4%bb%b6%e5%90%8d%e6%81%a2%e5%a4%8d/>
>
> 程序设计细节和恢复原理详见：<https://bbs.kanxue.com/thread-284324.htm>

## 鸣谢

winmt、大肥鱼
