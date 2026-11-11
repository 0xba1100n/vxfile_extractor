//! Netgear CHK：PK11 + [name\0][u32 LE size]... 数据顺序排列
use crate::minifs::Entry;

pub fn extract(blob: &[u8]) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut from = 0usize;
    while let Some(i) = memchr::memmem::find(&blob[from..], b"PK11") {
        let i = from + i;
        from = i + 4;
        let mut pos = i + 4;
        while pos < blob.len() && blob[pos] == 0 { pos += 1; }
        let mut ents: Vec<(String, usize)> = Vec::new();
        let mut q = pos;
        while q + 8 < blob.len() && ents.len() < 2000 {
            let e = match memchr::memchr(0, &blob[q..(q + 80).min(blob.len())]) {
                Some(x) => q + x,
                None => break,
            };
            let name = &blob[q..e];
            if name.is_empty() || name.iter().any(|c| *c < 32 || *c > 126) { break; }
            let size = u32::from_le_bytes([blob[e+1], blob[e+2], blob[e+3], blob[e+4]]) as usize;
            if size == 0 || size > 0x200000 { break; }
            ents.push((String::from_utf8_lossy(name).to_string(), size));
            q = e + 5;
        }
        let mut dp = q;
        for (name, size) in ents {
            if dp + size > blob.len() { break; }
            out.push(Entry { path: name, data: blob[dp..dp+size].to_vec() });
            dp += size;
        }
    }
    out
}