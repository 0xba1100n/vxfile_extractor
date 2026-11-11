//! MINIFS 解包（v1 逐文件 LZMA / v3 分块 LZMA）
//! 参考：H4lo/MINIFS-Decompression、RedaOps/minifs-extractor、arXiv 2407.05064
use std::io::Read;
use xz2::read::XzDecoder;
use xz2::stream::Stream;

pub struct Entry {
    pub path: String,
    pub data: Vec<u8>,
}

fn de_lzma(blob: &[u8]) -> Option<Vec<u8>> {
    let mut dec = XzDecoder::new_stream(blob, Stream::new_lzma_decoder(u64::MAX).ok()?);
    let mut out = Vec::new();
    let _ = dec.read_to_end(&mut out);
    if out.is_empty() { None } else { Some(out) }
}

fn cstr(data: &[u8], off: usize, limit: usize) -> String {
    if off >= data.len() { return String::new(); }
    let end = (off + limit).min(data.len());
    let mut e = end;
    for i in off..end {
        if data[i] == 0 { e = i; break; }
    }
    String::from_utf8_lossy(&data[off..e]).to_string()
}

fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if from >= hay.len() { return None; }
    memchr::memmem::find(&hay[from..], needle).map(|i| i + from)
}

/// v1：MINIFS + [size][offset][name:0x50]，每个文件一条 LZMA 流
pub fn extract_v1(data: &[u8]) -> Vec<Entry> {
    let m = match find(data, b"MINIFS", 0) { Some(x) => x, None => return Vec::new() };
    let base = match find(data, b"\x5a\x00\x00\x80", m) { Some(x) => x, None => return Vec::new() };
    let mut ents: Vec<(String, usize, usize)> = Vec::new(); // name, size, off
    let mut p = m + 20;
    while p + 88 <= data.len() {
        let size = u32::from_be_bytes([data[p], data[p+1], data[p+2], data[p+3]]) as usize;
        let off = u32::from_be_bytes([data[p+4], data[p+5], data[p+6], data[p+7]]) as usize;
        if data[p+8] != b'/' { break; }
        let name = cstr(data, p + 8, 0x50);
        ents.push((name, size, off));
        p += 88;
    }
    if ents.is_empty() { return Vec::new(); }
    ents.sort_by_key(|e| e.2);
    let mut out = Vec::new();
    for i in 0..ents.len() {
        let start = base + ents[i].2;
        let end = if i + 1 < ents.len() { base + ents[i+1].2 } else { data.len() };
        if start >= end || end > data.len() { continue; }
        if let Some(dec) = de_lzma(&data[start..end]) {
            out.push(Entry { path: ents[i].0.clone(), data: dec });
        }
    }
    out
}

/// v3：32 字节头 + ToN/ToF/ToC + chunk 级 LZMA
pub fn extract_chunked(data: &[u8]) -> Vec<Entry> {
    let m = match find(data, b"MINIFS", 0) { Some(x) => x, None => return Vec::new() };
    if m + 32 > data.len() { return Vec::new(); }
    let files_no = u32::from_be_bytes([data[m+0x14], data[m+0x15], data[m+0x16], data[m+0x17]]) as usize;
    let ton_size = u32::from_be_bytes([data[m+0x1c], data[m+0x1d], data[m+0x1e], data[m+0x1f]]) as usize;
    if files_no == 0 || files_no > 5000 || ton_size > (4 << 20) { return Vec::new(); }
    let ton = m + 32;
    let tof = ton + ton_size;
    let toc = tof + 20 * files_no;
    if toc + 12 > data.len() { return Vec::new(); }
    let mut files = Vec::with_capacity(files_no);
    for k in 0..files_no {
        let e = tof + 20 * k;
        if e + 20 > data.len() { return Vec::new(); }
        let po = u32::from_be_bytes([data[e], data[e+1], data[e+2], data[e+3]]) as usize;
        let no = u32::from_be_bytes([data[e+4], data[e+5], data[e+6], data[e+7]]) as usize;
        let ch = u32::from_be_bytes([data[e+8], data[e+9], data[e+10], data[e+11]]) as usize;
        let oc = u32::from_be_bytes([data[e+12], data[e+13], data[e+14], data[e+15]]) as usize;
        let sz = u32::from_be_bytes([data[e+16], data[e+17], data[e+18], data[e+19]]) as usize;
        files.push((po, no, ch, oc, sz));
    }
    let chunks_no = files[files.len()-1].2 + 1;
    let raw = toc + 12 * chunks_no;
    if raw + 4 > data.len() || &data[raw..raw+4] != b"\x5d\x00\x00\x80" { return Vec::new(); }
    let mut chunk_cache: Vec<Option<Vec<u8>>> = vec![None; chunks_no];
    let mut out = Vec::new();
    for (po, no, ch, oc, sz) in files {
        if ch >= chunks_no { continue; }
        if chunk_cache[ch].is_none() {
            let te = toc + 12 * ch;
            let co = u32::from_be_bytes([data[te], data[te+1], data[te+2], data[te+3]]) as usize;
            let cs = u32::from_be_bytes([data[te+4], data[te+5], data[te+6], data[te+7]]) as usize;
            let s = raw + co;
            if s + cs <= data.len() {
                chunk_cache[ch] = de_lzma(&data[s..s+cs]);
            }
        }
        if let Some(blob) = &chunk_cache[ch] {
            if oc + sz <= blob.len() {
                let path = cstr(data, ton + po, 512);
                let name = cstr(data, ton + no, 512);
                let full = format!("{}/{}", path.trim_matches('/'), name);
                out.push(Entry { path: full, data: blob[oc..oc+sz].to_vec() });
            }
        }
    }
    out
}

pub fn extract(data: &[u8]) -> Vec<Entry> {
    let v1 = extract_v1(data);
    if !v1.is_empty() { return v1; }
    extract_chunked(data)
}