#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""vxsym -- independent VxWorks symbol-table detectors.

Public API
----------
    hits = find_symbols(data, detectors=("A", "B", "C"), deep=True)

`data` is a bytes object (a firmware image, an extracted container member or
anything else).  The return value is a list of hit dicts, strongest first::

    {"detector": "A" | "B" | "C",
     "offset": absolute byte offset of the evidence inside `data`,
     "size": bytes covered by the recovered table / cluster,
     "names_count": number of recovered symbol names,
     "names": [str, ...]            # first 48 names, evidence for humans
     "has_table": bool,             # a real NUL separated name table was carved
     "label": str,                  # human readable description
     "table_size": int,             # bytes of the carved name table (0 if none)
     "blob": bytes}                 # SYMBOL_Table material ("" when not kept)

Detectors
---------
A  Compressed symbol table.  LZMA-alone streams (properties byte 0x5A/0x5C/
   0x5D/0x5E/0x6E, the VxWorks/LZMA default set) whose decompressed payload
   carries at least two VxWorks markers (`bzero`, `usrRoot`, `taskSpawn`,
   `windFindNextSym`, `WIND version`).  If the decompressed image embeds a
   NUL separated symbol name table, that table is carved (`has_table`).
B  Uncompressed symbol table.  A run of >= 40 NUL separated C identifiers,
   >= 10 of them ending in a VxWorks symbol suffix (Task/Init/Isr/Lib/Hook/
   Create/Delete/...), and (>= 2 known VxWorks API names inside the run, or a
   >= 200 name application table inside an image carrying VxWorks markers).
C  Symbol density.  >= 15 distinct VxWorks API names inside one 64 KiB window
   (used for symbol tables that are fragmented or whose run boundaries are
   destroyed by padding/pointers).

Only the Python standard library is used.  The module never writes files and
never raises on malformed input: every detector is wrapped so a failure in one
layer cannot abort the scan.
"""

import hashlib
import io
import lzma
import re
import struct
import zipfile

# ---------------------------------------------------------------------------
# tunables
# ---------------------------------------------------------------------------
MAX_INFLATE = 64 << 20      # refuse to inflate more than 64 MiB per stream
MAX_DICT = 32 << 20         # largest LZMA dictionary we are willing to allocate
MAX_LZMA_BLOB = 48 << 20    # do not brute force LZMA inside blobs larger than this
MAX_NAME_BLOB = 48 << 20    # do not regex-scan blobs larger than this
MAX_DEPTH = 2               # follow LZMA layers up to this nesting level
MAX_STREAMS = 64            # LZMA candidates per blob
MAX_RAW_PROBE = 0x2000      # offsets probed for header-less raw LZMA1 streams
MAX_PER_DETECTOR = 6        # hits kept per detector per call
EVIDENCE_NAMES = 48         # names copied into hit["names"]

LZMA_PROPS = (0x5A, 0x5C, 0x5D, 0x5E, 0x6E)
_PROPS_RE = re.compile(rb"[\x5a\x5c-\x5e\x6e]\x00\x00")

A_MIN_MARKERS = 2
B_MIN_NAMES = 40
B_MIN_SUFFIX = 10
B_MIN_API = 2
B_MIN_BIG = 200            # a large table may hold only application symbols
C_WINDOW = 64 << 10
C_MIN_API = 15
MIN_BLOB = 1024

# VxWorks / Wind River API and symbol names used both by detector B (a real
# table has to contain a couple of them) and by detector C (density).
VX_API = (
    b"taskSpawn", b"taskInit", b"taskDelete", b"taskSuspend", b"taskResume",
    b"taskDelay", b"taskIdSelf", b"taskIdVerify", b"taskPrioritySet",
    b"taskPriorityGet", b"taskNameToId", b"taskIdToName", b"taskLock",
    b"taskUnlock", b"taskRestart", b"taskExit", b"taskSwitchHookAdd",
    b"semBCreate", b"semCCreate", b"semMCreate", b"semTake", b"semGive",
    b"semFlush", b"semDelete", b"semBCoreInit",
    b"msgQCreate", b"msgQDelete", b"msgQSend", b"msgQReceive", b"msgQNumMsgs",
    b"intLock", b"intUnlock", b"intEnable", b"intDisable", b"intConnect",
    b"intLockLevelSet",
    b"memPartCreate", b"memPartAlloc", b"memPartFree", b"memPartAddToPool",
    b"memPartShow",
    b"wdCreate", b"wdDelete", b"wdStart", b"wdCancel",
    b"tShell", b"usrRoot", b"usrKernelInit", b"usrInit", b"usrShell",
    b"usrNetworkInit", b"usrToolsInit", b"usrSerialInit", b"usrShowInit",
    b"excJobAdd", b"excHookAdd", b"excTask", b"excShowInit",
    b"bzero", b"bcopy", b"bcmp",
    b"windFindNextSym", b"symFindByName", b"symFindByValue", b"symLibInit",
    b"symAdd", b"logMsg",
    b"iosDevAdd", b"iosDevFind", b"iosLibInit", b"iosOpen", b"iosClose",
    b"iosRead", b"iosWrite", b"iosIoctl",
    b"cacheFlush", b"cacheInvalidate", b"cacheEnable", b"cacheDisable",
    b"kernelInit", b"kernelVersion",
    b"objFree", b"objAlloc",
    b"tickGet", b"tickAnnounce", b"sysClkRateSet", b"sysClkRateGet",
    b"pipeDevCreate", b"pipeDevDelete",
    b"dosFsInit", b"nfsInit", b"ifLibInit", b"endFindByName",
)
_VX_API_SET = frozenset(VX_API)

VX_MARKERS = (b"bzero", b"usrRoot", b"taskSpawn", b"windFindNextSym",
              b"WIND version")

NAME_SUFFIXES = (
    b"Task", b"Init", b"Isr", b"Lib", b"Hook", b"Create", b"Delete",
    b"Rtn", b"Msg", b"Sem", b"Dev", b"Show", b"Free", b"Alloc",
)

# A run of NUL separated C identifiers.  VxWorks pads entries to 4/8 byte
# alignment, so 1..8 NULs may separate two names.  The trailing group keeps the
# match contiguous; at least 16 names are required before we bother scoring.
_NAME_RUN_RE = re.compile(rb"(?:[A-Za-z_][A-Za-z0-9_]{0,255}\x00{1,8}){16,}")



def _dec_names(names, limit=EVIDENCE_NAMES):
    out = []
    for n in names[:limit]:
        try:
            out.append(n.decode("ascii"))
        except UnicodeDecodeError:
            out.append(n.decode("ascii", "replace"))
    return out


# ---------------------------------------------------------------------------
# detector A helpers: LZMA-alone stream discovery
# ---------------------------------------------------------------------------
def _lzma_streams(blob, max_streams=MAX_STREAMS):
    """Yield (offset, decompressed_bytes) for LZMA-alone streams in `blob`.

    A candidate must look like a real LZMA-alone header: known properties byte,
    two zero bytes, a sane dictionary size and either an unknown (0xFFFF...)
    or a sane uncompressed size.  Only then is a bounded inflate attempted.
    """
    out = []
    for m in _PROPS_RE.finditer(blob):
        if len(out) >= max_streams:
            break
        i = m.start()
        if i > len(blob) - 16:
            break
        dict_size = struct.unpack_from("<I", blob, i + 1)[0]
        unpacked = struct.unpack_from("<Q", blob, i + 5)[0]
        if not (4096 <= dict_size <= MAX_DICT):
            continue
        if unpacked != 0xFFFFFFFFFFFFFFFF and not (4096 <= unpacked <= MAX_INFLATE):
            continue
        dec = None
        try:
            d = lzma.LZMADecompressor(format=lzma.FORMAT_ALONE)
            dec = d.decompress(blob[i:], max_length=MAX_INFLATE)
        except Exception:
            dec = None
        if dec is not None and MIN_BLOB <= len(dec) <= MAX_INFLATE:
            out.append((i, dec))
    return out


def _raw_lzma_streams(blob, max_streams=2):
    """Probe for header-less raw LZMA1 streams (Netgear .chk and friends).

    A raw LZMA1 stream always starts with the 0x00 range-coder byte, so only
    those offsets are attempted; that keeps the 8 KiB probe at a few dozen
    inflate calls even on multi-megabyte images.
    """
    out = []
    n = len(blob)
    limit = min(n - 16, MAX_RAW_PROBE)
    for off in range(limit):
        if blob[off] != 0x00:
            continue
        for ds in (8 << 20, 1 << 20, 64 << 10):
            dec = None
            try:
                d = lzma.LZMADecompressor(
                    format=lzma.FORMAT_RAW,
                    filters=[{"id": lzma.FILTER_LZMA1, "dict_size": ds}])
                dec = d.decompress(blob[off:], max_length=MAX_INFLATE)
            except Exception:
                dec = None
            if dec is not None and MIN_BLOB <= len(dec) <= MAX_INFLATE:
                out.append((off, dec))
                break
        if len(out) >= max_streams:
            break
    return out


# ---------------------------------------------------------------------------
# detector B helper: uncompressed name table carving
# ---------------------------------------------------------------------------
def _scan_name_table(blob, markers=None):
    """Find the largest VxWorks symbol name table in `blob`.

    Returns (offset, table_bytes, names, suffix_count) or None.
    """
    if len(blob) < B_MIN_NAMES * 2 or len(blob) > MAX_NAME_BLOB:
        return None
    if markers is None:
        markers = sum(1 for m in VX_MARKERS if m in blob)
    best = None
    for m in _NAME_RUN_RE.finditer(blob):
        start, end = m.start(), m.end()
        # The left edge of a regex match can fall on the tail of a non-symbol
        # string ("...Lib.c", or a stray byte pair inside code).  Drop up to a
        # few 1-2 character tokens so the carved table starts on a real name.
        for _ in range(8):
            nul = blob.find(b"\x00", start, end)
            if nul < 0:
                break
            tok = blob[start:nul]
            if len(tok) > 2:
                break
            start = nul + 1
            while start < end and blob[start] == 0:
                start += 1
        names = [t for t in blob[start:end].split(b"\x00") if t]
        if len(names) < B_MIN_NAMES:
            continue
        suffix = 0
        for t in names:
            if t.endswith(NAME_SUFFIXES):
                suffix += 1
        if suffix < B_MIN_SUFFIX:
            continue
        api = 0
        for t in names:
            if t in _VX_API_SET:
                api += 1
        # Normal case: the table itself carries VxWorks API symbols.  Some
        # application symbol tables (e.g. printer firmware) only hold program
        # symbols, so accept a very large table when the surrounding image has
        # the usual VxWorks markers.
        if api < B_MIN_API and not (len(names) >= B_MIN_BIG and
                                    markers >= A_MIN_MARKERS):
            continue
        if best is None or len(names) > len(best[2]):
            best = (start, blob[start:end], names, suffix)
    return best


# ---------------------------------------------------------------------------
# detector C helper: API name density
# ---------------------------------------------------------------------------
def _api_positions(blob):
    pos = []
    for name in VX_API:
        start = 0
        while True:
            i = blob.find(name, start)
            if i < 0:
                break
            pos.append((i, name))
            start = i + 1
    pos.sort()
    return pos


def _scan_density(blob):
    """Find a 64 KiB window with >= C_MIN_API distinct VxWorks API names.

    Returns (offset, size, distinct_names) or None.
    """
    if len(blob) < C_WINDOW // 2 or len(blob) > MAX_NAME_BLOB:
        return None
    pos = _api_positions(blob)
    if len(pos) < C_MIN_API:
        return None
    counts = {}
    right = 0
    best = None
    for left in range(len(pos)):
        if right < left:
            right = left
        limit = pos[left][0] + C_WINDOW
        while right < len(pos) and pos[right][0] < limit:
            name = pos[right][1]
            counts[name] = counts.get(name, 0) + 1
            right += 1
        if len(counts) > (best[2] if best else C_MIN_API - 1):
            if len(counts) >= C_MIN_API:
                win_start = pos[left][0]
                last_off, last_name = pos[right - 1]
                win_end = last_off + len(last_name)
                best = (win_start, win_end - win_start, len(counts),
                        sorted(counts))
        # slide left edge
        name = pos[left][1]
        c = counts.get(name, 1) - 1
        if c <= 0:
            counts.pop(name, None)
        else:
            counts[name] = c
    if best is None:
        return None
    return best[0], best[1], best[3]


# ---------------------------------------------------------------------------
# layer expansion
# ---------------------------------------------------------------------------
def _blob_key(blob):
    h = hashlib.md5()
    h.update(blob[:4096])
    h.update(blob[-4096:])
    return (len(blob), h.hexdigest())


def iter_layers(data, deep=True, max_depth=MAX_DEPTH):
    """Yield (offset, kind, blob, depth) for `data` and every decompressed layer.

    kind is "raw" for the input itself, "zip" for a member of an embedded zip
    and "lzma" for an LZMA stream.  `offset` is the absolute byte offset of the
    layer inside `data` for depth 1 (and relative to the parent layer's inflated
    image for deeper nesting, which is rare).
    """
    if not data:
        return
    seen = set()
    key = _blob_key(data)
    seen.add(key)
    yield 0, "raw", data, 0

    queue = []
    kind0 = "raw"
    if len(data) >= 4096 and zipfile.is_zipfile(io.BytesIO(data)):
        try:
            with zipfile.ZipFile(io.BytesIO(data)) as z:
                for item in z.infolist():
                    if item.is_dir() or item.file_size < MIN_BLOB:
                        continue
                    body = z.read(item)
                    k = _blob_key(body)
                    if k in seen:
                        continue
                    seen.add(k)
                    yield 0, "zip", body, 0
                    queue.append((0, body, 0))
            kind0 = "zip"
        except Exception:
            pass
    if kind0 == "raw":
        queue.append((0, data, 0))

    while queue:
        base, blob, depth = queue.pop(0)
        if depth >= (max_depth if deep else 0):
            continue
        if len(blob) < MIN_BLOB or len(blob) > MAX_LZMA_BLOB:
            continue
        try:
            streams = _lzma_streams(blob)
        except Exception:
            streams = []
        if not streams and depth == 0 and len(blob) <= (24 << 20):
            # Raw LZMA1 has no magic, so probing is the expensive path.  When
            # the blob already exposes an uncompressed name table there is
            # nothing to gain, so skip it.
            hint = None
            try:
                hint = _scan_name_table(blob)
            except Exception:
                hint = None
            if hint is None:
                try:
                    streams = _raw_lzma_streams(blob)
                except Exception:
                    streams = []
        for off, dec in streams:
            k = _blob_key(dec)
            if k in seen:
                continue
            seen.add(k)
            yield base + off, "lzma", dec, depth + 1
            queue.append((base + off, dec, depth + 1))


# ---------------------------------------------------------------------------
# public API
# ---------------------------------------------------------------------------
def _mk_hit(detector, offset, size, names, has_table, label, blob=b"", table_size=0):
    return {
        "detector": detector,
        "offset": int(offset),
        "size": int(size),
        "names_count": len(names),
        "names": _dec_names(names),
        "has_table": bool(has_table),
        "label": label,
        "table_size": int(table_size),
        "blob": blob,
    }


def _rank(h):
    return (1 if h["has_table"] else 0, h["names_count"], h["size"])


def find_symbols(data, detectors=("A", "B", "C"), deep=True, max_depth=MAX_DEPTH):
    """Recover VxWorks symbol table evidence from `data`.

    See the module docstring for the returned dict layout and the detector
    definitions.  The result is never None; an empty list means no evidence.
    """
    if not data or len(data) < MIN_BLOB:
        return []

    detectors = tuple(d.upper() for d in detectors)
    hits = []

    for offset, kind, blob, depth in iter_layers(data, deep=deep,
                                                 max_depth=max_depth):
        if len(blob) < MIN_BLOB:
            continue

        a_table = False
        if kind == "lzma" and "A" in detectors:
            hit = _detect_a(offset, blob)
            if hit is not None:
                hit["depth"] = depth
                hits.append(hit)
                a_table = hit["has_table"]

        table = None
        if kind != "lzma" and "B" in detectors:
            try:
                table = _scan_name_table(blob)
            except Exception:
                table = None
            if table is not None:
                start, raw, names, suffix = table
                label = ("uncompressed symbol table: %d names, %d suffixes, "
                         "kind=%s" % (len(names), suffix, kind))
                hit = _mk_hit("B", offset + start, len(raw), names, True,
                              label, raw, len(raw))
                hit["depth"] = depth
                hits.append(hit)

        if "C" in detectors and table is None and not a_table:
            try:
                dens = _scan_density(blob)
            except Exception:
                dens = None
            if dens is not None:
                start, size, names = dens
                label = ("symbol density: %d distinct VxWorks API names in "
                         "one 64 KiB window, kind=%s" % (len(names), kind))
                hit = _mk_hit("C", offset + start, size, names, False, label)
                hit["depth"] = depth
                hits.append(hit)

    return _dedupe_and_rank(hits)


def _detect_a(offset, dec):
    markers = sum(1 for m in VX_MARKERS if m in dec)
    if markers < A_MIN_MARKERS:
        return None
    api_names = [n for n in VX_API if n in dec]
    if not api_names:
        return None
    table = None
    try:
        table = _scan_name_table(dec, markers=markers)
    except Exception:
        table = None
    if table is not None:
        start, raw, names, suffix = table
        label = ("compressed symbol table: LZMA stream, markers=%d, carved "
                 "%d names / %d suffixes" % (markers, len(names), suffix))
        hit = _mk_hit("A", offset, len(raw), names, True, label, raw, len(raw))
        hit["table_offset"] = offset + start
        hit["stream_size"] = len(dec)
        return hit
    label = ("compressed VxWorks image: LZMA stream, markers=%d, %d API names"
             % (markers, len(api_names)))
    hit = _mk_hit("A", offset, len(dec), api_names, False, label)
    hit["stream_size"] = len(dec)
    return hit


def recover_symbol_table(data, min_names=B_MIN_NAMES):
    """Convenience wrapper for integration.

    Returns the strongest hit that carries a real carved name table
    (hit["blob"] is the SYMBOL_Table payload), or None.
    """
    for hit in find_symbols(data, detectors=("A", "B")):
        if hit["has_table"] and hit["names_count"] >= min_names and hit["blob"]:
            return hit
    return None


def _dedupe_and_rank(hits):
    best = {}
    for h in hits:
        key = (h["detector"], hashlib.md5(h["blob"][:4096] if h["blob"]
                                         else b"%d:%d" % (h["offset"], h["size"]))
               .hexdigest())
        cur = best.get(key)
        if cur is None or _rank(h) > _rank(cur):
            best[key] = h
    out = list(best.values())
    out.sort(key=lambda h: (h["detector"], -h["names_count"], h["offset"]))
    trimmed = []
    for det in ("A", "B", "C"):
        group = [h for h in out if h["detector"] == det]
        group.sort(key=_rank, reverse=True)
        kept = group[:MAX_PER_DETECTOR]
        # keep the blob only for the strongest hits to bound memory
        for i, h in enumerate(kept):
            if i >= 2:
                h["blob"] = b""
        trimmed.extend(kept)
    trimmed.sort(key=lambda h: (1 if h["has_table"] else 0, h["names_count"],
                                 h["size"], h["detector"]), reverse=True)
    return trimmed
