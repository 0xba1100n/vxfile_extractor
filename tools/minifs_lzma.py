#!/usr/bin/env python3
# coding: utf-8
"""MINIFS v1 解包：表 + 逐文件 LZMA 流。

结构（来自 H4lo/MINIFS-Decompression 与 arXiv 2407.05064）：
  MINIFS 头 16 字节 magic + 4 字节 flag
  file_info[]: [size:4BE][offset:4BE][name:0x50]，stride 0x58
  offset 相对第一个 LZMA 流（5A 00 00 80），每个文件一条独立流，
  流的结束位置 = 下一条流的起点。
"""
import lzma
import os
import struct

from pathsafe import sanitize_rel, lzma_try

MAGIC = b"MINIFS"
LZ_PROP = b"\x5a\x00\x00\x80"


def parse(data):
    m = data.find(MAGIC)
    if m < 0:
        return None
    base = data.find(LZ_PROP, m)
    if base < 0:
        return None
    ents = []
    p = m + 20
    while p + 88 <= len(data):
        size, off = struct.unpack(">II", data[p:p + 8])
        name = data[p + 8:p + 88].split(b"\x00")[0]
        if not name.startswith(b"/"):
            break
        ents.append((name.decode("latin1", "ignore"), size, off))
        p += 88
    if not ents:
        return None
    ents.sort(key=lambda e: e[2])
    return {"magic": m, "base": base, "entries": ents}


def extract(data, out_dir, limit=None):
    """返回 [(name, size, ok)]"""
    info = parse(data)
    if not info:
        return []
    base, ents = info["base"], info["entries"]
    out = []
    for i, (name, size, off) in enumerate(ents):
        start = base + off
        end = base + ents[i + 1][2] if i + 1 < len(ents) else len(data)
        if not (0 <= start < end <= len(data)):
            out.append((name, size, False))
            continue
        try:
            dec = lzma_try(data[start:end])
        except Exception:
            dec = None
        if dec is None:
            out.append((name, size, False))
            continue
        rel = sanitize_rel(name.lstrip("/"))
        dst = os.path.join(out_dir, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True) if os.path.dirname(dst) else None
        try:
            with open(dst, "wb") as f:
                f.write(dec)
        except Exception:
            out.append((name, size, False))
            continue
        out.append((name, len(dec), True))
        if limit and len([x for x in out if x[2]]) >= limit:
            break
    return out


if __name__ == "__main__":
    import sys
    p = sys.argv[1]
    data = open(p, "rb").read()
    res = extract(data, "minifs_out")
    ok = sum(1 for _n, _s, o in res if o)
    print("entries %d, extracted %d" % (len(res), ok))

# ---------------------------------------------------------------------------
# 分块版 MiniFS（v3：ToN/ToF/ToC + chunk 级 LZMA）
# 参考：https://github.com/RedaOps/minifs-extractor
# ---------------------------------------------------------------------------
def parse_chunked(data):
    m = data.find(MAGIC)
    if m < 0:
        return None
    try:
        files_no = struct.unpack(">I", data[m+0x14:m+0x18])[0]
        ton_size = struct.unpack(">I", data[m+0x1c:m+0x20])[0]
    except Exception:
        return None
    if not (0 < files_no < 5000) or not (0 < ton_size < (1 << 22)):
        return None
    ton = m + 32
    tof = ton + ton_size
    toc = tof + 20 * files_no
    if toc + 12 > len(data):
        return None
    ents = []
    for k in range(files_no):
        try:
            po, no, ch, oc, sz = struct.unpack(">IIIII", data[tof+20*k:tof+20*k+20])
        except Exception:
            return None
        ents.append((po, no, ch, oc, sz))
    chunks_no = ents[-1][2] + 1
    raw = toc + 12 * chunks_no
    if raw + 4 > len(data) or data[raw:raw+4] != b"\x5d\x00\x00\x80":
        return None
    chunks = []
    for k in range(chunks_no):
        chunks.append(struct.unpack(">III", data[toc+12*k:toc+12*k+12]))
    return {"magic": m, "ton": ton, "tof": tof, "toc": toc, "raw": raw,
            "files": ents, "chunks": chunks, "data": data}


def _cstr(data, off, limit=512):
    e = data.find(b"\x00", off, off + limit)
    if e < 0:
        e = off + limit
    return data[off:e].decode("latin1", "ignore")


def extract_chunked(data, out_dir):
    info = parse_chunked(data)
    if not info:
        return []
    cache = {}
    out = []
    for (po, no, ch, oc, sz) in info["files"]:
        name = _cstr(data, info["ton"] + no)
        path = _cstr(data, info["ton"] + po)
        if not name or ch >= len(info["chunks"]):
            out.append((name or "?", 0, False))
            continue
        if ch not in cache:
            co, cs, ds = info["chunks"][ch]
            try:
                dec = lzma_try(data[info["raw"] + co: info["raw"] + co + cs])
            except Exception:
                cache[ch] = None
                out.append((name, sz, False))
                continue
            cache[ch] = dec
        blob = cache[ch]
        if blob is None:
            out.append((name, sz, False))
            continue
        body = blob[oc:oc+sz]
        rel = sanitize_rel((path.strip("/") + "/" + name).lstrip("/"))
        dst = os.path.join(out_dir, rel)
        dd = os.path.dirname(dst)
        if dd:
            os.makedirs(dd, exist_ok=True)
        try:
            with open(dst, "wb") as f:
                f.write(body)
        except Exception:
            out.append((name, sz, False))
            continue
        out.append((name, len(body), True))
    return out

# ---------------------------------------------------------------------------
# v2：头 32 字节 + [hash 16B + 压缩长度 + name_off + 解压长度 + 0] + 流，交替排列
# 名字区无明文（已哈希/加密），只能按内容给个通用名。
# ---------------------------------------------------------------------------
def extract_v2(data, out_dir):
    m = data.find(MAGIC)
    if m < 0:
        return []
    try:
        ver = struct.unpack(">I", data[m+0x10:m+0x14])[0]
        files_no = struct.unpack(">I", data[m+0x14:m+0x18])[0]
    except Exception:
        return []
    if ver != 2 or not (0 < files_no < 5000):
        return []
    os.makedirs(out_dir, exist_ok=True)
    pos = m + 0x20
    out = []
    for k in range(files_no):
        if pos + 32 > len(data):
            break
        try:
            csize, noff, dsize, _z = struct.unpack(">IIII", data[pos+16:pos+32])
        except Exception:
            break
        body = data[pos+32: pos+32+csize]
        if len(body) != csize:
            break
        try:
            dec = lzma_try(body)
        except Exception:
            dec = None
        if dec is not None:
            ext = ".bin"
            head = dec[:4096]
            low = head.lstrip()
            if low[:1] == b"<":
                ext = ".html"
                if b"<?xml" in head.lower():
                    ext = ".xml"
            elif head.startswith(b"\x89PNG"):
                ext = ".png"
            elif head.startswith(b"GIF8"):
                ext = ".gif"
            elif head.startswith(b"-----BEGIN"):
                ext = ".pem"
            elif b"function" in head[:2048] or b"var " in head[:2048]:
                ext = ".js"
            name = "v2_%03d_%08X%s" % (k, pos, ext)
            with open(os.path.join(out_dir, name), "wb") as f:
                f.write(dec)
            out.append((name, len(dec), True))
        pos = pos + 32 + csize
    return out
