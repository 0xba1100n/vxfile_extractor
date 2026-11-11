//! VxWorks symbol-table detectors -- a Rust port of `tools/vxsym.py`.
//!
//! The module exposes two entry points:
//!
//! * [`find_symbols`] -- every piece of symbol-table evidence found in a blob,
//!   strongest first (detectors `A`, `B` and `C`).
//! * [`recover_symbol_table`] -- the strongest hit that actually carries a
//!   carved name table (`has_table == true`).
//!
//! Detectors (identical semantics to the Python reference):
//!
//! * `A` compressed symbol table.  LZMA-alone streams (properties byte
//!   0x5A/0x5C/0x5D/0x5E/0x6E, the VxWorks/LZMA default set) whose payload
//!   carries at least two VxWorks markers (`bzero`, `usrRoot`, `taskSpawn`,
//!   `windFindNextSym`, `WIND version`).  If the inflated image embeds a NUL
//!   separated name table, that table is carved (`has_table`).  Header-less raw
//!   LZMA1 streams (Netgear `.chk` and friends) are probed as well: they always
//!   start with the 0x00 range-coder byte, so only those offsets are tried.
//! * `B` uncompressed symbol table.  A run of >= 40 NUL separated C
//!   identifiers (padded to 4/8 byte alignment, 1..8 NULs between names), >= 10
//!   of them ending in a VxWorks symbol suffix, and (>= 2 known VxWorks API
//!   names, or a >= 200 name application table inside an image carrying
//!   VxWorks markers).
//! * `C` symbol density.  >= 15 distinct VxWorks API names inside one 64 KiB
//!   window.
//!
//! # Ported / not ported
//!
//! Embedded **zip containers are not expanded** here: this crate does not
//! depend on a zip reader and `vxsym.py`'s `zipfile` layer expansion is instead
//! the caller's job (pass the extracted member).  Everything else -- LZMA-alone
//! discovery, bounded inflate, raw LZMA1 probing, name-table carving, table
//! start trimming, density windows, blob-hash dedupe and strength ranking --
//! follows the Python implementation.
//!
//! `xz2` 0.1.7 has no `Stream::new_raw_decoder`, so a raw LZMA1 stream is
//! decoded through the equivalent LZMA-alone container that is synthesised on
//! the fly (props 0x5D = lc3/lp0/pb2, the Python `FILTER_LZMA1` defaults, plus
//! the probed dictionary size and an unknown uncompressed size).
//!
//! Unlike Python (which caps `hit["names"]` at 48 evidence strings but keeps
//! `names_count` at the full table size), [`SymHit::names`] always holds the
//! complete name list and `names_count == names.len()`.

use std::collections::{HashMap, HashSet, VecDeque};

use memchr::{memchr, memmem};
use xz2::stream::{Action, Status, Stream};

/// A single piece of symbol-table evidence.
#[derive(Debug, Clone)]
pub struct SymHit {
    /// 'A' compressed symbol table / 'B' uncompressed name table / 'C' density.
    pub detector: char,
    /// Offset of the evidence inside the input data (A: offset of the stream).
    pub offset: usize,
    /// Complete list of recovered symbol names.
    pub names: Vec<String>,
    pub names_count: usize,
    /// Carved symbol-table bytes (empty when only marker-level evidence exists).
    pub blob: Vec<u8>,
    /// Whether a real NUL separated name table was carved.
    pub has_table: bool,
    pub label: String,
}

// ---------------------------------------------------------------------------
// tunables (mirrors vxsym.py)
// ---------------------------------------------------------------------------
const MAX_INFLATE: usize = 64 << 20; // never inflate more than 64 MiB per stream
const MAX_DICT: u32 = 32 << 20; // largest LZMA dictionary we allocate
const MAX_LZMA_BLOB: usize = 48 << 20; // do not brute force LZMA in larger blobs
const MAX_NAME_BLOB: usize = 48 << 20; // do not scan larger blobs for names
const MAX_DEPTH: usize = 2; // follow LZMA layers up to this nesting level
const MAX_STREAMS: usize = 64; // LZMA candidates per blob
const MAX_RAW_PROBE: usize = 0x2000; // offsets probed for raw LZMA1 streams
const MAX_PER_DETECTOR: usize = 6; // hits kept per detector

const A_MIN_MARKERS: usize = 2;
const B_MIN_NAMES: usize = 40;
const B_MIN_SUFFIX: usize = 10;
const B_MIN_API: usize = 2;
const B_MIN_BIG: usize = 200; // a large table may hold only application symbols
const C_WINDOW: usize = 64 << 10;
const C_MIN_API: usize = 15;
const MIN_BLOB: usize = 1024;

/// VxWorks / Wind River API and symbol names, used by detector B (a real table
/// has to contain a couple of them) and by detector C (density).
const VX_API: &[&[u8]] = &[
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
];

const VX_MARKERS: &[&[u8]] = &[
    b"bzero",
    b"usrRoot",
    b"taskSpawn",
    b"windFindNextSym",
    b"WIND version",
];

const NAME_SUFFIXES: &[&[u8]] = &[
    b"Task", b"Init", b"Isr", b"Lib", b"Hook", b"Create", b"Delete", b"Rtn", b"Msg", b"Sem",
    b"Dev", b"Show", b"Free", b"Alloc",
];

/// A run of NUL separated C identifiers needs at least this many names before it
/// is scored (`{16,}` in the Python regex).
const RUN_MIN_TOKENS: usize = 16;
/// Identifiers longer than this cannot be a name-table entry (`{0,255}`).
const MAX_IDENT: usize = 256;

// ---------------------------------------------------------------------------
// small byte helpers
// ---------------------------------------------------------------------------
#[inline]
fn is_ident_start(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphabetic()
}

#[inline]
fn is_ident(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphanumeric()
}

#[inline]
fn is_lzma_props(c: u8) -> bool {
    matches!(c, 0x5A | 0x5C | 0x5D | 0x5E | 0x6E)
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    memmem::find(hay, needle).is_some()
}

fn in_api_set(tok: &[u8]) -> bool {
    VX_API.iter().any(|n| *n == tok)
}

// ---------------------------------------------------------------------------
// MD5 -- used only as a dedupe key, exactly like vxsym.py's hashlib.md5
// ---------------------------------------------------------------------------
const MD5_S: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, //
    5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, //
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, //
    6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const MD5_K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
    0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
    0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
    0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
    0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
    0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
    0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
    0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
    0xeb86d391,
];

struct Md5 {
    state: [u32; 4],
    buf: [u8; 64],
    buflen: usize,
    len: u64,
}

impl Md5 {
    fn new() -> Md5 {
        Md5 {
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476],
            buf: [0u8; 64],
            buflen: 0,
            len: 0,
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.len = self.len.wrapping_add(data.len() as u64);
        if self.buflen > 0 {
            let take = (64 - self.buflen).min(data.len());
            self.buf[self.buflen..self.buflen + take].copy_from_slice(&data[..take]);
            self.buflen += take;
            data = &data[take..];
            if self.buflen == 64 {
                let block = self.buf;
                self.compress(&block);
                self.buflen = 0;
            }
        }
        while data.len() >= 64 {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[..64]);
            self.compress(&block);
            data = &data[64..];
        }
        if !data.is_empty() {
            self.buf[..data.len()].copy_from_slice(data);
            self.buflen = data.len();
        }
    }

    fn compress(&mut self, block: &[u8; 64]) {
        let mut m = [0u32; 16];
        for (i, slot) in m.iter_mut().enumerate() {
            *slot = u32::from_le_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        let (mut a, mut b, mut c, mut d) =
            (self.state[0], self.state[1], self.state[2], self.state[3]);
        for i in 0..64 {
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let tmp = d;
            d = c;
            c = b;
            let sum = a.wrapping_add(f).wrapping_add(MD5_K[i]).wrapping_add(m[g]);
            b = b.wrapping_add(sum.rotate_left(MD5_S[i]));
            a = tmp;
        }
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
    }

    fn finish(mut self) -> [u8; 16] {
        let bitlen = self.len.wrapping_mul(8);
        let rem = ((self.len + 1) % 64) as usize;
        let zeros = if rem <= 56 { 56 - rem } else { 120 - rem };
        let mut pad = Vec::with_capacity(zeros + 9);
        pad.push(0x80u8);
        pad.resize(zeros + 1, 0u8);
        pad.extend_from_slice(&bitlen.to_le_bytes());
        self.update(&pad);
        let mut out = [0u8; 16];
        for i in 0..4 {
            out[i * 4..i * 4 + 4].copy_from_slice(&self.state[i].to_le_bytes());
        }
        out
    }
}

fn md5(input: &[u8]) -> [u8; 16] {
    let mut h = Md5::new();
    h.update(input);
    h.finish()
}

/// `(len(blob), md5(blob[:4096] + blob[-4096:]))`, the layer dedupe key of
/// `vxsym.py`.
fn blob_key(blob: &[u8]) -> (usize, [u8; 16]) {
    let mut h = Md5::new();
    let n = blob.len();
    h.update(&blob[..n.min(4096)]);
    h.update(&blob[n.saturating_sub(4096)..]);
    (n, h.finish())
}

// ---------------------------------------------------------------------------
// detector A helpers: LZMA stream discovery
// ---------------------------------------------------------------------------
/// Decode one LZMA stream with a hard output cap, mirroring
/// `LZMADecompressor.decompress(..., max_length=cap)`: truncated streams yield
/// the bytes decoded so far, corrupt streams yield `None`.
fn decode_stream(stream: &mut Stream, input: &[u8], cap: usize) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();
    let mut off = 0usize;
    loop {
        if out.len() >= cap {
            out.truncate(cap);
            return Some(out);
        }
        let want = (cap - out.len()).min(1 << 20);
        out.reserve(want);
        let before_in = stream.total_in();
        let before_out = stream.total_out();
        let status = stream
            .process_vec(&input[off..], &mut out, Action::Run)
            .ok()?;
        let used = (stream.total_in() - before_in) as usize;
        let produced = (stream.total_out() - before_out) as usize;
        off += used;
        match status {
            Status::StreamEnd => return Some(out),
            Status::Ok => {
                if produced == 0 && used == 0 {
                    // no more input and no more output: a truncated stream
                    return Some(out);
                }
            }
            _ => return Some(out),
        }
    }
}

/// Decode an LZMA-alone stream (`input` starts at its 13-byte header).
fn decode_lzma_alone(input: &[u8], cap: usize) -> Option<Vec<u8>> {
    let mut stream = Stream::new_lzma_decoder(1u64 << 32).ok()?;
    decode_stream(&mut stream, input, cap)
}

/// Decode a header-less raw LZMA1 stream.  xz2 0.1.7 exposes no
/// `new_raw_decoder`, so frame the payload in the equivalent LZMA-alone header
/// (props 0x5D = lc3/lp0/pb2, the `FILTER_LZMA1` defaults, plus `dict_size` and
/// an unknown uncompressed size).  This decodes to exactly the same bytes as
/// `lzma.FORMAT_RAW` with `{"id": FILTER_LZMA1, "dict_size": dict_size}`.
fn decode_raw_lzma1(input: &[u8], dict_size: u32, cap: usize) -> Option<Vec<u8>> {
    let mut framed = Vec::with_capacity(input.len() + 13);
    framed.push(0x5D);
    framed.extend_from_slice(&dict_size.to_le_bytes());
    framed.extend_from_slice(&u64::MAX.to_le_bytes());
    framed.extend_from_slice(input);
    decode_lzma_alone(&framed, cap)
}

/// LZMA-alone candidates: known properties byte, two zero bytes, sane
/// dictionary size and a sane or unknown uncompressed size.  Mirrors
/// `_PROPS_RE.finditer` + the bounded inflate of `vxsym.py`.
fn lzma_streams(blob: &[u8], max_streams: usize) -> Vec<(usize, Vec<u8>)> {
    let mut out: Vec<(usize, Vec<u8>)> = Vec::new();
    let n = blob.len();
    if n < 16 {
        return out;
    }
    let mut i = 0usize;
    while i + 2 < n {
        if !is_lzma_props(blob[i]) || blob[i + 1] != 0 || blob[i + 2] != 0 {
            i += 1;
            continue;
        }
        if out.len() >= max_streams {
            break;
        }
        if i > n - 16 {
            break;
        }
        let dict_size = u32::from_le_bytes([blob[i + 1], blob[i + 2], blob[i + 3], blob[i + 4]]);
        let unpacked = u64::from_le_bytes([
            blob[i + 5],
            blob[i + 6],
            blob[i + 7],
            blob[i + 8],
            blob[i + 9],
            blob[i + 10],
            blob[i + 11],
            blob[i + 12],
        ]);
        if !(4096..=MAX_DICT).contains(&dict_size) {
            i += 3;
            continue;
        }
        if unpacked != u64::MAX && !(4096..=MAX_INFLATE as u64).contains(&unpacked) {
            i += 3;
            continue;
        }
        if let Some(dec) = decode_lzma_alone(&blob[i..], MAX_INFLATE) {
            if dec.len() >= MIN_BLOB && dec.len() <= MAX_INFLATE {
                out.push((i, dec));
            }
        }
        i += 3;
    }
    out
}

/// Probe for header-less raw LZMA1 streams (Netgear `.chk` and friends).  A raw
/// LZMA1 stream always starts with the 0x00 range-coder byte, so only those
/// offsets are attempted.
fn raw_lzma_streams(blob: &[u8], max_streams: usize) -> Vec<(usize, Vec<u8>)> {
    let mut out: Vec<(usize, Vec<u8>)> = Vec::new();
    let n = blob.len();
    if n < 16 {
        return out;
    }
    let limit = (n - 16).min(MAX_RAW_PROBE);
    let mut off = 0usize;
    while off < limit {
        match memchr(0, &blob[off..limit]) {
            Some(k) => off += k,
            None => break,
        }
        for ds in [8u32 << 20, 1 << 20, 64 << 10] {
            if let Some(dec) = decode_raw_lzma1(&blob[off..], ds, MAX_INFLATE) {
                if dec.len() >= MIN_BLOB && dec.len() <= MAX_INFLATE {
                    out.push((off, dec));
                    break;
                }
            }
        }
        if out.len() >= max_streams {
            break;
        }
        off += 1;
    }
    out
}

// ---------------------------------------------------------------------------
// detector B helper: uncompressed name-table carving
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
struct NameTable {
    start: usize,
    end: usize,
    names: Vec<Vec<u8>>,
    suffix: usize,
}

/// Walk every run of NUL separated identifiers, exactly like
/// `_NAME_RUN_RE.finditer` (`(?:[A-Za-z_][A-Za-z0-9_]{0,255}\x00{1,8}){16,}`).
///
/// `f(start, end)` is invoked for each non-overlapping left-to-right match; the
/// match ends right after the last consumed NUL byte, so a following run is
/// picked up from there.
fn find_name_runs<F: FnMut(usize, usize)>(blob: &[u8], mut f: F) {
    let n = blob.len();
    let mut i = 0usize;
    while i < n {
        if !is_ident_start(blob[i]) {
            i += 1;
            continue;
        }
        // Maximal identifier length at `i`.  If it exceeds 256 the token can
        // never be followed by a NUL at `i`, and neither can the first
        // `l - 256` characters of it: jump straight to the first position whose
        // maximal identifier is exactly 256 bytes.
        let mut l = 1usize;
        while i + l < n && is_ident(blob[i + l]) {
            l += 1;
        }
        if l > MAX_IDENT {
            i += l - MAX_IDENT;
            continue;
        }
        if i + l >= n || blob[i + l] != 0 {
            i += 1;
            continue;
        }
        let mut pos = i;
        let mut cur = l;
        let mut tokens = 0usize;
        let mut end = i;
        loop {
            let mut p = pos + cur;
            let mut z = 0usize;
            while z < 8 && p < n && blob[p] == 0 {
                z += 1;
                p += 1;
            }
            if z == 0 {
                break; // token without its NUL padding: the run ends here
            }
            tokens += 1;
            end = p;
            if p >= n || !is_ident_start(blob[p]) {
                break;
            }
            let mut nl = 1usize;
            while p + nl < n && is_ident(blob[p + nl]) {
                nl += 1;
            }
            if nl > MAX_IDENT {
                break; // next entry cannot be an identifier of <= 255 chars
            }
            pos = p;
            cur = nl;
        }
        if tokens >= RUN_MIN_TOKENS {
            f(i, end);
            i = end;
        } else {
            i += 1;
        }
    }
}

/// Find the largest VxWorks symbol name table in `blob`, mirroring
/// `_scan_name_table`.
fn scan_name_table(blob: &[u8], markers: Option<usize>) -> Option<NameTable> {
    if blob.len() < B_MIN_NAMES * 2 || blob.len() > MAX_NAME_BLOB {
        return None;
    }
    let markers =
        markers.unwrap_or_else(|| VX_MARKERS.iter().filter(|m| contains(blob, m)).count());
    let mut best: Option<NameTable> = None;
    find_name_runs(blob, |match_start, match_end| {
        let end = match_end;
        let mut start = match_start;
        // The left edge of a match can fall on the tail of a non-symbol string
        // ("...Lib.c", or a stray byte pair inside code).  Drop up to a few
        // 1-2 character tokens so the carved table starts on a real name.
        for _ in 0..8 {
            let nul = match memchr(0, &blob[start..end]) {
                Some(k) => start + k,
                None => break,
            };
            let tok_len = nul - start;
            if tok_len > 2 {
                break;
            }
            start = nul + 1;
            while start < end && blob[start] == 0 {
                start += 1;
            }
        }
        let names: Vec<Vec<u8>> = blob[start..end]
            .split(|&b| b == 0)
            .filter(|t| !t.is_empty())
            .map(|t| t.to_vec())
            .collect();
        if names.len() < B_MIN_NAMES {
            return;
        }
        let suffix = names
            .iter()
            .filter(|t| NAME_SUFFIXES.iter().any(|s| t.ends_with(s)))
            .count();
        if suffix < B_MIN_SUFFIX {
            return;
        }
        let api = names.iter().filter(|t| in_api_set(t)).count();
        // Normal case: the table itself carries VxWorks API symbols.  Some
        // application symbol tables (e.g. printer firmware) only hold program
        // symbols, so accept a very large table when the surrounding image has
        // the usual VxWorks markers.
        if api < B_MIN_API && !(names.len() >= B_MIN_BIG && markers >= A_MIN_MARKERS) {
            return;
        }
        if best.as_ref().map_or(true, |b| names.len() > b.names.len()) {
            best = Some(NameTable {
                start,
                end,
                names,
                suffix,
            });
        }
    });
    best
}

// ---------------------------------------------------------------------------
// detector C helper: API name density
// ---------------------------------------------------------------------------
fn api_positions(blob: &[u8]) -> Vec<(usize, &'static [u8])> {
    let mut pos: Vec<(usize, &'static [u8])> = Vec::new();
    for name in VX_API {
        let mut start = 0usize;
        while let Some(k) = memmem::find(&blob[start..], name) {
            let i = start + k;
            pos.push((i, name));
            start = i + 1;
        }
    }
    pos.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(b.1)));
    pos
}

/// Find a 64 KiB window with >= C_MIN_API distinct VxWorks API names.
fn scan_density(blob: &[u8]) -> Option<(usize, usize, Vec<Vec<u8>>)> {
    if blob.len() < C_WINDOW / 2 || blob.len() > MAX_NAME_BLOB {
        return None;
    }
    let pos = api_positions(blob);
    if pos.len() < C_MIN_API {
        return None;
    }
    let mut counts: HashMap<&'static [u8], usize> = HashMap::new();
    let mut right = 0usize;
    let mut best: Option<(usize, usize, Vec<Vec<u8>>)> = None;
    for left in 0..pos.len() {
        if right < left {
            right = left;
        }
        let limit = pos[left].0 + C_WINDOW;
        while right < pos.len() && pos[right].0 < limit {
            *counts.entry(pos[right].1).or_insert(0) += 1;
            right += 1;
        }
        let best_count = best.as_ref().map_or(C_MIN_API - 1, |b| b.2.len());
        if counts.len() > best_count && counts.len() >= C_MIN_API {
            let win_start = pos[left].0;
            let (last_off, last_name) = pos[right - 1];
            let win_end = last_off + last_name.len();
            let mut names: Vec<Vec<u8>> = counts.keys().map(|k| k.to_vec()).collect();
            names.sort();
            best = Some((win_start, win_end - win_start, names));
        }
        // slide the left edge
        let name = pos[left].1;
        if let Some(c) = counts.get_mut(name) {
            *c -= 1;
            if *c == 0 {
                counts.remove(name);
            }
        }
    }
    best
}

// ---------------------------------------------------------------------------
// layer expansion
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Raw,
    Lzma,
}

struct Layer {
    offset: usize,
    kind: Kind,
    blob: Vec<u8>,
}

/// `data` plus every decompressed layer (BFS, dedupe by blob hash), mirroring
/// `iter_layers` minus the unported zip-member expansion.
fn iter_layers(data: &[u8], deep: bool, max_depth: usize) -> Vec<Layer> {
    let mut out: Vec<Layer> = Vec::new();
    if data.is_empty() {
        return out;
    }
    let mut seen: HashSet<(usize, [u8; 16])> = HashSet::new();
    seen.insert(blob_key(data));
    out.push(Layer {
        offset: 0,
        kind: Kind::Raw,
        blob: data.to_vec(),
    });

    let mut queue: VecDeque<(usize, Vec<u8>, usize)> = VecDeque::new();
    queue.push_back((0, data.to_vec(), 0));
    while let Some((base, blob, depth)) = queue.pop_front() {
        if depth >= if deep { max_depth } else { 0 } {
            continue;
        }
        if blob.len() < MIN_BLOB || blob.len() > MAX_LZMA_BLOB {
            continue;
        }
        let mut streams = lzma_streams(&blob, MAX_STREAMS);
        if streams.is_empty() && depth == 0 && blob.len() <= (24 << 20) {
            // Raw LZMA1 has no magic, so probing is the expensive path.  When
            // the blob already exposes an uncompressed name table there is
            // nothing to gain, so skip it.
            if scan_name_table(&blob, None).is_none() {
                streams = raw_lzma_streams(&blob, 2);
            }
        }
        for (off, dec) in streams {
            let k = blob_key(&dec);
            if !seen.insert(k) {
                continue;
            }
            out.push(Layer {
                offset: base + off,
                kind: Kind::Lzma,
                blob: dec.clone(),
            });
            queue.push_back((base + off, dec, depth + 1));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// public API
// ---------------------------------------------------------------------------
#[derive(Clone)]
struct Hit {
    h: SymHit,
    /// Python's `hit["size"]`; not part of the public struct, but needed for
    /// dedupe keys and strength ranking.
    size: usize,
}

fn mk_hit(
    detector: char,
    offset: usize,
    size: usize,
    names: Vec<Vec<u8>>,
    has_table: bool,
    label: String,
    blob: Vec<u8>,
) -> Hit {
    let names: Vec<String> = names
        .iter()
        .map(|n| String::from_utf8_lossy(n).into_owned())
        .collect();
    let names_count = names.len();
    Hit {
        h: SymHit {
            detector,
            offset,
            names,
            names_count,
            blob,
            has_table,
            label,
        },
        size,
    }
}

fn rank(h: &Hit) -> (u8, usize, usize) {
    (h.h.has_table as u8, h.h.names_count, h.size)
}

fn dedupe_key(h: &Hit) -> (char, [u8; 16]) {
    if !h.h.blob.is_empty() {
        let n = h.h.blob.len();
        (h.h.detector, md5(&h.h.blob[..n.min(4096)]))
    } else {
        let s = format!("{}:{}", h.h.offset, h.size);
        (h.h.detector, md5(s.as_bytes()))
    }
}

fn dedupe_and_rank(hits: Vec<Hit>) -> Vec<SymHit> {
    let mut best: Vec<Hit> = Vec::new();
    let mut index: HashMap<(char, [u8; 16]), usize> = HashMap::new();
    for h in hits {
        let key = dedupe_key(&h);
        match index.get(&key) {
            Some(&slot) => {
                if rank(&h) > rank(&best[slot]) {
                    best[slot] = h;
                }
            }
            None => {
                index.insert(key, best.len());
                best.push(h);
            }
        }
    }
    let mut out = best;
    out.sort_by(|a, b| {
        (a.h.detector, std::cmp::Reverse(a.h.names_count), a.h.offset).cmp(&(
            b.h.detector,
            std::cmp::Reverse(b.h.names_count),
            b.h.offset,
        ))
    });

    let mut trimmed: Vec<Hit> = Vec::new();
    for det in ['A', 'B', 'C'] {
        let mut group: Vec<Hit> = out.iter().filter(|h| h.h.detector == det).cloned().collect();
        group.sort_by(|a, b| rank(b).cmp(&rank(a)));
        group.truncate(MAX_PER_DETECTOR);
        // keep the blob only for the strongest hits to bound memory
        for (i, h) in group.iter_mut().enumerate() {
            if i >= 2 {
                h.h.blob.clear();
            }
        }
        trimmed.extend(group);
    }
    trimmed.sort_by(|a, b| {
        let ka = (a.h.has_table as u8, a.h.names_count, a.size, a.h.detector);
        let kb = (b.h.has_table as u8, b.h.names_count, b.size, b.h.detector);
        kb.cmp(&ka)
    });
    trimmed.into_iter().map(|h| h.h).collect()
}

/// Detector A: a compressed (LZMA) VxWorks image, with the embedded name table
/// carved when possible.
fn detect_a(offset: usize, dec: &[u8]) -> Option<Hit> {
    let markers = VX_MARKERS.iter().filter(|m| contains(dec, m)).count();
    if markers < A_MIN_MARKERS {
        return None;
    }
    let api_names: Vec<&&[u8]> = VX_API.iter().filter(|n| contains(dec, n)).collect();
    if api_names.is_empty() {
        return None;
    }
    if let Some(table) = scan_name_table(dec, Some(markers)) {
        let label = format!(
            "compressed symbol table: LZMA stream, markers={}, carved {} names / {} suffixes",
            markers,
            table.names.len(),
            table.suffix
        );
        let blob = dec[table.start..table.end].to_vec();
        return Some(mk_hit(
            'A',
            offset,
            blob.len(),
            table.names,
            true,
            label,
            blob,
        ));
    }
    let names: Vec<Vec<u8>> = api_names.iter().map(|n| n.to_vec()).collect();
    let label = format!(
        "compressed VxWorks image: LZMA stream, markers={}, {} API names",
        markers,
        api_names.len()
    );
    Some(mk_hit('A', offset, dec.len(), names, false, label, Vec::new()))
}

/// Recover VxWorks symbol-table evidence from `data`.
///
/// See the module documentation for the detector definitions.  The result is
/// never `None`; an empty vector means no evidence was found.
pub fn find_symbols(data: &[u8]) -> Vec<SymHit> {
    if data.is_empty() || data.len() < MIN_BLOB {
        return Vec::new();
    }

    let mut hits: Vec<Hit> = Vec::new();
    for layer in iter_layers(data, true, MAX_DEPTH) {
        let blob = layer.blob.as_slice();
        if blob.len() < MIN_BLOB {
            continue;
        }

        let mut a_table = false;
        if layer.kind == Kind::Lzma {
            if let Some(hit) = detect_a(layer.offset, blob) {
                a_table = hit.h.has_table;
                hits.push(hit);
            }
        }

        let mut table: Option<NameTable> = None;
        if layer.kind != Kind::Lzma {
            table = scan_name_table(blob, None);
            if let Some(t) = &table {
                let label = format!(
                    "uncompressed symbol table: {} names, {} suffixes, kind=raw",
                    t.names.len(),
                    t.suffix
                );
                hits.push(mk_hit(
                    'B',
                    layer.offset + t.start,
                    t.end - t.start,
                    t.names.clone(),
                    true,
                    label,
                    blob[t.start..t.end].to_vec(),
                ));
            }
        }

        if table.is_none() && !a_table {
            if let Some((start, size, names)) = scan_density(blob) {
                let label = format!(
                    "symbol density: {} distinct VxWorks API names in one 64 KiB window, kind=raw",
                    names.len()
                );
                hits.push(mk_hit(
                    'C',
                    layer.offset + start,
                    size,
                    names,
                    false,
                    label,
                    Vec::new(),
                ));
            }
        }
    }

    dedupe_and_rank(hits)
}

/// The strongest hit that carries a real carved name table (its `blob` is the
/// SYMBOL_Table payload), or `None`.
pub fn recover_symbol_table(data: &[u8]) -> Option<SymHit> {
    for hit in find_symbols(data) {
        if hit.has_table && hit.names_count >= B_MIN_NAMES && !hit.blob.is_empty() {
            return Some(hit);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn hex(d: &[u8; 16]) -> String {
        d.iter().map(|b| format!("{:02x}", b)).collect()
    }

    #[test]
    fn md5_known_vectors() {
        assert_eq!(hex(&md5(b"")), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(hex(&md5(b"abc")), "900150983cd24fb0d6963f7d28e17f72");
        // > 1 block, exercises the buffered update path
        let long = vec![b'a'; 1000];
        assert_eq!(hex(&md5(&long)), "cabe45dcc9ae5b66ba86600cca6b8ba8");
        // data split across updates must hash like the concatenation
        let mut h = Md5::new();
        h.update(&long[..13]);
        h.update(&long[13..64]);
        h.update(&long[64..]);
        assert_eq!(hex(&h.finish()), "cabe45dcc9ae5b66ba86600cca6b8ba8");
    }

    fn sample_table(names: &[&str]) -> Vec<u8> {
        let mut blob = vec![0xEEu8; 32];
        for n in names {
            blob.extend_from_slice(n.as_bytes());
            blob.push(0);
            while blob.len() % 4 != 0 {
                blob.push(0);
            }
        }
        blob.extend_from_slice(&[0x11, 0x22, 0x33]);
        blob
    }

    #[test]
    fn detector_b_finds_aligned_table() {
        let mut names: Vec<String> = vec![
            "taskSpawn".to_string(),
            "semTake".to_string(),
            "usrRoot".to_string(),
            "bzero".to_string(),
        ];
        for i in 0..60 {
            names.push(format!("sym{:03}Task", i));
        }
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let blob = sample_table(&refs);
        let t = scan_name_table(&blob, None).expect("table");
        assert_eq!(t.names.len(), names.len());
        assert_eq!(t.start, 32);
        assert_eq!(&t.names[0], b"taskSpawn");
        assert_eq!(t.end, blob.len() - 3);
        assert!(t.suffix >= B_MIN_SUFFIX);
    }

    #[test]
    fn detector_b_trims_short_leading_tokens() {
        let mut names: Vec<String> = vec!["taskSpawn".to_string(), "usrRoot".to_string()];
        for i in 0..50 {
            names.push(format!("sym{:03}Isr", i));
        }
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let mut blob = vec![0xEEu8; 16];
        // "Lib.c" tail: only the short "c" token survives regex matching
        blob.extend_from_slice(b"c\x00");
        blob.extend_from_slice(b"ab\x00");
        blob.extend_from_slice(&sample_table(&refs));
        let t = scan_name_table(&blob, None).expect("table");
        assert_eq!(&t.names[0], b"taskSpawn");
        assert_eq!(t.names.len(), names.len());
    }

    #[test]
    fn detector_c_counts_distinct_api_names() {
        let api: [&[u8]; 16] = [
            b"taskSpawn", b"taskInit", b"semTake", b"semGive", b"msgQSend", b"intLock",
            b"memPartAlloc", b"wdStart", b"usrRoot", b"excJobAdd", b"bzero", b"symAdd",
            b"iosOpen", b"cacheFlush", b"kernelInit", b"objAlloc",
        ];
        let mut blob = vec![0xAAu8; 128 * 1024];
        let mut first = 0usize;
        let mut last_end = 0usize;
        for (i, name) in api.iter().enumerate() {
            let at = 1000 + i * 1000;
            blob[at..at + name.len()].copy_from_slice(name);
            if i == 0 {
                first = at;
            }
            last_end = at + name.len();
        }
        let (start, size, names) = scan_density(&blob).expect("density");
        assert_eq!(start, first);
        assert_eq!(size, last_end - first);
        assert_eq!(names.len(), 16);
        let mut sorted: Vec<Vec<u8>> = api.iter().map(|n| n.to_vec()).collect();
        sorted.sort();
        assert_eq!(names, sorted);
        // no NUL separated table -> detector B does not fire
        assert!(scan_name_table(&blob, None).is_none());
    }

    #[test]
    fn find_symbols_rejects_tiny_input() {
        assert!(find_symbols(&[]).is_empty());
        assert!(find_symbols(&[0u8; 16]).is_empty());
        assert!(recover_symbol_table(&[0u8; 16]).is_none());
    }
}
