#!/usr/bin/env python3
"""Run vxfile_extractor over every VxWorks sample (inside WSL native fs, exFAT-safe)."""
import csv, os, re, shutil, subprocess, sys, zipfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from validate_recovery import looks_ok
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORKBIN = os.path.join(ROOT, "work", "bin")
RUNS = os.path.join(ROOT, "runs")
TOOL = "/mnt/e/ten_nday_difficult_exploit_homework/VxWorks_Firmwares/tools/vxfile_extractor/vxfile_extracter.py"
RESULTS = os.path.join(ROOT, "results.csv")
FUZZY = "--fuzzymode" in sys.argv

OK_MARKERS = ["文件偏移表所在位置", "已重命名文件", "最后匹配到的存在文件偏移表文件集"]
FAIL_MARKERS = {
    "no_table": "找不到binwalk -Me解压后的、明文可接触的文件偏移表",
    "hidden_table": "此形态的vxworks固件由于文件偏移表极有可能被以某种形式隐藏",
    "no_compressed": "未找到压缩数据",
    "no_binwalk": "未找到 binwalk",
}


def slug(path):
    b = os.path.basename(path)
    b = re.sub(r"[^A-Za-z0-9._-]+", "_", b)
    return b[:70]


def prepare_bin(row):
    src = row["file"]
    if not os.path.isabs(src):
        src = os.path.join(ROOT, src)
    if not os.path.exists(src):
        import glob as _g
        cand = _g.glob(os.path.join(ROOT, "samples", "**", os.path.basename(row["file"])), recursive=True)
        if cand:
            src = cand[0]
        else:
            return None
    name = slug(src)
    if name.lower().endswith(".bin"):
        out = os.path.join(WORKBIN, name)
        if not os.path.exists(out):
            shutil.copyfile(src, out)
        return out
    if src.lower().endswith((".rar", ".7z")):
        out = os.path.join(WORKBIN, name)
        if not os.path.exists(out):
            shutil.copyfile(src, out)
        return out
    if zipfile.is_zipfile(src):
        with zipfile.ZipFile(src) as z:
            cands = [i for i in z.infolist()
                     if not i.is_dir() and re.search(r"\.(bin|img|trx|chk|pkg|fw)$", i.filename, re.I)]
            if not cands:
                # 有些固件成员没有扩展名（TrendNet: FW_TEW-455APBO_EU(v1-8-9)），取最大成员兜底
                cands = [i for i in z.infolist() if not i.is_dir() and i.file_size > 4096]
            if not cands:
                return None
            item = max(cands, key=lambda i: i.file_size)
            out = os.path.join(WORKBIN, slug(item.filename))
            if not os.path.exists(out):
                with z.open(item) as fh, open(out, "wb") as w:
                    shutil.copyfileobj(fh, w)
            return out
    # 未知容器（如伪装成 .zip 的 RAR）也交给工具自己判断
    out = os.path.join(WORKBIN, name)
    if not os.path.exists(out):
        shutil.copyfile(src, out)
    return out


def win_to_wsl(path):
    p = os.path.abspath(path)
    return "/mnt/" + p[0].lower() + p[2:].replace("\\", "/")


def classify(log):
    if "已重命名文件" in log:
        off = re.search(r"偏移\((0x[0-9a-fA-F]+)\)", log)
        return "named_ok", off.group(1) if off else ""
    if any(m in log for m in OK_MARKERS):
        off = re.search(r"偏移\((0x[0-9a-fA-F]+)\)", log)
        return "table_ok", off.group(1) if off else ""
    for tag, m in FAIL_MARKERS.items():
        if m in log:
            return tag, ""
    if "Traceback" in log:
        return "traceback", ""
    if "Operation not permitted" in log:
        return "symlink_err", ""
    return "unknown", ""


def run_one(row):
    name = slug(row["file"])
    rundir = os.path.join(RUNS, name)
    # 每次清空，避免上一轮的文件被重复统计
    if os.path.isdir(rundir):
        shutil.rmtree(rundir, ignore_errors=True)
    os.makedirs(rundir, exist_ok=True)
    nout = 0
    binpath = prepare_bin(row)
    if not binpath:
        return {"sample": os.path.basename(row["file"]), "vendor": row.get("vendor", ""),
                "status": "nobin", "table_offset": "", "files_recovered": 0,
                "vx_version": row.get("version", "")}
    tmp = "/tmp/vxr_" + name
    script = ("rm -rf {t}; mkdir -p {t}; cp '{b}' {t}/input.bin; cd {t}; "
              "python3 {tool} input.bin {fz} 2>&1; echo EXIT=$?").format(
        t=tmp, b=win_to_wsl(binpath), tool=TOOL, fz="--fuzzymode" if FUZZY else "")
    try:
        r = subprocess.run(["wsl.exe", "-d", "Ubuntu-22.04", "--", "bash", "-lc", script],
                           capture_output=True, text=True, encoding="utf-8",
                           errors="replace", timeout=1800)
        log = (r.stdout or "") + (r.stderr or "")
    except subprocess.TimeoutExpired as e:
        log = "TIMEOUT 1800s\n" + (e.stdout or "")
    with open(os.path.join(rundir, "log.txt"), "w", encoding="utf-8") as f:
        f.write(log)
    status, off = classify(log)
    subprocess.run(["wsl.exe", "-d", "Ubuntu-22.04", "--", "bash", "-lc",
                    "cp -r {t}/vxfile_* '{d}/' 2>/dev/null; rm -rf {t}; true".format(
                        t=tmp, d=win_to_wsl(rundir))], capture_output=True, text=True, encoding="utf-8", errors="replace")
    for _, _, files in os.walk(rundir):
        nout += len(files)
    # 统计「内容校验通过」的恢复文件，以及是否提取到符号表
    valid = 0
    for dp, _, fs in os.walk(rundir):
        for f in fs:
            if "result_vxworks_file" in dp and looks_ok(os.path.join(dp, f)):
                valid += 1
    has_sym = any(f == "SYMBOL_Table" for dp, _, fs in os.walk(rundir) for f in fs)
    sym_names = 0
    for dp, _, fs in os.walk(rundir):
        if "SYMBOL_Names.txt" in fs:
            try:
                sym_names = sum(1 for _ in open(os.path.join(dp, "SYMBOL_Names.txt"), encoding="utf-8"))
            except Exception:
                sym_names = 0
            break
    names = 0
    names_only = 0
    for dp, _, fs in os.walk(rundir):
        if "NAMES_ONLY.csv" in fs:
            try:
                names_only = sum(1 for _ in open(os.path.join(dp, "NAMES_ONLY.csv"), encoding="utf-8")) - 1
            except Exception:
                names_only = 0
    for dp, _, fs in os.walk(rundir):
        if "MINIFS_Table.csv" in fs:
            try:
                names = sum(1 for _ in open(os.path.join(dp, "MINIFS_Table.csv"), encoding="utf-8")) - 1
            except Exception:
                names = 0
            break
    if status in ("no_compressed", "unknown", "hidden_table") and nout > 2:
        status = "carved_only"
    return {"sample": os.path.basename(row["file"]), "vendor": row.get("vendor", ""),
            "status": status, "table_offset": off, "files_recovered": nout,
            "vx_version": row.get("version", ""), "valid_files": valid,
            "symbol_table": 1 if has_sym else 0, "symbol_names": sym_names, "names": names, "names_only": names_only}


def main():
    limit = int(sys.argv[sys.argv.index("--limit") + 1]) if "--limit" in sys.argv else 0
    jobs = int(sys.argv[sys.argv.index("--jobs") + 1]) if "--jobs" in sys.argv else 2
    os.makedirs(WORKBIN, exist_ok=True)
    os.makedirs(RUNS, exist_ok=True)
    for name in ("CORPUS.csv", "corpus.csv"):
        corpus = os.path.join(ROOT, name)
        if os.path.exists(corpus):
            break
    else:
        corpus = os.path.join(ROOT, "manifest.csv")
    rows = list(csv.DictReader(open(corpus, encoding="utf-8-sig")))
    rows = [r for r in rows if r.get("file")]
    for r in rows:
        r.setdefault("version", r.get("vx_version", ""))
    if "--vendor" in sys.argv:
        want = sys.argv[sys.argv.index("--vendor") + 1]
        rows = [r for r in rows if r.get("vendor") == want]
    if limit:
        rows = rows[:limit]
    print("[*] %d samples to run" % len(rows))
    fields = ["sample", "vendor", "status", "table_offset", "files_recovered", "vx_version",
              "valid_files", "symbol_table", "symbol_names", "names", "names_only"]
    mode = "w"
    with open(RESULTS, mode, newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        with ThreadPoolExecutor(max_workers=jobs) as ex:
            for i, res in enumerate(ex.map(run_one, rows), 1):
                w.writerow(res)
                f.flush()
                print("  [%3d/%3d] %-12s %-9s files=%-4d %s" % (
                    i, len(rows), res["status"], res["vendor"],
                    res["files_recovered"], res["sample"][:60]), flush=True)


if __name__ == "__main__":
    main()

