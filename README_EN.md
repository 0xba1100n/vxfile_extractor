# vxfile_extractor V3.0.1

[中文说明](README.md)

One-click extraction of files inside vxWorks firmware, restoring real filenames from binwalk output and the filename offset table inside the uImage.

**Major update.** **300+** VxWorks firmware models covered. Unpack success **100%**, symbol-table detection **33.4%** (many images carry no symbol table at all; we export whatever is there), original-filename recovery **53.2%** (all 164 samples that carry a filename table were renamed successfully).

v3.0.1 adds support for a number of Mercury routers such as MW300R, MW151RM and MW150R (IMG0 header + LZMA stream).

## Three success rates

| Metric | Result | Definition |
|---|---:|---|
| Unpack success | 308 / 308 = **100%** | at least one artifact per firmware (files / kernel image / layers) |
| └ whole layer decoded | 209 / 308 = 67.9% | byte coverage of the decoded container >= 90% (strict) |
| Original filename recovery | 164 / 308 = 53.2% | real filenames restored; |
| Symbol-table detection | 103 / 308 = 33.4% | >= 20 symbol names found in one firmware |

Per-firmware data: [`stats/RUST_METRICS.csv`](stats/RUST_METRICS.csv). Type profile: [`stats/PROFILE.csv`](stats/PROFILE.csv).

## What is new

1. **Rewritten in Rust, about 20x faster.** End-to-end runtime went from 4-34 s per sample (pure Python) to 0.2-1.6 s (21.7x). Single binary, no Python and no binwalk.

2. **Filesystem-aware dispatching.** Before the AI era this required manual offset hunting, so the matching scale was limited and the result was somewhat rough. With 300+ real images classified, each of the five filesystem families now gets its own analysis and unpacking strategy:

   | Family | Representative |
   |---|---|
   | MINIFS v1 | Mercury MW/MR |
   | MINIFS v2 | MAC1200RE |
   | MINIFS v3 | Archer C80/C64 |
   | OW | MR900 |
   | PK11 | Netgear CHK |

   Byte layouts and parsing details: [`docs/FILESYSTEMS.md`](docs/FILESYSTEMS.md).

3. **New UI capability (CLI).** During extraction it reports the detected type and entropy per block, marks high-entropy ranges (`0xstart-0xend`) and classifies them as decoded or likely encrypted, helping analysts understand firmware layout.

4. **Reproducible corpus.** Vendor, model, version, sha256, source URL and filesystem family for all 308 samples: [`stats/CORPUS.csv`](stats/CORPUS.csv).

## Build

```bash
git clone https://github.com/0xba1100n/vxfile_extractor.git
cd vxfile_extractor
git checkout v3.0.1

cd rust/vxcore
cargo build --release
./target/release/vxfile_extractor --version     # vxfile_extractor 3.0.0
```

Cross compilation requires `rustup target add <target>` and a linker:

```bash
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target aarch64-apple-darwin
```

## Download (recommended)

| Your machine | Download | Size |
|---|---|---:|
| Linux x86_64 (Ubuntu / Debian / CentOS) | [`..._linux_x86_64_gnu`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_x86_64_gnu) | 1.44 MB |
| Linux x86_64 (Alpine / static) | [`..._linux_x86_64_musl`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_x86_64_musl) | 1.54 MB |
| Linux ARM64 (servers, 64-bit Pi OS) | [`..._linux_aarch64_gnu`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_aarch64_gnu) | 1.32 MB |
| Linux ARMv7 (older Pi, router Linux) | [`..._linux_armv7_musl`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_armv7_musl) | 1.39 MB |
| Windows 64-bit | [`..._windows_x86_64.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_x86_64.exe) | 1.11 MB |
| Windows 32-bit | [`..._windows_i686.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_i686.exe) | 1020 KB |
| Windows ARM64 | [`..._windows_aarch64.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_aarch64.exe) | 972 KB |
| macOS Apple Silicon | [`..._macos_aarch64`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_macos_aarch64) | 1.12 MB |
| macOS Intel | [`..._macos_x86_64`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_macos_x86_64) | 1.34 MB |

On Linux / macOS, make it executable:

```bash
chmod +x VxFirmwareExtractor_linux_x86_64_gnu
./VxFirmwareExtractor_linux_x86_64_gnu --version    # vxfile_extractor 3.0.0
```

Release page: <https://github.com/0xba1100n/vxfile_extractor/releases/tag/v3.0.0>

## Usage

```bash
vxfile_extractor fw.bin                 # one shot, output in ./out
vxfile_extractor fw.bin --out result    # custom output directory
```

Output:

```
out/
├─ minifs/      files from MINIFS v1/v3 (original paths)
├─ files/       files from PK11 (Netgear)
├─ streams/     files carved from LZMA streams
├─ kernel.bin   raw kernel (whole image, or with the 20-byte header stripped)
└─ SUMMARY.json layers / versions / counters
```

## Examples

Extract:

```
$ vxfile_extractor samples/tp-link/Archer_C80_US_V1_191122.zip -o out/
[+] Output layout
    bin/   8 file(s)  <- executable / firmware image candidates
    src/   285 file(s)  <- static resources (js 166, html 81, css 13, json 9, xml 6)
    cfg/   4 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x158910 (3947396 B)
[!] high-entropy 0x00158C00-0x0027E800 (1203 KiB, 99.4%) -> compressed (data was recovered)
```

High-entropy region verdict:

```
$ vxfile_extractor samples/expand-a/inner/FW300R__2012126162529.bin -o out/
[!] high-entropy 0x00040000-0x00141000 (1028 KiB, 99.4%) -> likely encrypted / unknown container
```

Full case list: 100 solved plus every unsolved sample, each with real output and an explanation: [`docs/EXAMPLES.md`](docs/EXAMPLES.md).

Sample collection and format knowledge come mostly from 0xba1100n's [vxfile_extractor](https://github.com/0xba1100n/vxfile_extractor) and the [Kanxue post](https://bbs.kanxue.com/thread-284324.htm); the chunked MINIFS layout follows [RedaOps/minifs-extractor](https://github.com/RedaOps/minifs-extractor) and winmt's [paper](https://arxiv.org/html/2407.05064v1).

## Credits

winmt, Da Feiyu.

Da Feiyu (DeepSeek V4.1 Flash) ran 99 rounds, 1260 steps, at 176 tok/s - 503M tokens in total - and built this project from the corpus up to the Rust engine.

Blog post (in Chinese): <https://ba1100n.tech/iot_security/%e6%8e%a2%e7%a9%b6vxworks%e6%96%87%e4%bb%b6%e5%90%8d%e6%81%a2%e5%a4%8d/>

Implementation details and recovery methodology: <https://bbs.kanxue.com/thread-284324.htm>
