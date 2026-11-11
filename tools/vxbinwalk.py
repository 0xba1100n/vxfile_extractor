#!/usr/bin/env python3
"""vxbinwalk - a dependency-free, cross-platform stand-in for binwalk v2.

Implements the subset vxfile_extractor needs:
    vxbinwalk -Me -C <outdir> <file>
It prints a binwalk-v2 style table and writes every extracted layer to
    <outdir>/_<basename>.extracted/<UPPERCASE_HEX_OFFSET>
so the existing tool can keep its logic untouched.

Supported containers:
  * archives / single-stream compressors: zip, gzip, bzip2, xz,
    LZMA-alone, raw LZMA1, raw/zlib/gzip DEFLATE
  * vendor wrappers: uImage, TP-Link RTOS, TRX (HDR0), D-Link DLOB,
    Ubicom boot image, generic TLV and length-prefixed containers
  * carving: JPEG/PNG/GIF, HTML documents, inline JS/CSS

Compressed payloads are searched at *every* offset (not only the first
kilobyte), and files larger than LARGE_SCAN are sampled per block instead
of being skipped, so a 200 MB disk image still yields hits.
"""
import argparse
import bz2
import hashlib
import io
import lzma
import os
import struct
import sys
import zipfile
import zlib

UIMAGE = b"\x27\x05\x19\x56"
TPLINK = b"\x00\x14\x2f\xc0"
TRX = b"HDR0"
DLOB = b"DLOB"
UBICOM = b"\xfa\x32\x00\x80"

OS_NAMES = {4: "Linux", 5: "Linux", 8: "VxWorks", 13: "VxWorks", 14: "VxWorks", 17: "Firmware"}
ARCH_NAMES = {2: "ARM", 3: "Intel x86", 5: "MIPS", 7: "PowerPC", 8: "PowerPC", 20: "MIPS"}
COMP_NAMES = {0: "none", 1: "gzip", 2: "bzip2", 3: "lzma", 4: "lzo"}

MAX_LAYERS = 12
MAX_OUT = 256 << 20
# Blobs bigger than this are sampled block by block instead of byte by byte.
LARGE_SCAN = 24 << 20
# Sampling step used for the cheap in-block probe of large blobs.
LARGE_STEP = 256
# Upper bound for everything a single scan may pull out of one file.
MAX_TOTAL_OUT = 512 << 20
# Hard cap on layers per sample: exFAT writes and the downstream tool both
# choke on hundreds of tiny carvings, so keep only the biggest ones.
MAX_ROWS = 300
# Web (HTML/JS/CSS) carving limits: it is only cheap on flat, not too big blobs.
WEB_MAX_FILE = 8 << 20
WEB_MAX_OFFSET = 32 << 20
WEB_MAX_BLOB = 16 << 20
# Carved out fragments smaller than this are noise: they inflate the "unpacked"
# rate without carrying real content, and every one costs an exFAT write.
CARVE_MIN = 4096
# Per sample cap on carved products; keep the biggest ones.
CARVE_MAX = 200
LAST_TRUNCATED = None
LAST_DROPPED_TINY = 0
LAST_CARVE_TRUNC = 0
LAST_CARVE_KEPT = 0

LZMA_PROPS = (0x5C, 0x5D, 0x5E, 0x5A, 0x6E)


# --------------------------------------------------------------------------
# decompressor helpers
# --------------------------------------------------------------------------
def _ok(body):
    return body if body is not None and 1024 < len(body) <= MAX_OUT else None


def _drain_lzma(d, out, budget):
    """LZMADecompressor.decompress(max_length=...) may return a partial chunk:
    keep pulling empty input until the stream ends (or the budget is gone)."""
    while not d.eof and len(out) < budget:
        try:
            more = d.decompress(b"", min(budget - len(out), MAX_OUT))
        except Exception:
            break
        if not more:
            break
        out += more
    return out


def unlzma_all(data, off):
    """Decode one LZMA-alone stream *and* any concatenated members after it.

    Firmware images (Mercury / TP-Link) often store a header followed by many
    tiny LZMA members; LZMADecompressor stops at the first member end, while
    lzma.decompress() concatenates them.  Both behaviours are folded in here
    with an explicit cap so a bogus header cannot explode memory.
    """
    view = memoryview(data)
    rest = view[off:]
    out = bytearray()
    members = 0
    while len(out) < MAX_OUT and members < 64:
        try:
            d = lzma.LZMADecompressor(format=lzma.FORMAT_ALONE)
            chunk = d.decompress(rest, MAX_OUT - len(out))
            chunk = _drain_lzma(d, chunk, MAX_OUT - len(out))
        except Exception:
            break
        if not chunk:
            break
        out += chunk
        members += 1
        if not d.eof or not d.unused_data:
            break
        rest = d.unused_data
    body = bytes(out)
    return _ok(body) if body else None


def lzma_alone_at(data, off):
    return unlzma_all(data, off)


def raw_lzma_at(data, off):
    view = memoryview(data)
    for ds in (1 << 23, 1 << 20, 1 << 16):
        try:
            d = lzma.LZMADecompressor(format=lzma.FORMAT_RAW,
                                      filters=[{"id": lzma.FILTER_LZMA1, "dict_size": ds}])
            out = d.decompress(view[off:], MAX_OUT)
            out = _drain_lzma(d, out, MAX_OUT)
        except Exception:
            continue
        body = _ok(out)
        if body:
            return body, ds
    return None, 0


def deflate_at(data, off):
    """Raw/zlib/gzip DEFLATE probe used for headerless streams."""
    view = memoryview(data)
    for wbits in (-15, 15, 31):
        try:
            d = zlib.decompressobj(wbits)
            out = d.decompress(view[off:], MAX_OUT)
        except Exception:
            continue
        if d.eof:
            body = _ok(out)
            if body:
                return body, wbits
    return None, 0


def _inflate(data, off, wbits):
    """Inflate one zlib/gzip/raw-deflate stream at off (must reach its end)."""
    try:
        d = zlib.decompressobj(wbits)
        out = d.decompress(memoryview(data)[off:], MAX_OUT)
        while not d.eof and len(out) < MAX_OUT:
            more = d.decompress(b"", MAX_OUT - len(out))
            if not more:
                break
            out += more
    except Exception:
        return None
    if not d.eof:
        return None
    return _ok(out)


def _bunzip(data, off):
    try:
        d = bz2.BZ2Decompressor()
        out = d.decompress(memoryview(data)[off:], MAX_OUT)
        while not d.eof and len(out) < MAX_OUT:
            more = d.decompress(b"", MAX_OUT - len(out))
            if not more:
                break
            out += more
    except Exception:
        return None
    if not d.eof:
        return None
    return _ok(out)


def _unxz(data, off):
    try:
        d = lzma.LZMADecompressor(format=lzma.FORMAT_XZ)
        out = d.decompress(memoryview(data)[off:], MAX_OUT)
    except Exception:
        return None
    if not d.eof:
        return None
    return _ok(out)


def _unlzma(data, off):
    return unlzma_all(data, off)


def _dict_ok(ds):
    """LZMA dictionary sizes are 2^n or 2^n + 2^(n-1); anything else is noise."""
    if ds < (4 << 10) or ds > (1 << 30):
        return False
    x = ds
    while x > 1 and (x & 1) == 0:
        x >>= 1
    return x in (1, 3)


def lzma_header_ok(data, off):
    """Cheap validity test for an LZMA-alone (13 byte) header at off."""
    if off + 13 > len(data) or data[off] not in LZMA_PROPS:
        return False
    ds = int.from_bytes(data[off + 1:off + 5], "little")
    if not _dict_ok(ds):
        return False
    size = int.from_bytes(data[off + 5:off + 13], "little")
    if size == 0xFFFFFFFFFFFFFFFF:
        return True
    return 1024 < size < (4 << 30)


# --------------------------------------------------------------------------
# carving helpers
# --------------------------------------------------------------------------
def _jpeg_end(d, i):
    j = d.find(b"\xff\xd9", i + 2)
    return j + 2 if j > 0 and j - i < (4 << 20) else None


def _png_end(d, i):
    j = d.find(b"IEND", i)
    return j + 8 if j > 0 and j - i < (8 << 20) else None


def _gif_end(d, i):
    j = d.find(b"\x3b", i + 6)
    return j + 1 if j > 0 and j - i < (4 << 20) else None


IMG_SIGS = [
    (b"\xff\xd8\xff", "JPEG image data", _jpeg_end),
    (b"\x89PNG\r\n\x1a\n", "PNG image", _png_end),
    (b"GIF8", "GIF image", _gif_end),
]

HTML_START = (b"<!DOCTYPE", b"<!doctype", b"<html", b"<HTML", b"<Html")
HTML_END = (b"</html>", b"</HTML>", b"</Html>")
HTML_HINTS = (b"<head", b"<HEAD", b"<body", b"<BODY", b"<title", b"<TITLE",
              b"<div", b"<table", b"<script", b"<SCRIPT", b"<p>", b"<P>")


def _printable_ratio(d, a, b, step=97):
    seg = d[a:b]
    if not seg:
        return 0.0
    sample = seg[::step] if len(seg) > 4096 else seg
    good = sum(1 for c in sample if 9 <= c <= 13 or 32 <= c < 127)
    return good / float(len(sample))


def _html_end(d, i):
    best = None
    for end in HTML_END:
        j = d.find(end, i)
        if j >= 0 and (best is None or j < best):
            best = j
    return best + 7 if best is not None else None


def find_html(data, start=0, end=None, limit=200):
    """Return [(start, end)] spans of complete HTML documents."""
    if end is None:
        end = len(data)
    spans = []
    pos = start
    while len(spans) < limit:
        hit = -1
        for sig in HTML_START:
            j = data.find(sig, pos, end)
            if j >= 0 and (hit < 0 or j < hit):
                hit = j
        if hit < 0:
            break
        tail = _html_end(data, hit)
        if tail is None or tail > end:
            pos = hit + 1
            continue
        size = tail - hit
        if size < 64:
            pos = hit + 1
            continue
        body = data[hit:tail]
        if size >= 1024:
            hints = sum(1 for h in HTML_HINTS if h in body)
            if hints < 2 and b"<" not in body[:256]:
                pos = hit + 1
                continue
        if _printable_ratio(data, hit, tail) < 0.75:
            pos = hit + 1
            continue
        spans.append((hit, tail))
        pos = tail
    return spans


def find_inline_web(data, spans, limit=100):
    """Carve standalone <script>/<style> blocks that are not inside an HTML doc."""
    found = []
    for tag, endtag, label in ((b"<script", b"</script>", "JavaScript source"),
                               (b"<style", b"</style>", "CSS stylesheet")):
        pos = 0
        while len(found) < limit:
            i = data.find(tag, pos)
            if i < 0:
                break
            gt = data.find(b">", i)
            if gt < 0:
                break
            j = data.find(endtag, gt)
            if j < 0:
                pos = i + 1
                continue
            tail = j + len(endtag)
            pos = tail
            if tail - i < 512:
                continue
            if any(a <= i < b or a < tail <= b for a, b in spans):
                continue
            if _printable_ratio(data, gt + 1, j) < 0.85:
                continue
            found.append((i, tail, label))
    return found


def emit_carved(data, base, out, limit=400, web=True):
    """Exactly what binwalk's -e does for images, plus web documents."""
    n = 0
    for sig, label, endfn in IMG_SIGS:
        pos = 0
        while n < limit:
            pos = data.find(sig, pos)
            if pos < 0:
                break
            end = endfn(data, pos)
            if end:
                out.append((base + pos, "%s, size: %d bytes" % (label, end - pos), data[pos:end]))
                n += 1
            pos += 2
    if not web or len(data) > WEB_MAX_FILE or base >= WEB_MAX_OFFSET:
        return
    spans = find_html(data, limit=min(limit, 120))
    for a, b in spans:
        if base + a >= WEB_MAX_OFFSET:
            break
        out.append((base + a, "HTML document, size: %d bytes" % (b - a), data[a:b]))
    for a, b, label in find_inline_web(data, spans, limit=min(limit, 60)):
        out.append((base + a, "%s, size: %d bytes" % (label, b - a), data[a:b]))


# --------------------------------------------------------------------------
# container parsers
# --------------------------------------------------------------------------
def _trx(data, base, depth, seen, out):
    """TRX (HDR0): 28 byte header, image size + 3 segment offsets, big endian CRC."""
    if len(data) < 28:
        return
    total, crc, flags = struct.unpack("<III", data[4:16])
    offs = struct.unpack("<III", data[16:28])
    if total <= 28 or total > len(data) + (16 << 20):
        total = len(data)
    total = min(total, len(data))
    desc = ("TRX firmware header, little endian, image size: %d bytes, CRC32: 0x%08X, "
            "flags: 0x%04X, version: %d, header size: 28 bytes"
            % (total, crc, flags >> 16, flags & 0xFFFF))
    out.append((base, desc, data[:28]))
    bounds = [28] + [o for o in offs if 28 < o < total] + [total]
    for i in range(len(bounds) - 1):
        a, b = bounds[i], bounds[i + 1]
        if b <= a:
            continue
        chunk = data[a:b]
        out.append((base + a, "TRX segment %d, offset: 0x%X, size: %d bytes" % (i, a, len(chunk)), chunk))
        _scan(chunk, base + a, depth + 1, seen, out)


def _dlob(data, base, depth, seen, out):
    """D-Link DLOB private header: magic, then 32bit lengths (either endian)."""
    if len(data) < 32:
        return
    vals = [struct.unpack("<I", data[4:8])[0], struct.unpack(">I", data[4:8])[0],
            struct.unpack("<I", data[8:12])[0], struct.unpack(">I", data[8:12])[0]]
    total = next((v for v in vals if 32 <= v <= len(data)), len(data))
    hdr = next((v for v in vals if 32 <= v <= min(total, 0x1000)), 0x20)
    out.append((base, "D-Link DLOB firmware header, header size: %d bytes, image size: %d bytes"
                % (hdr, total), data[:hdr]))
    body = data[hdr:total]
    if body:
        out.append((base + hdr, "D-Link DLOB payload, size: %d bytes" % len(body), body))
        _scan(body, base + hdr, depth + 1, seen, out)


def _ubicom(data, base, depth, seen, out):
    """Ubicom boot image (D-Link DGL-4xxx): 32 byte header then the payload."""
    if len(data) < 64:
        return
    words = struct.unpack("<8I", data[:32])
    out.append((base, "Ubicom boot image header, header size: 32 bytes, flags: 0x%08X, "
                "size: %d bytes" % (words[3], len(data)), data[:32]))
    body = data[32:]
    out.append((base + 32, "Ubicom boot image payload, size: %d bytes" % len(body), body))
    _scan(body, base + 32, depth + 1, seen, out)


def _tlv_entries(data):
    """[4 byte tag][4 byte big/little endian length][payload] repeated."""
    for endian in (">", "<"):
        off = 0
        entries = []
        while off + 8 <= len(data) and len(entries) < MAX_ROWS:
            tag = data[off:off + 4]
            ln = struct.unpack(endian + "I", data[off + 4:off + 8])[0]
            if ln <= 0 or ln > len(data) - off - 8:
                break
            good_tag = all(32 <= c < 127 for c in tag) or tag[:2] == b"\x00\x00"
            if not good_tag:
                break
            entries.append((off, tag, ln))
            off += 8 + ln
        if len(entries) >= 3 and off >= len(data) * 0.8:
            return entries, endian
    return None, None


def _lenprefixed_entries(data):
    """[4 byte length][payload] repeated until the end of the blob."""
    for endian in (">", "<"):
        off = 0
        entries = []
        while off + 4 <= len(data) and len(entries) < MAX_ROWS:
            ln = struct.unpack(endian + "I", data[off:off + 4])[0]
            if ln < 16 or ln > len(data) - off - 4:
                break
            entries.append((off, ln))
            off += 4 + ln
        if len(entries) >= 3 and off >= len(data) * 0.8:
            return entries, endian
    return None, None


def _tlv(data, base, depth, seen, out):
    entries, endian = _tlv_entries(data)
    if entries:
        desc = ("TLV container, %s endian, entries: %d, size: %d bytes"
                % ("big" if endian == ">" else "little", len(entries), len(data)))
        out.append((base, desc, data))
        for k, (off, tag, ln) in enumerate(entries):
            body = data[off + 8:off + 8 + ln]
            name = tag.decode("latin1", "replace") if tag[:2] != b"\x00\x00" else "0x%08X" % int.from_bytes(tag, "big")
            out.append((base + off + 8, "TLV entry %d, tag: %s, size: %d bytes" % (k, name, ln), body))
            _scan(body, base + off + 8, depth + 1, seen, out)
        return True
    entries, endian = _lenprefixed_entries(data)
    if entries:
        desc = ("Length-prefixed container, %s endian, entries: %d, size: %d bytes"
                % ("big" if endian == ">" else "little", len(entries), len(data)))
        out.append((base, desc, data))
        for k, (off, ln) in enumerate(entries):
            body = data[off + 4:off + 4 + ln]
            out.append((base + off + 4, "Length-prefixed record %d, size: %d bytes" % (k, ln), body))
            _scan(body, base + off + 4, depth + 1, seen, out)
        return True
    return False


# --------------------------------------------------------------------------
# whole-blob compressed stream scan
# --------------------------------------------------------------------------
def _stream_offsets(data, limit):
    """Yield (offset, kind) candidates for compressed streams inside data."""
    n = len(data)
    if n <= LARGE_SCAN:
        starts = (0, n)
    else:
        starts = None  # large blob: full magic search, sampled zlib probe

    # exact multi-byte magics: a C level find, cheap even for huge files
    for magic, kind in ((b"\x1f\x8b\x08", "gzip"), (b"BZh", "bzip2"), (b"\xfd7zXZ\x00", "xz")):
        pos = 0
        c = 0
        while c < limit:
            pos = data.find(magic, pos)
            if pos < 0:
                break
            c += 1
            yield pos, kind
            pos += 1

    # zlib: CMF 0x78 (deflate, 32K window) + FLG such that (CMF<<8|FLG) % 31 == 0
    pos = 0
    c = 0
    if starts is None:
        for pos in range(0, n - 2, LARGE_STEP):
            if data[pos] != 0x78:
                continue
            if ((0x78 << 8) | data[pos + 1]) % 31:
                continue
            yield pos, "zlib"
            c += 1
            if c >= limit:
                break
    else:
        while c < limit:
            pos = data.find(b"\x78", pos)
            if pos < 0:
                break
            if pos + 1 < n and ((0x78 << 8) | data[pos + 1]) % 31 == 0:
                c += 1
                yield pos, "zlib"
            pos += 1

    # LZMA-alone: 13 byte header with a legitimate dictionary size
    pos = 0
    c = 0
    if starts is None:
        for pos in range(0, n - 13, LARGE_STEP):
            if data[pos] in LZMA_PROPS and lzma_header_ok(data, pos):
                c += 1
                yield pos, "lzma"
                if c >= limit:
                    break
    else:
        for m in range(0, n - 13):
            if data[m] not in LZMA_PROPS:
                continue
            if lzma_header_ok(data, m):
                c += 1
                yield m, "lzma"
                if c >= limit:
                    break


def scan_streams(data, base, depth, seen, out, budget):
    """Find and extract compressed streams anywhere inside data."""
    if depth > 3 or len(data) < 64:
        return 0
    caps = {"gzip": 64, "bzip2": 16, "xz": 8, "zlib": 24, "lzma": 32}
    hits = {"gzip": 0, "bzip2": 0, "xz": 0, "zlib": 0, "lzma": 0}
    tried = set()
    for off, kind in _stream_offsets(data, 200):
        if hits[kind] >= caps[kind]:
            continue
        if (off, kind) in tried:
            continue
        tried.add((off, kind))
        if kind == "gzip":
            body = _inflate(data, off, 31)
            desc = "gzip compressed data, uncompressed size: %d bytes" % len(body) if body else None
        elif kind == "bzip2":
            body = _bunzip(data, off)
            desc = "bzip2 compressed data, uncompressed size: %d bytes" % len(body) if body else None
        elif kind == "xz":
            body = _unxz(data, off)
            desc = "XZ compressed data, uncompressed size: %d bytes" % len(body) if body else None
        elif kind == "zlib":
            body = _inflate(data, off, 15)
            desc = "Zlib compressed data, uncompressed size: %d bytes" % len(body) if body else None
        else:
            body = _unlzma(data, off)
            desc = ("LZMA compressed data, properties: 0x%02X, dictionary size: %d bytes, "
                    "uncompressed size: %d bytes"
                    % (data[off], struct.unpack("<I", data[off + 1:off + 5])[0], len(body))) if body else None
        if not body:
            continue
        if budget["bytes"] < len(body) or len(out) >= MAX_ROWS:
            break
        budget["bytes"] -= len(body)
        out.append((base + off, desc, body))
        hits[kind] += 1
        _scan(body, base + off, depth + 1, seen, out, budget)
    return sum(hits.values())


def _cap_rows(out):
    """Keep at most MAX_ROWS layers, preferring the biggest payloads."""
    global LAST_TRUNCATED
    if len(out) <= MAX_ROWS:
        return out
    keep = sorted(range(len(out)), key=lambda i: len(out[i][2]), reverse=True)[:MAX_ROWS]
    LAST_TRUNCATED = (len(out), len(keep))
    return [out[i] for i in sorted(keep)]


def scan_all(data):
    global LAST_TRUNCATED, LAST_DROPPED_TINY, LAST_CARVE_TRUNC, LAST_CARVE_KEPT
    LAST_TRUNCATED = None
    LAST_DROPPED_TINY = 0
    LAST_CARVE_TRUNC = 0
    LAST_CARVE_KEPT = 0
    out = []
    seen = set()
    budget = {"bytes": MAX_TOTAL_OUT}
    _scan(data, 0, 0, seen, out, budget)

    # 明文镜像本身也要扫内嵌图片/网页（Linksys/Belkin 这类没有压缩层的固件全靠这一步）
    emitted = set()
    for _off, _desc, blob in out:
        if len(blob) <= (4 << 20):
            emitted.add(hashlib.md5(blob).hexdigest())
    carve = []
    emit_carved(data, 0, carve, web=len(data) <= WEB_MAX_FILE)
    for _off, _desc, blob in list(out):
        emit_carved(blob, _off, carve, web=len(blob) <= WEB_MAX_BLOB)

    uniq = []
    for row in carve:
        h = hashlib.md5(row[2]).hexdigest()
        if h in emitted:
            continue          # already extracted as a decompressed layer
        emitted.add(h)
        uniq.append(row)
    carve = uniq

    big = [r for r in carve if len(r[2]) >= CARVE_MIN]
    LAST_DROPPED_TINY = len(carve) - len(big)
    if len(big) > CARVE_MAX:
        LAST_CARVE_TRUNC = len(big) - CARVE_MAX
        keep = sorted(range(len(big)), key=lambda i: len(big[i][2]), reverse=True)[:CARVE_MAX]
        big = [big[i] for i in sorted(keep)]
    LAST_CARVE_KEPT = len(big)
    out.extend(big)
    return _cap_rows(out)


# --------------------------------------------------------------------------
# recursive scanner
# --------------------------------------------------------------------------
def _scan(data, base, depth, seen, out, budget=None):
    if budget is None:
        budget = {"bytes": MAX_TOTAL_OUT}
    if not data or len(data) < 32 or depth > 5:
        return
    key = hashlib.md5(data[:4096] + bytes(str(len(data)), "ascii")).hexdigest()
    if key in seen:
        return
    seen.add(key)

    # ---- ZIP -------------------------------------------------------------
    if zipfile.is_zipfile(io.BytesIO(data)):
        try:
            with zipfile.ZipFile(io.BytesIO(data)) as z:
                for item in z.infolist():
                    if item.is_dir() or item.file_size < 1024:
                        continue
                    body = z.read(item)
                    out.append((base, "Zip archive data, name: %s, uncompressed size: %d bytes"
                                % (item.filename, len(body)), body))
                    _scan(body, base, depth + 1, seen, out, budget)
        except Exception:
            pass
        return

    # ---- single-stream compressors at offset 0 ---------------------------
    if data[:3] == b"\x1f\x8b\x08":
        body = _inflate(data, 0, 31)
        if body:
            out.append((base, "gzip compressed data, uncompressed size: %d bytes" % len(body), body))
            _scan(body, base, depth + 1, seen, out, budget)
            return
    if data[:3] == b"BZh":
        body = _bunzip(data, 0)
        if body:
            out.append((base, "bzip2 compressed data, uncompressed size: %d bytes" % len(body), body))
            _scan(body, base, depth + 1, seen, out, budget)
            return
    if data[:6] == b"\xfd7zXZ\x00":
        body = _unxz(data, 0)
        if body:
            out.append((base, "XZ compressed data, uncompressed size: %d bytes" % len(body), body))
            _scan(body, base, depth + 1, seen, out, budget)
            return

    # ---- TP-Link RTOS wrapper -------------------------------------------
    if data[:4] == TPLINK:
        out.append((base, "TP-Link RTOS firmware, header size: 148 bytes, total size: %d bytes"
                    % len(data), data))
        _scan(data[0x94:], base + 0x94, depth + 1, seen, out, budget)
        return

    # ---- uImage (only at a header boundary) ------------------------------
    upos = -1
    for cand in (0, 0x40, 0x94, 0x100):
        if data[cand:cand + 4] == UIMAGE:
            upos = cand
            break
    if upos >= 0 and upos + 64 <= len(data):
        h = data[upos:upos + 64]
        size = struct.unpack(">I", h[12:16])[0]
        load = struct.unpack(">I", h[16:20])[0]
        ep = struct.unpack(">I", h[20:24])[0]
        os_, arch, typ, comp = h[28], h[29], h[30], h[31]
        name = h[32:64].split(b"\x00")[0].decode("latin1", "replace")
        big = data[upos + 1] != 0x00
        payload = data[upos + 64:upos + 64 + size]
        desc = ("uImage header, header size: 64 bytes, image size: %d bytes, load address: 0x%08X, "
                "entry point: 0x%08X, OS: %s, CPU: %s, image type: %d, compression type: %s, "
                "image name: \"%s\", %s"
                % (size, load, ep, OS_NAMES.get(os_, "Unix"), ARCH_NAMES.get(arch, "unknown"),
                   typ, COMP_NAMES.get(comp, "unknown"), name,
                   "big endian" if big else "little endian"))
        out.append((base + upos, desc, payload))
        inner = None
        if comp == 3:
            inner = lzma_alone_at(payload, 0) or raw_lzma_at(payload, 0)[0]
        elif comp == 1:
            try:
                inner = _inflate(payload, 0, 31)
            except Exception:
                inner = None
        elif comp == 2:
            inner = _bunzip(payload, 0)
        if inner:
            out.append((base + upos + 64, "LZMA compressed data, uncompressed size: %d bytes"
                        % len(inner), inner))
            _scan(inner, base + upos + 64, depth + 1, seen, out, budget)
        else:
            _scan(payload, base + upos + 64, depth + 1, seen, out, budget)

    # ---- vendor firmware headers ----------------------------------------
    if data[:4] == TRX:
        _trx(data, base, depth, seen, out)
    elif data[:4] == DLOB:
        _dlob(data, base, depth, seen, out)
    elif data[:4] == UBICOM:
        _ubicom(data, base, depth, seen, out)
    else:
        _tlv(data, base, depth, seen, out)

    # ---- compressed streams anywhere in the blob ------------------------
    found = scan_streams(data, base, depth, seen, out, budget)

    # ---- raw LZMA1 / raw DEFLATE near the start (headerless formats) -----
    # Both are brute force probes: only worth it when no header-carrying
    # stream was found, otherwise a plain image burns minutes here.
    # Headerless probes are O(n) attempts; keep the historical 24 MB ceiling
    # for them so a 200 MB disk image is sampled (see _stream_offsets) instead
    # of poked byte by byte.
    small = len(data) <= LARGE_SCAN
    if depth == 0 and not found and small:
        for m in range(0, min(len(data) - 16, 2048)):
            body, ds = raw_lzma_at(data, m)
            if not body:
                continue
            out.append((base + m, "LZMA compressed data, dictionary size: %d bytes, "
                        "uncompressed size: %d bytes" % (ds, len(body)), body))
            _scan(body, base + m, depth + 1, seen, out, budget)
            found = 1
            break
    if depth == 0 and not found and small:
        for m in range(0, min(len(data) - 16, 1024)):
            body, wb = deflate_at(data, m)
            if not body:
                continue
            out.append((base + m, "Zlib compressed data, uncompressed size: %d bytes" % len(body), body))
            _scan(body, base + m, depth + 1, seen, out, budget)
            break


def main():
    ap = argparse.ArgumentParser(add_help=True)
    ap.add_argument("-M", action="store_true")
    ap.add_argument("-e", action="store_true")
    ap.add_argument("-C", "--directory", default=None)
    ap.add_argument("file", nargs="?")
    a, _unknown = ap.parse_known_args()
    if not a.file:
        print("usage: vxbinwalk -Me -C <dir> <file>")
        return 2

    path = a.file
    data = open(path, "rb").read()
    base = os.path.basename(path)

    try:
        sys.stdout.reconfigure(errors="replace")
    except Exception:
        pass
    rows = scan_all(data)
    rows.sort(key=lambda r: r[0])

    outdir = a.directory or "."
    dest = os.path.join(outdir, "_%s.extracted" % base)
    os.makedirs(dest, exist_ok=True)
    written = set()
    for off, desc, payload in rows:
        name = "%X" % off
        if not payload or name in written:
            idx = 2
            while "%X_%d" % (off, idx) in written:
                idx += 1
            name = "%X_%d" % (off, idx)
        written.add(name)
        with open(os.path.join(dest, name), "wb") as f:
            f.write(payload[:MAX_OUT])

    print("%-34s %s" % ("DECIMAL", "HEXADECIMAL") + " " * 22 + "DESCRIPTION")
    print("-" * 110)
    for off, desc, _ in rows:
        desc = desc.encode("ascii", "replace").decode("ascii")
        print("%-34d 0x%-32X %s" % (off, off, desc))
    print("-" * 110)
    print("[i] carve: kept %d, dropped_tiny: %d (< %d bytes), truncated: %d (max %d)"
          % (LAST_CARVE_KEPT, LAST_DROPPED_TINY, CARVE_MIN, LAST_CARVE_TRUNC, CARVE_MAX))
    if LAST_TRUNCATED:
        print("[!] truncated: kept %d of %d layers (largest payloads)"
              % (LAST_TRUNCATED[1], LAST_TRUNCATED[0]))
    print("\nAnalyzed 1 file, %d signatures found, %d layers extracted"
          % (len(rows), len(written)))
    return 0


if __name__ == "__main__":
    sys.exit(main())