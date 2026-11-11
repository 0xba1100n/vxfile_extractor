#!/usr/bin/env python3
"""四类封装的冒烟测试：改完代码跑一遍，确认没有回归。

覆盖：TP-Link RTOS 壳（C80，符号表+命名恢复）、Mercury MINIFS 表导出、
Linksys 明文镜像、Netgear CHK（裸 LZMA）。

用法：python tools\smoke_test.py [--keep]
"""
import os
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TOOL = os.path.join(ROOT, "tools", "vxfile_extractor", "vxfile_extracter.py")

CASES = [
    {
        "name": "tp-link-c80",
        "file": os.path.join(ROOT, "samples", "tp-link", "Archer_C80_US_V1_191122.zip"),
        "must": ["SYMBOL_Table"],
        "min_files": 3,
        "desc": "TP-Link RTOS 壳：应解出符号表 + 至少 3 个经内容校验的命名文件",
    },
    {
        "name": "mercury-minifs",
        "file": os.path.join(ROOT, "samples", "mercury", "MW313R_V4.0_160602.zip"),
        "must": ["MINIFS_Table.csv"],
        "min_files": 0,
        "desc": "Mercury MINIFS：应导出文件表（>=100 条）",
    },
    {
        "name": "linksys-plain",
        "file": os.path.join(ROOT, "samples", "jp", "linksys_WRT54Gv5v6_1.02.5.bin"),
        "must": [],
        "min_files": 40,
        "desc": "Linksys 明文镜像：应 carve 出大量内嵌文件",
    },
    {
        "name": "netgear-chk",
        "file": os.path.join(ROOT, "samples", "netgear", "WGR614v3_FW2.00.zip"),
        "must": [],
        "min_files": 5,
        "desc": "Netgear CHK（裸 LZMA1）：应解出至少 5 个文件",
    },
]


def find_tree(root, needles):
    hits = {n: None for n in needles}
    for dp, _, fs in os.walk(root):
        for f in fs:
            for n in needles:
                if f == n and hits[n] is None:
                    hits[n] = os.path.join(dp, f)
    return hits


def count_files(root):
    return sum(len(fs) for _, _, fs in os.walk(root))


def run_case(case):
    tmp = tempfile.mkdtemp(prefix="vxsmoke_")
    try:
        src = case["file"]
        if not os.path.exists(src):
            return False, "样本不存在: %s" % src
        r = subprocess.run([sys.executable, TOOL, src], cwd=tmp,
                           capture_output=True, text=True, encoding="utf-8",
                           errors="replace", timeout=1800)
        log = (r.stdout or "") + (r.stderr or "")
        if "Traceback" in log:
            return False, "出现 traceback"
        hits = find_tree(tmp, case["must"])
        missing = [n for n, p in hits.items() if p is None]
        if missing:
            return False, "缺少产物: %s" % ",".join(missing)
        if "MINIFS_Table.csv" in hits and hits["MINIFS_Table.csv"]:
            n = sum(1 for _ in open(hits["MINIFS_Table.csv"], encoding="utf-8")) - 1
            if n < 100:
                return False, "MINIFS 表只有 %d 条" % n
        nf = count_files(tmp)
        if nf < case["min_files"]:
            return False, "只产出 %d 个文件（期望 >=%d）" % (nf, case["min_files"])
        return True, "%d 个文件" % nf
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def main():
    keep = "--keep" in sys.argv
    ok = 0
    for c in CASES:
        passed, detail = run_case(c)
        print("%-4s %-16s %s" % ("PASS" if passed else "FAIL", c["name"], detail))
        print("      %s" % c["desc"])
        ok += passed
    print("\n%d/%d 通过" % (ok, len(CASES)))
    return 0 if ok == len(CASES) else 1


if __name__ == "__main__":
    sys.exit(main())
