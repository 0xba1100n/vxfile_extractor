#!/usr/bin/env python3
# coding: utf-8
"""Classify unpacked files into bin/ (main logic) and src/ (static assets).

Layout created next to the extracted tree:
    bin/   executable / firmware image candidates
    src/   static web resources (html/js/css/xml/images/fonts)
    cfg/   certificates and config-ish files

Hardlinks are used when possible, otherwise files are copied.
"""
import os
import shutil

STATIC_EXT = {".htm", ".html", ".js", ".css", ".xml", ".json", ".map", ".svg",
              ".png", ".gif", ".jpg", ".jpeg", ".ico", ".bmp", ".webp",
              ".woff", ".woff2", ".ttf", ".eot", ".otf"}
CFG_EXT = {".pem", ".cer", ".crt", ".key", ".der", ".conf", ".cfg", ".ini",
           ".sh", ".txt", ".html.bak"}
BIN_EXT = {".bin", ".out", ".elf", ".img", ".rom", ".fw", ".axf", ".mot", ".srec"}
ELF_MAGIC = b"\x7fELF"
PE_MAGIC = b"MZ"
VX_MARKERS = (b"vxWorks", b"WIND version", b"symTbl", b"taskSpawn", b"_WRS_")


def _kind(path):
    try:
        with open(path, "rb") as f:
            head = f.read(4096)
    except Exception:
        head = b""
    ext = os.path.splitext(path)[1].lower()
    if head.startswith(ELF_MAGIC):
        return "bin", "ELF"
    if head.startswith(PE_MAGIC) and len(head) > 64:
        return "bin", "PE"
    if ext in STATIC_EXT:
        return "src", ext.lstrip(".")
    if ext in BIN_EXT:
        return "bin", "raw"
    if ext in CFG_EXT:
        return "cfg", ext.lstrip(".")
    if any(m in head for m in VX_MARKERS):
        return "bin", "vxworks-raw"
    return "other", ext.lstrip(".") or "dat"


def _link(src, dst):
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if os.path.exists(dst):
        base, ext = os.path.splitext(dst)
        k = 1
        while os.path.exists("%s_%d%s" % (base, k, ext)):
            k += 1
        dst = "%s_%d%s" % (base, k, ext)
    try:
        os.link(src, dst)
    except Exception:
        try:
            shutil.copy2(src, dst)
        except Exception:
            return None
    return dst


def classify(root):
    """Return dict with counts and main-logic candidates."""
    bins, srcs, cfgs = [], [], []
    for dp, _dn, fns in os.walk(root):
        rel_dir = os.path.relpath(dp, root)
        if rel_dir.split(os.sep)[0] in ("bin", "src", "cfg"):
            continue
        for fn in fns:
            p = os.path.join(dp, fn)
            bucket, sub = _kind(p)
            if bucket == "bin":
                bins.append((p, sub))
            elif bucket == "src":
                srcs.append((p, sub))
            elif bucket == "cfg":
                cfgs.append((p, sub))
    for lst, name in ((bins, "bin"), (srcs, "src"), (cfgs, "cfg")):
        for p, _sub in lst:
            rel = os.path.relpath(p, root)
            _link(p, os.path.join(root, name, rel))
    # main logic candidates: ELF/PE first, then largest raw
    GOOD = ("httpd", "goahead", "webs", "boa", "lighttpd", "cgi", "init", "shell")
    BAD = ("wifi", "ram_code", "mcu", "patch", "hdr", "dat", "nvram", "boot")

    def score(item):
        p, sub = item
        low = os.path.basename(p).lower()
        rank = {"ELF": 0, "PE": 0, "vxworks-raw": 1, "raw": 2}.get(sub, 3)
        if any(k in low for k in BAD):
            rank += 3
        if any(k in low for k in GOOD):
            rank -= 1
        try:
            size = os.path.getsize(p)
        except Exception:
            size = 0
        return (rank, -size)

    raw_kernel = any(os.path.basename(p) == "kernel.bin" for p, _s in bins)
    if raw_kernel:
        bins = [(p, s2) for p, s2 in bins if os.path.basename(p) != "kernel.bin"]
        srcs = srcs
    cands = sorted(bins, key=score)[:3]
    kernel = largest_layer(root)
    return {
        "bin": len(bins) + (1 if raw_kernel else 0), "src": len(srcs), "cfg": len(cfgs),
        "kernel": kernel, "raw_kernel": raw_kernel,
        "src_by_ext": _count_ext(srcs),
        "candidates": [(os.path.relpath(p, root), sub, os.path.getsize(p)) for p, sub in cands],
    }


def largest_layer(root):
    """binwalk 解出来的最大一层，通常就是 VxWorks 内核本体。"""
    best = None
    for dp, _dn, fns in os.walk(root):
        if ".extracted" not in dp:
            continue
        if os.sep + "result_vxworks_file" in dp:
            continue
        for fn in fns:
            p = os.path.join(dp, fn)
            try:
                size = os.path.getsize(p)
            except Exception:
                continue
            if size < 65536:
                continue
            if best is None or size > best[1]:
                best = (p, size)
    if best is None:
        return None
    name = os.path.basename(best[0])
    try:
        label = "layer @0x%X" % int(name, 16)
    except Exception:
        label = name
    return (best[0], best[1], label)


def _count_ext(items):
    out = {}
    for _p, sub in items:
        out[sub] = out.get(sub, 0) + 1
    return out


def format_hint(info):
    lines = []
    lines.append("[+] Output layout")
    if info["bin"]:
        lines.append("    bin/   %d file(s)  <- executable / firmware image candidates" % info["bin"])
    if info["src"]:
        top = ", ".join("%s %d" % (k, v) for k, v in sorted(info["src_by_ext"].items(), key=lambda z: -z[1])[:5])
        lines.append("    src/   %d file(s)  <- static resources (%s)" % (info["src"], top))
    if info["cfg"]:
        lines.append("    cfg/   %d file(s)  <- certificates / configs" % info["cfg"])
    if info.get("raw_kernel"):
        lines.append("[*] Likely main logic: raw kernel image (kernel.bin, header stripped)")
    if info.get("kernel") and not info.get("raw_kernel"):
        p, size, label = info["kernel"]
        lines.append("[*] Likely main logic: kernel image %s (%d B)" % (label, size))
        lines.append("    main HTTP/CGI code lives inside this image, not as a separate file")
    if info["candidates"]:
        lines.append("[*] Other binary candidates:")
        for rel, sub, size in info["candidates"]:
            lines.append("    %-52s %-8s %8d B" % (rel[:52], sub, size))
    elif not info.get("kernel") and not info.get("raw_kernel"):
        lines.append("[*] Likely main logic: none found (no ELF/PE/raw binary in this sample)")
    return "\n".join(lines)


if __name__ == "__main__":
    import sys
    import json
    r = sys.argv[1]
    info = classify(r)
    print(format_hint(info))
    print(json.dumps(info, indent=2))