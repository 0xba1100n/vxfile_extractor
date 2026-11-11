#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""vxtable.py — 在固件镜像里定位并解析「文件名 / 偏移表」。

对外接口::

    from vxtable import find_tables, Table, mapped_ratio
    for t in find_tables(data):
        print(t.layout, len(t), t.confidence)
        for name, size, offset in t.entries:
            ...

命令行::

    python tools/vxtable.py <file> [<file> ...]      # 逐文件打印找到的表
    python tools/vxtable.py --scan                   # 对 table_scan.csv 里
                                                     # 「有名字」的样本跑一遍，
                                                     # 产出 table_scan2.csv
    python tools/vxtable.py --scan --out xxx.csv

支持的布局
----------
1. ``minifs-fixed``   ``MINIFS`` + 0x10 头，记录 ``[size:4BE][offset:4BE][name:0x50]``，
                      stride 0x58。name 字段长度按 0x50/0x48/0x40/0x30 兼容。
2. ``minifs-pool``    ``MINIFS`` + ``[ver=3][count][data_size][name_block_len]``，
                      后面是名字池（路径/文件名分开放，NUL 分隔），再后面是
                      ``count`` 条 20 字节记录 ``[path_off][name_off][chunk][in_chunk][size]``。
                      数据按 chunk 压缩，raw 偏移未知，所以 offset 记为 -1。
3. ``fixed-stride``   任意定长记录 ``[name:L][size][offset]``：stride、L、端序、对齐
                      全部从数据推断。语料里实测三种：``L=0x28/trailer BE``(Mercury/TP-Link
                      早期)、``L=0x0D/stride 0x18/trailer LE``(Linksys WRT54G v5/v6)、
                      ``L=0x40/stride 0x4C/trailer LE``(Linksys WRT54G2 v1.x)。
   ``offsets-only``    ``[name:L][offset:4]``：只有偏移没有 size（offset 单调递增校验）。
4. ``nameblock``      连续 NUL 分隔的名字块 + 等长的 4 字节偏移数组（数量可能写在
                      名字块前 4 / 12 字节）。
5. ``lenprefix``      长度前缀记录 ``[len][name][size][offset]`` 或
                      ``[size][offset][len][name]``（1/2 字节长度，BE/LE）。

判真原则（防假阳性）
--------------------
* 名字必须是可打印的类文件名（含已知扩展名；定长表允许「字段填满被截断」的名字，
  但比例不超过 35%）。
* 定长表 / 长度前缀表要求 offset 单调且 ``offset[i+1] ≈ offset[i] + size[i]``
  （允许 0x1000 以内的对齐填充），命中率 >= 85%。
* 名字块布局要求偏移数组单调递增，且数量前缀能对上（对不上则降置信度）。
* 空槽（全 0 名字字段）允许跳过，但不计入 entries。
"""

from __future__ import print_function

import csv
import os
import re
import struct
import subprocess
import sys
import tempfile
import zipfile

__all__ = ["Table", "find_tables", "mapped_ratio", "load_container"]

# --------------------------------------------------------------------------
# 常量
# --------------------------------------------------------------------------

MAX_ENTRIES = 4000          # 单表条目上限，防解析跑飞
MAX_VALUE = 0x4000000       # size/offset 上限（64MB），超过视为跑偏
CUM_PAD = 0x1000            # 允许的对齐填充（offset[i+1] - offset[i] - size[i]）
MAX_CUT_RATIO = 0.35        # 截断名字占比上限

_EXT = (b"htm|html|shtml|css|js|xml|png|gif|jpg|jpeg|bmp|ico|swf|"
        b"bin|dat|cfg|ini|txt|pem|cer|crt|key|asp|cgi|json|db|"
        b"img|trx|rom|chk|pdf|zip|gz|tar|so|o|h|ttf|wav")

NAME_HIT = re.compile(rb"[A-Za-z0-9_\-./]{1,80}\.(?:" + _EXT + rb")", re.IGNORECASE)
NAME_FULL = re.compile(rb"[A-Za-z0-9_\-./]{1,80}\.(?:" + _EXT + rb")\Z", re.IGNORECASE)
RAW_OK = re.compile(rb"[A-Za-z0-9_\-./]{4,80}\Z")


# --------------------------------------------------------------------------
# 数据结构
# --------------------------------------------------------------------------

class Table(object):
    """一张「文件名表」。

    属性
    ----
    layout      : 布局名（见模块文档）
    entries     : [(name, size, offset), ...]
    start       : 表在数据里的起始偏移
    end         : 表结束偏移（最后一个字节 +1，未知则 None）
    confidence  : 0..1，越高越可信
    meta        : 布局参数（stride/name_len/endian/count_hint/note ...）
    """

    __slots__ = ("layout", "entries", "start", "end", "confidence", "meta")

    def __init__(self, layout, entries, start, end=None, confidence=0.0, **meta):
        self.layout = layout
        self.entries = list(entries)
        self.start = int(start)
        self.end = None if end is None else int(end)
        self.confidence = float(confidence)
        self.meta = meta

    # -- 便捷属性 --------------------------------------------------------
    @property
    def names(self):
        return [e[0] for e in self.entries]

    @property
    def name_set(self):
        return set(e[0] for e in self.entries)

    def __len__(self):
        return len(self.entries)

    def __iter__(self):
        return iter(self.entries)

    def __repr__(self):
        return "<Table %s n=%d conf=%.2f @0x%X>" % (
            self.layout, len(self.entries), self.confidence, self.start)

    def to_dict(self):
        d = {"layout": self.layout, "entries": len(self.entries),
             "start": self.start, "end": self.end,
             "confidence": round(self.confidence, 3)}
        d.update(self.meta)
        return d


# --------------------------------------------------------------------------
# 名字判定
# --------------------------------------------------------------------------

def _name_kind(nm, field_len=None):
    """返回 'full' / 'cut' / None。

    full: 完整文件名（含已知扩展名）
    cut : 名字把定长字段撑满，扩展名被截断（例如 13 字节字段里的
          ``lastpassword.``）——只在 field_len 已知且确实填满时接受。
    """
    if not nm or len(nm) < 4 or len(nm) > 80:
        return None
    if not RAW_OK.match(nm):
        return None
    if NAME_FULL.match(nm):
        return "full"
    # 名字把定长字段撑满 → 大概率是被截断的长文件名（如 13 字节字段里的
    # "Outbreak_Aler" / "lastpassword."），扩展名本身可能已经被切掉。
    if field_len is not None and len(nm) >= field_len:
        letters = sum(1 for c in nm if (65 <= c <= 90) or (97 <= c <= 122))
        if letters >= 4 and (b"." in nm or len(nm) >= 8):
            return "cut"
    return None


def _decode(nm):
    return nm.decode("latin-1")


def _cum_score(entries, pad=CUM_PAD):
    """offset 连续性的命中率：offset[i+1] 是否约等于 offset[i]+size[i]。"""
    if len(entries) < 2:
        return 0.0
    ok = 0
    for i in range(len(entries) - 1):
        size, off = entries[i][1], entries[i][2]
        size2, off2 = entries[i + 1][1], entries[i + 1][2]
        gap = off2 - (off + size)
        if 0 <= gap <= pad:
            ok += 1
    return float(ok) / (len(entries) - 1)


def _mono_score(offsets):
    if len(offsets) < 2:
        return 0.0
    ok = sum(1 for i in range(len(offsets) - 1) if offsets[i] <= offsets[i + 1])
    return float(ok) / (len(offsets) - 1)


def _name_runs(data, min_run=3, max_gap=0x200):
    """把「像文件名的字符串」按位置聚成候选 run。"""
    hits = [(m.start(), m.group()) for m in NAME_HIT.finditer(data)]
    runs, cur = [], []
    for pos, name in hits:
        if cur and pos - cur[-1][0] > max_gap:
            if len(cur) >= min_run:
                runs.append(cur)
            cur = []
        cur.append((pos, name))
    if len(cur) >= min_run:
        runs.append(cur)
    runs.sort(key=len, reverse=True)
    return runs


# --------------------------------------------------------------------------
# 1) MINIFS 定长表
# --------------------------------------------------------------------------

def _minifs_fixed(data, magic_off):
    """``[size:4BE][offset:4BE][name:0x50]``，stride 0x58（name 字段长度可不同）。"""
    n = len(data)
    best = None
    # (name 字段长度, stride)；第一条记录前 8 字节就是 size/offset
    layouts = ((0x50, 0x58), (0x48, 0x50), (0x40, 0x48), (0x30, 0x38), (0x60, 0x68))
    for name_len, stride in layouts:
        for p0 in range(magic_off + 0x18, min(magic_off + 0x34, n - 8)):
            try:
                size0, off0 = struct.unpack(">II", data[p0 - 8:p0])
            except struct.error:
                continue
            if size0 > MAX_VALUE or off0 > MAX_VALUE:
                continue
            first = data[p0:p0 + name_len].split(b"\0")[0]
            if _name_kind(first) is None:
                continue
            ents = [(_decode(first), size0, off0)]
            p = p0 + stride
            while p + name_len <= n and len(ents) < MAX_ENTRIES:
                field = data[p:p + name_len]
                nm = field.split(b"\0")[0]
                if not nm:
                    break
                if len(nm) < name_len and data[p + len(nm)] != 0:
                    break
                if _name_kind(nm) is None:
                    break
                size, off = struct.unpack(">II", data[p - 8:p])
                if size > MAX_VALUE or off > MAX_VALUE:
                    break
                ents.append((_decode(nm), size, off))
                p += stride
            if len(ents) < 4:
                continue
            cum = _cum_score(ents)
            if cum < 0.75:
                cum = _cum_score(ents[1:])          # 首条可能是证书这种「异类」
            if cum < 0.75:
                continue
            score = (len(ents), cum)
            if best is None or score > best[0]:
                best = (score, ents, name_len, stride, cum)
    if not best:
        return None
    _score, ents, name_len, stride, cum = best
    end = min(magic_off + 0x1c + stride * len(ents), len(data))
    conf = min(1.0, 0.95 + 0.04 * cum + 0.01 * min(1.0, len(ents) / 40.0))
    return Table("minifs-fixed", ents, magic_off,
                 start_detail="MINIFS@0x%X" % magic_off,
                 end=end, confidence=conf,
                 name_len=name_len, stride=stride, cum=round(cum, 3))


# --------------------------------------------------------------------------
# 2) MINIFS 名字池 + 20 字节记录
# --------------------------------------------------------------------------

def _minifs_pool(data, magic_off):
    """``[MINIFS][ver=3][count][data_size][name_block_len]`` + 名字池 + 记录表。"""
    n = len(data)
    if magic_off + 0x20 > n:
        return None
    ver, count, total = struct.unpack(">III", data[magic_off + 0x10:magic_off + 0x1c])
    if ver != 3 or not (4 <= count <= 20000):
        return None
    name_len = struct.unpack(">I", data[magic_off + 0x1c:magic_off + 0x20])[0]
    ns = magic_off + 0x20
    if not (0 < name_len <= n - ns):
        return None
    pool = data[ns:ns + name_len]
    strings = {}
    p = 0
    while p < len(pool) and len(strings) < MAX_ENTRIES:
        e = pool.find(b"\0", p)
        if e < 0:
            break
        strings[p] = _decode(pool[p:e])
        p = e + 1
    if len(strings) < 4:
        return None
    tab = ns + name_len
    if tab + count * 20 > n:
        return None
    ents = []
    resolved = 0
    for i in range(count):
        base = tab + i * 20
        path_off, name_off, chunk, in_chunk, size = struct.unpack(
            ">IIIII", data[base:base + 20])
        if path_off not in strings or name_off not in strings:
            continue
        resolved += 1
        path = strings[path_off]
        fname = strings[name_off]
        if not fname:
            continue
        name = (path.rstrip("/") + "/" + fname) if path else fname
        enc = name.encode("latin-1", "replace")
        if _name_kind(enc) is None:
            continue
        # 这种布局的数据是按 chunk 压缩存放的，raw 里没有直接可用的文件偏移；
        # offset 置 -1 表示「索引里有、但 raw 偏移未知」，避免给出假的偏移。
        ents.append((name, size, -1))
    if resolved < max(4, int(0.8 * count)) or len(ents) < 4:
        return None
    if len(ents) < 0.5 * count:
        return None
    end = tab + count * 20
    ratio_resolved = float(resolved) / count
    conf = min(0.99, 0.88 + 0.06 * ratio_resolved + 0.05 * min(1.0, len(ents) / 50.0))
    return Table("minifs-pool", ents, magic_off, end=end, confidence=conf,
                 version=ver, count=count, name_block=name_len,
                 data_size=total,
                 note="名字池布局；数据按 chunk 压缩，offset 未知（-1）")


# --------------------------------------------------------------------------
# 3) 定长记录（自动推断 name 字段长度 / stride / 端序）
# --------------------------------------------------------------------------

def _parse_fixed(data, start, stride, name_len, endian):
    """解析一串定长记录，返回 (entries, cut_count)。"""
    fmt = ">" if endian == "BE" else "<"
    ents = []
    cut = 0
    empty = 0
    p = start
    n = len(data)
    while p + name_len + 8 <= n:
        field = data[p:p + name_len]
        nm = field.split(b"\0")[0]
        if not nm:
            if field.count(0) != name_len:      # 非空非名：解析已跑偏
                break
            empty += 1                          # 空槽（被删条目）
            if empty > MAX_ENTRIES:
                break
            p += stride
            continue
        if len(nm) < name_len and data[p + len(nm)] != 0:
            break
        kind = _name_kind(nm, name_len)
        if kind is None:
            break
        if kind == "cut":
            cut += 1
            # 允许截断名字，但别无限跑飞；真正的比例在 _detect_fixed 里卡
            if cut > 4 + int(0.5 * len(ents)):
                break
        size, off = struct.unpack(fmt + "II", data[p + name_len:p + name_len + 8])
        if size > MAX_VALUE or off > MAX_VALUE:
            break
        ents.append((_decode(nm), size, off))
        if len(ents) >= MAX_ENTRIES:
            break
        p += stride
    return ents, cut


def _parse_offsets_only(data, start, stride, name_len, endian):
    """``[name:name_len][offset:4]``（没有 size）。"""
    fmt = ">" if endian == "BE" else "<"
    ents = []
    p = start
    n = len(data)
    while p + name_len + 4 <= n:
        field = data[p:p + name_len]
        nm = field.split(b"\0")[0]
        if not nm:
            if field.count(0) != name_len:
                break
            p += stride
            continue
        if len(nm) < name_len and data[p + len(nm)] != 0:
            break
        if _name_kind(nm, name_len) is None:
            break
        off = struct.unpack(fmt + "I", data[p + name_len:p + name_len + 4])[0]
        if off > MAX_VALUE:
            break
        ents.append((_decode(nm), 0, off))
        if len(ents) >= MAX_ENTRIES:
            break
        p += stride
    return ents


def _detect_offsets_only(data, runs):
    """``name + NUL pad + 4 字节 offset``（只有偏移，没有 size）。"""
    out = []
    for run in runs:
        if len(run) < 8:
            continue
        gaps = [run[i + 1][0] - run[i][0] for i in range(len(run) - 1)]
        strides = sorted(set(g for g in gaps if 0x10 <= g <= 0x400))[:6]
        for stride in strides:
            name_len = stride - 4
            if name_len < 4:
                continue
            starts = set()
            for pos, _nm in run[:4]:
                for d in range(-4, 5):
                    if pos + d >= 0:
                        starts.add(pos + d)
                for k in range(1, 4):
                    q = pos - k * stride
                    for d in range(-2, 3):
                        if q + d >= 0:
                            starts.add(q + d)
            for endian in ("BE", "LE"):
                best = None
                for s0 in sorted(starts):
                    ents = _parse_offsets_only(data, s0, stride, name_len, endian)
                    if len(ents) < 8:
                        continue
                    offs = [e[2] for e in ents]
                    strict = float(sum(1 for i in range(len(offs) - 1)
                                       if offs[i] < offs[i + 1])) / (len(offs) - 1)
                    if strict < 0.95:
                        continue
                    if offs[0] > 0x10000:
                        continue
                    cov = float(len(ents)) / len(run)
                    if cov < 0.6:
                        continue
                    conf = min(0.87, 0.50 + 0.20 * strict
                               + 0.12 * min(1.0, len(ents) / 30.0)
                               + 0.08 * min(1.0, cov))
                    key = (conf, len(ents))
                    if best is None or key > best[0]:
                        best = (key, s0, ents, strict, cov, conf)
                if not best:
                    continue
                _key, s0, ents, strict, cov, conf = best
                out.append(Table(
                    "offsets-only", ents, s0,
                    end=min(len(data), s0 + stride * len(ents)),
                    confidence=conf, stride=stride, name_len=name_len,
                    endian=endian, strict=round(strict, 3),
                    cov=round(min(1.0, cov), 2), note="只有 offset，没有 size"))
    return out


def _count_hint(data, start):
    """表头里可能写着条目数：看看 start 前 4 / 8 / 12 字节。"""
    hints = []
    for back in (4, 8, 12):
        if start - back < 0:
            continue
        raw = data[start - back:start - back + 4]
        for endian in (">", "<"):
            try:
                v = struct.unpack(endian + "I", raw)[0]
            except struct.error:
                continue
            if 0 < v <= MAX_ENTRIES:
                hints.append(v)
    return hints


def _detect_fixed(data, runs):
    out = []
    for run in runs:
        if len(run) < 6:
            continue
        gaps = [run[i + 1][0] - run[i][0] for i in range(len(run) - 1)]
        strides = sorted(set(g for g in gaps if 0x10 <= g <= 0x400))[:6]
        if not strides:
            continue
        for stride in strides:
            for trailer in (8, 11, 12, 16, 6, 4):
                name_len = stride - trailer
                if name_len < 4:
                    continue
                # 候选起点：run 里前几条名字的位置，以及按 stride 往前推出来的
                # 位置（名字前面可能粘着表头/计数，regex 会把它们算进名字）。
                starts = set()
                for pos, _nm in run[:4]:
                    for d in range(-4, 5):
                        q = pos + d
                        if q >= 0:
                            starts.add(q)
                    for k in range(1, 4):
                        q = pos - k * stride
                        for d in range(-2, 3):
                            if q + d >= 0:
                                starts.add(q + d)
                starts = sorted(starts)
                for endian in ("BE", "LE"):
                    best = None
                    for s0 in starts:
                        ents, cut = _parse_fixed(data, s0, stride, name_len, endian)
                        if len(ents) < 6:
                            continue
                        if not any(e[1] for e in ents):
                            # size 全是 0：其实是 [name][offset] 布局被按 8 字节
                            # trailer 误读了，交给 _detect_offsets_only。
                            continue
                        cum = _cum_score(ents)
                        if cum < 0.85:
                            continue
                        cov = float(len(ents)) / len(run)
                        if cov < 0.6:
                            continue
                        cut_ratio = float(cut) / len(ents)
                        if cut_ratio > MAX_CUT_RATIO:
                            continue
                        conf = min(0.97, (0.45 + 0.30 * cum
                                         + 0.20 * min(1.0, len(ents) / 30.0)
                                         + 0.05 * min(1.0, cov)
                                         - 0.10 * cut_ratio))
                        key = (conf, len(ents))
                        if best is None or key > best[0]:
                            best = (key, s0, ents, cut, cum, cut_ratio, conf)
                    if not best:
                        continue
                    _key, s0, ents, cut, cum, cut_ratio, conf = best
                    hints = _count_hint(data, s0)
                    if hints and any(abs(h - len(ents)) <= max(2, 0.05 * len(ents))
                                     for h in hints):
                        conf = min(0.97, conf + 0.08)
                    end = s0 + stride * (len(ents) + 0)
                    out.append(Table(
                        "fixed-stride", ents, s0, end=min(end, len(data)),
                        confidence=max(0.0, min(1.0, conf)),
                        stride=stride, name_len=name_len, endian=endian,
                        trailer=trailer, cum=round(cum, 3),
                        cut=cut, cov=round(min(1.0, float(len(ents)) / len(run)), 2),
                        count_hint=hints[0] if hints else None))
    return out
# --------------------------------------------------------------------------
# 4) 名字块 + 偏移数组
# --------------------------------------------------------------------------

def _read_name_block(data, start, max_entries=MAX_ENTRIES):
    """从 start 起读连续 NUL 分隔的名字，返回 (names, end_pos)。"""
    names = []
    p = start
    n = len(data)
    while p < n and len(names) < max_entries:
        end = data.find(b"\0", p, min(n, p + 0x101))
        if end < 0:
            break
        raw = data[p:end]
        if _name_kind(raw) is None:
            break
        names.append(_decode(raw))
        p = end + 1
    return names, p


def _detect_nameblock(data, runs):
    """名字连续排，后面跟同样数量的 4 字节偏移；数量可能在前 4/12 字节。"""
    out = []
    n = len(data)
    for run in runs:
        if len(run) < 8:
            continue
        for pos, _nm in run[:2]:
            names, p = _read_name_block(data, pos)
            if len(names) < 8:
                continue
            cnt = len(names)
            for align in (0, 1, 2, 3):
                q = p + align
                if q + 4 * cnt > n:
                    continue
                for endian in ("BE", "LE"):
                    fmt = ">" if endian == "BE" else "<"
                    offs = struct.unpack_from(fmt + "%dI" % cnt, data, q)
                    if any(o > MAX_VALUE for o in offs):
                        continue
                    mono = float(sum(1 for i in range(cnt - 1)
                                     if offs[i] < offs[i + 1])) / (cnt - 1)
                    if mono < 0.9:
                        continue
                    if offs[-1] == 0:
                        continue
                    hint = None
                    for back in (4, 12):
                        if pos - back < 0:
                            continue
                        raw = data[pos - back:pos - back + 4]
                        for e2 in (">", "<"):
                            v = struct.unpack(e2 + "I", raw)[0]
                            if v == cnt:
                                hint = (back, e2, v)
                    conf = (0.42 + 0.22 * mono + 0.18 * min(1.0, cnt / 24.0)
                            + (0.18 if hint else 0.0))
                    if hint:
                        conf = min(0.90, conf + 0.05)
                    conf = min(0.90, conf)
                    ents = [(names[i], 0, offs[i]) for i in range(cnt)]
                    out.append(Table("nameblock", ents, pos,
                                     end=min(n, q + 4 * cnt),
                                     confidence=max(0.0, min(1.0, conf)),
                                     endian=endian, count=cnt,
                                     count_prefix=hint,
                                     align=align, mono=round(mono, 3),
                                     note="只有 offset，没有 size"))
    return out


# --------------------------------------------------------------------------
# 5) 长度前缀记录
# --------------------------------------------------------------------------

def _parse_lenprefix(data, rec, first_len, endian, order, suffix, mode,
                     prefix_len=1):
    """链式解析长度前缀记录（名字长度逐条读，可变）。

    mode='lenfirst' : [len][name][pad:suffix][f1:4][f2:4]
    mode='lenlast'  : [f1:4][f2:4][len][name]
    order=0 时 f1=size, f2=offset；order=1 时反过来。
    prefix_len 为 1 或 2。
    """
    fmt = ">" if endian == "BE" else "<"
    ents = []
    p = rec
    n = len(data)
    while len(ents) < MAX_ENTRIES:
        if mode == "lenfirst":
            if p + prefix_len > n:
                break
            ln = (data[p] if prefix_len == 1
                  else struct.unpack(fmt + "H", data[p:p + 2])[0])
            if not (3 <= ln <= 0x80):
                break
            nstart = p + prefix_len
            if nstart + ln > n:
                break
            nm = data[nstart:nstart + ln]
            if _name_kind(nm) is None:
                break
            fpos = nstart + ln + suffix
            if fpos + 8 > n:
                break
            a, b = struct.unpack(fmt + "II", data[fpos:fpos + 8])
            nxt = fpos + 8
        else:  # lenlast: [size][offset][len][name]
            if p + 8 + prefix_len > n:
                break
            a, b = struct.unpack(fmt + "II", data[p:p + 8])
            lp = p + 8
            ln = (data[lp] if prefix_len == 1
                  else struct.unpack(fmt + "H", data[lp:lp + 2])[0])
            if not (3 <= ln <= 0x80):
                break
            nstart = lp + prefix_len
            if nstart + ln > n:
                break
            nm = data[nstart:nstart + ln]
            if _name_kind(nm) is None:
                break
            nxt = nstart + ln
        size, off = (a, b) if order == 0 else (b, a)
        if size > MAX_VALUE or off > MAX_VALUE:
            break
        ents.append((_decode(nm), size, off))
        p = nxt
        if len(ents) == 1:
            pass
    return ents


def _detect_lenprefix(data, runs):
    out = []
    for run in runs:
        if len(run) < 6:
            continue
        for pos, nm in run[:3]:
            name_len = len(nm)
            if not (3 <= name_len <= 0x7f):
                continue
            # 长度字节/字在名字前面
            prefixes = []
            if pos >= 1 and data[pos - 1] == name_len:
                prefixes.append((1, pos - 1, "le1"))
            if pos >= 2:
                for e2, tag in (("<", "le2"), (">", "be2")):
                    v = struct.unpack(e2 + "H", data[pos - 2:pos])[0]
                    if v == name_len and name_len > 0xff:
                        prefixes.append((2, pos - 2, tag))
            for prefix_len, rec, _tag in prefixes:
                for endian in ("BE", "LE"):
                    for order in (0, 1):
                        for suffix in (0, 1, 2, 3):
                            ents = _parse_lenprefix(data, rec, name_len, endian,
                                                    order, suffix, "lenfirst",
                                                    prefix_len)
                            if len(ents) < 6:
                                continue
                            cum = _cum_score(ents)
                            if cum < 0.85:
                                continue
                            conf = min(0.92, 0.55 + 0.3 * cum
                                       + 0.15 * min(1.0, len(ents) / 20.0))
                            out.append(Table(
                                "lenprefix", ents, rec,
                                confidence=min(1.0, conf),
                                order="size,offset" if order == 0 else "offset,size",
                                endian=endian, prefix_len=prefix_len,
                                suffix=suffix, cum=round(cum, 3),
                                note="[len][name][size][offset]"))
                # [size][offset][len][name]
                for endian in ("BE", "LE"):
                    for order in (0, 1):
                        rec2 = rec - 8
                        if rec2 < 0:
                            continue
                        ents = _parse_lenprefix(data, rec2, name_len, endian,
                                                order, 0, "lenlast", prefix_len)
                        if len(ents) < 6:
                            continue
                        cum = _cum_score(ents)
                        if cum < 0.85:
                            continue
                        conf = min(0.92, 0.55 + 0.3 * cum
                                   + 0.15 * min(1.0, len(ents) / 20.0))
                        out.append(Table(
                            "lenprefix", ents, rec2,
                            confidence=min(1.0, conf),
                            order="size,offset" if order == 0 else "offset,size",
                            endian=endian, prefix_len=prefix_len, suffix=0,
                            cum=round(cum, 3),
                            note="[size][offset][len][name]"))
    return out


# --------------------------------------------------------------------------
# 汇总
# --------------------------------------------------------------------------

def _detect_minifs(data):
    out = []
    start = 0
    guard = 0
    while guard < 32:
        guard += 1
        m = data.find(b"MINIFS", start)
        if m < 0:
            break
        start = m + 1
        t = _minifs_pool(data, m) or _minifs_fixed(data, m)
        if t is not None:
            out.append(t)
    return out


def _set_overlap(a, b, ratio=0.9):
    """两个集合是否「基本是同一批名字」（互相包含率都 >= ratio）。"""
    inter = len(a & b)
    if not inter:
        return False
    return (float(inter) / len(a) >= ratio and float(inter) / len(b) >= ratio)


def _dedupe(tables):
    """去掉重复/被包含的表。"""
    tables = sorted(tables, key=lambda t: (t.confidence, len(t)), reverse=True)
    minifs_layouts = ("minifs-fixed", "minifs-pool")
    kept = []
    for t in tables:
        dup = False
        for k in kept:
            if t.layout == k.layout and abs(t.start - k.start) <= 0x20:
                if len(t) <= len(k):
                    dup = True
                break
            ns_t, ns_k = t.name_set, k.name_set
            if not (ns_t and ns_k):
                continue
            overlap = _set_overlap(ns_t, ns_k)
            if not overlap:
                # 名字被切掉头/尾的「残片表」（例如从名字中间起读）
                # 用 basename 也能认出来
                bt = set(n.rsplit("/", 1)[-1] for n in ns_t)
                bk = set(n.rsplit("/", 1)[-1] for n in ns_k)
                overlap = _set_overlap(bt, bk)
            if not overlap:
                continue
            # 同一批名字：MINIFS 布局（有魔数）永远压过靠推断的 fixed-stride，
            # 因为后者是把 [size][offset][name] 误当 [name][size][offset]，
            # 名字对、值错位一条。
            if (k.layout in minifs_layouts and t.layout not in minifs_layouts
                    and len(t) <= 1.5 * len(k)):
                dup = True
                break
            if len(t) <= len(k):
                dup = True
            break
        if not dup:
            kept.append(t)
    kept.sort(key=lambda t: (t.confidence, len(t)), reverse=True)
    return kept


def find_tables(data, min_entries=6, max_tables=48):
    """在 data（固件镜像原始字节）里找出所有可信的「文件名表」。

    返回按 (置信度, 条目数) 降序排列的 :class:`Table` 列表。
    """
    if not data or len(data) < 0x40:
        return []
    runs = _name_runs(data)
    cands = []
    cands.extend(_detect_minifs(data))
    if runs:
        cands.extend(_detect_fixed(data, runs))
        cands.extend(_detect_offsets_only(data, runs))
        cands.extend(_detect_nameblock(data, runs))
        cands.extend(_detect_lenprefix(data, runs))
    cands = [t for t in cands if len(t) >= min_entries]
    kept = _dedupe(cands)
    kept.sort(key=lambda t: (t.confidence, len(t)), reverse=True)
    return kept[:max_tables]


# --------------------------------------------------------------------------
# 内容映射率（名字 → 内容的交叉验证）
# --------------------------------------------------------------------------

def _looks_like(name, blob):
    """按扩展名判断 blob 开头是否像该类型的真实内容；无法判定返回 None。"""
    if not blob:
        return None
    ext = name.lower().rsplit(".", 1)[-1] if "." in name else ""
    head = blob[:64]
    if ext in ("htm", "html", "shtml"):
        return head.lstrip()[:1] == b"<"
    if ext == "xml":
        return head.lstrip()[:1] == b"<"
    if ext == "png":
        return head.startswith(b"\x89PNG")
    if ext == "gif":
        return head.startswith(b"GIF8")
    if ext in ("jpg", "jpeg"):
        return head.startswith(b"\xff\xd8")
    if ext == "bmp":
        return head.startswith(b"BM")
    if ext == "ico":
        return head.startswith(b"\x00\x00\x01\x00")
    if ext == "pem":
        return b"-----BEGIN" in blob[:256]
    if ext in ("cer", "crt"):
        return head[:1] == b"\x30"
    if ext in ("css", "js"):
        printable = sum(1 for c in head if 9 <= c <= 126)
        frac = float(printable) / max(1, len(head))
        marks = sum(1 for k in (b"{", b";", b"=", b"function", b"var ")
                    if k in blob[:512])
        return frac >= 0.95 and marks >= 1
    return None


def mapped_ratio(data, table):
    """按 offset 取内容，统计「扩展名 ↔ 内容」对得上的比例，返回 (ratio, hit, tot)。"""
    hit = tot = 0
    for name, size, off in table.entries:
        if off is None or off < 0 or off + 8 > len(data):
            continue
        ext = name.lower().rsplit(".", 1)[-1] if "." in name else ""
        if ext in ("bin", "dat", "cfg", "ini", "txt", "json", "db", "img",
                   "trx", "rom", "chk", "pdf", "zip", "gz", "tar", "so", "o",
                   "h", "ttf", "wav", "swf", "cgi", "asp"):
            continue
        n = 64 if size <= 0 else max(8, min(64, size))
        verdict = _looks_like(name, data[off:off + n])
        if verdict is None:
            continue
        tot += 1
        if verdict:
            hit += 1
    return (float(hit) / tot if tot else 0.0), hit, tot

# --------------------------------------------------------------------------
# 容器读取（zip / rar / 7z / 裸镜像）
# --------------------------------------------------------------------------

def load_container(path):
    """取出固件镜像本体：zip 取最大成员，rar/7z 用 7-Zip，其它原样读。"""
    try:
        if zipfile.is_zipfile(path):
            with zipfile.ZipFile(path) as z:
                members = [i for i in z.infolist() if not i.is_dir()]
                if members:
                    it = max(members, key=lambda i: i.file_size)
                    return z.read(it)
    except Exception:
        pass
    try:
        with open(path, "rb") as f:
            head = f.read(8)
    except Exception:
        return b""
    if head.startswith(b"Rar!") or head.startswith(b"7z\xbc\xaf"):
        seven = os.environ.get("SEVENZIP", r"C:\Program Files\7-Zip\7z.exe")
        if os.path.exists(seven):
            tmp = tempfile.mkdtemp()
            try:
                subprocess.run([seven, "x", "-y", "-o" + tmp, path],
                               stdout=subprocess.DEVNULL,
                               stderr=subprocess.DEVNULL, timeout=300)
                files = [os.path.join(dp, f)
                         for dp, _dn, fs in os.walk(tmp) for f in fs
                         if os.path.getsize(os.path.join(dp, f)) > 4096]
                if files:
                    with open(max(files, key=os.path.getsize), "rb") as f:
                        return f.read()
            except Exception:
                pass
    with open(path, "rb") as f:
        return f.read()


# --------------------------------------------------------------------------
# 命令行
# --------------------------------------------------------------------------

def _force_utf8_stdout():
    """控制台不是 UTF-8 时也别因为打印中文崩掉。"""
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except Exception:
            pass


def _print_tables(path, data, tables=None, show_names=12):
    if tables is None:
        tables = find_tables(data)
    print("== %s  (%d bytes)  tables=%d" % (path, len(data), len(tables)))
    for t in tables:
        ratio, hit, tot = mapped_ratio(data, t)
        print("   [%s] n=%d conf=%.2f @0x%X mapped=%.2f (%d/%d) %s"
              % (t.layout, len(t), t.confidence, t.start, ratio, hit, tot,
                 {k: v for k, v in t.meta.items()
                  if k in ("stride", "name_len", "endian", "count", "cum",
                           "cut", "cov", "count_hint", "version", "note")}))
        for name, size, off in t.entries[:show_names]:
            print("        %-50s size=%-8d off=0x%X" % (name, size, off))
        if len(t) > show_names:
            print("        ...（共 %d 条）" % len(t))
    return tables


def _scan(rows_csv, out_csv, show_progress=True):
    """对 table_scan.csv 里「有名字」的样本跑 find_tables，写 table_scan2.csv。"""
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    scan_path = rows_csv or os.path.join(root, "table_scan.csv")
    corpus_path = os.path.join(root, "CORPUS.csv")
    out_path = out_csv or os.path.join(root, "table_scan2.csv")

    paths = {}
    with open(corpus_path, encoding="utf-8-sig", newline="") as f:
        for r in csv.DictReader(f):
            base = os.path.basename(r["file"])
            cand = os.path.join(root, r["file"])
            # CORPUS 里偶尔有同名/失效路径：优先保留真实存在的那个
            if base not in paths or (not os.path.exists(paths[base])
                                     and os.path.exists(cand)):
                paths[base] = cand

    with open(scan_path, encoding="utf-8", newline="") as f:
        rows = list(csv.DictReader(f))

    targets = [r for r in rows if int(r.get("name_runs") or 0) > 0]
    out_rows = []
    for i, r in enumerate(targets, 1):
        sample = r["sample"]
        path = paths.get(sample)
        if not path or not os.path.exists(path):
            out_rows.append({"sample": sample, "layout": "-", "entries": 0,
                             "mapped_ratio": "0.00", "confidence": "0.00",
                             "vendor": r.get("vendor", ""),
                             "note": "file missing"})
            continue
        try:
            data = load_container(path)
        except Exception as exc:                     # pragma: no cover
            out_rows.append({"sample": sample, "layout": "-", "entries": 0,
                             "mapped_ratio": "0.00", "confidence": "0.00",
                             "vendor": r.get("vendor", ""),
                             "note": "load error: %s" % exc})
            continue
        tables = find_tables(data)
        best = None
        for t in tables:
            if best is None or (len(t), t.confidence) > (len(best), best.confidence):
                best = t
        if best is None:
            out_rows.append({"sample": sample, "layout": "-", "entries": 0,
                             "mapped_ratio": "0.00", "confidence": "0.00",
                             "vendor": r.get("vendor", ""),
                             "note": "no table"})
        else:
            ratio, hit, tot = mapped_ratio(data, best)
            extra = []
            if best.meta.get("stride") is not None:
                fmt = "stride=0x%X,L=0x%X" % (best.meta["stride"],
                                              best.meta["name_len"])
                if best.meta.get("endian"):
                    fmt += "," + best.meta["endian"]
                extra.append(fmt)
            if best.meta.get("version") is not None:
                extra.append("ver=%s,count=%s" % (best.meta["version"],
                                                  best.meta["count"]))
            if best.meta.get("cut"):
                extra.append("cut=%d" % best.meta["cut"])
            if len(tables) > 1:
                extra.append("tables=%d" % len(tables))
            out_rows.append({
                "sample": sample,
                "layout": best.layout,
                "entries": len(best),
                "mapped_ratio": "%.2f" % ratio,
                "confidence": "%.2f" % best.confidence,
                "vendor": r.get("vendor", ""),
                "note": "; ".join(extra) + (" | " if extra else "") +
                        ("%s / %s" % (best.entries[0][0], best.entries[-1][0])
                         if best.entries else ""),
            })
        if show_progress:
            row = out_rows[-1]
            print("%3d/%d %-46s %-14s n=%-4s mapped=%s conf=%s"
                  % (i, len(targets), sample, row["layout"], row["entries"],
                     row["mapped_ratio"], row["confidence"]), flush=True)

    fields = ["sample", "layout", "entries", "mapped_ratio", "confidence",
              "vendor", "note"]
    with open(out_path, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fields)
        w.writeheader()
        w.writerows(out_rows)

    got = [r for r in out_rows if r["entries"] and int(r["entries"]) > 0]
    total = sum(int(r["entries"]) for r in out_rows)
    print("\n有名字表的样本：%d / %d" % (len(got), len(out_rows)))
    print("清单总条数：%d" % total)
    print("明细：%s" % out_path)
    return out_rows


def main(argv=None):
    _force_utf8_stdout()
    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv:
        print(__doc__)
        return 0
    if argv[0] == "--scan":
        rows_csv = None
        out_csv = None
        rest = argv[1:]
        i = 0
        while i < len(rest):
            if rest[i] == "--out" and i + 1 < len(rest):
                out_csv = rest[i + 1]
                i += 2
            elif rest[i] == "--in" and i + 1 < len(rest):
                rows_csv = rest[i + 1]
                i += 2
            else:
                i += 1
        _scan(rows_csv, out_csv)
        return 0
    rc = 0
    for path in argv:
        if not os.path.exists(path):
            print("[-] 不存在：%s" % path, file=sys.stderr)
            rc = 1
            continue
        data = load_container(path)
        if not data:
            print("[-] 读不出内容：%s" % path, file=sys.stderr)
            rc = 1
            continue
        tables = find_tables(data)
        if tables:
            _print_tables(path, data, tables=tables)
        else:
            print("== %s  (%d bytes)  没有找到可信的文件名表" % (path, len(data)))
    return rc


if __name__ == "__main__":
    sys.exit(main())