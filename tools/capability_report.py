#!/usr/bin/env python3
"""三项能力矩阵：正常解包 / 恢复符号表 / 恢复文件名。

读 results.csv（由 run_extractor.py 产出）生成：
  CAPABILITY.csv  逐样本
  CAPABILITY.md   汇总 + 分厂商 + 三个能力各自的清单
"""
import collections
import csv
import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "results.csv")
CSV_OUT = os.path.join(ROOT, "CAPABILITY.csv")
MD_OUT = os.path.join(ROOT, "CAPABILITY.md")


def _kernel_only(sample, files):
    """只有裸内核判定、没有文件系统、且产物不超过 2 个的样本。"""
    log = os.path.join(ROOT, "runs", sample, "log.txt")
    if not os.path.exists(log):
        return False
    try:
        t = open(log, encoding="utf-8", errors="replace").read()
    except Exception:
        return False
    if files > 2:
        return False
    raw = ("Payload unpacked (arm-kernel-with-20b-header" in t
           or "Payload unpacked (flat-vxworks-kernel" in t)
    fs = ("Payload unpacked (MINIFS" in t or "Payload unpacked (OW" in t
          or "Payload unpacked (PK11" in t)
    return raw and not fs


def load():
    rows = list(csv.DictReader(open(SRC, encoding="utf-8-sig")))
    fam = {}
    try:
        cpath = os.path.join(ROOT, "CORPUS.csv")
        for r in csv.DictReader(open(cpath, encoding="utf-8")):
            f = r.get("family") or ""
            if f:
                fam.setdefault(os.path.basename(r["file"]), f)
    except Exception:
        fam = {}
    out = []
    for r in rows:
        files = int(r.get("files_recovered") or 0)
        valid = int(r.get("valid_files") or 0)
        names = int(r.get("names") or 0)
        sym = str(r.get("symbol_table")) == "1"
        symn = int(r.get("symbol_names") or 0)
        kern = _kernel_only(r.get("sample", ""), files)
        out.append({
            "vendor": r.get("vendor", "?"),
            "sample": r.get("sample", ""),
            "unpack": "yes" if (files > 1 and not kern) else "no",
            "kernel_only": "yes" if kern else "no",
            "files": files,
            "symbol": "yes" if sym else "no",
            "symbol_names": symn,
            "filename_list": names,
            "names_only": int(r.get("names_only") or 0),
            "named_content": valid,
            "status": r.get("status", ""),
            "family": fam.get(r.get("sample", ""), ""),
        })
    return out


def main():
    rows = load()
    n = len(rows)
    up = [r for r in rows if r["unpack"] == "yes"]
    sy = [r for r in rows if r["symbol"] == "yes"]
    fl = [r for r in rows if r["filename_list"] > 0 or r["names_only"] > 0
          or r["named_content"] > 0]
    fonly = [r for r in rows if r["filename_list"] == 0 and r["names_only"] > 0
             and r["named_content"] == 0]
    nc = [r for r in rows if r["named_content"] > 0]
    all3 = [r for r in rows if r["unpack"] == "yes" and r["symbol"] == "yes"
            and (r["filename_list"] > 0 or r["named_content"] > 0)]

    with open(CSV_OUT, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(sorted(rows, key=lambda x: (x["vendor"], x["sample"])))

    L = []
    A = L.append
    A("# 三项能力矩阵\n")
    A("| 能力 | 命中样本 | 占比 |")
    A("|---|---|---|")
    A("| 正常解包（解出 >=1 个内嵌文件） | **%d** | %.0f%% |" % (len(up), 100.0 * len(up) / n))
    A("| 恢复符号表 | **%d** | %.0f%% |" % (len(sy), 100.0 * len(sy) / n))
    A("| └ 其中符号名总数 | %d 个 | — |" % sum(r["symbol_names"] for r in sy))
    A("| 恢复文件名（清单） | **%d** | %.0f%% |" % (len(fl), 100.0 * len(fl) / n))
    A("| └ 其中仅靠名字收割（无偏移） | %d | %.0f%% |" % (len(fonly), 100.0 * len(fonly) / n))
    A("| 恢复文件名（清单+内容） | **%d** | %.0f%% |" % (len(nc), 100.0 * len(nc) / n))
    A("| 三项同时满足 | **%d** | %.0f%% |" % (len(all3), 100.0 * len(all3) / n))
    A("| 样本总数 | %d | 100%% |\n" % n)

    A("## 分厂商\n")
    A("| 厂商 | 样本 | 解包 | 符号表 | 文件名清单 | 文件名+内容 |")
    A("|---|---|---|---|---|---|")
    by = collections.defaultdict(lambda: [0, 0, 0, 0, 0])
    for r in rows:
        d = by[r["vendor"]]
        d[0] += 1
        d[1] += r["unpack"] == "yes"
        d[2] += r["symbol"] == "yes"
        d[3] += (r["filename_list"] > 0 or r["named_content"] > 0)
        d[4] += r["named_content"] > 0
    for v, d in sorted(by.items(), key=lambda x: -x[1][0]):
        A("| %s | %d | %d | %d | %d | %d |" % (v, d[0], d[1], d[2], d[3], d[4]))
    A("")

    if any(r["family"] for r in rows):
        A("## 按型号族\n")
        A("| 族 | 样本 | 解包 | 符号表 | 文件名(有表) | 文件名(仅名字) |")
        A("|---|---|---|---|---|---|")
        byf = collections.defaultdict(lambda: [0, 0, 0, 0, 0])
        for r in rows:
            k = (r["vendor"] + ":" + r["family"]) if r["family"] else r["vendor"]
            d = byf[k]
            d[0] += 1
            d[1] += r["unpack"] == "yes"
            d[2] += r["symbol"] == "yes"
            d[3] += r["filename_list"] > 0
            d[4] += r["names_only"] > 0
        for k, d in sorted(byf.items(), key=lambda x: -x[1][0]):
            A("| %s | %d | %d | %d | %d | %d |" % (k, d[0], d[1], d[2], d[3], d[4]))
        A("")

    A("## 恢复符号表的样本（%d）\n" % len(sy))
    for r in sy:
        A("- [%s] %s — %d 个符号名" % (r["vendor"], r["sample"], r["symbol_names"]))
    A("")

    A("## 能恢复文件名的样本（%d，名字清单共 %d 条）\n"
      % (len(fl), sum(r["filename_list"] for r in fl)))
    for r in sorted(fl, key=lambda x: (-x["filename_list"], -x["named_content"])):
        A("- [%s] %s — 清单 %d 条 / 有效文件 %d 个"
          % (r["vendor"], r["sample"], r["filename_list"], r["named_content"]))
    A("")

    A("## 文件名 + 内容都恢复（%d）\n" % len(nc))
    for r in nc:
        A("- [%s] %s — %d 个有效文件" % (r["vendor"], r["sample"], r["named_content"]))
    A("")

    A("## 完全无产出的样本（%d）\n" % (n - len(up)))
    for r in rows:
        if r["unpack"] != "yes":
            A("- [%s] %s" % (r["vendor"], r["sample"]))
    A("")

    with open(MD_OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(L))
    print("written:", CSV_OUT)
    print("written:", MD_OUT)
    print("unpack=%d symbol=%d names=%d named_content=%d all3=%d total=%d"
          % (len(up), len(sy), len(fl), len(nc), len(all3), n))


if __name__ == "__main__":
    main()
