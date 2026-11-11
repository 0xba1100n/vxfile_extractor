//! v2：头 32 字节 + 记录(32) 与流交替
pub fn extract_v2(data: &[u8]) -> Vec<crate::minifs::Entry> {
    use std::io::Read;
    let m = match memchr::memmem::find(data, b"MINIFS") { Some(x) => x, None => return Vec::new() };
    if m + 0x20 > data.len() { return Vec::new(); }
    let ver = u32::from_be_bytes([data[m+0x10], data[m+0x11], data[m+0x12], data[m+0x13]]);
    let files_no = u32::from_be_bytes([data[m+0x14], data[m+0x15], data[m+0x16], data[m+0x17]]) as usize;
    if ver != 2 || files_no == 0 || files_no > 5000 { return Vec::new(); }
    let mut pos = m + 0x20;
    let mut out = Vec::new();
    for k in 0..files_no {
        if pos + 32 > data.len() { break; }
        let csize = u32::from_be_bytes([data[pos+16], data[pos+17], data[pos+18], data[pos+19]]) as usize;
        let end = pos + 32 + csize;
        if end > data.len() || csize == 0 { break; }
        let mut dec = Vec::new();
        if let Ok(st) = xz2::stream::Stream::new_lzma_decoder(u64::MAX) {
            let mut d = xz2::read::XzDecoder::new_stream(&data[pos+32..end], st);
            let _ = d.read_to_end(&mut dec);
        }
        if !dec.is_empty() {
            let ext = super::lzcarve::guess_ext(&dec);
            out.push(crate::minifs::Entry { path: format!("v2_{:03}_{:08X}{}", k, pos, ext), data: dec });
        }
        pos = end;
    }
    out
}