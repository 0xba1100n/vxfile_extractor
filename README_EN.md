# vxfile_extractor V3.0.0

**Major update.** 300+ VxWorks firmware models covered. Unpack success **100%**, symbol-table detection **33.4%**, original-filename recovery **53.2%**.

## Rates

| Metric | Result | Definition |
|---|---:|---|
| Unpack | 308 / 308 = **100%** | at least one artifact per firmware |
| └ whole-layer | 209 / 308 = 67.9% | byte coverage of the decoded container >= 90% |
| Original filenames | 164 / 308 = 53.2% | real names restored, not generated ones |
| Symbol names | 103 / 308 = 33.4% | >= 20 symbols found |

Per-firmware numbers: [`stats/RUST_METRICS.csv`](stats/RUST_METRICS.csv).

## Highlights

1. Rewritten in Rust, ~20x faster end to end (4-34 s -> 0.2-1.6 s per sample), single binary, no Python and no binwalk.
2. Five filesystem families reverse engineered from 308 real images: MINIFS v1/v2/v3, OW (MR900), PK11 (Netgear).
3. `vx profile` reports per-block type and entropy; `vx entropy` marks high-entropy ranges as compressed or likely-encrypted.
4. Corpus with models, versions, sha256 and source URLs: [`stats/CORPUS.csv`](stats/CORPUS.csv).

## Build

```bash
cd rust/vxcore
cargo build --release
./target/release/vx --version   # vx 3.0.0
```

Prebuilt binaries for linux (x86_64/aarch64/armv7), windows (x86_64/i686/aarch64) and macOS (x86_64/aarch64) are attached to the release.

## Usage

```bash
vx verdict fw.bin
vx extract fw.bin -o out/
vx profile fw.bin --block 1024 --ascii
vx batch . -o stats/RUST_METRICS.csv
```

See [README.md](README.md) for details.
