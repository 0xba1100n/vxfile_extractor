#!/usr/bin/env python3
# coding: utf-8
"""通用文件系统解包：PK11（Netgear）、OW（MR900）、MINIFS 三族。

对给定的 blob（原始镜像或解压层）依次尝试，返回解出的文件列表。
"""
import lzma
import os
import struct

from pathsafe import sanitize_rel


def _mkpath(out_dir, rel):
    rel = sanitize_rel(rel.lstrip("/"))
    dst = os.path.join(out_dir, rel)
    dd = os.path.dirname(dst)
    if dd:
        os.makedirs(dd, exist_ok=True)
    return dst


def extract_pk11(blob, out_dir):
    """Netgear CHK：PK11 + [name\\0][u32 LE size]... 数据顺序排列。"""
    out = []
    i = blob.find(b"PK11")
    while i >= 0:
        pos = i + 4
        while pos < len(blob) and blob[pos] == 0:
            pos += 1
        ents = []
        q = pos
        while q < len(blob) - 8 and len(ents) < 2000:
            e = blob.find(b"\x00", q, q + 80)
            if e < 0:
                break
            name = blob[q:e]
            if not name or any(c < 32 or c > 126 for c in name):
                break
            size = struct.unpack("<I", blob[e+1:e+5])[0]
            if size == 0 or size > 0x200000:
                break
            ents.append((name.decode("latin1"), size))
            q = e + 5
        data_pos = q
        n_ok = 0
        for name, size in ents:
            if data_pos + size > len(blob):
                break
            body = blob[data_pos:data_pos+size]
            if any(c == 0 for c in body[:4]) and not name.endswith((".gif", ".png", ".ico")):
                # 数据开始就不像文本，可能布局不同，放弃这条链
                pass
            try:
                with open(_mkpath(out_dir, name), "wb") as f:
                    f.write(body)
            except Exception:
                break
            data_pos += size
            n_ok += 1
        if n_ok:
            out.append(("PK11@0x%X" % i, n_ok, True))
        i = blob.find(b"PK11", i + 4)
    return out


def extract_ow(blob, out_dir):
    from lzma_carve import parse_ow
    ow = parse_ow(blob)
    if not ow:
        return []
    n = 0
    for name, body in ow:
        try:
            with open(_mkpath(out_dir, name), "wb") as f:
                f.write(body)
            n += 1
        except Exception:
            pass
    return [("OW", n, True)] if n else []


def extract_minifs(blob, out_dir):
    import minifs_lzma
    for fn in (minifs_lzma.extract, minifs_lzma.extract_chunked, minifs_lzma.extract_v2):
        try:
            r = fn(blob, out_dir)
        except Exception:
            r = []
        r = [x for x in r if x[2]]
        if r:
            return r
    return []


def extract_all(blob, out_dir):
    """返回 {"label": 名称, "count": 文件数}，未命中返回 None。"""
    for fn, label in ((extract_minifs, "MINIFS"), (extract_ow, "OW"),
                      (extract_pk11, "PK11"), (extract_raw_kernel, "raw-kernel")):
        try:
            r = fn(blob, out_dir)
        except Exception:
            r = []
        if not r:
            continue
        if fn is extract_minifs:
            n = sum(1 for x in r if len(x) > 2 and x[2])
        elif fn is extract_raw_kernel:
            n = int(r["count"])
        else:
            n = sum(int(x[1]) for x in r)
        if n:
            return {"label": r["label"] if isinstance(r, dict) else label, "count": n}
    return None


if __name__ == "__main__":
    import sys, zipfile
    p = sys.argv[1]
    if zipfile.is_zipfile(p):
        z = zipfile.ZipFile(p)
        d = z.read(max(z.infolist(), key=lambda i: i.file_size))
    else:
        d = open(p, "rb").read()
    r = extract_all(d, "fs_out")
    print("results:", r)

# ---------------------------------------------------------------------------
# 裸内核 / 带头内核
#   情况一：整片 flash 镜像，含 "WIND version"，没有容器
#   情况二：20 字节头（u32 BE = 载荷长度）+ 裸 ARM 内核
# ---------------------------------------------------------------------------
def _arm_prologue(blob, off):
    """判断 off 处是否像 ARM 指令：条件码 AL + 数据处理/分支。"""
    if off + 8 > len(blob):
        return False
    import struct as _s
    a = _s.unpack("<I", blob[off:off+4])[0]
    b = _s.unpack("<I", blob[off+4:off+8])[0]
    def ok(x):
        if (x >> 28) != 0xE:
            return False
        return ((x >> 26) & 3) == 0 or ((x >> 25) & 7) == 5
    return ok(a) and ok(b)


def detect_raw_kernel(blob):
    """返回 (类别, 头部长度) 或 None。"""
    if len(blob) < 64:
        return None
    import struct as _s
    # 带头内核：u32 BE 恰好等于 len-20，且 0x14 处是 ARM 代码
    if len(blob) > 64:
        n = _s.unpack(">I", blob[:4])[0]
        if n == len(blob) - 20 and _arm_prologue(blob, 20):
            return ("arm-kernel-with-20b-header", 20)
    # 裸内核：有 WIND version，且没有已知文件系统
    if b"WIND version" in blob[: (8 << 20)]:
        return ("flat-vxworks-kernel", 0)
    return None


def extract_raw_kernel(blob, out_dir):
    info = detect_raw_kernel(blob)
    if not info:
        return None
    label, hdr = info
    body = blob[hdr:]
    os.makedirs(out_dir, exist_ok=True)
    try:
        with open(os.path.join(out_dir, "kernel.bin"), "wb") as f:
            f.write(body)
    except Exception:
        return None
    return {"label": label, "count": 1}
