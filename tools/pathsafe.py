#!/usr/bin/env python3
# coding: utf-8
"""把固件里的原始路径转成 Windows/Linux 都能落盘的安全路径。"""
import os
import re

_BAD = re.compile(r'[<>:"\\|?*\x00-\x1f]')


def sanitize_rel(name):
    parts = []
    for p in (name or "").replace("\\", "/").split("/"):
        p = _BAD.sub("_", p).rstrip(" .")
        if p in ("", ".", ".."):
            p = "_"
        parts.append(p)
    rel = "/".join(parts)
    if len(rel) > 240:
        base, ext = os.path.splitext(rel)
        rel = base[:240 - len(ext)] + ext
    return rel

def lzma_try(blob):
    """尽量解开一条 LZMA-alone 流；尾部有多余字节时保留已解出的部分。"""
    import lzma
    dec = lzma.LZMADecompressor(format=lzma.FORMAT_ALONE)
    out = b""
    step = 65536
    for i in range(0, len(blob), step):
        try:
            out += dec.decompress(blob[i:i + step])
        except lzma.LZMAError:
            break
        if dec.eof:
            break
    if out:
        return out
    try:
        return lzma.decompress(blob, format=lzma.FORMAT_ALONE)
    except Exception:
        return None
