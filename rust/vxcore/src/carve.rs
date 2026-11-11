//! Content carving: images, HTML, JS, CSS (port of vxbinwalk's emit_carved).
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Carved {
    pub kind: String,
    pub offset: usize,
    pub size: usize,
    #[serde(skip)]
    pub data: Vec<u8>,
}

const MIN_SIZE: usize = 4096;
const MAX_FILES: usize = 200;

fn jpeg_end(d: &[u8], i: usize) -> Option<usize> {
    let mut p = i + 2;
    while p + 1 < d.len() {
        if d[p] == 0xFF && d[p + 1] == 0xD9 {
            return Some(p + 2);
        }
        p += 1;
    }
    None
}

fn png_end(d: &[u8], i: usize) -> Option<usize> {
    const IEND: &[u8] = b"IEND\xaeB`\x82";
    memchr::memmem::find(&d[i..], IEND).map(|r| i + r + IEND.len())
}

fn gif_end(d: &[u8], i: usize) -> Option<usize> {
    memchr::memmem::rfind(&d[i..], &b"\x00\x3b"[..]).map(|r| i + r + 2)
}

fn html_end(d: &[u8], i: usize) -> Option<usize> {
    let rest = &d[i..];
    let lower_hit = memchr::memmem::find(rest, b"</html>");
    let upper_hit = memchr::memmem::find(rest, b"</HTML>");
    match (lower_hit, upper_hit) {
        (Some(a), Some(b)) => Some(i + a.min(b) + 7),
        (Some(a), None) => Some(i + a + 7),
        (None, Some(b)) => Some(i + b + 7),
        (None, None) => None,
    }
}

fn printable_ratio(d: &[u8], a: usize, b: usize) -> f64 {
    let end = b.min(d.len());
    if end <= a {
        return 0.0;
    }
    let mut ok = 0usize;
    let mut n = 0usize;
    let mut p = a;
    while p < end {
        let c = d[p];
        n += 1;
        if c == 9 || c == 10 || c == 13 || (32..127).contains(&c) {
            ok += 1;
        }
        p += 1;
    }
    ok as f64 / n as f64
}

/// Find embedded images / HTML documents / JS / CSS blocks.
pub fn carve(data: &[u8]) -> Vec<Carved> {
    let mut out: Vec<Carved> = Vec::new();
    let mut push = |kind: &str, off: usize, end: usize| {
        if end <= off || end > data.len() || out.len() >= MAX_FILES {
            return;
        }
        let size = end - off;
        if size < MIN_SIZE {
            return;
        }
        out.push(Carved { kind: kind.to_string(), offset: off, size, data: data[off..end].to_vec() });
    };

    // JPEG
    let mut p = 0usize;
    while let Some(i) = memchr::memmem::find(&data[p..], &b"\xff\xd8\xff"[..]) {
        let off = p + i;
        p = off + 3;
        if let Some(end) = jpeg_end(data, off) {
            push("jpeg", off, end);
        }
    }
    // PNG
    let mut p = 0usize;
    while let Some(i) = memchr::memmem::find(&data[p..], &b"\x89PNG\r\n\x1a\n"[..]) {
        let off = p + i;
        p = off + 8;
        if let Some(end) = png_end(data, off) {
            push("png", off, end);
        }
    }
    // GIF
    let mut p = 0usize;
    while let Some(i) = memchr::memmem::find(&data[p..], &b"GIF8"[..]) {
        let off = p + i;
        p = off + 6;
        if let Some(end) = gif_end(data, off) {
            push("gif", off, end);
        }
    }
    // HTML documents
    let mut p = 0usize;
    for pat in [&b"<!DOCTYPE html"[..], &b"<html"[..], &b"<HTML"[..]] {
        let mut q = 0usize;
        while let Some(i) = memchr::memmem::find(&data[q..], pat) {
            let off = q + i;
            q = off + pat.len();
            if off > 0 {
                let prev = data[off - 1];
                if prev.is_ascii_alphanumeric() {
                    continue;
                }
            }
            let end = match html_end(data, off) {
                Some(e) => e,
                None => continue,
            };
            if printable_ratio(data, off, end) > 0.85 {
                push("html", off, end);
            }
        }
    }
    out.sort_by_key(|c| c.offset);
    out
}