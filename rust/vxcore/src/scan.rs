//! Container / compressed-stream scanner -- Rust port of `tools/vxbinwalk.py`.
//!
//! The Python tool does two unrelated jobs in one pass:
//!   1. it emits "layers" (containers, decompressed streams), and
//!   2. it carves JPEG/PNG/GIF and HTML/JS/CSS fragments out of every blob.
//! `Layer`/`scan_all` model only job (1); the kind vocabulary in the porting
//! contract has no image/web members, so the Python carving rows are filtered
//! out by `tests/parity.rs` before the two layer lists are compared.
//!
//! Everything else follows the Python control flow closely: same magic tests,
//! same recursion order, same `seen` de-duplication key (first 4096 bytes plus
//! the decimal length), same per-kind stream caps, same 512 MiB budget, same
//! 24 MiB large-file sampling and the same 300-layer cap (keep the biggest
//! payloads, restore discovery order).
//!
//! `xz2` 0.1.7 (the version pinned by Cargo.toml and available offline) has no
//! `Stream::new_raw_decoder`; the documented `new_raw_decoder` entry point does
//! not exist in this release. Raw LZMA1 is therefore decoded by synthesising
//! the 13-byte LZMA-alone header (properties 0x5D, the requested dictionary
//! size, unknown uncompressed size) in front of the raw stream and using
//! `Stream::new_lzma_decoder`, which is bit-for-bit the same LZMA1 decoder and
//! reproduces Python's `FORMAT_RAW` + `FILTER_LZMA1` results (verified against
//! the Netgear CHK streams). No format is silently skipped.

use std::collections::HashSet;

use bzip2::Decompress as BzDecompress;
use bzip2::Status as BzStatus;
use flate2::{Decompress as FlateDecompress, FlushDecompress, Status as FlateStatus};
use memchr::memmem;
use xz2::stream::{Action, Status as XzStatus, Stream};

/// One extracted layer.
#[derive(Debug, Clone)]
pub struct Layer {
    /// Offset of the layer inside the input data.
    pub offset: usize,
    /// "lzma-alone" / "raw-lzma" / "deflate" / "gzip" / "bzip2" / "xz" /
    /// "zip" / "uimage" / "tplink" / "trx" / "dlob" / "ubicom" / "tlv".
    /// "unsupported-*" is emitted only where the pinned dependency set cannot
    /// reproduce a Python-supported format (currently only rare ZIP methods).
    pub kind: String,
    /// Extracted payload.
    pub data: Vec<u8>,
}

const UIMAGE: &[u8] = b"\x27\x05\x19\x56";
const TPLINK: &[u8] = b"\x00\x14\x2f\xc0";
const TRX: &[u8] = b"HDR0";
const DLOB: &[u8] = b"DLOB";
const UBICOM: &[u8] = b"\xfa\x32\x00\x80";

const MAX_OUT: usize = 256 << 20;
/// Blobs bigger than this are sampled block by block instead of byte by byte.
const LARGE_SCAN: usize = 24 << 20;
/// Sampling step used for the cheap in-block probe of large blobs.
const LARGE_STEP: usize = 256;
/// Upper bound for everything a single scan may pull out of one file.
const MAX_TOTAL_OUT: usize = 512 << 20;
/// Hard cap on layers per sample (keep the biggest payloads).
const MAX_ROWS: usize = 300;
/// Historical constant from the Python module: declared but never enforced.
/// The effective recursion limits are `MAX_DEPTH` and `MAX_STREAM_DEPTH`.
#[allow(dead_code)]
const MAX_LAYERS: usize = 12;
/// `_scan()` refuses to recurse deeper than this (Python: `depth > 5`).
const MAX_DEPTH: usize = 5;
/// `scan_streams()` refuses to recurse deeper than this (Python: `depth > 3`).
const MAX_STREAM_DEPTH: usize = 3;

const LZMA_PROPS: [u8; 5] = [0x5C, 0x5D, 0x5E, 0x5A, 0x6E];

fn ok_layer(body: Vec<u8>) -> Option<Vec<u8>> {
    if body.len() > 1024 && body.len() <= MAX_OUT {
        Some(body)
    } else {
        None
    }
}

fn le16(d: &[u8], o: usize) -> usize {
    u16::from_le_bytes([d[o], d[o + 1]]) as usize
}

fn le32(d: &[u8], o: usize) -> usize {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]) as usize
}

fn be32(d: &[u8], o: usize) -> usize {
    u32::from_be_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]) as usize
}

// --------------------------------------------------------------------------
// public entry point
// --------------------------------------------------------------------------

/// Scan `data` and return every layer Python's `vxbinwalk.scan_all` would
/// produce, minus the image/HTML/JS carvings.
pub fn scan_all(data: &[u8]) -> Vec<Layer> {
    let mut ctx = Ctx {
        out: Vec::new(),
        seen: HashSet::new(),
        budget: MAX_TOTAL_OUT,
    };
    ctx.scan(data, 0, 0);
    cap_rows(ctx.out)
}

/// Keep at most `MAX_ROWS` layers, preferring the biggest payloads, then
/// restore the original discovery order (Python `_cap_rows`).
fn cap_rows(out: Vec<Layer>) -> Vec<Layer> {
    if out.len() <= MAX_ROWS {
        return out;
    }
    let mut slots: Vec<Option<Layer>> = out.into_iter().map(Some).collect();
    let mut keep: Vec<usize> = (0..slots.len()).collect();
    // Stable sort: equal payload sizes stay in discovery order.
    keep.sort_by(|&a, &b| {
        let la = slots[b].as_ref().map(|l| l.data.len()).unwrap_or(0);
        let lb = slots[a].as_ref().map(|l| l.data.len()).unwrap_or(0);
        la.cmp(&lb)
    });
    keep.truncate(MAX_ROWS);
    keep.sort_unstable();
    keep.into_iter()
        .filter_map(|i| slots[i].take())
        .collect()
}

// --------------------------------------------------------------------------
// recursive scanner
// --------------------------------------------------------------------------

struct Ctx {
    out: Vec<Layer>,
    seen: HashSet<(usize, Vec<u8>)>,
    budget: usize,
}

impl Ctx {
    /// Push a layer and recurse into its payload at `scan_base`.
    ///
    /// The payload is moved into the layer *before* recursing (so output order
    /// matches Python), then temporarily taken out to satisfy the borrow
    /// checker. This avoids cloning multi-megabyte payloads.
    fn emit_scan(&mut self, offset: usize, kind: &str, body: Vec<u8>, scan_base: usize, depth: usize) {
        self.out.push(Layer {
            offset,
            kind: kind.to_string(),
            data: body,
        });
        let idx = self.out.len() - 1;
        let tmp = std::mem::take(&mut self.out[idx].data);
        self.scan(&tmp, scan_base, depth);
        self.out[idx].data = tmp;
    }

    fn plain(&mut self, offset: usize, kind: &str, body: Vec<u8>) {
        self.out.push(Layer {
            offset,
            kind: kind.to_string(),
            data: body,
        });
    }

    fn scan(&mut self, data: &[u8], base: usize, depth: usize) {
        if data.len() < 32 || depth > MAX_DEPTH {
            return;
        }
        // Same de-duplication key as Python: md5(data[:4096] + str(len)).
        // Comparing (len, prefix bytes) is exactly equivalent without md5.
        let key = (data.len(), data[..data.len().min(4096)].to_vec());
        if !self.seen.insert(key) {
            return;
        }

        // ---- ZIP ---------------------------------------------------------
        if is_zipfile(data) {
            self.zip_branch(data, base, depth);
            return;
        }

        // ---- single-stream compressors at offset 0 -----------------------
        if data.starts_with(b"\x1f\x8b\x08") {
            if let Some(body) = inflate(data, 0, 31) {
                self.emit_scan(base, "gzip", body, base, depth + 1);
                return;
            }
        }
        if data.starts_with(b"BZh") {
            if let Some(body) = bunzip(data, 0) {
                self.emit_scan(base, "bzip2", body, base, depth + 1);
                return;
            }
        }
        if data.starts_with(b"\xfd7zXZ\x00") {
            if let Some(body) = unxz(data, 0) {
                self.emit_scan(base, "xz", body, base, depth + 1);
                return;
            }
        }

        // ---- TP-Link RTOS wrapper ----------------------------------------
        if data.starts_with(TPLINK) {
            // Python stores the whole blob as the layer payload.
            self.plain(base, "tplink", data.to_vec());
            let sub = if data.len() > 0x94 { &data[0x94..] } else { &[][..] };
            self.scan(sub, base + 0x94, depth + 1);
            return;
        }

        // ---- uImage (only at a header boundary) --------------------------
        let mut upos: Option<usize> = None;
        for cand in [0usize, 0x40, 0x94, 0x100] {
            if data.len() >= cand + 4 && &data[cand..cand + 4] == UIMAGE {
                upos = Some(cand);
                break;
            }
        }
        if let Some(upos) = upos {
            if upos + 64 <= data.len() {
                let h = &data[upos..upos + 64];
                let size = be32(h, 12);
                // load address / entry point / OS / arch / name are only used
                // for the textual description in Python, which Layer has no
                // field for.
                let comp = h[31];
                let payload_end = (upos + 64).saturating_add(size).min(data.len());
                let payload = &data[upos + 64..payload_end];
                self.plain(base + upos, "uimage", payload.to_vec());

                let mut inner: Option<Vec<u8>> = None;
                let mut inner_kind = "lzma-alone";
                if comp == 3 {
                    if let Some(body) = unlzma_all(payload, 0) {
                        inner = Some(body);
                    } else if let (Some(body), _ds) = raw_lzma_at(payload, 0) {
                        inner = Some(body);
                        inner_kind = "raw-lzma";
                    }
                } else if comp == 1 {
                    // Python labels this row "LZMA compressed data" even though
                    // it is gzip; the real format is reported here.
                    if let Some(body) = inflate(payload, 0, 31) {
                        inner = Some(body);
                        inner_kind = "gzip";
                    }
                } else if comp == 2 {
                    if let Some(body) = bunzip(payload, 0) {
                        inner = Some(body);
                        inner_kind = "bzip2";
                    }
                }

                if let Some(body) = inner {
                    self.emit_scan(base + upos + 64, inner_kind, body, base + upos + 64, depth + 1);
                } else {
                    self.scan(payload, base + upos + 64, depth + 1);
                }
            }
        }

        // ---- vendor firmware headers -------------------------------------
        if data.starts_with(TRX) {
            self.trx(data, base, depth);
        } else if data.starts_with(DLOB) {
            self.dlob(data, base, depth);
        } else if data.starts_with(UBICOM) {
            self.ubicom(data, base, depth);
        } else {
            self.tlv(data, base, depth);
        }

        // ---- compressed streams anywhere in the blob ---------------------
        let found = self.scan_streams(data, base, depth);

        // ---- raw LZMA1 / raw DEFLATE near the start (headerless formats) --
        let small = data.len() <= LARGE_SCAN;
        let mut found = found;
        if depth == 0 && found == 0 && small {
            let hi = data.len().saturating_sub(16).min(2048);
            for m in 0..hi {
                let (body, ds) = raw_lzma_at(data, m);
                if let Some(body) = body {
                    self.emit_scan(
                        base + m,
                        "raw-lzma",
                        body,
                        base + m,
                        depth + 1,
                    );
                    let _ = ds;
                    found = 1;
                    break;
                }
            }
        }
        if depth == 0 && found == 0 && small {
            let hi = data.len().saturating_sub(16).min(1024);
            for m in 0..hi {
                let (body, _wb) = deflate_at(data, m);
                if let Some(body) = body {
                    self.emit_scan(base + m, "deflate", body, base + m, depth + 1);
                    break;
                }
            }
        }
    }

    // ---- container parsers -------------------------------------------

    fn zip_branch(&mut self, data: &[u8], base: usize, depth: usize) {
        if let Some(items) = zip_items(data) {
            for item in items {
                match item {
                    ZipItem::Payload(body) => {
                        self.emit_scan(base, "zip", body, base, depth + 1)
                    }
                    ZipItem::Unsupported(kind, body) => self.plain(base, &kind, body),
                }
            }
        }
    }

    /// TRX (HDR0): 28 byte header, image size + 3 segment offsets, LE CRC.
    fn trx(&mut self, data: &[u8], base: usize, depth: usize) {
        let total_raw = le32(data, 4);
        let mut total = total_raw;
        if total <= 28 || total > data.len() + (16 << 20) {
            total = data.len();
        }
        total = total.min(data.len());
        self.plain(base, "trx", data[..28].to_vec());

        let mut bounds = vec![28usize];
        for k in 0..3 {
            let o = le32(data, 16 + k * 4);
            if o > 28 && o < total {
                bounds.push(o);
            }
        }
        bounds.push(total);
        for i in 0..bounds.len().saturating_sub(1) {
            let (a, b) = (bounds[i], bounds[i + 1]);
            if b <= a {
                continue;
            }
            let chunk = data[a..b].to_vec();
            self.emit_scan(base + a, "trx", chunk, base + a, depth + 1);
        }
    }

    /// D-Link DLOB private header: magic, then 32-bit lengths either endian.
    fn dlob(&mut self, data: &[u8], base: usize, depth: usize) {
        if data.len() < 32 {
            return;
        }
        let vals = [le32(data, 4), be32(data, 4), le32(data, 8), be32(data, 8)];
        let total = vals
            .iter()
            .copied()
            .find(|&v| v >= 32 && v <= data.len())
            .unwrap_or(data.len());
        let hdr_cap = total.min(0x1000);
        let hdr = vals
            .iter()
            .copied()
            .find(|&v| v >= 32 && v <= hdr_cap)
            .unwrap_or(0x20);
        self.plain(base, "dlob", data[..hdr].to_vec());
        let body = data[hdr..total].to_vec();
        if !body.is_empty() {
            self.emit_scan(base + hdr, "dlob", body, base + hdr, depth + 1);
        }
    }

    /// Ubicom boot image (D-Link DGL-4xxx): 32 byte header then the payload.
    fn ubicom(&mut self, data: &[u8], base: usize, depth: usize) {
        if data.len() < 64 {
            return;
        }
        self.plain(base, "ubicom", data[..32].to_vec());
        let body = data[32..].to_vec();
        self.emit_scan(base + 32, "ubicom", body, base + 32, depth + 1);
    }

    /// TLV container / length-prefixed record container.
    fn tlv(&mut self, data: &[u8], base: usize, depth: usize) {
        if let Some((entries, _big)) = tlv_entries(data) {
            self.plain(base, "tlv", data.to_vec());
            for (off, _tag, ln) in entries {
                let body = data[off + 8..off + 8 + ln].to_vec();
                self.emit_scan(base + off + 8, "tlv", body, base + off + 8, depth + 1);
            }
            return;
        }
        if let Some((entries, _big)) = lenprefixed_entries(data) {
            self.plain(base, "tlv", data.to_vec());
            for (off, ln) in entries {
                let body = data[off + 4..off + 4 + ln].to_vec();
                self.emit_scan(base + off + 4, "tlv", body, base + off + 4, depth + 1);
            }
        }
    }

    // ---- compressed streams anywhere in the blob ----------------------

    fn scan_streams(&mut self, data: &[u8], base: usize, depth: usize) -> usize {
        if depth > MAX_STREAM_DEPTH || data.len() < 64 {
            return 0;
        }
        // index: 0 gzip, 1 bzip2, 2 xz, 3 zlib/deflate, 4 lzma-alone
        let caps = [64usize, 16, 8, 24, 32];
        let mut hits = [0usize; 5];
        let mut tried: HashSet<(usize, usize)> = HashSet::new();
        for (off, ki) in stream_offsets(data, 200) {
            if hits[ki] >= caps[ki] {
                continue;
            }
            if !tried.insert((off, ki)) {
                continue;
            }
            let body = match ki {
                0 => inflate(data, off, 31),
                1 => bunzip(data, off),
                2 => unxz(data, off),
                3 => inflate(data, off, 15),
                _ => unlzma_all(data, off),
            };
            let body = match body {
                Some(b) => b,
                None => continue,
            };
            if self.budget < body.len() || self.out.len() >= MAX_ROWS {
                break;
            }
            self.budget -= body.len();
            let kind = match ki {
                0 => "gzip",
                1 => "bzip2",
                2 => "xz",
                3 => "deflate",
                _ => "lzma-alone",
            };
            self.emit_scan(base + off, kind, body, base + off, depth + 1);
            hits[ki] += 1;
        }
        hits.iter().sum()
    }
}

// --------------------------------------------------------------------------
// TLV helpers
// --------------------------------------------------------------------------

fn tlv_entries(data: &[u8]) -> Option<(Vec<(usize, [u8; 4], usize)>, bool)> {
    for big in [true, false] {
        let mut off = 0usize;
        let mut entries: Vec<(usize, [u8; 4], usize)> = Vec::new();
        while off + 8 <= data.len() && entries.len() < MAX_ROWS {
            let tag = [data[off], data[off + 1], data[off + 2], data[off + 3]];
            let ln = if big {
                be32(data, off + 4)
            } else {
                le32(data, off + 4)
            };
            if ln == 0 || ln > data.len() - off - 8 {
                break;
            }
            let printable = tag.iter().all(|&c| (32..127).contains(&c));
            if !printable && tag[..2] != [0, 0] {
                break;
            }
            entries.push((off, tag, ln));
            off += 8 + ln;
        }
        if entries.len() >= 3 && (off as f64) >= (data.len() as f64) * 0.8 {
            return Some((entries, big));
        }
    }
    None
}

fn lenprefixed_entries(data: &[u8]) -> Option<(Vec<(usize, usize)>, bool)> {
    for big in [true, false] {
        let mut off = 0usize;
        let mut entries: Vec<(usize, usize)> = Vec::new();
        while off + 4 <= data.len() && entries.len() < MAX_ROWS {
            let ln = if big {
                be32(data, off)
            } else {
                le32(data, off)
            };
            if ln < 16 || ln > data.len() - off - 4 {
                break;
            }
            entries.push((off, ln));
            off += 4 + ln;
        }
        if entries.len() >= 3 && (off as f64) >= (data.len() as f64) * 0.8 {
            return Some((entries, big));
        }
    }
    None
}

// --------------------------------------------------------------------------
// candidate offsets for compressed streams
// --------------------------------------------------------------------------

fn stream_offsets(data: &[u8], limit: usize) -> Vec<(usize, usize)> {
    let n = data.len();
    let large = n > LARGE_SCAN;
    let mut out: Vec<(usize, usize)> = Vec::new();

    // Exact multi-byte magics: a C-level find, cheap even for huge files.
    for (magic, ki) in [
        (&b"\x1f\x8b\x08"[..], 0usize),
        (&b"BZh"[..], 1),
        (&b"\xfd7zXZ\x00"[..], 2),
    ] {
        let mut pos = 0usize;
        let mut c = 0usize;
        while c < limit && pos <= n {
            match memmem::find(&data[pos..], magic) {
                Some(i) => {
                    let p = pos + i;
                    out.push((p, ki));
                    c += 1;
                    pos = p + 1;
                }
                None => break,
            }
        }
    }

    // zlib: CMF 0x78 (deflate, 32K window) + FLG with (CMF<<8|FLG) % 31 == 0.
    let mut c = 0usize;
    if large {
        let mut pos = 0usize;
        while pos < n.saturating_sub(2) {
            if data[pos] == 0x78 && ((0x78usize << 8) | data[pos + 1] as usize) % 31 == 0 {
                out.push((pos, 3));
                c += 1;
                if c >= limit {
                    break;
                }
            }
            pos += LARGE_STEP;
        }
    } else {
        let mut pos = 0usize;
        while c < limit {
            match memchr::memchr(0x78, &data[pos..]) {
                Some(i) => {
                    let p = pos + i;
                    if p + 1 < n && ((0x78usize << 8) | data[p + 1] as usize) % 31 == 0 {
                        out.push((p, 3));
                        c += 1;
                    }
                    pos = p + 1;
                }
                None => break,
            }
        }
    }

    // LZMA-alone: 13 byte header with a legitimate dictionary size.
    let mut c = 0usize;
    if large {
        let mut pos = 0usize;
        while pos < n.saturating_sub(13) {
            if LZMA_PROPS.contains(&data[pos]) && lzma_header_ok(data, pos) {
                out.push((pos, 4));
                c += 1;
                if c >= limit {
                    break;
                }
            }
            pos += LARGE_STEP;
        }
    } else {
        for m in 0..n.saturating_sub(13) {
            if LZMA_PROPS.contains(&data[m]) && lzma_header_ok(data, m) {
                out.push((m, 4));
                c += 1;
                if c >= limit {
                    break;
                }
            }
        }
    }
    out
}

fn dict_ok(ds: usize) -> bool {
    if ds < (4 << 10) || ds > (1 << 30) {
        return false;
    }
    let mut x = ds;
    while x > 1 && (x & 1) == 0 {
        x >>= 1;
    }
    x == 1 || x == 3
}

fn lzma_header_ok(data: &[u8], off: usize) -> bool {
    if off + 13 > data.len() || !LZMA_PROPS.contains(&data[off]) {
        return false;
    }
    let ds = le32(data, off + 1);
    if !dict_ok(ds) {
        return false;
    }
    let mut size = [0u8; 8];
    size.copy_from_slice(&data[off + 5..off + 13]);
    let size = u64::from_le_bytes(size);
    if size == 0xFFFF_FFFF_FFFF_FFFF {
        return true;
    }
    size > 1024 && size < (4u64 << 30)
}

// --------------------------------------------------------------------------
// LZMA
// --------------------------------------------------------------------------

/// Drive one LZMA-alone stream.
///
/// Returns `(output, bytes_consumed, eof)`. `Err` means the very first call
/// failed while input was still left, which is how Python's
/// `LZMADecompressor.decompress()` surfaces a bad header; partial output that
/// only fails at the drain step is *kept*, exactly like Python's
/// `_drain_lzma()` swallowing the trailing error.
fn lzma_alone_member(input: &[u8], max_out: usize) -> Result<(Vec<u8>, usize, bool), ()> {
    let mut st = Stream::new_lzma_decoder(u64::MAX).map_err(|_| ())?;
    let mut out: Vec<u8> = Vec::new();
    let mut consumed = 0usize;
    loop {
        if out.len() >= max_out {
            return Ok((out, consumed, false));
        }
        let want = (max_out - out.len()).min(1 << 20).max(1);
        let mut buf = vec![0u8; want];
        let before_out = st.total_out() as usize;
        let res = st.process(&input[consumed..], &mut buf, Action::Run);
        let produced = st.total_out() as usize - before_out;
        let new_in = st.total_in() as usize;
        match res {
            Ok(XzStatus::StreamEnd) => {
                out.extend_from_slice(&buf[..produced]);
                return Ok((out, new_in, true));
            }
            Ok(XzStatus::Ok) => {
                out.extend_from_slice(&buf[..produced]);
                consumed = new_in;
                if produced == 0 {
                    return Ok((out, consumed, false));
                }
                // Keep pulling while the decoder has buffered output; the next
                // loop pass feeds the (possibly empty) remainder.
            }
            Ok(_) => {
                out.extend_from_slice(&buf[..produced]);
                return Ok((out, consumed, false));
            }
            Err(_) => {
                if new_in < input.len() {
                    // Corrupt while input remains: Python's initial
                    // decompress() call raises and discards this member.
                    return Err(());
                }
                return Ok((out, new_in, false));
            }
        }
    }
}

/// Decode one LZMA-alone stream *and* any concatenated members after it.
fn unlzma_all(data: &[u8], off: usize) -> Option<Vec<u8>> {
    if off >= data.len() {
        return None;
    }
    let mut out: Vec<u8> = Vec::new();
    let mut pos = off;
    let mut members = 0usize;
    while out.len() < MAX_OUT && members < 64 {
        let rest = &data[pos..];
        let limit = MAX_OUT - out.len();
        match lzma_alone_member(rest, limit) {
            Err(_) => break,
            Ok((chunk, consumed, eof)) => {
                if chunk.is_empty() {
                    break;
                }
                out.extend_from_slice(&chunk);
                members += 1;
                if !eof || consumed == 0 || pos + consumed >= data.len() {
                    break;
                }
                pos += consumed;
            }
        }
    }
    ok_layer(out)
}

/// Raw LZMA1: Python tries 8 MiB, 1 MiB and 64 KiB dictionaries, with the
/// default `lc=3, lp=0, pb=2`. `xz2` 0.1.7 has no raw decoder constructor, so
/// the equivalent LZMA-alone header is synthesised and fed to the real LZMA1
/// decoder (see the module comment).
fn raw_lzma_at(data: &[u8], off: usize) -> (Option<Vec<u8>>, usize) {
    if off >= data.len() {
        return (None, 0);
    }
    for ds in [1usize << 23, 1 << 20, 1 << 16] {
        let mut framed: Vec<u8> = Vec::with_capacity(13 + data.len() - off);
        framed.push(0x5D);
        framed.extend_from_slice(&(ds as u32).to_le_bytes());
        framed.extend_from_slice(&u64::MAX.to_le_bytes());
        framed.extend_from_slice(&data[off..]);
        if let Ok((out, _consumed, _eof)) = lzma_alone_member(&framed, MAX_OUT) {
            if let Some(body) = ok_layer(out) {
                return (Some(body), ds);
            }
        }
    }
    (None, 0)
}

// --------------------------------------------------------------------------
// DEFLATE / gzip
// --------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Wrap {
    Raw,
    Zlib,
    Gzip,
}

fn crc_table() -> &'static [u32; 256] {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for (i, slot) in t.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            *slot = c;
        }
        t
    })
}

fn crc32(data: &[u8]) -> u32 {
    let table = crc_table();
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    !c
}

/// Parse a gzip member header, returning the offset of the raw DEFLATE data.
fn gzip_header(input: &[u8]) -> Option<usize> {
    if input.len() < 10 || &input[..3] != b"\x1f\x8b\x08" || input[3] & 0xE0 != 0 {
        return None;
    }
    let flg = input[3];
    let mut p = 10usize;
    if flg & 4 != 0 {
        if p + 2 > input.len() {
            return None;
        }
        let xlen = le16(input, p);
        p += 2 + xlen;
        if p > input.len() {
            return None;
        }
    }
    if flg & 8 != 0 {
        let j = memchr::memchr(0, &input[p..])?;
        p += j + 1;
    }
    if flg & 16 != 0 {
        let j = memchr::memchr(0, &input[p..])?;
        p += j + 1;
    }
    if flg & 2 != 0 {
        if p + 2 > input.len() {
            return None;
        }
        let want = le16(input, p) as u32;
        if (crc32(&input[..p]) & 0xFFFF) != want {
            return None;
        }
        p += 2;
    }
    if p > input.len() {
        return None;
    }
    Some(p)
}

fn flate_decode(input: &[u8], wrap: Wrap, cap: usize, require_eof: bool) -> Option<Vec<u8>> {
    let (mut d, start, is_gzip) = match wrap {
        Wrap::Raw => (FlateDecompress::new(false), 0usize, false),
        Wrap::Zlib => (FlateDecompress::new(true), 0usize, false),
        Wrap::Gzip => {
            let h = gzip_header(input)?;
            (FlateDecompress::new(false), h, true)
        }
    };
    let mut out: Vec<u8> = Vec::new();
    let mut consumed = start;
    let mut eof = false;
    loop {
        let mut buf = vec![0u8; (cap.saturating_sub(out.len())).min(1 << 20).max(1)];
        let before = d.total_out() as usize;
        let res = d.decompress(&input[consumed..], &mut buf, FlushDecompress::None);
        let produced = d.total_out() as usize - before;
        out.extend_from_slice(&buf[..produced]);
        consumed = start + d.total_in() as usize;
        match res {
            Ok(FlateStatus::StreamEnd) => {
                eof = true;
                break;
            }
            Ok(_) => {
                if produced == 0 {
                    break;
                }
                if out.len() >= cap {
                    break;
                }
            }
            Err(_) => return None,
        }
    }
    if require_eof && !eof {
        return None;
    }
    if is_gzip {
        // zlib's wbits=31 validates the trailer; do the same.
        if consumed + 8 > input.len() {
            return None;
        }
        let crc = le32(input, consumed);
        let isize = le32(input, consumed + 4);
        if crc32(&out) != crc as u32 || out.len() as u32 != isize as u32 {
            return None;
        }
    }
    Some(out)
}

fn inflate(data: &[u8], off: usize, wbits: i32) -> Option<Vec<u8>> {
    if off >= data.len() {
        return None;
    }
    let wrap = match wbits {
        -15 => Wrap::Raw,
        15 => Wrap::Zlib,
        _ => Wrap::Gzip,
    };
    let body = flate_decode(&data[off..], wrap, MAX_OUT, true)?;
    ok_layer(body)
}

fn deflate_at(data: &[u8], off: usize) -> (Option<Vec<u8>>, i32) {
    for wbits in [-15, 15, 31] {
        if let Some(body) = inflate(data, off, wbits) {
            return (Some(body), wbits);
        }
    }
    (None, 0)
}

// --------------------------------------------------------------------------
// bzip2 / xz
// --------------------------------------------------------------------------

fn bunzip_raw(input: &[u8], cap: usize) -> Option<Vec<u8>> {
    let mut d = BzDecompress::new(false);
    let mut out: Vec<u8> = Vec::new();
    let mut consumed = 0usize;
    loop {
        let mut buf = vec![0u8; (cap.saturating_sub(out.len())).min(1 << 20).max(1)];
        let before = d.total_out() as usize;
        let res = d.decompress(&input[consumed..], &mut buf);
        let produced = d.total_out() as usize - before;
        out.extend_from_slice(&buf[..produced]);
        match res {
            Ok(BzStatus::StreamEnd) => return Some(out),
            Ok(_) => {
                consumed = d.total_in() as usize;
                if produced == 0 || out.len() >= cap {
                    return None;
                }
            }
            Err(_) => return None,
        }
    }
}

fn bunzip(data: &[u8], off: usize) -> Option<Vec<u8>> {
    if off >= data.len() {
        return None;
    }
    let body = bunzip_raw(&data[off..], MAX_OUT)?;
    ok_layer(body)
}

fn unxz(data: &[u8], off: usize) -> Option<Vec<u8>> {
    if off >= data.len() {
        return None;
    }
    let mut st = Stream::new_stream_decoder(u64::MAX, 0).ok()?;
    let input = &data[off..];
    let mut out: Vec<u8> = Vec::new();
    let mut consumed = 0usize;
    loop {
        if out.len() >= MAX_OUT {
            return None;
        }
        let want = (MAX_OUT - out.len()).min(1 << 20).max(1);
        let mut buf = vec![0u8; want];
        let before = st.total_out() as usize;
        let res = st.process(&input[consumed..], &mut buf, Action::Run);
        let produced = st.total_out() as usize - before;
        let new_in = st.total_in() as usize;
        out.extend_from_slice(&buf[..produced]);
        match res {
            Ok(XzStatus::StreamEnd) => return ok_layer(out),
            Ok(XzStatus::Ok) => {
                consumed = new_in;
                if produced == 0 {
                    return None;
                }
            }
            Ok(_) => return None,
            Err(_) => return None,
        }
    }
}

// --------------------------------------------------------------------------
// ZIP
// --------------------------------------------------------------------------

enum ZipItem {
    Payload(Vec<u8>),
    Unsupported(String, Vec<u8>),
}

fn find_eocd(data: &[u8]) -> Option<usize> {
    if data.len() < 22 {
        return None;
    }
    let last = data.len() - 22;
    let start = data.len().saturating_sub(22 + 65535);
    let mut i = last;
    loop {
        if data.len() >= i + 22 && &data[i..i + 4] == b"PK\x05\x06" {
            let comment = le16(data, i + 20);
            if i + 22 + comment <= data.len() {
                return Some(i);
            }
        }
        if i == start {
            break;
        }
        i -= 1;
    }
    None
}

fn is_zipfile(data: &[u8]) -> bool {
    find_eocd(data).is_some()
}

fn zip_items(data: &[u8]) -> Option<Vec<ZipItem>> {
    let eocd = find_eocd(data)?;
    if data.len() < eocd + 22 {
        return None;
    }
    let count = le16(data, eocd + 10);
    let cd_size = le32(data, eocd + 12);
    let cd_off = le32(data, eocd + 16);
    if cd_off == 0xFFFF_FFFF || count == 0xFFFF {
        // ZIP64 central directory: Python handles it, the pinned dependency
        // set has no zip64 helper. Report it instead of silently dropping it.
        return Some(vec![ZipItem::Unsupported(
            "unsupported-zip64".to_string(),
            Vec::new(),
        )]);
    }
    // zipfile._EndRecData: data before the archive (SFX stubs) shifts offsets.
    let concat = eocd as i64 - cd_size as i64 - cd_off as i64;
    let cd = cd_off as i64 + concat;
    if cd < 0 {
        return None;
    }
    let mut p = cd as usize;
    let mut items: Vec<ZipItem> = Vec::new();
    for _ in 0..count {
        if p + 46 > data.len() || &data[p..p + 4] != b"PK\x01\x02" {
            return None;
        }
        let method = le16(data, p + 10);
        let crc = le32(data, p + 16) as u32;
        let comp_size = le32(data, p + 20);
        let uncomp_size = le32(data, p + 24);
        let fn_len = le16(data, p + 28);
        let extra_len = le16(data, p + 30);
        let comment_len = le16(data, p + 32);
        let local_off = le32(data, p + 42);
        let name_end = p + 46 + fn_len;
        if name_end > data.len() {
            return None;
        }
        let name = &data[p + 46..name_end];
        p = name_end + extra_len + comment_len;

        if name.ends_with(b"/") || uncomp_size < 1024 {
            continue;
        }
        if comp_size == 0xFFFF_FFFF || uncomp_size == 0xFFFF_FFFF || local_off == 0xFFFF_FFFF {
            items.push(ZipItem::Unsupported(
                "unsupported-zip64".to_string(),
                Vec::new(),
            ));
            continue;
        }
        let lp = local_off as i64 + concat;
        if lp < 0 {
            return None;
        }
        let lp = lp as usize;
        if lp + 30 > data.len() || &data[lp..lp + 4] != b"PK\x03\x04" {
            return None;
        }
        let lfn = le16(data, lp + 26);
        let lex = le16(data, lp + 28);
        let ds = lp + 30 + lfn + lex;
        if ds > data.len() || ds + comp_size > data.len() {
            return None;
        }
        let raw = &data[ds..ds + comp_size];

        let body = match method {
            0 => raw.to_vec(),
            8 => match flate_decode(raw, Wrap::Raw, uncomp_size, true) {
                Some(b) => b,
                None => return Some(items),
            },
            12 => match bunzip_raw(raw, uncomp_size) {
                Some(b) => b,
                None => return Some(items),
            },
            14 => match zip_lzma(raw, uncomp_size) {
                Some(b) => b,
                None => return Some(items),
            },
            m => {
                items.push(ZipItem::Unsupported(
                    format!("unsupported-zip-method-{}", m),
                    raw.to_vec(),
                ));
                continue;
            }
        };
        if body.len() != uncomp_size || crc32(&body) != crc {
            // zipfile.read() raises BadZipFile here, aborting the archive.
            return Some(items);
        }
        items.push(ZipItem::Payload(body));
    }
    Some(items)
}

fn zip_lzma(raw: &[u8], cap: usize) -> Option<Vec<u8>> {
    if raw.len() < 9 {
        return None;
    }
    let psize = le16(raw, 2);
    if psize != 5 || raw.len() < 4 + psize {
        return None;
    }
    let mut framed: Vec<u8> = Vec::with_capacity(13 + raw.len() - 4 - psize);
    framed.extend_from_slice(&raw[4..4 + psize]);
    framed.extend_from_slice(&u64::MAX.to_le_bytes());
    framed.extend_from_slice(&raw[4 + psize..]);
    let (out, _, _) = lzma_alone_member(&framed, cap).ok()?;
    Some(out)
}

// --------------------------------------------------------------------------
// unit tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Generated with CPython's lzma/zlib/bz2/gzip modules for the payload
    // b"hello world! ".repeat(400) (5200 bytes).
    const ALONE: &str = "5d00008000ffffffffffffffff00341949ee8de917893a335ffd86053c23899a96221c5f05b90381d02fea25523ac42cfc1fac8ebe45581255ffffa99a0000";
    const RAW: &str = "00341949ee8de917893a335ffd86053c23899a96221c5f05b90381d02fea25523ac42cfc1fac8ebe45581255ffffa99a0000";
    const ZLIB: &str = "789cedc7b10900200c00b057ea6f0a1d0a852ebeef057e906cc953d5717b6aaf48111111111111111111917f1ed1b735ba";
    const GZIP: &str = "1f8b0800ecd9f26a02ffedc7b10900200c00b057ea6f0a1d0a852ebeef057e906cc953d5717b6aaf48111111111111111111917f1eca027b0050140000";
    const BZIP2: &str = "425a6839314159265359329a7cb40004af918060000644908020007041931014aa0d9474116045e0458116045c08b422d08bd045f045a117022fc5dc914e14240ca69f2d00";
    const XZ: &str = "fd377a585a000004e6d6b4460200210116000000742fe5a3e0144f002b5d00341949ee8de917893a335ffd86053c23899a96221c5f05b90381d02fea25523ac42cfc1fac8ebe4463dcd00000b4b01193551bf812000147d02800000081167b3db1c467fb020000000004595a";
    const ZIP: &str = "504b0304140000000800dc856a5dca027b002b000000501400000c0000006669726d776172652e62696eedc7b10900200c00b057ea6f0a1d0a852ebeef057e906cc953d5717b6aaf48111111111111111111917f1e504b01021400140000000800dc856a5dca027b002b000000501400000c00000000000000000000008001000000006669726d776172652e62696e504b050600000000010001003a000000550000000000";
    // First 84 bytes of the real Netgear WGR614v3 CHK header, used to make the
    // raw-LZMA1 probe land on the same offset as the Python scanner.
    const HDR84: &str = "414830303f1e415b000f08ba0395291c5db30287b32ca5289aff580a00010014553132483031335430305f4e455447454152000073717a02238601019a080f0000000000607e3b00000000005c0f20c9c9f4df72";

    fn hx(s: &str) -> Vec<u8> {
        assert_eq!(s.len() % 2, 0);
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    fn payload() -> Vec<u8> {
        b"hello world! ".repeat(400)
    }

    #[test]
    fn header_and_dict_checks() {
        assert!(dict_ok(1 << 23));
        assert!(dict_ok((1 << 23) + (1 << 22)));
        assert!(dict_ok(1 << 16));
        assert!(!dict_ok(0));
        assert!(!dict_ok(1 << 40));
        assert!(!dict_ok(12345));
        let hdr = hx(ALONE);
        assert!(lzma_header_ok(&hdr, 0));
        assert!(!lzma_header_ok(&hdr, 1));
        assert_eq!(ok_layer(vec![0u8; 1024]), None);
        assert_eq!(ok_layer(vec![0u8; 1025]).map(|v| v.len()), Some(1025));
    }

    #[test]
    fn lzma_alone_stream() {
        let mut d = hx(ALONE);
        d.extend_from_slice(&[0u8; 100]); // force len >= 64 for scan_streams()
        let layers = scan_all(&d);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].offset, 0);
        assert_eq!(layers[0].kind, "lzma-alone");
        assert_eq!(layers[0].data, payload());
    }

    #[test]
    fn tiny_lzma_alone_falls_back_to_raw_probe() {
        // The 63-byte vector is below scan_streams()'s 64-byte gate, so Python
        // finds the same stream through the raw-LZMA1 probe at offset 13.
        let layers = scan_all(&hx(ALONE));
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].offset, 13);
        assert_eq!(layers[0].kind, "raw-lzma");
        assert_eq!(layers[0].data, payload());
    }

    #[test]
    fn raw_lzma1_netgear_chk() {
        let raw = hx(RAW);
        let (body, ds) = raw_lzma_at(&raw, 0);
        assert_eq!(ds, 1 << 23);
        assert_eq!(body.unwrap(), payload());

        let mut d = hx(HDR84);
        d.extend_from_slice(&raw);
        let layers = scan_all(&d);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].offset, 84);
        assert_eq!(layers[0].kind, "raw-lzma");
        assert_eq!(layers[0].data, payload());
    }

    #[test]
    fn single_stream_formats() {
        for (hex, kind) in [
            (ZLIB, "deflate"),
            (GZIP, "gzip"),
            (BZIP2, "bzip2"),
            (XZ, "xz"),
        ] {
            let layers = scan_all(&hx(hex));
            assert_eq!(layers.len(), 1, "unexpected layer count for {}", kind);
            assert_eq!(layers[0].offset, 0);
            assert_eq!(layers[0].kind, kind);
            assert_eq!(layers[0].data, payload());
        }
    }

    #[test]
    fn uimage_wrapper() {
        let alone = hx(ALONE);
        let mut h = vec![0u8; 64];
        h[0..4].copy_from_slice(UIMAGE);
        h[12..16].copy_from_slice(&(alone.len() as u32).to_be_bytes());
        h[28] = 5;
        h[29] = 2;
        h[30] = 2;
        h[31] = 3;
        h[32..39].copy_from_slice(b"testimg");
        let mut d = h;
        d.extend_from_slice(&alone);

        let layers = scan_all(&d);
        assert_eq!(layers.len(), 3);
        assert_eq!((layers[0].offset, layers[0].kind.as_str()), (0, "uimage"));
        assert_eq!(layers[0].data, alone);
        assert_eq!(
            (layers[1].offset, layers[1].kind.as_str()),
            (64, "lzma-alone")
        );
        assert_eq!(layers[1].data, payload());
        assert_eq!(
            (layers[2].offset, layers[2].kind.as_str()),
            (64, "lzma-alone")
        );
        assert_eq!(layers[2].data, payload());
    }

    #[test]
    fn tplink_wrapper() {
        let alone = hx(ALONE);
        let mut d = vec![0u8; 148];
        d[0..4].copy_from_slice(TPLINK);
        d.extend_from_slice(&alone);
        d.extend_from_slice(&[0u8; 100]);

        let layers = scan_all(&d);
        assert_eq!(layers.len(), 2);
        assert_eq!((layers[0].offset, layers[0].kind.as_str()), (0, "tplink"));
        assert_eq!(layers[0].data, d);
        assert_eq!(
            (layers[1].offset, layers[1].kind.as_str()),
            (148, "lzma-alone")
        );
        assert_eq!(layers[1].data, payload());
    }

    #[test]
    fn trx_container_and_segment() {
        let gz = hx(GZIP);
        let total = 28 + gz.len();
        let mut d = vec![0u8; 28];
        d[0..4].copy_from_slice(TRX);
        d[4..8].copy_from_slice(&(total as u32).to_le_bytes());
        d[16..20].copy_from_slice(&28u32.to_le_bytes());
        d[20..24].copy_from_slice(&28u32.to_le_bytes());
        d[24..28].copy_from_slice(&(total as u32).to_le_bytes());
        d.extend_from_slice(&gz);

        let layers = scan_all(&d);
        let shapes: Vec<(usize, &str, usize)> = layers
            .iter()
            .map(|l| (l.offset, l.kind.as_str(), l.data.len()))
            .collect();
        assert_eq!(
            shapes,
            vec![
                (0, "trx", 28),
                (28, "trx", gz.len()),
                (28, "gzip", 5200),
                (28, "gzip", 5200),
            ]
        );
    }

    #[test]
    fn dlob_ubicom_and_tlv() {
        // DLOB: big-endian total (96) at +4, little-endian header size (32) at +8.
        let mut d = vec![0u8; 96];
        d[0..4].copy_from_slice(DLOB);
        d[4..8].copy_from_slice(&[0, 0, 0, 96]);
        d[8..12].copy_from_slice(&32u32.to_le_bytes());
        // Python picks the 96-byte value as the header size and then finds the
        // same raw-LZMA1 false positive at offset 6; both are reproduced here.
        let layers = scan_all(&d);
        let shapes: Vec<(usize, &str, usize)> = layers
            .iter()
            .map(|l| (l.offset, l.kind.as_str(), l.data.len()))
            .collect();
        assert_eq!(shapes, vec![(0, "dlob", 96), (6, "raw-lzma", 1804)]);

        // Ubicom: magic + zeros, 32-byte header and payload.
        let mut u = vec![0u8; 64];
        u[0..4].copy_from_slice(UBICOM);
        let layers = scan_all(&u);
        let shapes: Vec<(usize, &str, usize)> = layers
            .iter()
            .map(|l| (l.offset, l.kind.as_str(), l.data.len()))
            .collect();
        assert_eq!(shapes, vec![(0, "ubicom", 32), (32, "ubicom", 32)]);

        // TLV: three big-endian entries.
        let mut tlv = Vec::new();
        for (i, pl) in [vec![b'a'; 40], vec![b'b'; 36], vec![b'c'; 35]]
            .iter()
            .enumerate()
        {
            tlv.extend_from_slice(format!("AB{:02}", i).as_bytes());
            tlv.extend_from_slice(&(pl.len() as u32).to_be_bytes());
            tlv.extend_from_slice(pl);
        }
        assert_eq!(tlv.len(), 135);
        let layers = scan_all(&tlv);
        let shapes: Vec<(usize, &str, usize)> = layers
            .iter()
            .map(|l| (l.offset, l.kind.as_str(), l.data.len()))
            .collect();
        assert_eq!(
            shapes,
            vec![
                (0, "tlv", 135),
                (8, "tlv", 40),
                (56, "tlv", 36),
                (100, "tlv", 35),
            ]
        );
    }

    #[test]
    fn zip_member() {
        let layers = scan_all(&hx(ZIP));
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].offset, 0);
        assert_eq!(layers[0].kind, "zip");
        assert_eq!(layers[0].data, payload());
    }

    #[test]
    fn cap_rows_keeps_largest_and_order() {
        let layers: Vec<Layer> = (0..305)
            .map(|i| Layer {
                offset: i,
                kind: "x".to_string(),
                data: vec![0u8; if i < 5 { 10_000 + i } else { 100 + i }],
            })
            .collect();
        let capped = cap_rows(layers);
        assert_eq!(capped.len(), MAX_ROWS);
        assert_eq!(capped[0].offset, 0); // biggest payloads survive, order kept
        assert!(capped.iter().all(|l| !(5..10).contains(&l.offset)));
        assert!(capped.iter().any(|l| l.data.len() == 10_004));
    }
}
