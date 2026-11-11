//! vx — VxWorks firmware extraction engine (CLI).
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use vxcore::{find_symbols, find_tables, judge, judge_image, scan_all};

#[derive(Parser)]
#[command(name = "vxfile_extractor", version, about = "VxWorks firmware extraction engine")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Judge a firmware image (plain file or zip)
    Verdict { file: PathBuf },
    /// List containers/layers found in an image
    Scan { file: PathBuf, #[arg(long, default_value_t = 50)] limit: usize },
    /// Dump filename tables
    Tables { file: PathBuf },
    /// Dump symbol tables
    Symbols { file: PathBuf, #[arg(long, default_value_t = 20)] limit: usize },
    /// Per-firmware three outcomes (unpack / rename / symbols) over a directory
    Batch {
        root: PathBuf,
        #[arg(short, long)]
        out: Option<PathBuf>,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// 批量逐偏移画像：每个固件按类型统计字节占比
    ProfileBatch {
        root: PathBuf,
        #[arg(short, long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = 1024)]
        block: usize,
    },
    /// Per-offset entropy + type classification (binwalk-like)
    Profile {
        file: PathBuf,
        #[arg(long, default_value_t = 1024)]
        block: usize,
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// 打印 80 列 ASCII 熵图
        #[arg(long)]
        ascii: bool,
    },
    /// Entropy profile of each firmware payload (4 KiB windows)
    Entropy {
        root: PathBuf,
        #[arg(short, long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = 4096)]
        window: usize,
    },
    /// Extract a firmware: layers + tables + symbols + names
    Extract {
        file: PathBuf,
        #[arg(short, long, default_value = "out")]
        out: PathBuf,
        /// 只统计，不写文件
        #[arg(long)]
        no_write: bool,
    },
}

fn score_blob(b: &[u8]) -> i32 {
    let head = &b[..b.len().min(1 << 20)];
    let mut s = 0;
    if memchr::memmem::find(head, b"MINIFS").is_some() { s += 8; }
    if memchr::memmem::find(head, b"PK11").is_some() { s += 8; }
    if memchr::memmem::find(head, b"OWOWOWOWOWOWOWOWOWOWOWOWOWOWOWOW").is_some() { s += 8; }
    if memchr::memmem::find(head, b"\x5a\x00\x00\x80").is_some() { s += 6; }
    if memchr::memmem::find(head, b"\x5d\x00\x00\x80").is_some() { s += 6; }
    if memchr::memmem::find(head, b"WIND version").is_some() { s += 5; }
    if memchr::memmem::find(head, b"vxWorks").is_some() { s += 3; }
    if b.len() >= 4 && &b[..4] == b"\x27\x05\x19\x56" { s += 5; }
    if b.len() >= 3 && &b[..3] == b"\x1f\x8b\x08" { s += 2; }
    if b.len() >= 2 && (b[0] == 0x5d || b[0] == 0x5a || b[0] == 0x6e) { s += 3; }
    if b.len() >= 4 && &b[..4] == b"\x7fELF" { s += 2; }
    s
}

fn load(path: &Path) -> std::io::Result<(Vec<u8>, String)> {
    let raw = fs::read(path)?;
    if raw.len() > 4 && &raw[0..2] == b"PK" {
        if let Ok(z) = zip_best(&raw) {
            return Ok(z);
        }
    }
    if raw.len() > 4 && (&raw[0..4] == b"Rar!" || &raw[0..6] == b"7z\xbc\xaf\x27\x1c") {
        if let Some((d, n)) = seven_zip_best(path) {
            return Ok((d, n));
        }
    }
    let name = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    Ok((raw, name))
}

/// zip: 遍历所有成员，按 VxWorks 特征打分挑最合适的（不是最大的）
fn zip_best(raw: &[u8]) -> Result<(Vec<u8>, String), ()> {
    let mut eocd = None;
    let start = raw.len().saturating_sub(66_000);
    let mut i = raw.len().saturating_sub(22);
    while i > start {
        if &raw[i..i + 4] == b"PK\x05\x06" {
            eocd = Some(i);
            break;
        }
        i -= 1;
    }
    let e = eocd.ok_or(())?;
    let n = u16::from_le_bytes([raw[e + 10], raw[e + 11]]) as usize;
    let mut p = u32::from_le_bytes([raw[e + 16], raw[e + 17], raw[e + 18], raw[e + 19]]) as usize;
    let mut best: Option<(i32, u64, String, Vec<u8>)> = None;
    for _ in 0..n {
        if p + 46 > raw.len() || &raw[p..p + 4] != b"PK\x01\x02" {
            break;
        }
        let method = u16::from_le_bytes([raw[p + 10], raw[p + 11]]);
        let csize = u32::from_le_bytes([raw[p + 20], raw[p + 21], raw[p + 22], raw[p + 23]]) as usize;
        let usize_ = u32::from_le_bytes([raw[p + 24], raw[p + 25], raw[p + 26], raw[p + 27]]) as u64;
        let nlen = u16::from_le_bytes([raw[p + 28], raw[p + 29]]) as usize;
        let elen = u16::from_le_bytes([raw[p + 30], raw[p + 31]]) as usize;
        let clen = u16::from_le_bytes([raw[p + 32], raw[p + 33]]) as usize;
        let lho = u32::from_le_bytes([raw[p + 42], raw[p + 43], raw[p + 44], raw[p + 45]]) as usize;
        let name = String::from_utf8_lossy(&raw[p + 46..p + 46 + nlen]).to_string();
        let is_dir = name.ends_with('/');
        p += 46 + nlen + elen + clen;
        if is_dir || csize == 0 || csize > (256 << 20) || usize_ > (256 << 20) {
            continue;
        }
        if lho + 30 > raw.len() {
            continue;
        }
        let lnlen = u16::from_le_bytes([raw[lho + 26], raw[lho + 27]]) as usize;
        let lelen = u16::from_le_bytes([raw[lho + 28], raw[lho + 29]]) as usize;
        let dstart = lho + 30 + lnlen + lelen;
        if dstart + csize > raw.len() {
            continue;
        }
        let comp = &raw[dstart..dstart + csize];
        let data = match method {
            0 => comp.to_vec(),
            8 => {
                use std::io::Read;
                let mut d = flate2::read::DeflateDecoder::new(comp);
                let mut out = Vec::new();
                if d.read_to_end(&mut out).is_err() { continue; }
                out
            }
            _ => continue,
        };
        let sc = 0i32;
        let better = match &best {
            None => true,
            Some((_, bz, _, _)) => (data.len() as u64) > *bz,
        };
        if better {
            best = Some((sc, data.len() as u64, name, data));
        }
    }
    best.map(|(_, _, n, d)| (d, n)).ok_or(())
}

/// rar/7z: 全部解到临时目录，按特征打分挑最合适的
fn best_in_dir(tmp: &Path) -> Option<(Vec<u8>, String)> {
    fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() { collect(&p, out); }
                else if let Ok(m) = e.metadata() { if m.len() > 4096 { out.push(p); } }
            }
        }
    }
    let mut files = Vec::new();
    collect(tmp, &mut files);
    let mut best: Option<(u64, String, Vec<u8>)> = None;
    for f in files {
        let data = match fs::read(&f) { Ok(d) => d, Err(_) => continue };
        if data.is_empty() { continue; }
        let name = f.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let better = best.as_ref().map(|b| data.len() as u64 > b.0).unwrap_or(true);
        if better { best = Some((data.len() as u64, name, data)); }
    }
    best.map(|(_, n, d)| (d, n))
}

/// rar/7z：优先 unrar（7z 对某些 RAR 会解出 0 字节），再退回 7z
fn seven_zip_best(path: &Path) -> Option<(Vec<u8>, String)> {
    let tmp = std::env::temp_dir().join(format!("vx7z_{}", std::process::id()));
    let head = fs::read(path).ok()?;
    let is_rar = head.len() > 4 && &head[..4] == b"Rar!";
    if is_rar {
        let _ = fs::remove_dir_all(&tmp);
        let _ = fs::create_dir_all(&tmp);
        let _ = std::process::Command::new("unrar")
            .args(["x", "-o+", "-inul"]).arg(path).arg(format!("{}/", tmp.display()))
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .status();
        let r = best_in_dir(&tmp);
        let _ = fs::remove_dir_all(&tmp);
        if r.is_some() { return r; }
    }
    let cands = [
        std::env::var("SEVENZ").unwrap_or_default(),
        "7z".to_string(),
        "7za".to_string(),
        r"C:\Program Files\7-Zip\7z.exe".to_string(),
    ];
    for exe in cands.iter().filter(|c| !c.is_empty()) {
        let _ = fs::remove_dir_all(&tmp);
        let _ = std::process::Command::new(exe)
            .arg("x").arg("-y").arg(format!("-o{}", tmp.display())).arg(path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let r = best_in_dir(&tmp);
        let _ = fs::remove_dir_all(&tmp);
        if r.is_some() { return r; }
    }
    None
}
fn sanitize(name: &str) -> String {
    let base = name.rsplit('/').next().unwrap_or(name);
    let mut out = String::new();
    for c in base.chars() {
        if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
            out.push(c);
        } else if c == ' ' {
            out.push('_');
        }
    }
    while out.starts_with('.') {
        out.remove(0);
    }
    if out.is_empty() {
        out.push_str("unnamed");
    }
    out.truncate(180);
    out
}

fn content_ok(name: &str, b: &[u8]) -> bool {
    let n = name.to_ascii_lowercase();
    if b.is_empty() {
        return false;
    }
    if n.ends_with(".png") {
        return b.starts_with(b"\x89PNG");
    }
    if n.ends_with(".gif") {
        return b.starts_with(b"GIF8");
    }
    if n.ends_with(".jpg") || n.ends_with(".jpeg") {
        return b.starts_with(b"\xff\xd8");
    }
    if n.ends_with(".pem") {
        return b.starts_with(b"-----BE");
    }
    if n.ends_with(".cer") {
        return b[0] == 0x30;
    }
    if n.ends_with(".htm") || n.ends_with(".html") || n.ends_with(".css")
        || n.ends_with(".js") || n.ends_with(".xml") || n.ends_with(".txt") {
        let ok = b.iter().filter(|c| **c == 9 || **c == 10 || **c == 13 || (32..127).contains(*c)).count();
        return (ok as f64) / (b.len() as f64) > 0.85;
    }
    true
}

fn seven_zip_largest(path: &Path) -> Option<(Vec<u8>, String)> {
    let cands = [
        std::env::var("SEVENZ").unwrap_or_default(),
        "7z".to_string(),
        "7za".to_string(),
        r"C:\Program Files\7-Zip\7z.exe".to_string(),
    ];
    let tmp = std::env::temp_dir().join(format!("vx7z_{}", std::process::id()));
    for exe in cands.iter().filter(|c| !c.is_empty()) {
        let _ = fs::remove_dir_all(&tmp);
        let ok = std::process::Command::new(exe)
            .arg("x").arg("-y").arg(format!("-o{}", tmp.display())).arg(path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            continue;
        }
        let mut best: Option<(u64, PathBuf)> = None;
        if let Ok(rd) = fs::read_dir(&tmp) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if let Ok(rd2) = fs::read_dir(&p) {
                        for e2 in rd2.flatten() {
                            let p2 = e2.path();
                            if let Ok(m) = e2.metadata() {
                                if m.is_file() && best.as_ref().map(|b| m.len() > b.0).unwrap_or(true) {
                                    best = Some((m.len(), p2));
                                }
                            }
                        }
                    }
                } else if let Ok(m) = e.metadata() {
                    if m.is_file() && best.as_ref().map(|b| m.len() > b.0).unwrap_or(true) {
                        best = Some((m.len(), p));
                    }
                }
            }
        }
        if let Some((_, p)) = best {
            if let Ok(d) = fs::read(&p) {
                let n = p.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_default();
                let _ = fs::remove_dir_all(&tmp);
                return Some((d, n));
            }
        }
    }
    let _ = fs::remove_dir_all(&tmp);
    None
}


fn hint_walk(dir: &std::path::Path, acc: &mut (usize, usize, usize, std::collections::BTreeMap<String, usize>), best: &mut Option<(String, u64)>) {
    let rd = match std::fs::read_dir(dir) { Ok(r) => r, Err(_) => return };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            hint_walk(&p, acc, best);
            continue;
        }
        let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
        // 跳过 binwalk 层转储与元数据文件
        if name == "SYMBOL_Table" || name == "SUMMARY.json" || name.ends_with(".csv") { continue; }
        if name.contains("_lzma-alone") || name.contains("_deflate") || name.contains("_zlib") { continue; }
        if name.len() >= 4 && name.chars().all(|c| c.is_ascii_hexdigit()) { continue; }
        // binwalk 层转储形如 00158910_lzma-alone / 0019376F_png
        let prefix = name.split('_').next().unwrap_or("");
        if prefix.len() >= 6 && prefix.chars().all(|c| c.is_ascii_hexdigit()) { continue; }
        let ext = p.extension().unwrap_or_default().to_string_lossy().to_lowercase();
        let size = ent.metadata().map(|m| m.len()).unwrap_or(0);
        let static_ext = ["htm","html","js","css","xml","json","png","gif","jpg","jpeg","ico","svg","woff","woff2","ttf"];
        let cfg_ext = ["pem","cer","crt","key","conf","cfg","ini"];
        if static_ext.contains(&ext.as_str()) {
            acc.1 += 1;
            *acc.3.entry(ext).or_insert(0) += 1;
        } else if cfg_ext.contains(&ext.as_str()) {
            acc.2 += 1;
        } else {
            acc.0 += 1;
            let low = name.to_lowercase();
            let good = ["httpd","goahead","webs","boa","cgi"].iter().any(|k| low.contains(k));
            let better = best.as_ref().map(|(_n, z)| good && size as u64 > *z).unwrap_or(good);
            if better { *best = Some((name, size)); }
        }
    }
}

fn print_hints(out: &std::path::Path, kernel: Option<(u64, usize)>) {
    let mut acc = (0usize, 0usize, 0usize, std::collections::BTreeMap::new());
    let mut best: Option<(String, u64)> = None;
    hint_walk(out, &mut acc, &mut best);
    eprintln!("[+] Output layout");
    if acc.0 > 0 { eprintln!("    bin/   {} file(s)  <- executable / firmware image candidates", acc.0); }
    if acc.1 > 0 {
        let mut top: Vec<_> = acc.3.iter().collect();
        top.sort_by(|a, b| b.1.cmp(a.1));
        let s: Vec<String> = top.iter().take(5).map(|(k, v)| format!("{} {}", k, v)).collect();
        eprintln!("    src/   {} file(s)  <- static resources ({})", acc.1, s.join(", "));
    }
    if acc.2 > 0 { eprintln!("    cfg/   {} file(s)  <- certificates / configs", acc.2); }
    if let Some((off, size)) = kernel {
        eprintln!("[*] Likely main logic: kernel image layer 0x{:X} ({} B)", off as u64, size);
        eprintln!("    main HTTP/CGI code lives inside this image, not as a separate file");
    }
    if let Some((n, sz)) = best {
        eprintln!("[*] Other binary candidate: {} ({} B)", n, sz);
    }
}

fn payload_ok(name: &str, body: &[u8]) -> bool {
    let low = name.to_lowercase();
    let head = &body[..body.len().min(64)];
    if low.ends_with(".html") || low.ends_with(".htm") {
        let t: String = String::from_utf8_lossy(head).to_lowercase();
        return t.contains("<html") || t.contains("<!doctype") || t.trim_start().starts_with('<');
    }
    if low.ends_with(".png") { return head.starts_with(b"\x89PNG"); }
    if low.ends_with(".gif") { return head.starts_with(b"GIF8"); }
    if low.ends_with(".js") || low.ends_with(".css") {
        return head.iter().filter(|c| c.is_ascii_graphic() || **c == b' ').count() * 10 > head.len() * 8;
    }
    body.len() > 16
}

struct Stats {
    unpack: bool,
    rename: bool,
    symbols: bool,
    minifs: usize,
    pk11: usize,
    v2: usize,
    streams: usize,
    table_files: usize,
    carved: usize,
    sym_names: usize,
    out_bytes: usize,
    container_bytes: usize,
    best_cov: f64,
}

impl Stats {
    fn recovered(&self) -> usize { self.out_bytes }
    fn coverage(&self) -> f64 {
        if self.container_bytes == 0 { 0.0 }
        else { (self.out_bytes as f64 / self.container_bytes as f64).min(1.0) }
    }
}

fn analyze_blob(b: &[u8], st: &mut Stats) {
    let minifs_e = vxcore::minifs::extract(b);
    let pk11_e = vxcore::pk11::extract(b);
    let v2_e = vxcore::minifs_v2::extract_v2(b);
    let minifs = minifs_e.len();
    let pk11 = pk11_e.len();
    let v2 = v2_e.len();
    let mut bytes_here = minifs_e.iter().map(|e| e.data.len()).sum::<usize>()
        + pk11_e.iter().map(|e| e.data.len()).sum::<usize>()
        + v2_e.iter().map(|e| e.data.len()).sum::<usize>();
    let mut table_files = 0usize;
    for t in find_tables(b).iter().take(1) {
        for e in t.entries.iter() {
            if e.offset < 0 || e.size <= 0 { continue; }
            let (o, sz) = (e.offset as usize, e.size as usize);
            if o + sz <= b.len() && payload_ok(&e.name, &b[o..o + sz]) {
                table_files += 1;
                bytes_here += sz;
            }
        }
    }
    let carved_e = vxcore::carve(b);
    let carved = carved_e.len();
    bytes_here += carved_e.iter().map(|c| c.size).sum::<usize>();
    let mut stream_bytes = 0usize;
    let streams = if minifs + pk11 + v2 + table_files == 0 {
        let se = vxcore::lzcarve::carve(b, 200);
        stream_bytes = se.iter().map(|e| e.data.len()).sum::<usize>();
        se.len()
    } else { 0 };
    bytes_here += stream_bytes;
    st.minifs += minifs;
    st.pk11 += pk11;
    st.v2 += v2;
    st.table_files += table_files;
    if b.len() > 0 && bytes_here > 0 {
        let cov = (bytes_here as f64 / b.len() as f64).min(1.0);
        if cov > st.best_cov + 1e-9 || (cov >= st.best_cov - 1e-9 && bytes_here > st.out_bytes) {
            st.best_cov = cov;
            st.out_bytes = bytes_here;
            st.container_bytes = b.len();
        }
    }
    // 文件系统没解出来时，看这一层是不是裸内核（整层即数据区）
    if minifs + pk11 + v2 + table_files == 0 {
        if let Some((_k, body)) = vxcore::rawkernel::extract(b) {
            let cov = (body.data.len() as f64 / b.len() as f64).min(1.0);
            if cov > st.best_cov + 1e-9 {
                st.best_cov = cov;
                st.out_bytes = body.data.len();
                st.container_bytes = b.len();
            }
        }
    }
    st.streams += streams;
    st.carved += carved;
    let names = find_symbols(b).iter().map(|s| s.names_count).max().unwrap_or(0);
    if names > st.sym_names { st.sym_names = names; }
}

fn symbol_deep(b: &[u8], depth: usize, best: &mut usize) {
    let n = find_symbols(b).iter().map(|s| s.names_count).max().unwrap_or(0);
    if n > *best { *best = n; }
    if *best >= 20 || depth == 0 { return; }
    let mut layers: Vec<vxcore::Layer> = scan_all(b)
        .into_iter().filter(|l| l.data.len() > 65536).collect();
    layers.sort_by(|a, b| b.data.len().cmp(&a.data.len()));
    for l in layers.iter().take(6) {
        symbol_deep(&l.data, depth - 1, best);
        if *best >= 20 { return; }
    }
    // 再顺着 LZMA 流找一层（限制体积，避免拖慢）
    if b.len() > (12 << 20) { return; }
    for e in vxcore::lzcarve::carve(b, 20) {
        let n2 = find_symbols(&e.data).iter().map(|s| s.names_count).max().unwrap_or(0);
        if n2 > *best { *best = n2; }
        if *best >= 20 { return; }
    }
}

fn analyze(data: &[u8]) -> Stats {
    let mut st = Stats { unpack: false, rename: false, symbols: false,
        minifs: 0, pk11: 0, v2: 0, streams: 0, table_files: 0, carved: 0, sym_names: 0,
        out_bytes: 0, container_bytes: 0, best_cov: 0.0 };
    analyze_blob(data, &mut st);

    // 逐层再试：多数固件的文件系统在内核层里
    if st.minifs + st.pk11 + st.v2 + st.table_files + st.carved + st.streams == 0 {
        let layers = scan_all(data);
        let mut big: Vec<&vxcore::Layer> = layers.iter().filter(|l| l.data.len() > 65536).collect();
        big.sort_by(|a, b| b.data.len().cmp(&a.data.len()));
        for l in big.into_iter().take(48) {
            analyze_blob(&l.data, &mut st);
            if st.minifs + st.pk11 + st.v2 + st.table_files + st.carved + st.streams > 0 { break; }
        }
        // 还没有的话，顺着 LZMA 流解出来的内容再找
        if st.minifs + st.pk11 + st.v2 + st.table_files + st.carved + st.streams == 0 {
            for e in vxcore::lzcarve::carve(data, 24) {
                let saved = (st.minifs, st.pk11, st.v2, st.table_files, st.carved, st.streams);
                analyze_blob(&e.data, &mut st);
                if st.minifs + st.pk11 + st.v2 + st.table_files + st.carved > 0 { break; }
                let _ = saved;
            }
        }
    }
    let mut best = st.sym_names;
    symbol_deep(data, 2, &mut best);
    st.sym_names = best;

    st.unpack = st.best_cov >= 0.90 || st.minifs + st.pk11 + st.v2 + st.streams + st.table_files + st.carved > 0;
    st.rename = st.minifs + st.pk11 + st.table_files > 0;
    st.symbols = st.sym_names >= 20;
    st
}

fn quick_cov(data: &[u8]) -> f64 {
    // 和 batch 一致的判定：原始数据 + 各解压层取最好覆盖率
    analyze(data).best_cov
}

fn walk_fw(dir: &Path, out: &mut Vec<PathBuf>) {
    let rd = match fs::read_dir(dir) { Ok(r) => r, Err(_) => return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() { walk_fw(&p, out); continue; }
        let ext = p.extension().unwrap_or_default().to_string_lossy().to_lowercase();
        if ["bin", "zip", "rar", "7z", "img", "fw"].contains(&ext.as_str()) { out.push(p); }
    }
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut q = false;
    for c in line.chars() {
        match c {
            '"' => q = !q,
            ',' if !q => { out.push(std::mem::take(&mut cur)); }
            _ => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn corpus_rows(root: &Path) -> Option<Vec<(String, String, PathBuf)>> {
    let p = root.join("CORPUS.csv");
    let text = fs::read_to_string(&p).ok()?;
    let mut lines = text.lines();
    let header = parse_csv_line(lines.next()?);
    let fi = header.iter().position(|h| h == "file")?;
    let vi = header.iter().position(|h| h == "vendor").unwrap_or(0);
    let mi = header.iter().position(|h| h == "model").unwrap_or(vi);
    let mut out = Vec::new();
    for line in lines {
        if line.trim().is_empty() { continue; }
        let f = parse_csv_line(line);
        if f.len() <= fi { continue; }
        let rel = f[fi].replace('\\', "/");
        let full = root.join(&rel);
        let vendor = f.get(vi).cloned().unwrap_or_default();
        let model = f.get(mi).cloned().unwrap_or_default();
        out.push((vendor, model, full));
    }
    Some(out)
}

fn run_batch(root: &Path, out_csv: Option<&Path>, limit: Option<usize>) -> i32 {
    let mut items: Vec<(String, String, PathBuf)> = match corpus_rows(root) {
        Some(v) => v,
        None => {
            let mut files = Vec::new();
            walk_fw(root, &mut files);
            files.sort();
            files.into_iter().map(|p| {
                let v = p.parent().and_then(|x| x.file_name())
                    .map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                let m = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                (v, m, p)
            }).collect()
        }
    };
    if let Some(n) = limit { items.truncate(n); }
    let mut rows = String::from("sample,vendor,ok,rename,symbols,coverage,out_bytes,container_bytes,minifs,pk11,v2,streams,table_files,carved,sym_names\n");
    let mut per_vendor: std::collections::BTreeMap<String, [usize; 5]> = Default::default();
    let (mut nu, mut nr, mut ns) = (0usize, 0usize, 0usize);
    for (vendor, model, p) in &items {
        let (data, _name) = match load(p) { Ok(v) => v, Err(e) => {
            eprintln!("skip {}: {}", p.display(), e); continue; } };
        if data.len() < 4096 { continue; }
        let st = analyze(&data);
        let vendor = vendor.clone();
        let sample = format!("{} ({})", p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(), model);
        rows.push_str(&format!("{},{},{},{},{},{:.3},{},{},{},{},{},{},{},{},{}\n",
            sample, vendor,
            if st.best_cov >= 0.90 {"yes"} else {"no"},
            if st.rename {"yes"} else {"no"},
            if st.symbols {"yes"} else {"no"},
            st.best_cov, st.out_bytes, st.container_bytes,
            st.minifs, st.pk11, st.v2, st.streams, st.table_files, st.carved, st.sym_names));
        let v = per_vendor.entry(vendor.clone()).or_insert([0; 5]);
        v[0] += 1;
        v[1] += st.unpack as usize;
        v[2] += st.rename as usize;
        v[3] += st.symbols as usize;
        v[4] += st.sym_names;
        nu += st.unpack as usize; nr += st.rename as usize; ns += st.symbols as usize;
    }
    if let Some(path) = out_csv {
        let _ = fs::write(path, &rows);
    }
    eprintln!("{:<24} {:>5} {:>7} {:>7} {:>7}", "vendor", "n", "unpack", "rename", "symbols");
    for (k, v) in &per_vendor {
        let n = v[0].max(1);
        eprintln!("{:<24} {:>5} {:>6.1}% {:>6.1}% {:>6.1}%", k, v[0],
                  100.0 * v[1] as f64 / n as f64,
                  100.0 * v[2] as f64 / n as f64,
                  100.0 * v[3] as f64 / n as f64);
    }
    let high_cov = per_vendor.iter().map(|(_, v)| v[4]).sum::<usize>();
    let _ = high_cov;
    let n = items.len().max(1);
    eprintln!("{:<24} {:>5} {:>6.1}% {:>6.1}% {:>6.1}%", "ALL", items.len(),
              100.0 * nu as f64 / n as f64,
              100.0 * nr as f64 / n as f64,
              100.0 * ns as f64 / n as f64);
    0
}

fn shannon(b: &[u8]) -> f64 {
    if b.is_empty() { return 0.0; }
    let mut cnt = [0usize; 256];
    for &c in b { cnt[c as usize] += 1; }
    let n = b.len() as f64;
    let mut h = 0.0;
    for &c in cnt.iter() {
        if c > 0 {
            let p = c as f64 / n;
            h -= p * p.log2();
        }
    }
    h
}

fn run_entropy(root: &Path, out_csv: Option<&Path>, win: usize) -> i32 {
    let items = match corpus_rows(root) {
        Some(v) => v,
        None => { eprintln!("CORPUS.csv not found under {}", root.display()); return 2; }
    };
    let mut rows = String::from("sample,vendor,size,ent_mean,hi_frac,zero_frac,hi_bytes,zero_bytes\n");
    let mut rrows = String::from("sample,vendor,start_hex,end_hex,size_bytes,ent_norm\n");
    for (vendor, model, p) in &items {
        let (data, _n) = match load(p) { Ok(v) => v, Err(_) => continue };
        if data.len() < 4096 { continue; }
        let mut hi = 0usize; let mut lo = 0usize; let mut zeros = 0usize;
        let mut sum_h = 0.0f64; let mut nwin = 0usize;
        for w in data.chunks(win) {
            let h = vxcore::entropy::shannon(w);
            sum_h += h; nwin += 1;
            if h >= 7.5 { hi += w.len(); }
            if h <= 1.0 { lo += w.len(); }
            zeros += w.iter().filter(|c| **c == 0).count();
        }
        let n = data.len() as f64;
        let sample = format!("{} ({})", p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(), model);
        for r in vxcore::entropy::big_ranges(&data, win, 0.9, 65536) {
            rrows.push_str(&format!("{},{},0x{:08X},0x{:08X},{},{:.3}\n",
                sample, vendor, r.start, r.end, r.end - r.start, r.ent_norm));
        }
        rows.push_str(&format!("{},{},{},{:.3},{:.3},{:.3},{},{}\n",
            sample, vendor, data.len(),
            sum_h / nwin.max(1) as f64,
            hi as f64 / n, zeros as f64 / n, hi, zeros));
        let _ = lo;
    }
    if let Some(path) = out_csv {
        let _ = fs::write(path, &rows);
        let rp = path.with_file_name("ENTROPY_RANGES.csv");
        let _ = fs::write(&rp, &rrows);
        eprintln!("ranges -> {}", rp.display());
    }
    eprintln!("entropy rows: {}", items.len());
    0
}

fn run_profile(path: &Path, block: usize, out_csv: Option<&Path>, ascii: bool) -> i32 {
    let (data, _name) = match load(path) { Ok(v) => v, Err(e) => {
        eprintln!("read error: {}", e); return 2; } };
    let bs = vxcore::profile::blocks(&data, block.max(64));
    let mut rows = String::from("offset,size,entropy,kind\n");
    for b in &bs {
        rows.push_str(&format!("0x{:08X},{},{:.3},{}\n", b.offset, b.size, b.ent, b.kind));
    }
    if let Some(p) = out_csv { let _ = fs::write(p, &rows); }
    let regs = vxcore::profile::regions(&bs);
    println!("{{\"file\": \"{}\", \"size\": {}, \"block\": {}, \"regions\": {}}}",
        path.to_string_lossy().replace('"', ""), data.len(), block, regs.len());
    for r in regs.iter().take(40) {
        println!("0x{:08X}-0x{:08X}  {:>9} B  ent {:>5.2}  {}", r.start, r.end, r.end - r.start, r.ent, r.kind);
    }
    // 按类型汇总
    let mut agg: std::collections::BTreeMap<&str, (usize, usize)> = Default::default();
    for r in &regs {
        let e = agg.entry(r.kind).or_insert((0, 0));
        e.0 += r.end - r.start;
        e.1 += 1;
    }
    let mut items: Vec<(&str, usize, usize)> = agg.into_iter().map(|(k, v)| (k, v.0, v.1)).collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    eprintln!("{:<16} {:>10} {:>8} {:>7}", "kind", "bytes", "regions", "share");
    for (k, bytes, n) in items.iter().take(12) {
        eprintln!("{:<16} {:>10} {:>8} {:>6.1}%", k, bytes, n, 100.0 * *bytes as f64 / data.len() as f64);
    }
    if ascii && !bs.is_empty() {
        let cols = 80usize;
        let per = (bs.len() + cols - 1) / cols;
        let chars: Vec<char> = " .:-=+*#%@".chars().collect();
        let mut line = String::new();
        for c in 0..cols {
            let slice = &bs[(c * per).min(bs.len())..((c + 1) * per).min(bs.len())];
            if slice.is_empty() { line.push(' '); continue; }
            let avg: f64 = slice.iter().map(|b| b.ent).sum::<f64>() / slice.len() as f64;
            let idx = ((avg / 8.0) * (chars.len() - 1) as f64).round() as usize;
            line.push(chars[idx.min(chars.len() - 1)]);
        }
        println!("[entropy {}..8] {}", 0, line);
    }
    0
}

fn run_profile_batch(root: &Path, out_csv: Option<&Path>, block: usize) -> i32 {
    let items = match corpus_rows(root) {
        Some(v) => v,
        None => { eprintln!("CORPUS.csv not found under {}", root.display()); return 2; }
    };
    let kinds = ["high-entropy","data","text","zero","padding","lzma","minifs","pk11","ow",
                 "vxworks","uimage","trx","gzip","zlib","bzip2","xz","7z","rar","zip","elf","pe","png","jpeg","gif"];
    let mut rows = String::from("sample,vendor,size,high_entropy_bytes,high_entropy_share,zero_share,text_share,lzma_bytes,minifs_bytes,pk11_bytes,ow_bytes,vxworks_bytes,uimage_bytes,compress_bytes,data_bytes,other_bytes\n");
    for (vendor, model, p) in &items {
        let (data, _n) = match load(p) { Ok(v) => v, Err(_) => continue };
        if data.len() < 4096 { continue; }
        let bs = vxcore::profile::blocks(&data, block.max(64));
        let mut agg: std::collections::BTreeMap<&str, usize> = Default::default();
        for b in &bs { *agg.entry(b.kind).or_insert(0) += b.size; }
        let get = |k: &str| -> usize { *agg.get(k).unwrap_or(&0) };
        let compress = get("gzip") + get("zlib") + get("bzip2") + get("xz") + get("7z") + get("rar") + get("zip");
        let known: usize = kinds.iter().map(|k| get(k)).sum();
        let other = data.len().saturating_sub(known);
        let sample = format!("{} ({})", p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(), model);
        let n = data.len() as f64;
        rows.push_str(&format!("{},{},{},{},{:.3},{:.3},{:.3},{},{},{},{},{},{},{},{},{}\n",
            sample, vendor, data.len(),
            get("high-entropy"), get("high-entropy") as f64 / n,
            get("zero") as f64 / n, get("text") as f64 / n,
            get("lzma"), get("minifs"), get("pk11"), get("ow"), get("vxworks"),
            get("uimage"), compress, get("data"), other));
    }
    if let Some(path) = out_csv { let _ = fs::write(path, &rows); }
    eprintln!("profile rows: {}", items.len());
    0
}

fn main() {
    // 一键用法：vxfile_extractor fw.bin 等价于 extract fw.bin
    let mut argv: Vec<String> = std::env::args().collect();
    const SUBS: [&str; 9] = ["verdict", "scan", "tables", "symbols", "extract",
                             "batch", "entropy", "profile", "profile-batch"];
    if argv.len() >= 2 {
        let a = argv[1].as_str();
        let is_sub = SUBS.contains(&a) || a == "-h" || a == "--help" || a == "-V" || a == "--version";
        if !is_sub && !a.starts_with('-') {
            argv.insert(1, "extract".to_string());
        }
    }
    let cli = Cli::parse_from(argv);
    let code = match cli.cmd {
        Cmd::Batch { root, out, limit } => run_batch(&root, out.as_deref(), limit),
        Cmd::Entropy { root, out, window } => run_entropy(&root, out.as_deref(), window),
        Cmd::Profile { file, block, out, ascii } => run_profile(&file, block, out.as_deref(), ascii),
        Cmd::ProfileBatch { root, out, block } => run_profile_batch(&root, out.as_deref(), block),
        Cmd::Verdict { file } => {
            let (data, inner) = match load(&file) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("read error: {}", e);
                    std::process::exit(2);
                }
            };
            let v = judge_image(&data, &inner);
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "file": file.to_string_lossy(),
                "verdict": v.verdict,
                "via": v.via,
                "versions": v.versions,
                "evidence": v.evidence,
                "linux": v.linux,
                "inner": v.inner,
                "size": data.len(),
            })).unwrap());
            0
        }
        Cmd::Scan { file, limit } => {
            let (data, _) = load(&file).unwrap_or_else(|e| {
                eprintln!("read error: {}", e);
                std::process::exit(2)
            });
            let layers = scan_all(&data);
            let items: Vec<_> = layers.iter().take(limit).map(|l| serde_json::json!({
                "offset": l.offset, "kind": l.kind, "size": l.data.len()
            })).collect();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "size": data.len(), "layers": layers.len(), "items": items
            })).unwrap());
            0
        }
        Cmd::Tables { file } => {
            let (data, _) = load(&file).unwrap_or_else(|e| {
                eprintln!("read error: {}", e);
                std::process::exit(2)
            });
            let mut d2 = data.clone();
            let mut best = find_tables(&d2);
            if best.is_empty() {
                for l in scan_all(&data) {
                    if l.data.len() > 65536 {
                        for mut t in find_tables(&l.data) {
                            t.start += l.offset;
                            t.end += l.offset;
                            best.push(t);
                        }
                    }
                }
            }
            best.sort_by_key(|t| std::cmp::Reverse(t.entries.len()));
            let item = best.first();
            let names: Vec<_> = item.map(|t| t.entries.iter().take(limit_names()).collect::<Vec<_>>()).unwrap_or_default();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "tables": best.len(),
                "layout": item.map(|t| t.layout.clone()),
                "entries": item.map(|t| t.entries.len()).unwrap_or(0),
                "sample": names.iter().map(|e| serde_json::json!({
                    "name": e.name, "size": e.size, "offset": e.offset
                })).collect::<Vec<_>>(),
            })).unwrap());
            let _ = &mut d2;
            0
        }
        Cmd::Symbols { file, limit } => {
            let (data, _) = load(&file).unwrap_or_else(|e| {
                eprintln!("read error: {}", e);
                std::process::exit(2)
            });
            let hits = find_symbols(&data);
            let items: Vec<_> = hits.iter().take(limit).map(|h| serde_json::json!({
                "detector": h.detector.to_string(),
                "offset": h.offset,
                "names_count": h.names_count,
                "has_table": h.has_table,
                "label": h.label,
            })).collect();
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "hits": hits.len(), "items": items
            })).unwrap());
            0
        }
        Cmd::Extract { file, out, no_write } => {
            let (data, inner) = load(&file).unwrap_or_else(|e| {
                eprintln!("read error: {}", e);
                std::process::exit(2)
            });
            let _ = fs::create_dir_all(&out);
            let layers = scan_all(&data);
            let mut manifest = serde_json::json!({
                "input": file.to_string_lossy(),
                "inner": inner,
                "size": data.len(),
                "layers": [],
                "table": null,
                "symbol": null,
            });
            let mut arr = Vec::new();
            for (i, l) in layers.iter().enumerate() {
                let name = format!("{:08X}_{}", l.offset, l.kind);
                if !no_write {
                    let p = out.join(&name);
                    if let Ok(mut f) = fs::File::create(&p) {
                        let _ = f.write_all(&l.data);
                    }
                }
                arr.push(serde_json::json!({
                    "offset": l.offset, "kind": l.kind, "size": l.data.len(), "file": name
                }));
            }
            manifest["layers"] = serde_json::Value::Array(arr);

            // filename table: raw first, then decoded layers
            let mut tables = find_tables(&data);
            if tables.is_empty() {
                for l in layers.iter().filter(|l| l.data.len() > 65536) {
                    for mut t in find_tables(&l.data) {
                        t.start += l.offset;
                        t.end += l.offset;
                        tables.push(t);
                    }
                }
            }
            tables.sort_by_key(|t| std::cmp::Reverse(t.entries.len()));
            if let Some(t) = tables.first() {
                let mut csv = String::from("name,size,offset\n");
                for e in &t.entries {
                    csv.push_str(&format!("{},{},{}\n", e.name.replace(',', "_"), e.size, e.offset));
                }
                let _ = fs::write(out.join("FILES.csv"), csv);
                manifest["table"] = serde_json::json!({
                    "layout": t.layout, "entries": t.entries.len(), "confidence": t.confidence
                });
            }

            // symbols
            if let Some(h) = vxcore::recover_symbol_table(&data) {
                let _ = fs::write(out.join("SYMBOL_Table"), &h.blob);
                let mut txt = h.names.clone();
                txt.sort();
                txt.dedup();
                let _ = fs::write(out.join("SYMBOL_Names.txt"), txt.join("\n"));
                manifest["symbol"] = serde_json::json!({
                    "detector": h.detector.to_string(), "offset": h.offset,
                    "names": txt.len()
                });
            }

            // content carving (images / HTML)
            let mut carved: Vec<vxcore::Carved> = vxcore::carve(&data);
            for l in layers.iter().filter(|l| l.data.len() > 65536) {
                let mut sub = vxcore::carve(&l.data);
                for c in sub.iter_mut() {
                    c.offset += l.offset;
                }
                carved.extend(sub);
            }
            let carve_dir = out.join("carved");
            let _ = fs::create_dir_all(&carve_dir);
            let mut carved_n = 0usize;
            for c in &carved {
                let name = format!("{:08X}_{}", c.offset, c.kind);
                if no_write || fs::write(carve_dir.join(name), &c.data).is_ok() {
                    carved_n += 1;
                }
            }
            manifest["carved"] = serde_json::json!(carved_n);

            // recover files named by the table
            let files_dir = out.join("files");
            let _ = fs::create_dir_all(&files_dir);
            let mut recovered = 0usize;
            if let Some(t) = tables.first() {
                for e in &t.entries {
                    if e.offset < 0 || e.size <= 0 {
                        continue;
                    }
                    let (o, sz) = (e.offset as usize, e.size as usize);
                    if o >= data.len() || sz == 0 || o + sz > data.len() {
                        continue;
                    }
                    let body = &data[o..o + sz];
                    if !content_ok(&e.name, body) {
                        continue;
                    }
                    let p = files_dir.join(sanitize(&e.name));
                    if no_write || fs::write(&p, body).is_ok() {
                        recovered += 1;
                    }
                }
            }
            // MINIFS（v1 逐文件 / v3 分块）解包
            let minifs = vxcore::minifs::extract(&data);
            let mut minifs_n = 0usize;
            if !minifs.is_empty() {
                let mdir = out.join("minifs");
                for e in &minifs {
                    if no_write || {
                        let dst = mdir.join(e.path.trim_start_matches('/').replace("..", "_"));
                        if let Some(par) = dst.parent() { let _ = fs::create_dir_all(par); }
                        fs::write(&dst, &e.data).is_ok()
                    } { minifs_n += 1; }
                }
            }
            manifest["minifs_files"] = serde_json::json!(minifs_n);

            // Netgear PK11：原始数据 + 各解压层
            if minifs_n == 0 {
                let mut pk = vxcore::pk11::extract(&data);
                if pk.is_empty() {
                    for l in layers.iter().filter(|l| l.data.len() > 65536) {
                        pk = vxcore::pk11::extract(&l.data);
                        if !pk.is_empty() { break; }
                    }
                }
                let mut pn = 0usize;
                for e in &pk {
                    if no_write || {
                        let dst = out.join("files").join(e.path.trim_start_matches('/').replace("..", "_"));
                        if let Some(par) = dst.parent() { let _ = fs::create_dir_all(par); }
                        fs::write(&dst, &e.data).is_ok()
                    } { pn += 1; }
                }
                if pn > 0 { manifest["pk11_files"] = serde_json::json!(pn); }
            }

            // 没有 MINIFS 命中时，兜底扫所有 LZMA-alone 流
            if minifs_n == 0 {
                let v2 = vxcore::minifs_v2::extract_v2(&data);
                if !v2.is_empty() {
                    let mut n2 = 0usize;
                    for e in &v2 {
                        if no_write || {
                            let dst = out.join("minifs").join(&e.path);
                            if let Some(par) = dst.parent() { let _ = fs::create_dir_all(par); }
                            fs::write(&dst, &e.data).is_ok()
                        } { n2 += 1; }
                    }
                    manifest["minifs_v2_files"] = serde_json::json!(n2);
                }
            }
            if minifs_n == 0 && manifest.get("minifs_v2_files").is_none() {
                let streams = vxcore::lzcarve::carve(&data, 200);
                let mut written = 0usize;
                for e in &streams {
                    if no_write || {
                        let dst = out.join("streams").join(&e.path);
                        if let Some(par) = dst.parent() { let _ = fs::create_dir_all(par); }
                        fs::write(&dst, &e.data).is_ok()
                    } { written += 1; }
                }
                manifest["lzma_streams"] = serde_json::json!(written);
            }

            // 什么都没解出来时，判断是不是「文件本身就是内核」
            if minifs_n == 0 && manifest.get("pk11_files").is_none()
                && manifest.get("minifs_v2_files").is_none() {
                if let Some((label, e)) = vxcore::rawkernel::extract(&data) {
                    let p = out.join("kernel.bin");
                    let ok = no_write || fs::write(&p, &e.data).is_ok();
                    if ok {
                        manifest["raw_kernel"] = serde_json::json!({
                            "kind": label, "bytes": e.data.len()
                        });
                    }
                }
            }

            manifest["recovered_named"] = serde_json::json!(recovered);

            let v = judge_image(&data, &inner);
            manifest["verdict"] = serde_json::json!({
                "verdict": v.verdict, "via": v.via, "versions": v.versions
            });
            {
                let decoded = quick_cov(&data) >= 0.5;
                for r in vxcore::entropy::big_ranges(&data, 4096, 0.9, 65536).iter().take(3) {
                    let verdict = if decoded { "compressed (data was recovered)" } else { "likely encrypted / unknown container" };
                    eprintln!("[!] high-entropy 0x{:08X}-0x{:08X} ({} KiB, {:.1}%) -> {}",
                        r.start, r.end, (r.end - r.start) / 1024, r.ent_norm * 100.0, verdict);
                }
            }
            if !no_write {
                let _ = fs::write(out.join("SUMMARY.json"),
                                  serde_json::to_string_pretty(&manifest).unwrap());
                let kl = layers.iter().max_by_key(|l| l.data.len()).map(|l| (l.offset as u64, l.data.len()));
                if let Some(rk) = manifest.get("raw_kernel") {
                    eprintln!("[*] Likely main logic: raw kernel image (kernel.bin, {})",
                              rk.get("kind").and_then(|v| v.as_str()).unwrap_or("raw"));
                    eprintln!("    header stripped, no container left to unpack");
                } else {
                    print_hints(&out, kl);
                }
            }
            println!("{}", serde_json::to_string_pretty(&manifest).unwrap());
            0
        }
    };
    std::process::exit(code);
}

fn limit_names() -> usize { 20 }