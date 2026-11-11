//! 通用 LZMA-alone 流兜底：扫镜像里所有合法流并解出。
use crate::minifs::Entry;

fn is_pow2_dict(d: u32) -> bool {
    (4096..=(1u32 << 28)).contains(&d) && (d & (d - 1)) == 0
}

fn decode(blob: &[u8]) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut dec = xz2::read::XzDecoder::new_stream(
        blob, xz2::stream::Stream::new_lzma_decoder(u64::MAX).ok()?);
    let mut out = Vec::new();
    let _ = dec.read_to_end(&mut out);
    if out.len() >= 4096 { Some(out) } else { None }
}

pub fn guess_ext(blob: &[u8]) -> &'static str {
    let head = &blob[..blob.len().min(16)];
    if head.starts_with(b"\x89PNG") { return ".png"; }
    if head.starts_with(b"GIF8") { return ".gif"; }
    if head.starts_with(b"-----BEGIN") { return ".pem"; }
    let t = head.iter().position(|c| !c.is_ascii_whitespace()).map(|i| &blob[i..]).unwrap_or(blob);
    if t.first() == Some(&b'<') {
        let low = &blob[..blob.len().min(4096)];
        let s = String::from_utf8_lossy(low).to_lowercase();
        if s.contains("<?xml") { return ".xml"; }
        return ".html";
    }
    let low = &blob[..blob.len().min(2048)];
    let s = String::from_utf8_lossy(low);
    if s.contains("function") || s.contains("var ") { return ".js"; }
    ".bin"
}

/// 返回解出的流（最多 200 条，按内容去重）
pub fn carve(data: &[u8], max: usize) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let n = data.len();
    for o in 0..n.saturating_sub(16) {
        if out.len() >= max { break; }
        if data[o] >= 225 { continue; }
        let dict = u32::from_le_bytes([data[o+1], data[o+2], data[o+3], data[o+4]]);
        if !is_pow2_dict(dict) { continue; }
        let us = u64::from_le_bytes([data[o+5], data[o+6], data[o+7], data[o+8],
                                     data[o+9], data[o+10], data[o+11], data[o+12]]);
        if us != u64::MAX && us > (256 << 20) { continue; }
        let dec = match decode(&data[o..]) { Some(d) => d, None => continue };
        if let Some(ow) = parse_ow(&dec) {
            out.extend(ow);
            continue;
        }
        let mut h: u64 = 1469598103934665603;
        for b in dec.iter().step_by(97) { h ^= *b as u64; h = h.wrapping_mul(1099511628211); }
        if !seen.insert(h) { continue; }
        let ext = guess_ext(&dec);
        out.push(Entry { path: format!("stream_{:08X}{}", o, ext), data: dec });
    }
    out
}

/// MR900 系列：流里是 "OW"*16 魔数的文件系统
pub fn parse_ow(blob: &[u8]) -> Option<Vec<Entry>> {
    if blob.len() < 0x28 { return None; }
    if &blob[..32] != &b"OW".repeat(16)[..] { return None; }
    let count = u32::from_be_bytes([blob[0x24], blob[0x25], blob[0x26], blob[0x27]]) as usize;
    if count == 0 || count > 5000 || 0x28 + 76 * count > blob.len() { return None; }
    let mut out = Vec::new();
    for k in 0..count {
        let off = 0x28 + 76 * k;
        if off + 76 > blob.len() { break; }
        let name_end = blob[off..off+60].iter().position(|c| *c == 0).unwrap_or(60);
        let name = String::from_utf8_lossy(&blob[off..off+name_end]).to_string();
        let size = u32::from_be_bytes([blob[off+64], blob[off+65], blob[off+66], blob[off+67]]) as usize;
        let foff = u32::from_be_bytes([blob[off+68], blob[off+69], blob[off+70], blob[off+71]]) as usize;
        if name.is_empty() || size == 0 || foff + size > blob.len() { continue; }
        out.push(Entry { path: name.trim_start_matches('/').to_string(), data: blob[foff..foff+size].to_vec() });
    }
    if out.is_empty() { None } else { Some(out) }
}
