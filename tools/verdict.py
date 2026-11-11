#!/usr/bin/env python3
"""Canonical VxWorks judge for firmware images (U-Boot-string-table aware).

Rules (all evidence collected, verdict derived):
  version   : regex VxWorks\d+\.\d+            -> decisive
  kernel    : "WIND version" / "vxWorks.tgt" / "windVersion" / taskSpawn+usrRoot
  uboot_tbl : "Invalid OS"+"NetBSD" (U-Boot's OS name table => NOT evidence)
  linux     : hsqs/cramfs/UBI#/jffs2/busybox
"""
import glob, lzma, os, re, subprocess, tempfile, zipfile, zlib

VX_VER = re.compile(rb"VxWorks\s?(\d+\.\d+(?:\.\d+)?)")
SIG = [b"WIND version", b"vxWorks.tgt", b"windVersion", b"taskSpawn", b"usrRoot", b"windFindNextSym"]
LINUX = [b"hsqs", b"cramfs", b"UBI#", b"jffs2", b"busybox"]
LZ = [b"\x5d\x00\x00", b"\x6e\x00\x00"]


def judge(data):
    ev = {}
    vers = sorted({m.decode() for m in VX_VER.findall(data)})
    for s in SIG:
        n = data.count(s)
        if n:
            ev[s.decode()] = n
    linux = [l.decode() for l in LINUX if l in data]
    uboot_tbl = (b"Invalid OS" in data) or (b"OpenBSD" in data and b"NetBSD" in data and b"VxWorks" in data)
    kernel = (b"WIND version" in data) or (b"vxWorks.tgt" in data) or (b"windVersion" in data) \
             or ((b"taskSpawn" in data) and (b"usrRoot" in data))
    is_vx = bool(vers) or (kernel and not uboot_tbl)
    return {"is_vx": is_vx, "versions": vers, "evidence": ev, "linux": linux,
            "uboot_table": uboot_tbl}


def _lzma_streams(data, limit=80):
    n = 0
    for pat in LZ:
        off = 0
        while n < limit:
            off = data.find(pat, off)
            if off < 0:
                break
            n += 1
            try:
                dec = lzma.decompress(data[off:], format=lzma.FORMAT_ALONE)
            except Exception:
                off += 1
                continue
            if 1024 < len(dec) <= (96 << 20):
                yield off, dec
            off += 1




def _raw_lzma_streams(data, max_off=1024, limit=48 << 20):
    """Broadcom/Netgear style: raw LZMA1 (no container header) right after a small header."""
    n = min(len(data) - 16, max_off)
    for off in range(0, max(0, n)):
        for ds in (1 << 23, 1 << 20, 1 << 16):
            try:
                dec = lzma.LZMADecompressor(format=lzma.FORMAT_RAW,
                                            filters=[{"id": lzma.FILTER_LZMA1, "dict_size": ds}])
                out = dec.decompress(data[off:], limit)
            except Exception:
                continue
            if len(out) > 65536:
                yield off, ds, out
                break




SEVENZ = r"C:\Program Files\7-Zip\7z.exe"


def _rar_extract(path):
    """RAR masquerading as .zip (TL-R460 v5.0, TL-WR340G v2) -> largest member bytes."""
    if not os.path.exists(SEVENZ):
        return None
    tmp = tempfile.mkdtemp()
    try:
        subprocess.run([SEVENZ, "x", "-y", "-o" + tmp, path],
                       capture_output=True, timeout=180)
        best, size = None, -1
        for f in glob.glob(os.path.join(tmp, "**", "*"), recursive=True):
            if os.path.isfile(f) and os.path.getsize(f) > size:
                best, size = f, os.path.getsize(f)
        if best and size > 4096:
            return open(best, "rb").read(200 << 20)
    except Exception:
        return None
    return None




def _deflate_streams(data, max_off=0x10000, limit=48 << 20):
    """zlib 流按 78 9C/01/DA 头快速定位；裸 deflate 只在头部 4KB 内试。"""
    for sig, wbits in ((b"\x78\x9c", 15), (b"\x78\x01", 15), (b"\x78\xda", 15), (b"\x78\x5e", 15)):
        pos = 0
        while True:
            pos = data.find(sig, pos)
            if pos < 0 or pos > len(data) - 16:
                break
            try:
                d = zlib.decompressobj(wbits)
                out = d.decompress(data[pos:], limit)
            except Exception:
                pos += 1
                continue
            if len(out) > 65536:
                yield pos, wbits, out
            pos += 1
    for off in range(0, min(len(data) - 16, 4096)):
        try:
            d = zlib.decompressobj(-15)
            out = d.decompress(data[off:], limit)
        except Exception:
            continue
        if len(out) > 65536:
            yield off, -15, out
            break


def inspect_file(path):
    """Return dict(file-level) with inner image info."""
    from os.path import basename
    out = {"verdict": "unknown", "via": "none", "versions": [], "evidence": "",
           "linux": "", "inner": basename(path), "uboot_table": False}
    try:
        head = open(path, "rb").read(8)
        via0 = "plaintext"
        if head.startswith(b"Rar!\x1a\x07"):
            data = _rar_extract(path)
            if not data:
                out["evidence"] = "rar-unreadable"
                return out
            via0 = "rar"
            out["inner"] = os.path.basename(path)
        elif zipfile.is_zipfile(path):
            with zipfile.ZipFile(path) as z:
                items = [i for i in z.infolist() if not i.is_dir()]
                if not items:
                    return out
                item = max(items, key=lambda i: i.file_size)
                if item.file_size > 200 << 20:
                    return out
                data = z.read(item)
                out["inner"] = item.filename
        else:
            data = open(path, "rb").read(200 << 20)
    except Exception as e:
        out["evidence"] = "read-err:%s" % e
        return out

    j = judge(data)
    if j["is_vx"]:
        out.update(verdict="vxworks", via=via0, versions=j["versions"],
                   evidence=str(j["evidence"]), linux=",".join(j["linux"]))
        return out
    for off, dec in _lzma_streams(data):
        j2 = judge(dec)
        if j2["is_vx"]:
            out.update(verdict="vxworks", via="lzma@0x%x" % off, versions=j2["versions"],
                       evidence=str(j2["evidence"]), linux=",".join(j2["linux"]))
            return out
    for off, ds, dec in _raw_lzma_streams(data):
        j3 = judge(dec)
        if j3["is_vx"]:
            out.update(verdict="vxworks", via="rawlzma@0x%x/dict%d" % (off, ds),
                       versions=j3["versions"], evidence=str(j3["evidence"]),
                       linux=",".join(j3["linux"]))
            return out
    for off, wbits, dec in _deflate_streams(data):
        j4 = judge(dec)
        if j4["is_vx"]:
            out.update(verdict="vxworks", via="deflate@0x%x/w%d" % (off, wbits),
                       versions=j4["versions"], evidence=str(j4["evidence"]),
                       linux=",".join(j4["linux"]))
            return out
    if j["linux"]:
        out.update(verdict="linux", via=via0, linux=",".join(j["linux"]),
                   evidence=str(j["evidence"]))
    elif j["uboot_table"]:
        out.update(verdict="linux", via="uboot-table", evidence="uboot OS-name table only")
    return out


if __name__ == "__main__":
    import json, sys
    for p in sys.argv[1:]:
        print(p, json.dumps(inspect_file(p), ensure_ascii=False))

