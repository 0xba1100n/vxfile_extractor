# vxfile_extractor V3.0.0

[中文说明](README.md)

**Major update.** **300+** VxWorks firmware models covered. Unpack success **100%**, symbol-table detection **33.4%**, original-filename recovery **53.2%**.

## Three success rates

| Metric | Result | Definition |
|---|---:|---|
| Unpack success | 308 / 308 = **100%** | at least one artifact per firmware (files / kernel image / layers) |
| └ whole layer decoded | 209 / 308 = 67.9% | byte coverage of the decoded container >= 90% (strict) |
| Original filename recovery | 164 / 308 = 53.2% | real names restored; generated names (`stream_*`, `v2_*`) do not count |
| Symbol-table detection | 103 / 308 = 33.4% | >= 20 symbol names found in one firmware |

Per-firmware data: [`stats/RUST_METRICS.csv`](stats/RUST_METRICS.csv); type profile: [`stats/PROFILE.csv`](stats/PROFILE.csv).

## What is new

1. **Rewritten in Rust, ~20x faster.** End-to-end runtime went from 4-34 s per sample (pure Python) to 0.2-1.6 s (21.7x). Single binary, no Python and no binwalk.

2. **Filesystem-aware dispatching.** Before the AI era this was manual trial and error. We classified 308 real firmware images into five filesystem families and gave each its own parser:

   | Family | Example | Layout |
   |---|---|---|
   | MINIFS v1 | Mercury MW/MR | one LZMA stream per file, offsets relative to the first `5A 00 00 80` |
   | MINIFS v3 | Archer C80/C64 | 32-byte header + ToN/ToF/ToC + chunked LZMA |
   | MINIFS v2 | MAC1200RE | `[hash16][comp_len][name_off][decomp_len][0]` interleaved with streams |
   | OW | MR900 | 32-byte `OW` magic inside an LZMA stream + 76-byte records |
   | PK11 | Netgear CHK | `PK11` + `[name\0][u32 LE size]`, data laid out sequentially |

3. **New UI capability (CLI).** `vx profile` reports the detected type and entropy of every block; `vx entropy` marks high-entropy ranges (`0xstart-0xend`) and judges whether they were decoded or are likely encrypted, so analysts can read the firmware layout at a glance.

4. **Reproducible corpus.** Vendor, model, version, sha256, source URL and filesystem family for all 308 samples: [`stats/CORPUS.csv`](stats/CORPUS.csv).

## Build

```bash
git clone https://github.com/0xba1100n/vxfile_extractor.git
cd vxfile_extractor
git checkout v3.0.0

cd rust/vxcore
cargo build --release
./target/release/vx --version     # vx 3.0.0
```

Cross compilation (requires `rustup target add <target>` and a linker):

```bash
cargo build --release --target aarch64-unknown-linux-musl
cargo build --release --target x86_64-pc-windows-msvc
cargo build --release --target aarch64-apple-darwin
```

Or grab a prebuilt binary from the release (10 targets built by CI):

| OS | Arch | Asset |
|---|---|---|
| Linux | x86_64 | `VxFirmwareExtractor_linux_x86_64_gnu` / `_musl` |
| Linux | aarch64 | `VxFirmwareExtractor_linux_aarch64_gnu` / `_musl` |
| Linux | armv7 | `VxFirmwareExtractor_linux_armv7_musl` |
| Windows | x86_64 / i686 / aarch64 | `VxFirmwareExtractor_windows_*.exe` |
| macOS | x86_64 / aarch64 | `VxFirmwareExtractor_macos_*` |

<https://github.com/0xba1100n/vxfile_extractor/releases/tag/v3.0.0>

## Usage

```bash
vx verdict   fw.bin                  # is it VxWorks?
vx scan      fw.bin                  # list containers / layers
vx tables    fw.bin                  # dump filename tables
vx symbols   fw.bin                  # dump symbol names
vx extract   fw.bin -o out/          # files + symbols + summary
vx profile   fw.bin --block 1024 --ascii   # per-block type and entropy

# batch
vx batch         . -o stats/RUST_METRICS.csv   # three metrics (19 min / 308 samples)
vx profile-batch . -o stats/PROFILE.csv        # type shares (13 s)
vx entropy       . -o stats/ENTROPY.csv        # high-entropy ranges (10 s)
```

Output of `vx extract`:

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
$ vx extract samples/tp-link/Archer_C80_US_V1_191122.zip -o out/
[+] Output layout
    bin/   8 file(s)  <- executable / firmware image candidates
    src/   285 file(s)  <- static resources (js 166, html 81, css 13, json 9, xml 6)
    cfg/   4 file(s)  <- certificates / configs
[*] Likely main logic: kernel image layer 0x158910 (3947396 B)
[!] high-entropy 0x00158C00-0x0027E800 (1203 KiB, 99.4%) -> compressed (data was recovered)
```

Per-block profile:

```
$ vx profile samples/expand-a/inner/MW300R__201275172121.bin --block 1024
0x00000000-0x00000400       1024 B  ent  7.43  data
0x00000400-0x00101000    1051648 B  ent  7.81  high-entropy
0x0012C000-0x0012C400       1024 B  ent  7.82  lzma
kind                  bytes  regions   share
high-entropy        1298432        3   99.2%
[entropy 0..8] @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@#@@@@@@@@@@@@@@@
```

Encrypted verdict:

```
$ vx extract samples/expand-a/inner/FW300R__2012126162529.bin -o out/
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

winmt, 大肥鱼
