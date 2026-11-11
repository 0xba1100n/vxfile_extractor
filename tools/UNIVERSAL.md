# 普适性改造记录（vxfile_extractor）

目标：让工具不再绑定「binwalk v2 + Linux + 特定厂商偏移表形态」，在 Windows/WSL 都能跑，
并直接吃厂商下发的原始容器。

## 已完成

| 改动 | 位置 | 说明 |
|---|---|---|
| 可插拔解包后端 | `resolve_binwalk()` / `BINWALK` | 优先用同目录的 `tools/vxbinwalk.py`，其次才找系统 `binwalk`；可用 `VXBINWALK` 环境变量指定 |
| 纯 Python binwalk 兼容层 | `tools/vxbinwalk.py` | 支持 zip / gzip / bzip2 / xz / LZMA-alone / 裸 LZMA1 / 裸 DEFLATE / uImage（含压缩负载）/ TP-Link RTOS 壳 / TRX；输出 binwalk v2 风格表格，按 `<UPPER_HEX>` 命名解出文件 |
| 直接吃压缩包 | `normalize_input()` | 输入 `.zip/.rar/.7z` 时自动取**镜像类成员**（.bin/.img/.trx/.chk/.fw/.rom，而非体积最大的说明书 PDF/DOC） |
| 崩溃修复 1 | `rename_extracted_files()` | `re.match` 未命中时 `.group()` 抛 `AttributeError` → 现在回退到 `0xFFFFFFFF` |
| 崩溃修复 2 | `fuzzy_search_file_contain_table()` 调用处 | 该函数返回 dict，原代码当路径用 → `TypeError`；现在取 score 最高的键，取不到就优雅退出 |
| 崩溃修复 3 | `extract_web_source_filenames()` | `grep` 无匹配返回码 1 被 `check=True` 当异常抛 → 现在 `check=False` |
| 主程序兜底 | `main()` | 没有压缩层时，用「解出的最大文件」代替 `NONE` |

## 实测（148 样本，第 21 轮最终）

| 指标 | 数值 |
|---|---|
| 样本总数 | 148 |
| 有产出（解出 >=1 文件） | **106 / 148（72%）** |
| **能导出真实文件名清单** | **36 个样本 / 4,148 条文件名** |
| 文件名 + 内容都恢复 | **2 个样本 / 6 个文件**（Archer C80 v1 2019、2020） |
| 符号表提取 | 15 个样本 |
| 崩溃 | 0 |

范围说明（与用户确认过）：**只恢复「能拿到文件名」的部分**。
- 36 个 Mercury 型号的 MINIFS 表是明文，工具导出 `MINIFS_Table.csv`（真名 + 大小 + 偏移），但表后的 fs 是密文，
  所以只给清单、不给内容；
- C80 的表和数据都在明文 LZMA 层里，名字和内容都能恢复；
- 其余 112 个样本：73 个原始字节里连文件名都没有（纯内核/加密包），39 个有名字但无可用偏移表。

为此修的最后两个 bug：
1. `find_files_offset_table` 返回 None 时 `extract_file_info_type2` 会 `TypeError` 崩溃（MW300R v10 触发）；
2. 未找到表时不再中断，仍会导出清单与 SUMMARY。

## MINIFS 逆向：定论（第 10 轮）

绕了五轮，这轮用「区域大小 + 熵」把结论钉死了：

| 样本 | 表内 fs 跨度 | 表后到文件尾 | 熵 |
|---|---|---|---|
| MW313R v4 | 288,808 B | 292,292 B | **7.98** |
| MW300R v15 | 288,811 B | 292,292 B | **7.98** |

也就是说：**表是明文，表后面紧跟着的那块就是整个文件系统，但它是整块加密的。**
表里的 `offset/size` 描述的是**解密后**的布局，所以在密文上怎么对都对不上——
这正是前面「PNG 投票 2/18」「htm 首字符 4/74」「全层平移最高 5/74」这些失败的原因。

熵 7.98–8.0 说明是强加密（AES 级别），不是 XOR 混淆，没有密钥无法恢复。

结论：**文件名恢复的瓶颈已经从「格式没逆出来」变成「没有密钥」**。
需要的外部输入（任一即可）：
1. 同型号真机上抓一份解密后的 `config.bin`／web 分区；
2. 固件里解密函数的密钥（通常在 ARM 代码里，需要 IDA 逆 `minifs` 的读流程）；
3. 已知的 TP-Link RTOS 解密脚本（如果这代用的是公开方案）。

## 第 6 轮：MINIFS 表导出 + 一个重要负结论

新增能力：工具现在会把 MINIFS 文件表导出成 `MINIFS_Table.csv`（name,size,offset）。
即使内容抽不出来，**原始文件名/大小/偏移**也能先拿到。实测 MW313R v4：124 条，含
`/conf/oem.txt`、`/web/common/AccessCtrl.htm`、`/web/images/advance_me.png` 等，内容正确。

但抽样 15 个 Mercury 型号后得到一个必须记录的事实：**MINIFS 并不通用**。

| 型号 | MINIFS 表 | 说明 |
|---|---|---|
| MW313R v4 / MW300R v15 | 有（明文） | 124 条，可读 |
| MR100P-AC | 有（明文） | 106 条 |
| MAC100 / MAC1200R / MAC1300R / MAC2600R / MAC500 / MAC1200RE | 无或加密 | 14 个抽样里 13 个拿不到表 |

以 MAC1200RE 为例：`MINIFS` 头之后直接是 `00 00 00 02 00 00 00 af` 再往后就是高熵数据——
**表本身被加密**。这说明各型号的 fs 封装并不统一，静态分析能做的就是「遇到明文表就导出、遇到加密就止步」。

## 第 9 轮：结构化输出 SUMMARY.json

每次运行现在会在输出目录写一份 `SUMMARY.json`：

```json
{"input":"c80.bin","size":2617628,"layers":48,
 "vxworks_versions":["5.5.1"],"symbol_table":"SYMBOL_Table",
 "recovered_named":3,"recovered_valid":3,
 "layer_detail":[{"offset":0,"desc":"TP-Link RTOS firmware..."}, ...]}
```

批量分析不用再翻日志：层数、VxWorks 版本（含从符号表里补扫的）、符号表、命名/有效恢复数全在里面。

## 第 7 轮：冒烟测试 + 明文镜像漏抽修复

- 新增 `tools/smoke_test.py`：四类封装各一个用例（C80 符号表+命名、Mercury MINIFS 表、Linksys 明文、Netgear CHK），
  改完代码跑一遍即可确认没有回归。当前 **4/4 通过**。
- 第 8 轮又消灭了 3 个 `nobin`：伪装成 `.zip` 的 RAR（TL-R460、TL-WR340G v2）和 200MB 的 Nortel 镜像
  之前被 harness 直接跳过。现在未知容器一律透传给工具；同时给 `vxbinwalk` 加了 24MB 上限，
  大镜像不再逐字节爆破（Nortel 那个 200MB 图之前会把扫描拖死）。
- 修了一个真 bug：`vxbinwalk` 的 `emit_images` 只对「解出来的层」跑，**明文镜像本身没被扫**，
  导致 Linksys 这类无压缩层的固件签名数为 0。现在输入也参与图片扫描（Linksys 从 0 → 24 个签名）。

## 本轮事故与修复（记录在案）

用 PowerShell 的 `Get-Content -Raw | Set-Content` 改写工具源码时，UTF-8 被按 GBK 往返，全文中文变乱码、并引入非法字符导致无法编译。
处理：`git checkout` 还原 `vxfile_extracter.py`，再用 **Python 脚本**（`encoding="utf-8"`）把全部补丁重新打了一遍，逐条 `assert` 无 miss。
教训：这个文件只能用 Python 改，禁止 PowerShell 文本读写。

## 仍待解决

1. **MINIFS entry 数据块**（上面已述）——最大的一块。
2. **Windows 原生**：仍依赖 `strings` / `grep` / `file`；WSL 下有、Windows 下没有。需把 5 个函数换成纯 Python。
3. **符号表提取**：已改成遍历所有层 + 三种 props（5A/5D/6E），但语料里多数镜像的符号表被裁掉了，实测命中率低；
   建议再加「符号名密度」判定（连续出现多个 `xxxTask`/`xxxInit` 之类的字符串即认定为符号表）。
