# vxfile_extractor V3.0.0

[English](README_EN.md)

一键提取 vxwork 固件内部文件，并依据 binwalk 分析结果和 uboot 镜像内文件名偏移表，正确恢复 vxwork 文件名。

**重大更新！** 适配 **300+** 型号 VxWorks 固件，解包成功率 **100%**，符号表识别率 **33.4%**（有很多没有符号表，尽可能导出了），文件名重命名成功率 **53.2%**（有文件名表的 164 个样本全部成功）。

## 三个成功率

| 指标 | 结果 | 口径 |
|---|---:|---|
| 解包成功率 | 308 / 308 = **100%** | 每个固件至少解出产物（文件 / 内核镜像 / 解压层） |
| └ 整层全解出 | 209 / 308 = 67.9% | 所解容器的字节覆盖率 ≥ 90%，严格口径 |
| 文件名重命名成功率 | 164 / 308 = 53.2% | 恢复出原始文件名； |
| 符号表识别率 | 103 / 308 = 33.4% | 单个固件内识别到 ≥ 20 个符号名 |

逐固件数据在 [`stats/RUST_METRICS.csv`](stats/RUST_METRICS.csv)，类型画像在 [`stats/PROFILE.csv`](stats/PROFILE.csv)。

## 新特性

1. **以 Rust 重构，速度提升约 20 倍**。端到端从纯 Python 版的 4–34 秒/样本降到 0.2–1.6 秒/样本（21.7×），单文件二进制，不再依赖 Python 和 binwalk。

2. **按文件系统分类适配**。前 AI 时代只能靠人力逐个试偏移，因此受限于匹配规模，做的略显粗糙；这次用 300+ 个真实固件做分类匹配，按五类文件系统的特点，采用不同的分析方案和解包方法：

   | 族 | 代表 |
   |---|---|
   | MINIFS v1 | Mercury MW/MR |
   | MINIFS v2 | MAC1200RE |
   | MINIFS v3 | Archer C80/C64 |
   | OW | MR900 |
   | PK11 | Netgear CHK |

   各类的字节布局和解析细节见 [`docs/FILESYSTEMS.md`](docs/FILESYSTEMS.md)。

3. **新的 UI 能力（命令行）**。`vxfile_extractor profile` 展示每个文件块的识别类型与熵值，`vxfile_extractor entropy` 标出高熵区间（`0x起-0x止`）并判断该区域已解出还是疑似加密，辅助分析人员快速理解固件结构组成。

4. **固件案例可复现**。308 个样本的厂商、型号、版本、sha256、来源 URL、文件系统族见 [`stats/CORPUS.csv`](stats/CORPUS.csv)。

## 编译

```bash
git clone https://github.com/0xba1100n/vxfile_extractor.git
cd vxfile_extractor
git checkout v3.0.0

cd rust/vxcore
cargo build --release
./target/release/vxfile_extractor --version     # vxfile_extractor 3.0.0
```

交叉编译需要 `rustup target add <target>` 和对应 linker：

```bash
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target aarch64-apple-darwin
```

## 直接下载（推荐）

| 你的机器 | 下载 | 大小 |
|---|---|---:|
| Linux x86_64（Ubuntu / Debian / CentOS） | [`..._linux_x86_64_gnu`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_x86_64_gnu) | 1.44 MB |
| Linux x86_64（Alpine、全静态） | [`..._linux_x86_64_musl`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_x86_64_musl) | 1.54 MB |
| Linux ARM64（服务器 / 64 位树莓派系统） | [`..._linux_aarch64_gnu`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_aarch64_gnu) | 1.32 MB |
| Linux ARMv7（老树莓派 / 路由器 Linux） | [`..._linux_armv7_musl`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_armv7_musl) | 1.39 MB |
| Windows 64 位 | [`..._windows_x86_64.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_x86_64.exe) | 1.11 MB |
| Windows 32 位 | [`..._windows_i686.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_i686.exe) | 1020 KB |
| Windows ARM64 | [`..._windows_aarch64.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_aarch64.exe) | 972 KB |
| macOS Apple Silicon | [`..._macos_aarch64`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_macos_aarch64) | 1.12 MB |
| macOS Intel | [`..._macos_x86_64`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_macos_x86_64) | 1.34 MB |

Linux / macOS 下载后赋予执行权限：

```bash
chmod +x VxFirmwareExtractor_linux_x86_64_gnu
./VxFirmwareExtractor_linux_x86_64_gnu --version    # vxfile_extractor 3.0.0
```

Release 页：<https://github.com/0xba1100n/vxfile_extractor/releases/tag/v3.0.0>

## 用法

```bash
vxfile_extractor fw.bin                      # 一键解包，产物在 ./out
vxfile_extractor extract fw.bin -o out/      # 同上，显式指定输出目录
vxfile_extractor verdict   fw.bin            # 判定是否为 VxWorks
vxfile_extractor scan      fw.bin            # 列出容器 / 解压层
vxfile_extractor tables    fw.bin            # 打印文件名偏移表
vxfile_extractor symbols   fw.bin            # 打印符号名表
vxfile_extractor profile   fw.bin --block 1024 --ascii   # 逐块类型 + 熵

# 批量
vxfile_extractor batch         . -o stats/RUST_METRICS.csv   # 三项能力（19 分钟 / 308 样本）
vxfile_extractor profile-batch . -o stats/PROFILE.csv        # 类型占比（13 秒）
vxfile_extractor entropy       . -o stats/ENTROPY.csv        # 高熵区间（10 秒）
```

`vxfile_extractor extract` 的产物：

```
out/
├─ minifs/      MINIFS v1/v3 解出的文件（保留原路径）
├─ files/       PK11（Netgear）解出的文件
├─ streams/     LZMA 流兜底解出的文件
├─ kernel.bin   裸内核（整片镜像，或剥掉 20 字节头）
└─ SUMMARY.json 层数 / 版本 / 计数
```

## 运行示例

解包：

```
$ vxfile_extractor samples/tp-link/Archer_C80_US_V1_191122.zip -o out/
[+] Output layout
    bin/   8 file(s)  <- executable / firmware image candidates
    src/   285 file(s)  <- static resources (js 166, html 81, css 13, json 9, xml 6)
    cfg/   4 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x158910 (3947396 B)
[!] high-entropy 0x00158C00-0x0027E800 (1203 KiB, 99.4%) -> compressed (data was recovered)
```

逐块画像：

```
$ vxfile_extractor profile samples/expand-a/inner/MW300R__201275172121.bin --block 1024
0x00000000-0x00000400       1024 B  ent  7.43  data
0x00000400-0x00101000    1051648 B  ent  7.81  high-entropy
0x0012C000-0x0012C400       1024 B  ent  7.82  lzma
kind                  bytes  regions   share
high-entropy        1298432        3   99.2%
[entropy 0..8] @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@#@@@@@@@@@@@@@@@
```

高熵加密区域判定：

```
$ vxfile_extractor samples/expand-a/inner/FW300R__2012126162529.bin -o out/
[!] high-entropy 0x00040000-0x00141000 (1028 KiB, 99.4%) -> likely encrypted / unknown container
```

全部案例：100 个解开的 + 全部解不开的，逐条带实际输出和解读，见 [`docs/EXAMPLES.md`](docs/EXAMPLES.md)。

样本和格式认识基本都来自 0xba1100n 的 [vxfile_extractor](https://github.com/0xba1100n/vxfile_extractor) 和那篇[看雪帖](https://bbs.kanxue.com/thread-284324.htm)，MINIFS 分块结构参考了 [RedaOps/minifs-extractor](https://github.com/RedaOps/minifs-extractor) 与 winmt 师傅的[论文](https://arxiv.org/html/2407.05064v1)。

## 鸣谢

winmt、大肥鱼。

大肥鱼（DeepSeek V4.1 Flash）跑了 99 轮、1260 步，176 tok/s，一共 503M tokens，把这个项目从语料一路搭到 Rust 引擎。

附上博客文章作为补充说明：<https://ba1100n.tech/iot_security/%e6%8e%a2%e7%a9%b6vxworks%e6%96%87%e4%bb%b6%e5%90%8d%e6%81%a2%e5%a4%8d/>

程序设计细节和恢复原理详见：<https://bbs.kanxue.com/thread-284324.htm>
