//! VxWorks judge — port of tools/verdict.py
use std::io::Read;
use xz2::read::XzDecoder;
use xz2::stream::Stream;

const SIG: &[&[u8]] = &[
    b"WIND version",
    b"vxWorks.tgt",
    b"windVersion",
    b"taskSpawn",
    b"usrRoot",
    b"windFindNextSym",
];
const LINUX: &[&[u8]] = &[b"hsqs", b"cramfs", b"UBI#", b"jffs2", b"busybox"];

#[derive(Debug, Clone, Default)]
pub struct Verdict {
    pub is_vx: bool,
    pub versions: Vec<String>,
    pub evidence: Vec<(String, usize)>,
    pub linux: Vec<String>,
    pub uboot_table: bool,
}

fn count(hay: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() || needle.len() > hay.len() {
        return 0;
    }
    memchr::memmem::find_iter(hay, needle).count()
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    memchr::memmem::find(hay, needle).is_some()
}

/// Extract "VxWorks\x20?<major>.<minor>[.<patch>]" version strings.
fn versions_of(data: &[u8]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let key = b"VxWorks";
    let mut pos = 0usize;
    while let Some(i) = memchr::memmem::find(&data[pos..], key) {
        let mut p = pos + i + key.len();
        pos = p;
        while p < data.len() && (data[p] == b' ' || data[p] == b'\t') {
            p += 1;
        }
        let start = p;
        let mut dots = 0usize;
        while p < data.len() {
            let c = data[p];
            if c.is_ascii_digit() {
                p += 1;
            } else if c == b'.' && dots < 2 {
                dots += 1;
                p += 1;
            } else {
                break;
            }
        }
        if p > start && dots >= 1 {
            if let Ok(s) = std::str::from_utf8(&data[start..p]) {
                if s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
                    && s.rsplit('.').next().map(|t| !t.is_empty()).unwrap_or(false)
                {
                    if !out.iter().any(|x| x == s) {
                        out.push(s.to_string());
                    }
                }
            }
        }
        pos = pos.max(start + 1);
    }
    out.sort();
    out
}

pub fn judge(data: &[u8]) -> Verdict {
    let versions = versions_of(data);
    let mut evidence = Vec::new();
    for s in SIG {
        let n = count(data, s);
        if n > 0 {
            evidence.push((String::from_utf8_lossy(s).to_string(), n));
        }
    }
    let linux: Vec<String> = LINUX
        .iter()
        .filter(|l| contains(data, l))
        .map(|l| String::from_utf8_lossy(l).to_string())
        .collect();
    let uboot_table = contains(data, b"Invalid OS")
        || (contains(data, b"OpenBSD") && contains(data, b"NetBSD") && contains(data, b"VxWorks"));
    let kernel = contains(data, b"WIND version")
        || contains(data, b"vxWorks.tgt")
        || contains(data, b"windVersion")
        || (contains(data, b"taskSpawn") && contains(data, b"usrRoot"));
    let is_vx = !versions.is_empty() || (kernel && !uboot_table);
    Verdict { is_vx, versions, evidence, linux, uboot_table }
}

/// Decode an LZMA-alone (.lzma) stream that starts at `data[0]`.
/// Format: 1 byte props (lc/lp/pb), 4 bytes dict size LE, 8 bytes usize, then raw stream.
fn decode_alone(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 14 {
        return None;
    }
    let stream = Stream::new_lzma_decoder(u64::MAX).ok()?;
    let mut dec = XzDecoder::new_stream(&data[..], stream);
    let mut buf = Vec::new();
    let _ = dec.read_to_end(&mut buf);
    if buf.len() > 1024 && buf.len() <= (96 << 20) {
        Some(buf)
    } else {
        None
    }
}

/// Decode a headerless raw LZMA1 stream by synthesising a 13-byte .lzma header.
fn decode_raw_lzma(data: &[u8], dict: u32) -> Option<Vec<u8>> {
    if data.len() < 16 {
        return None;
    }
    let mut framed = Vec::with_capacity(data.len() + 13);
    framed.push(0x5D); // lc=3, lp=0, pb=2
    framed.extend_from_slice(&dict.max(4096).to_le_bytes());
    framed.extend_from_slice(&[0xFF; 8]); // unknown uncompressed size
    framed.extend_from_slice(data);
    let stream = Stream::new_lzma_decoder(u64::MAX).ok()?;
    let mut dec = XzDecoder::new_stream(&framed[..], stream);
    let mut buf = Vec::new();
    let _ = dec.read_to_end(&mut buf);
    if buf.len() > 65536 {
        Some(buf)
    } else {
        None
    }
}

/// LZMA-alone streams (props 5d/6e + 8-byte dict).
pub fn lzma_alone_streams(data: &[u8], limit: usize) -> Vec<(usize, Vec<u8>)> {
    let mut out = Vec::new();
    let mut n = 0usize;
    for pat in [&b"\x5d\x00\x00"[..], &b"\x6e\x00\x00"[..]] {
        let mut off = 0usize;
        while n < limit {
            let rel = match memchr::memmem::find(&data[off..], pat) {
                Some(i) => i,
                None => break,
            };
            off += rel;
            n += 1;
            if let Some(buf) = decode_alone(&data[off..]) {
                out.push((off, buf));
            }
            off += 1;
        }
    }
    out
}

/// Raw LZMA1 streams (Netgear CHK style), try a few dict sizes.
pub fn raw_lzma_streams(data: &[u8], max_off: usize) -> Vec<(usize, u32, Vec<u8>)> {
    let mut out = Vec::new();
    let end = max_off.min(data.len().saturating_sub(16));
    for off in 0..end {
        for ds in [8u32 << 20, 1 << 20, 1 << 16] {
            if let Some(buf) = decode_raw_lzma(&data[off..], ds) {
                out.push((off, ds, buf));
                break;
            }
        }
    }
    out
}

/// zlib streams by header, plus raw deflate in the first 4 KiB.
pub fn deflate_streams(data: &[u8]) -> Vec<(usize, i32, Vec<u8>)> {
    let mut out = Vec::new();
    for sig in [&b"\x78\x9c"[..], &b"\x78\x01"[..], &b"\x78\xda"[..], &b"\x78\x5e"[..]] {
        let mut pos = 0usize;
        while let Some(i) = memchr::memmem::find(&data[pos..], sig) {
            pos += i;
            let mut dec = flate2::read::ZlibDecoder::new(&data[pos..]);
            let mut buf = Vec::new();
            let _ = dec.read_to_end(&mut buf);
            if buf.len() > 65536 {
                out.push((pos, 15, buf));
            }
            pos += 1;
        }
    }
    let raw_end = 4096.min(data.len().saturating_sub(16));
    for off in 0..raw_end {
        let mut dec = flate2::read::DeflateDecoder::new(&data[off..]);
        let mut buf = Vec::new();
        let _ = dec.read_to_end(&mut buf);
        if buf.len() > 65536 {
            out.push((off, -15, buf));
            break;
        }
    }
    out
}

#[derive(Debug, Clone, Default)]
pub struct FileVerdict {
    pub verdict: String,
    pub via: String,
    pub versions: Vec<String>,
    pub evidence: Vec<(String, usize)>,
    pub linux: Vec<String>,
    pub uboot_table: bool,
    pub inner: String,
}

/// Judge a complete image (already unwrapped from zip/rar).
pub fn judge_image(data: &[u8], inner: &str) -> FileVerdict {
    let mut out = FileVerdict {
        verdict: "unknown".into(),
        via: "none".into(),
        inner: inner.to_string(),
        ..Default::default()
    };
    let j = judge(data);
    if j.is_vx {
        out.verdict = "vxworks".into();
        out.via = "plaintext".into();
        out.versions = j.versions;
        out.evidence = j.evidence;
        out.linux = j.linux;
        out.uboot_table = j.uboot_table;
        return out;
    }
    for (off, dec) in lzma_alone_streams(data, 80) {
        let j2 = judge(&dec);
        if j2.is_vx {
            out.verdict = "vxworks".into();
            out.via = format!("lzma@0x{:x}", off);
            out.versions = j2.versions;
            out.evidence = j2.evidence;
            return out;
        }
    }
    for (off, ds, dec) in raw_lzma_streams(data, 1024) {
        let j3 = judge(&dec);
        if j3.is_vx {
            out.verdict = "vxworks".into();
            out.via = format!("rawlzma@0x{:x}/dict{}", off, ds);
            out.versions = j3.versions;
            out.evidence = j3.evidence;
            return out;
        }
    }
    for (off, wbits, dec) in deflate_streams(data) {
        let j4 = judge(&dec);
        if j4.is_vx {
            out.verdict = "vxworks".into();
            out.via = format!("deflate@0x{:x}/w{}", off, wbits);
            out.versions = j4.versions;
            out.evidence = j4.evidence;
            return out;
        }
    }
    if !j.linux.is_empty() {
        out.verdict = "linux".into();
        out.via = "plaintext".into();
        out.linux = j.linux;
        out.evidence = j.evidence;
    } else if j.uboot_table {
        out.verdict = "linux".into();
        out.via = "uboot-table".into();
        out.evidence = vec![("uboot OS-name table only".into(), 1)];
    }
    out
}