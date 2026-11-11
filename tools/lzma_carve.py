#!/usr/bin/env python3
# coding: utf-8
"""通用兜底：扫出镜像里所有 LZMA-alone 流并落盘。

真实编码器的 dict 都是 2 的幂；据此过滤候选，避免逐字节解压爆炸。
"""
import hashlib
import lzma
import os
import struct

from pathsafe import sanitize_rel, lzma_try

DICT_OK = {1 << i for i in range(12, 29)}  # 4K .. 256M
MIN_OUT = 4096
MAX_STREAMS = 200


def _cands(data):
    n = len(data)
    for o in range(0, n - 16):
        p = data[o]
        if p >= 225:
            continue
        d = struct.unpack("<I", data[o + 1:o + 5])[0]
        if d not in DICT_OK:
            continue
        us = struct.unpack("<Q", data[o + 5:o + 13])[0]
        if us != 0xFFFFFFFFFFFFFFFF and us > (256 << 20):
            continue
        yield o


def _guess_ext(blob):
    b = blob[:16]
    if b.lstrip()[:1] == b"<":
        low = blob[:4096].lower()
        if b"<?xml" in low:
            return ".xml"
        if b"<html" in low or b"<!doctype" in low:
            return ".html"
        return ".html"
    if b.startswith(b"\x89PNG"):
        return ".png"
    if b.startswith(b"GIF8"):
        return ".gif"
    if b.startswith(b"-----BEGIN"):
        return ".pem"
    if b.startswith(b"{\"") or b.startswith(b"{\n"):
        return ".json"
    low = blob[:2048]
    if b"function" in low or b"var " in low:
        return ".js"
    return ".bin"


def carve(data, out_dir, max_streams=MAX_STREAMS):
    os.makedirs(out_dir, exist_ok=True)
    seen = set()
    out = []
    for o in _cands(data):
        if len(out) >= max_streams:
            break
        dec = lzma_try(data[o:o + (64 << 20)])
        if dec is None:
            continue
        if len(dec) < MIN_OUT:
            continue
        ow = parse_ow(dec)
        if ow:
            for name, body in ow:
                rel = sanitize_rel(name.lstrip("/"))
                dst = os.path.join(out_dir, rel)
                dd = os.path.dirname(dst)
                if dd:
                    os.makedirs(dd, exist_ok=True)
                try:
                    with open(dst, "wb") as f:
                        f.write(body)
                except Exception:
                    continue
                out.append((rel, len(body), True))
            continue
        h = hashlib.sha256(dec).hexdigest()[:16]
        if h in seen:
            continue
        seen.add(h)
        ext = _guess_ext(dec)
        name = "stream_%08X%s" % (o, ext)
        try:
            with open(os.path.join(out_dir, name), "wb") as f:
                f.write(dec)
        except Exception:
            continue
        out.append((name, len(dec), True))
    return out


if __name__ == "__main__":
    import sys
    d = open(sys.argv[1], "rb").read()
    r = carve(d, "lzma_out")
    print("streams:", len(r), "bytes:", sum(x[1] for x in r))

# ---------------------------------------------------------------------------
# MR900 系列：LZMA 流里是 "OW"*16 魔数的文件系统
#   头 40 字节：magic[32] + version(4) + count(4)
#   记录 76 字节：name[60] + 0(4) + size(4) + offset(4) + 0(4)
#   offset 相对 blob 起点
# ---------------------------------------------------------------------------
OW_MAGIC = b"OW" * 16


def parse_ow(blob):
    if len(blob) < 0x28 or blob[:32] != OW_MAGIC:
        return None
    try:
        count = struct.unpack(">I", blob[0x24:0x28])[0]
    except Exception:
        return None
    if not (0 < count < 5000) or 0x28 + 76 * count > len(blob):
        return None
    out = []
    for k in range(count):
        off = 0x28 + 76 * k
        name = blob[off:off+60].split(b"\x00")[0].decode("latin1", "ignore")
        _z, size, foff, _c = struct.unpack(">IIII", blob[off+60:off+76])
        if not name or size == 0 or foff + size > len(blob):
            continue
        out.append((name, blob[foff:foff+size]))
    return out or None
