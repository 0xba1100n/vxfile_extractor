# 编译产物与跨平台构建矩阵

## 命名规则

```
VxFirmwareExtractor_<os>_<arch>.<ext>
```
例：`VxFirmwareExtractor_linux_x86_64.elf`、`VxFirmwareExtractor_windows_x86_64.exe`。

## 扩展名是什么意思

| 扩展名 | 真实格式 | 用在哪 |
|---|---|---|
| `.elf` | **ELF**（Executable and Linkable Format） | Linux / BSD / 各种嵌入式 Unix。Linux 上其实可以不写扩展名，加 `.elf` 只是为了和 Windows 产物区分 |
| `.exe` | **PE / COFF**（Portable Executable） | Windows（所有架构） |
| `.macho` | **Mach-O** | macOS / iOS。Apple 官方分发一般不用扩展名，而是打包成 `.dmg` 或 `.tar.gz`；这里用 `.macho` 仅为一眼能认出格式 |
| `.apk` / `.ipa` | Android / iOS 应用包 | 本项目不涉及 |

补充：Linux 的 `.so` 是动态库，Windows 的 `.dll` 是动态库，`.a` / `.lib` 是静态库——我们的产物是**单个可执行文件**，不涉及这些。

## 构建矩阵

| # | 目标三元组 | 产物名 | 格式 | 状态 |
|---|---|---|---|---|
| 1 | `x86_64-unknown-linux-gnu` | `VxFirmwareExtractor_linux_x86_64.elf` | ELF | built |
| 2 | `aarch64-unknown-linux-gnu` | `VxFirmwareExtractor_linux_aarch64.elf` | ELF | needs rust-std + aarch64 gcc (network) |
| 3 | `x86_64-unknown-linux-musl` | `VxFirmwareExtractor_linux_x86_64_musl.elf` | ELF (static) | needs musl-gcc + rust-std (network) |
| 4 | `x86_64-pc-windows-msvc` | `VxFirmwareExtractor_windows_x86_64.exe` | PE | needs MSVC + rust-std (network / Windows host) |
| 5 | `aarch64-pc-windows-msvc` | `VxFirmwareExtractor_windows_aarch64.exe` | PE | needs MSVC ARM toolchain + rust-std |
| 6 | `x86_64-apple-darwin` | `VxFirmwareExtractor_macos_x86_64.macho` | Mach-O | needs Apple SDK (macOS runner) |
| 7 | `aarch64-apple-darwin` | `VxFirmwareExtractor_macos_aarch64.macho` | Mach-O | needs Apple SDK (macOS runner) |

## 本机构建能力（实测）

- Windows 主机：**没有安装 Rust**（`cargo` / `rustc` / `rustup` 均不在 PATH）。
- WSL Ubuntu 22.04：有 `cargo 1.85.0`，但**网络全断**（crates.io 与 aliyun / ustc / sjtu 镜像均返回 `000`）。
- 因此只能离线编译，依赖 `~/.cargo/registry` 里已缓存的 289 个 crate。
- `rust-std` 只装了 `x86_64-unknown-linux-gnu` 一个目标；`rustup` 不存在，无法添加其它 target。
- **macOS 目标必须有 Apple SDK**，在 Linux/Windows 上无法官方交叉编译（osxcross 属于非官方且需要 Apple 的 SDK 文件）。

结论：本机可产出 **1 / 7** 个目标（Linux x86_64）。其余目标需要在有网络的环境执行：

```bash
rustup target add <target>
cd rust/vxcore && cargo build --release --target <target>
```
或者直接推 GitHub，用仓库里的 `.github/workflows/release.yml`（已配置 6 个 runner，含 macOS）。

## 已产出

| 文件 | 大小 | SHA-256 |
|---|---|---|
| `VxFirmwareExtractor_linux_x86_64.elf` | 1,288,288 bytes | `6d3d8069f1541fd95c9a3e5552e4cbdd5492e93a333930bbb96f83ef86c03a78` |

## 验证方式

```bash
./dist/VxFirmwareExtractor_linux_x86_64.elf --version
./dist/VxFirmwareExtractor_linux_x86_64.elf verdict <firmware>
./dist/VxFirmwareExtractor_linux_x86_64.elf extract <firmware> -o out/
```
