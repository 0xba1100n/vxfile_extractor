# vxfile_extractor V3.0.0

[English](README_EN.md)

**重大更新！** 这次把 300+ 个型号的 VxWorks 固件都过了一遍：解包成功率 **100%**，符号表识别率 **33.4%**，文件名重命名成功率 **53.2%**。

## 三个成功率

先把数摆出来，免得看完一堆功能还不知道到底能干什么：

| 指标 | 结果 | 口径 |
|---|---:|---|
| 解包成功率 | 308 / 308 = **100%** | 每个固件至少解出产物（文件 / 内核镜像 / 解压层） |
| └ 整层全解出 | 209 / 308 = 67.9% | 所解容器的字节覆盖率 ≥ 90%，这个口径更严 |
| 文件名重命名成功率 | 164 / 308 = 53.2% | 恢复出**原始文件名**，`stream_*`、`v2_*` 这种生成名不算 |
| 符号表识别率 | 103 / 308 = 33.4% | 单个固件里识别到 ≥ 20 个符号名 |

逐固件的数据在 [`stats/RUST_METRICS.csv`](stats/RUST_METRICS.csv)，类型画像在 [`stats/PROFILE.csv`](stats/PROFILE.csv)，都能直接拉进表格里看。

## 新特性

1. **以 Rust 重构，速度提升约 20 倍**。这块是这次改动最大的地方：端到端从纯 Python 版的 4–34 秒/样本降到 0.2–1.6 秒/样本（21.7×）。而且现在是单文件二进制，不用再装 Python，也不用 binwalk。

2. **按文件系统分类适配**。前 AI 时代只能靠人力一个个去试偏移；这次拿 300+ 个真实固件做分类匹配，才敢说摸清了五类文件系统，每类走不同的分析方案和解包方法：

   | 族 | 代表 | 结构 |
   |---|---|---|
   | MINIFS v1 | Mercury MW/MR | 每文件一条 LZMA 流，offset 相对首个 `5A 00 00 80` |
   | MINIFS v3 | Archer C80/C64 | 32 字节头 + ToN/ToF/ToC + chunk 级 LZMA |
   | MINIFS v2 | MAC1200RE | `[hash16][压缩长][name_off][解压长][0]` 与流交替 |
   | OW | MR900 | LZMA 流内 32 字节 `OW` 魔数 + 76 字节记录 |
   | PK11 | Netgear CHK | `PK11` + `[name\0][u32 LE size]` |

3. **新的 UI 能力（命令行）**。顺手做了个能「看」结构的东西：`vxfile_extractor profile` 逐块给出识别类型和熵值，`vxfile_extractor entropy` 直接把高熵区间标成 `0x起-0x止`，并判断这一段是已经解出来了、还是疑似加密。分析的时候不用再猜这块到底是什么。

4. **固件案例可复现**。308 个样本的厂商、型号、版本、sha256、来源 URL、文件系统族都在 [`stats/CORPUS.csv`](stats/CORPUS.csv)。

## 编译

想自己编的话，就三步：

```bash
git clone https://github.com/0xba1100n/vxfile_extractor.git
cd vxfile_extractor
git checkout v3.0.0

cd rust/vxcore
cargo build --release
./target/release/vx --version     # vxfile_extractor 3.0.0
```

要交叉编译的话，先 `rustup target add <target>` 再配上对应 linker，比如：

```bash
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target aarch64-apple-darwin
```

## 直接下载（推荐）

不想装 Rust 环境的话，直接下对应平台的文件就行：

| 你的机器 | 下这个 | 大小 |
|---|---|---:|
| Linux x86_64（Ubuntu / Debian / CentOS） | [`..._linux_x86_64_gnu`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_x86_64_gnu) | 1.44 MB |
| Linux x86_64（Alpine、要全静态） | [`..._linux_x86_64_musl`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_x86_64_musl) | 1.54 MB |
| Linux ARM64（服务器 / 64 位树莓派系统） | [`..._linux_aarch64_gnu`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_aarch64_gnu) | 1.32 MB |
| Linux ARMv7（老树莓派 / 路由器上的 Linux） | [`..._linux_armv7_musl`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_armv7_musl) | 1.39 MB |
| Windows 64 位 | [`..._windows_x86_64.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_x86_64.exe) | 1.1 MB |
| Windows 32 位 | [`..._windows_i686.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_i686.exe) | 1020 KB |
| Windows ARM64 | [`..._windows_aarch64.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_aarch64.exe) | 970 KB |
| macOS Apple Silicon（M1/M2/M3/M4） | [`..._macos_aarch64`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_macos_aarch64) | 1.1 MB |
| macOS Intel | [`..._macos_x86_64`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_macos_x86_64) | 1.34 MB |

Linux / macOS 下载后给个执行权限就能跑：

```bash
chmod +x VxFirmwareExtractor_linux_x86_64_gnu
./VxFirmwareExtractor_linux_x86_64_gnu --version    # vxfile_extractor 3.0.0
```

校验和（sha256）在 Release 页面每个文件下面，也可以直接对：

| 文件 | sha256 |
|---|---|
| `VxFirmwareExtractor_linux_x86_64_gnu` | `2ee12fbb0ec55fe6783fd77f2cbda55324894b0a136c57a8df1edac9454acfa2` |
| `VxFirmwareExtractor_linux_x86_64_musl` | `a49ff20fe15ecd01dca353317b32c7f42e07d977a10948d0aa0612a899c8a57a` |
| `VxFirmwareExtractor_linux_aarch64_gnu` | `d7bf3d7f97bc05311e96874a7f7cbe99a861a424e17ab610817e04ab48e7319e` |
| `VxFirmwareExtractor_linux_aarch64_musl` | `2b930a50551d73f09e1e65642bac339f0e8b93087e4264a300d65f01fe14ef9e` |
| `VxFirmwareExtractor_linux_armv7_musl` | `9723df8bcf1a54a198829cbd2c7b04f12b946e1eab6b8645ef76451d116edc03` |
| `VxFirmwareExtractor_windows_x86_64.exe` | `1c1130475c433c95e80e4a3e5b974d85be579817a7f7b0658ba80218db22bda5` |
| `VxFirmwareExtractor_windows_i686.exe` | `6b204eda34c30124b99544d67c981a4034a3424904aedc39ebc5bcd62ac326cf` |
| `VxFirmwareExtractor_windows_aarch64.exe` | `bed7b3a5a984ea0d49cc4609ab8791483d23087e60187446fb7af47471777835` |
| `VxFirmwareExtractor_macos_aarch64` | `c3d5cc7069418fb3179583e0f27c58155350bd671d138fa7c9b11a4f2de868ed` |
| `VxFirmwareExtractor_macos_x86_64` | `344eb1ff8f3a09d8166611a9f1f1f081dd55b29a5392da1a636f8dcdc2075e84` |

Release 页：<https://github.com/0xba1100n/vxfile_extractor/releases/tag/v3.0.0>

## 用法

最省事的就一条：

```bash
vxfile_extractor fw.bin            # 一键解包，产物落在当前目录的 out/
```

等价于 `vxfile_extractor extract fw.bin -o out/`。想细看结构再用子命令：

```bash
vxfile_extractor verdict   fw.bin                  # 先看是不是 VxWorks
vxfile_extractor scan      fw.bin                  # 列出容器 / 解压层
vxfile_extractor tables    fw.bin                  # 打印文件名偏移表
vxfile_extractor symbols   fw.bin                  # 打印符号名表
vxfile_extractor extract   fw.bin -o out/          # 解包：文件 + 符号 + 摘要
vxfile_extractor profile   fw.bin --block 1024 --ascii   # 逐块类型 + 熵

# 批量跑
vxfile_extractor batch         . -o stats/RUST_METRICS.csv   # 三项能力（19 分钟 / 308 样本）
vxfile_extractor profile-batch . -o stats/PROFILE.csv        # 类型占比（13 秒）
vxfile_extractor entropy       . -o stats/ENTROPY.csv        # 高熵区间（10 秒）
```

`vxfile_extractor extract` 出来的东西长这样：

```
out/
├─ minifs/      MINIFS v1/v3 解出的文件（保留原路径）
├─ files/       PK11（Netgear）解出的文件
├─ streams/     LZMA 流兜底解出的文件
├─ kernel.bin   裸内核（整片镜像，或剥掉 20 字节头）
└─ SUMMARY.json 层数 / 版本 / 计数
```

## 运行示例

跑起来大概是这样——先看解包：

```
$ vxfile_extractor extract samples/tp-link/Archer_C80_US_V1_191122.zip -o out/
[+] Output layout
    bin/   8 file(s)  <- executable / firmware image candidates
    src/   285 file(s)  <- static resources (js 166, html 81, css 13, json 9, xml 6)
    cfg/   4 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x158910 (3947396 B)
[!] high-entropy 0x00158C00-0x0027E800 (1203 KiB, 99.4%) -> compressed (data was recovered)
```

再看逐块画像：

```
$ vxfile_extractor profile samples/expand-a/inner/MW300R__201275172121.bin --block 1024
0x00000000-0x00000400       1024 B  ent  7.43  data
0x00000400-0x00101000    1051648 B  ent  7.81  high-entropy
0x0012C000-0x0012C400       1024 B  ent  7.82  lzma
kind                  bytes  regions   share
high-entropy        1298432        3   99.2%
[entropy 0..8] @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@#@@@@@@@@@@@@@@@
```

以及疑似加密的判定：

```
$ vxfile_extractor extract samples/expand-a/inner/FW300R__2012126162529.bin -o out/
[!] high-entropy 0x00040000-0x00141000 (1028 KiB, 99.4%) -> likely encrypted / unknown container
```

更多案例（20 个解开 + 20 个解不开）放在 [`docs/EXAMPLES.md`](docs/EXAMPLES.md)。

## v2.1（原文）

> 一键提取 vxwork 固件内部文件，并依据 binwalk 分析结果和 uboot 镜像内文件名偏移表，正确恢复 vxwork 文件名。
>
> binwalk 尚未支持对 vxworks 系统固件的正确解包，而且已经停止更新很久了。直接用 binwalk 一把梭会遇到这种情况：全是莫名其妙的文件名。
>
> 附上博客文章作为补充说明：<https://ba1100n.tech/iot_security/%e6%8e%a2%e7%a9%b6vxworks%e6%96%87%e4%bb%b6%e5%90%8d%e6%81%a2%e5%a4%8d/>
>
> 程序设计细节和恢复原理详见：<https://bbs.kanxue.com/thread-284324.htm>

## 鸣谢

样本和格式认识基本都来自 0xba1100n 的 [vxfile_extractor](https://github.com/0xba1100n/vxfile_extractor) 和那篇[看雪帖](https://bbs.kanxue.com/thread-284324.htm)，MINIFS 分块结构参考了 [RedaOps/minifs-extractor](https://github.com/RedaOps/minifs-extractor) 与 winmt 师傅的[论文](https://arxiv.org/html/2407.05064v1)。

大肥鱼（DeepSeek V4.1 Flash）跑了 99 轮、1260 步，176 tok/s，一共 503M tokens，把这个项目从语料一路搭到 Rust 引擎。
