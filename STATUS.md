# 当前状态（STATUS）

> 本页是 `CAPABILITY.md`（唯一验收口径）的摘要，由 `results.csv` + `CORPUS.csv` 实测生成。

## 语料

| 项 | 值 |
|---|---|
| 确认样本 | **308** |
| 不同型号 | 148 |
| 厂商 | mercury 143、tp-link 58、tektronix 45、fast 14、anritsu 12、linksys 10、netgear 9、belkin 7、trendnet 7、intellistar 2、nortel 1 |

## 三项能力（全量）

| 能力 | 命中 | 占比 |
|---|---|---|
| 正常解包 | 298 | 97% |
| 恢复符号表 | 116 | 38%（符号名 824,018 个）|
| 恢复文件名（有表，强证据） | 209 | 68% |
| 恢复文件名（仅名字，弱证据） | 27 | 9% |
| 文件名 + 内容 | 69 | 22% |

## 按型号族

| 族 | 样本 |
|---|---|
| mercury:opaque | 103 |
| tektronix:instrument | 45 |
| mercury:minifs | 40 |
| tp-link:legacy | 34 |
| tp-link:rtos | 23 |
| fast:opaque | 13 |
| anritsu:instrument | 12 |
| linksys:troc | 9 |
| netgear:chk | 9 |
| trendnet:other | 7 |
| belkin:deflate-stub | 6 |
| intellistar:other | 2 |
| belkin:other | 1 |
| fast:minifs | 1 |
| linksys:other | 1 |
| nortel:other | 1 |
| tp-link:other | 1 |

## 工具

| 文件 | 作用 |
|---|---|
| `tools/vxbinwalk.py` | binwalk 兼容层：zip/rar/gzip/bz2/xz/LZMA/裸 LZMA1/DEFLATE/uImage/TRX/DLOB/Ubicom/TLV + 图片/HTML 抽取 |
| `tools/vxsym.py` | 符号表检测：压缩/未压缩、NUL 对齐、裸 LZMA1、递归层 |
| `tools/vxtable.py` | 文件名表检测：6 种布局（MINIFS 定长/名字池、stride、TROC 等）|
| `tools/verdict.py` | 判定：明文/zip/rar/LZMA/deflate 全链路 |
| `tools/run_extractor.py` / `batch_run.py` | 批量跑 / 多进程+超时+断点续跑 |
| `tools/capability_report.py` | 生成 CAPABILITY.md / CAPABILITY.csv |
| `tools/eval_suite.py` / `eval_compare.py` | 分层抽样评测 + 防造假 + 回归对比 |

## 复现

```powershell
python tools\run_extractor.py --jobs 4
python tools\capability_report.py
python tools\smoke_test.py
python tools\eval_suite.py --subset 44 --group-by family --out-json eval_new.json
python tools\eval_compare.py eval_baseline.json eval_new.json
```

## 已知天花板

- **Mercury 加密内核**：符号被剥、文件系统整块加密（熵 7.98），无密钥不可恢复。
- **TP-Link 老机型纯内核**：固件里没有文件系统。
- **Netgear/Belkin**：无目录表，文件名只能从 HTML 引用收割（弱证据）。
- **仪器**（泰克/Anritsu）：有符号表但无文件系统。
