# vxfile_extractor V3.0.0

[中文说明](README.md)

**Major update.** We went through 300+ VxWorks firmware models: unpack success **100%**, symbol-table detection **33.4%**, original-filename recovery **53.2%**.

## Three success rates

Numbers first, so you know what this thing actually does before reading the feature list:

| Metric | Result | Definition |
|---|---:|---|
| Unpack success | 308 / 308 = **100%** | at least one artifact per firmware (files / kernel image / layers) |
| └ whole layer decoded | 209 / 308 = 67.9% | byte coverage of the decoded container >= 90% (stricter) |
| Original filename recovery | 164 / 308 = 53.2% | real names restored; generated names (`stream_*`, `v2_*`) do not count |
| Symbol-table detection | 103 / 308 = 33.4% | >= 20 symbol names found in one firmware |

Per-firmware data lives in [`stats/RUST_METRICS.csv`](stats/RUST_METRICS.csv), type profile in [`stats/PROFILE.csv`](stats/PROFILE.csv).

## What is new

1. **Rewritten in Rust, about 20x faster.** This is the biggest change in this release: end-to-end runtime went from 4-34 s per sample (pure Python) to 0.2-1.6 s (21.7x). It is a single binary now - no Python, no binwalk.

2. **Filesystem-aware dispatching.** Before the AI era this was manual trial and error, one offset table at a time. With 300+ real images classified, we could finally say we understand five families, each with its own parser:

   | Family | Example | Layout |
   |---|---|---|
   | MINIFS v1 | Mercury MW/MR | one LZMA stream per file, offsets relative to the first `5A 00 00 80` |
   | MINIFS v3 | Archer C80/C64 | 32-byte header + ToN/ToF/ToC + chunked LZMA |
   | MINIFS v2 | MAC1200RE | `[hash16][comp_len][name_off][decomp_len][0]` interleaved with streams |
   | OW | MR900 | 32-byte `OW` magic inside an LZMA stream + 76-byte records |
   | PK11 | Netgear CHK | `PK11` + `[name\0][u32 LE size]`, data laid out sequentially |

3. **New UI capability (CLI).** So you can actually *look* at the structure: `vxfile_extractor profile` reports the detected type and entropy of every block, and `vxfile_extractor entropy` marks high-entropy ranges as `0xstart-0xend`, telling you whether that region was decoded or is likely encrypted. No more guessing what a chunk is.

4. **Reproducible corpus.** Vendor, model, version, sha256, source URL and filesystem family for all 308 samples: [`stats/CORPUS.csv`](stats/CORPUS.csv).

## Build

Three steps:

```bash
git clone https://github.com/0xba1100n/vxfile_extractor.git
cd vxfile_extractor
git checkout v3.0.0

cd rust/vxcore
cargo build --release
./target/release/vx --version     # vxfile_extractor 3.0.0
```

For cross builds, `rustup target add <target>` and point cargo at a linker:

```bash
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target aarch64-apple-darwin
```

## Download (recommended)

Skip the toolchain and grab the binary for your platform:

| Your machine | Download | Size |
|---|---|---:|
| Linux x86_64 (Ubuntu / Debian / CentOS) | [`..._linux_x86_64_gnu`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_x86_64_gnu) | 1.44 MB |
| Linux x86_64 (Alpine / fully static) | [`..._linux_x86_64_musl`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_x86_64_musl) | 1.54 MB |
| Linux ARM64 (servers, 64-bit Pi OS) | [`..._linux_aarch64_gnu`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_aarch64_gnu) | 1.32 MB |
| Linux ARMv7 (older Pi, router Linux) | [`..._linux_armv7_musl`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_linux_armv7_musl) | 1.39 MB |
| Windows 64-bit | [`..._windows_x86_64.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_x86_64.exe) | 1.1 MB |
| Windows 32-bit | [`..._windows_i686.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_i686.exe) | 1020 KB |
| Windows ARM64 | [`..._windows_aarch64.exe`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_windows_aarch64.exe) | 970 KB |
| macOS Apple Silicon (M1-M4) | [`..._macos_aarch64`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_macos_aarch64) | 1.1 MB |
| macOS Intel | [`..._macos_x86_64`](https://github.com/0xba1100n/vxfile_extractor/releases/download/v3.0.0/VxFirmwareExtractor_macos_x86_64) | 1.34 MB |

On Linux / macOS just make it executable:

```bash
chmod +x VxFirmwareExtractor_linux_x86_64_gnu
./VxFirmwareExtractor_linux_x86_64_gnu --version    # vxfile_extractor 3.0.0
```

sha256 checksums are on the release page next to each file (they change on every CI rebuild, so they are not copied here).

Release page: <https://github.com/0xba1100n/vxfile_extractor/releases/tag/v3.0.0>

## Usage

The short version:

```bash
vxfile_extractor fw.bin            # one shot: unpack into ./out
```

Same as `vxfile_extractor extract fw.bin -o out/`. Subcommands when you want details:

```bash
vxfile_extractor verdict   fw.bin                  # is it VxWorks?
vxfile_extractor scan      fw.bin                  # list containers / layers
vxfile_extractor tables    fw.bin                  # dump filename tables
vxfile_extractor symbols   fw.bin                  # dump symbol names
vxfile_extractor extract   fw.bin -o out/          # files + symbols + summary
vxfile_extractor profile   fw.bin --block 1024 --ascii   # per-block type and entropy

# batch
vxfile_extractor batch         . -o stats/RUST_METRICS.csv   # three metrics (19 min / 308 samples)
vxfile_extractor profile-batch . -o stats/PROFILE.csv        # type shares (13 s)
vxfile_extractor entropy       . -o stats/ENTROPY.csv        # high-entropy ranges (10 s)
```

What `vxfile_extractor extract` produces:

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
$ vxfile_extractor extract samples/tp-link/Archer_C80_US_V1_191122.zip -o out/
[+] Output layout
    bin/   8 file(s)  <- executable / firmware image candidates
    src/   285 file(s)  <- static resources (js 166, html 81, css 13, json 9, xml 6)
    cfg/   4 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x158910 (3947396 B)
[!] high-entropy 0x00158C00-0x0027E800 (1203 KiB, 99.4%) -> compressed (data was recovered)
```

Per-block profile:

```
$ vxfile_extractor profile samples/expand-a/inner/MW300R__201275172121.bin --block 1024
0x00000000-0x00000400       1024 B  ent  7.43  data
0x00000400-0x00101000    1051648 B  ent  7.81  high-entropy
0x0012C000-0x0012C400       1024 B  ent  7.82  lzma
kind                  bytes  regions   share
high-entropy        1298432        3   99.2%
[entropy 0..8] @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@#@@@@@@@@@@@@@@@
```

Encrypted verdict:

```
$ vxfile_extractor extract samples/expand-a/inner/FW300R__2012126162529.bin -o out/
[!] high-entropy 0x00040000-0x00141000 (1028 KiB, 99.4%) -> likely encrypted / unknown container
```

More cases (20 solved + 20 unsolved): [`docs/EXAMPLES.md`](docs/EXAMPLES.md).

## v2.1 (original text)

> One-click extraction of files inside vxWorks firmware; restores real filenames using binwalk output and the filename offset table inside the uImage.
>
> binwalk does not support unpacking vxWorks firmware and has been unmaintained for a long time. Running binwalk directly gives you a pile of meaningless filenames.
>
> Blog post: <https://ba1100n.tech/iot_security/%e6%8e%a2%e7%a9%b6vxworks%e6%96%87%e4%bb%b6%e5%90%8d%e6%81%a2%e5%a4%8d/>
>
> Implementation details: <https://bbs.kanxue.com/thread-284324.htm>

## Credits

Sample collection and format knowledge come mostly from 0xba1100n's [vxfile_extractor](https://github.com/0xba1100n/vxfile_extractor) and the [Kanxue post](https://bbs.kanxue.com/thread-284324.htm); the chunked MINIFS layout follows [RedaOps/minifs-extractor](https://github.com/RedaOps/minifs-extractor) and winmt's [paper](https://arxiv.org/html/2407.05064v1).

Da Feiyu (DeepSeek V4.1 Flash) ran 99 rounds, 1260 steps, at 176 tok/s - 503M tokens in total - and built this project from the corpus up to the Rust engine.
