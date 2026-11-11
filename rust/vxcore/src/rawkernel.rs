//! 裸内核识别：整片镜像，或 20 字节头 + ARM 内核。
use crate::minifs::Entry;

fn arm_prologue(blob: &[u8], off: usize) -> bool {
    if off + 8 > blob.len() { return false; }
    let a = u32::from_le_bytes([blob[off], blob[off+1], blob[off+2], blob[off+3]]);
    let b = u32::from_le_bytes([blob[off+4], blob[off+5], blob[off+6], blob[off+7]]);
    fn ok(x: u32) -> bool {
        if (x >> 28) != 0xE { return false; }
        ((x >> 26) & 3) == 0 || ((x >> 25) & 7) == 5
    }
    ok(a) && ok(b)
}

/// 返回 (类别, 头部长度)
pub fn detect(blob: &[u8]) -> Option<(&'static str, usize)> {
    if blob.len() < 64 { return None; }
    let n = u32::from_be_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    if n == blob.len() - 20 && arm_prologue(blob, 20) {
        return Some(("arm-kernel-with-20b-header", 20));
    }
    if memchr::memmem::find(&blob[..blob.len().min(8 << 20)], b"WIND version").is_some() {
        return Some(("flat-vxworks-kernel", 0));
    }
    None
}

pub fn extract(blob: &[u8]) -> Option<(&'static str, Entry)> {
    let (label, hdr) = detect(blob)?;
    Some((label, Entry { path: "kernel.bin".to_string(), data: blob[hdr..].to_vec() }))
}