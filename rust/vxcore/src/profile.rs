//! 逐偏移画像：每个窗口的熵 + 类型定性（仿 binwalk -E / 签名扫描）

pub fn shannon(b: &[u8]) -> f64 {
    if b.is_empty() { return 0.0; }
    let mut cnt = [0usize; 256];
    for &c in b { cnt[c as usize] += 1; }
    let n = b.len() as f64;
    let mut h = 0.0;
    for &c in cnt.iter() {
        if c > 0 { let p = c as f64 / n; h -= p * p.log2(); }
    }
    h
}

fn printable_ratio(b: &[u8]) -> f64 {
    if b.is_empty() { return 0.0; }
    let n = b.iter().filter(|c| (32u8..127u8).contains(c) || **c == b'\n' || **c == b'\r' || **c == b'\t').count();
    n as f64 / b.len() as f64
}

/// 对窗口起点做类型判定（只看窗口内容）
pub fn classify(win: &[u8]) -> &'static str {
    let w = win;
    let has = |p: &[u8]| memchr::memmem::find(w, p).is_some();
    if w.iter().all(|c| *c == 0) { return "zero"; }
    if w.len() >= 4 && &w[..4] == b"\x27\x05\x19\x56" { return "uimage"; }
    if w.len() >= 4 && &w[..4] == b"HDR0" { return "trx"; }
    if has(b"MINIFS") { return "minifs"; }
    if has(b"PK11") { return "pk11"; }
    if has(b"OWOWOWOWOWOWOWOWOWOWOWOWOWOWOWOW") { return "ow"; }
    if w.len() >= 5 && matches!(w[0], 0x5d | 0x5a | 0x6e | 0x5c | 0x5e) {
        let dict = u32::from_le_bytes([w[1], w[2], w[3], w[4]]);
        if (4096..=(1 << 28)).contains(&dict) && (dict & (dict - 1)) == 0 {
            return "lzma";
        }
    }
    if w.len() >= 3 && &w[..3] == b"\x1f\x8b\x08" { return "gzip"; }
    if w.len() >= 2 && w[0] == 0x78 && (w[1] == 0x9c || w[1] == 0x01 || w[1] == 0xda || w[1] == 0x5e) { return "zlib"; }
    if has(b"BZh9") { return "bzip2"; }
    if w.len() >= 6 && &w[..6] == b"\xfd7zXZ\x00" { return "xz"; }
    if w.len() >= 6 && &w[..6] == b"7z\xbc\xaf\x27\x1c" { return "7z"; }
    if w.len() >= 4 && &w[..4] == b"Rar!" { return "rar"; }
    if w.len() >= 4 && &w[..4] == b"PK\x03\x04" { return "zip"; }
    if w.len() >= 4 && &w[..4] == b"\x7fELF" { return "elf"; }
    if w.len() >= 2 && &w[..2] == b"MZ" { return "pe"; }
    if w.len() >= 8 && &w[..8] == b"\x89PNG\r\n\x1a\n" { return "png"; }
    if w.len() >= 3 && &w[..3] == b"\xff\xd8\xff" { return "jpeg"; }
    if w.len() >= 4 && &w[..4] == b"GIF8" { return "gif"; }
    if has(b"WIND version") || has(b"vxWorks") { return "vxworks"; }
    let h = shannon(w);
    if h >= 7.5 { return "high-entropy"; }
    if printable_ratio(w) > 0.85 { return "text"; }
    if h <= 1.0 { return "padding"; }
    "data"
}

#[derive(Debug, Clone)]
pub struct Block {
    pub offset: usize,
    pub size: usize,
    pub ent: f64,
    pub kind: &'static str,
}

pub fn blocks(data: &[u8], win: usize) -> Vec<Block> {
    let mut out = Vec::new();
    for (i, w) in data.chunks(win).enumerate() {
        out.push(Block {
            offset: i * win,
            size: w.len(),
            ent: shannon(w),
            kind: classify(w),
        });
    }
    out
}

#[derive(Debug, Clone)]
pub struct Region {
    pub start: usize,
    pub end: usize,
    pub kind: &'static str,
    pub ent: f64,
    pub blocks: usize,
}

/// 合并相邻同类块成区间
pub fn regions(bs: &[Block]) -> Vec<Region> {
    let mut out: Vec<Region> = Vec::new();
    for b in bs {
        match out.last_mut() {
            Some(r) if r.kind == b.kind => {
                r.end = b.offset + b.size;
                r.ent = (r.ent * r.blocks as f64 + b.ent) / (r.blocks + 1) as f64;
                r.blocks += 1;
            }
            _ => out.push(Region { start: b.offset, end: b.offset + b.size, kind: b.kind, ent: b.ent, blocks: 1 }),
        }
    }
    out
}