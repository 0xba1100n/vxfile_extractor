#!/usr/bin/env python3
"""给 CORPUS.csv 加 family 列（按输入侧结构特征分组，不按结果，避免选择偏差）。"""
import csv, os, subprocess, sys, tempfile, zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def load(path):
    try:
        if zipfile.is_zipfile(path):
            with zipfile.ZipFile(path) as z:
                return z.read(max(z.infolist(), key=lambda i: i.file_size))
    except Exception:
        pass
    try:
        if path.lower().endswith((".rar", ".7z")):
            tmp = tempfile.mkdtemp()
            subprocess.run([r"C:\Program Files\7-Zip\7z.exe", "x", "-y", "-o" + tmp, path],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=120)
            fs = [os.path.join(dp, f) for dp, _, fs in os.walk(tmp) for f in fs]
            if fs:
                return open(max(fs, key=os.path.getsize), "rb").read()
            return b""
    except Exception:
        pass
    try:
        return open(path, "rb").read()
    except Exception:
        return b""


def family(row, data):
    v = (row.get("vendor") or "").lower()
    model = str(row.get("model", "")) + " " + str(row.get("file", ""))
    m = model.upper()
    if v == "tp-link":
        if any(k in m for k in ("ARCHER C64", "ARCHER C80", "WR820N", "C64(EU", "C80(EU")):
            return "rtos"
        if "TL-" in m or "TL_" in m:
            return "legacy"
        return "other"
    if v == "mercury":
        return "minifs" if b"MINIFS" in data else "opaque"
    if v == "linksys":
        return "troc" if b"TROC" in data else "other"
    if v == "belkin":
        return "deflate-stub" if b"\x78\x9c" in data[0x3000:0x5000] else "other"
    if v == "netgear":
        return "chk"
    if v in ("tektronix", "anritsu", "keysight", "lecroy", "agilent"):
        return "instrument"
    if v in ("fast",):
        return "minifs" if b"MINIFS" in data else "opaque"
    return "other"


def main():
    p = os.path.join(ROOT, "CORPUS.csv")
    rows = list(csv.DictReader(open(p, encoding="utf-8")))
    for i, r in enumerate(rows, 1):
        if r.get("family"):
            continue
        path = os.path.join(ROOT, r["file"])
        d = load(path) if os.path.exists(path) else b""
        r["vendor"] = (r.get("vendor") or "").strip().lower()
        r["family"] = family(r, d)
        if i % 25 == 0:
            print("  %d/%d" % (i, len(rows)), flush=True)
    fields = list(rows[0].keys())
    if "family" not in fields:
        fields.append("family")
    with open(p, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(rows)
    import collections
    c = collections.Counter((r["vendor"], r["family"]) for r in rows)
    for k, n in sorted(c.items()):
        print("%-12s %-12s %d" % (k[0], k[1], n))


if __name__ == "__main__":
    main()
