//! 文件名 / 偏移表探测器 —— `tools/vxtable.py` 的 Rust 移植。
//!
//! 对外接口只有 [`find_tables`] / [`Table`] / [`Entry`]，语义与 Python 版逐条对齐：
//!
//! 1. `minifs-fixed`  `MINIFS` + 0x10 头，magic+0x1c 处首条记录
//!    `[size:4BE][offset:4BE][name:0x50]`，stride 0x58；name 字段长度兼容
//!    0x50 / 0x48 / 0x40 / 0x30 / 0x60。
//! 2. `minifs-pool`   `MINIFS` + `[ver=3][count][data_size][name_block_len]`，
//!    名字池（NUL 分隔，路径与文件名共用）后面是 `count` 条 20 字节记录
//!    `[path_off][name_off][chunk][in_chunk][size]`；数据按 chunk 压缩，raw
//!    偏移未知，所以 offset 记为 -1。
//! 3. `fixed-stride`  任意定长记录 `[name:L][size][offset]`：stride / L / 端序
//!    全部从数据推断（Linksys TROC stride 0x18、WRT54G2 stride 0x4C、
//!    `[name:0x28][size:4BE][off:4BE]` stride 0x30 等）。
//! 4. `offsets-only`  `[name:L][offset:4]`：只有偏移没有 size（offset 严格递增）。
//! 5. `nameblock`     连续 NUL 分隔的名字块 + 等长的 4 字节偏移数组（数量可能写在
//!    名字块前 4 / 12 字节）。
//! 6. `lenprefix`     长度前缀记录 `[len][name][size][offset]` 或
//!    `[size][offset][len][name]`。
//!
//! 与 Python 版唯一的表示差异：Python 的 `Table.end` 对 `lenprefix` 是 `None`，
//! 而本模块的 `end` 是 `usize`，因此 `lenprefix` 取「最后一条成功记录之后的
//! 偏移」。其余布局的 `end` 与 Python 完全一致。
//!
//! 判真原则（防假阳性）也与 Python 一致：名字必须是可打印的类文件名（含已知
//! 扩展名；定长表允许字段被填满而截断的名字，但比例不超过 35%）；定长表 /
//! 长度前缀表要求 offset 单调且 `offset[i+1] ≈ offset[i]+size[i]`（允许 0x1000
//! 以内对齐），命中率 >= 85%；名字块要求偏移数组单调递增；空槽跳过不计入。

use memchr::{memchr, memmem};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;

// --------------------------------------------------------------------------
// 常量
// --------------------------------------------------------------------------

/// 单表条目上限，防解析跑飞。
const MAX_ENTRIES: usize = 4000;
/// size / offset 上限（64MB），超过视为跑偏。
const MAX_VALUE: u32 = 0x400_0000;
/// 允许的对齐填充（offset[i+1] - offset[i] - size[i]）。
const CUM_PAD: i64 = 0x1000;
/// 截断名字占比上限。
const MAX_CUT_RATIO: f64 = 0.35;
/// 名字 run 聚类的最大间隔。
const RUN_MAX_GAP: usize = 0x200;
/// 名字 run 的最小条数。
const RUN_MIN_HITS: usize = 3;

/// 已知扩展名，顺序与 vxtable.py 的 `_EXT` 完全一致。
///
/// 顺序有意义：Python 的正则是 `\.(?:htm|html|...)`，正则选择是「从左到右第一个
/// 能匹配的前缀」，所以 `x.json` 的 `NAME_HIT` 实际只匹配到 `x.js`，`index.html`
/// 只匹配到 `index.htm`。这里用有序数组 + 前缀匹配复刻同一行为。
const EXT: [&[u8]; 39] = [
    b"htm", b"html", b"shtml", b"css", b"js", b"xml", b"png", b"gif", b"jpg", b"jpeg", b"bmp",
    b"ico", b"swf", b"bin", b"dat", b"cfg", b"ini", b"txt", b"pem", b"cer", b"crt", b"key", b"asp",
    b"cgi", b"json", b"db", b"img", b"trx", b"rom", b"chk", b"pdf", b"zip", b"gz", b"tar", b"so",
    b"o", b"h", b"ttf", b"wav",
];

// --------------------------------------------------------------------------
// 数据结构
// --------------------------------------------------------------------------

/// 一条「文件名 + size + offset」。size / offset 未知用 -1。
#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub size: i64,
    pub offset: i64,
}

/// 一张「文件名表」。
#[derive(Debug, Clone)]
pub struct Table {
    /// `minifs-fixed` / `minifs-pool` / `fixed-stride` / `offsets-only` /
    /// `nameblock` / `lenprefix`
    pub layout: String,
    pub entries: Vec<Entry>,
    pub confidence: f64,
    pub start: usize,
    pub end: usize,
}

impl Table {
    #[inline]
    fn new(layout: &str, entries: Vec<Entry>, confidence: f64, start: usize, end: usize) -> Table {
        Table {
            layout: layout.to_string(),
            entries,
            confidence,
            start,
            end,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

/// 端序。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endian {
    Be,
    Le,
}

/// 名字字段的分类结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameKind {
    /// 完整文件名（含已知扩展名）。
    Full,
    /// 名字把定长字段撑满，扩展名被截断（如 13 字节字段里的 `lastpassword.`）。
    Cut,
}

/// 长度前缀记录的两种排布。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LenMode {
    /// `[len][name][pad:suffix][f1:4][f2:4]`
    LenFirst,
    /// `[f1:4][f2:4][len][name]`
    LenLast,
}

// --------------------------------------------------------------------------
// 字节读取（Python struct.unpack 的边界语义：越界即失败）
// --------------------------------------------------------------------------

#[inline]
fn be32(d: &[u8], p: usize) -> Option<u32> {
    let s = d.get(p..p.wrapping_add(4))?;
    Some(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

#[inline]
fn le32(d: &[u8], p: usize) -> Option<u32> {
    let s = d.get(p..p.wrapping_add(4))?;
    Some(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

#[inline]
fn rd32(d: &[u8], p: usize, e: Endian) -> Option<u32> {
    match e {
        Endian::Be => be32(d, p),
        Endian::Le => le32(d, p),
    }
}

#[inline]
fn rd16(d: &[u8], p: usize, e: Endian) -> Option<u16> {
    let s = d.get(p..p.wrapping_add(2))?;
    Some(match e {
        Endian::Be => u16::from_be_bytes([s[0], s[1]]),
        Endian::Le => u16::from_le_bytes([s[0], s[1]]),
    })
}

/// latin-1 解码：每个字节映射成同码位的字符（与 Python `bytes.decode("latin-1")`
/// 完全等价，不会失败也不会丢字节）。
fn decode_latin1(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len());
    for &c in b {
        s.push(c as char);
    }
    s
}

// --------------------------------------------------------------------------
// 名字判定
// --------------------------------------------------------------------------

/// 名字字段允许的字符类 `[A-Za-z0-9_\-./]`。
#[inline]
fn is_name_char(c: u8) -> bool {
    matches!(c,
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/')
}

/// `re` 的 `\.(?:ext)`：返回第一个能匹配上的扩展名的长度。
#[inline]
fn ext_prefix_len(text: &[u8]) -> Option<usize> {
    for e in EXT.iter() {
        if text.len() >= e.len() && text[..e.len()].eq_ignore_ascii_case(e) {
            return Some(e.len());
        }
    }
    None
}

/// `NAME_FULL`：整个名字是 `class{1,80}\.ext\Z`。
fn is_name_full(nm: &[u8]) -> bool {
    // 贪婪的 class 段会先试最右边的点，逐个向左回溯 → 任意一个点满足即可。
    for (i, &c) in nm.iter().enumerate().skip(1) {
        if c == b'.' {
            let suf = &nm[i + 1..];
            if EXT.iter().any(|e| suf.eq_ignore_ascii_case(e)) {
                return true;
            }
        }
    }
    false
}

/// 返回 `Full` / `Cut` / `None`。
///
/// `field_len` 已知且名字确实填满字段时，扩展名被截断的名字也算命中。
fn name_kind(nm: &[u8], field_len: Option<usize>) -> Option<NameKind> {
    if nm.len() < 4 || nm.len() > 80 {
        return None;
    }
    if !nm.iter().all(|&c| is_name_char(c)) {
        return None;
    }
    if is_name_full(nm) {
        return Some(NameKind::Full);
    }
    if let Some(fl) = field_len {
        if nm.len() >= fl {
            let letters = nm.iter().filter(|c| c.is_ascii_alphabetic()).count();
            if letters >= 4 && (nm.contains(&b'.') || nm.len() >= 8) {
                return Some(NameKind::Cut);
            }
        }
    }
    None
}

// --------------------------------------------------------------------------
// 打分
// --------------------------------------------------------------------------

/// offset 连续性的命中率：`offset[i+1]` 是否约等于 `offset[i] + size[i]`。
fn cum_score(entries: &[Entry]) -> f64 {
    if entries.len() < 2 {
        return 0.0;
    }
    let mut ok = 0usize;
    for w in entries.windows(2) {
        let (size, off) = (w[0].size, w[0].offset);
        let off2 = w[1].offset;
        let gap = off2 - (off + size);
        if gap >= 0 && gap <= CUM_PAD {
            ok += 1;
        }
    }
    ok as f64 / (entries.len() - 1) as f64
}

/// `_name_runs`：把「像文件名的字符串」按位置聚成候选 run，按条数降序（稳定）。
fn name_runs(data: &[u8]) -> Vec<Vec<(usize, usize)>> {
    let hits = name_hits(data);
    let mut runs: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut cur: Vec<(usize, usize)> = Vec::new();
    for &(pos, ln) in hits.iter() {
        if let Some(&(last, _)) = cur.last() {
            if pos - last > RUN_MAX_GAP {
                if cur.len() >= RUN_MIN_HITS {
                    runs.push(std::mem::take(&mut cur));
                } else {
                    cur.clear();
                }
            }
        }
        cur.push((pos, ln));
    }
    if cur.len() >= RUN_MIN_HITS {
        runs.push(cur);
    }
    // Python: runs.sort(key=len, reverse=True)，稳定排序。
    runs.sort_by(|a, b| b.len().cmp(&a.len()));
    runs
}

/// `NAME_HIT.finditer`：非重叠、最左、每个起点取窗口内最右边的「点+扩展名」。
///
/// 返回 `(名字起点, 匹配到的名字长度)`；长度是 Python 里 `m.group()` 的长度
/// （注意 `x.json` 只匹配到 `x.js`）。
fn name_hits(data: &[u8]) -> Vec<(usize, usize)> {
    let n = data.len();
    // 所有「点 + 已知扩展名」的位置（扩展名长度一并记下）。
    let mut dots: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        if data[i] == b'.' {
            if let Some(el) = ext_prefix_len(&data[i + 1..]) {
                dots.push((i, el));
            }
        }
    }
    let mut hits: Vec<(usize, usize)> = Vec::new();
    let mut i = 0usize;
    while i < n {
        if !is_name_char(data[i]) {
            i += 1;
            continue;
        }
        // 名字不可能跨过字符类的边界。
        let run_start = i;
        let mut run_end = i;
        while run_end < n && is_name_char(data[run_end]) {
            run_end += 1;
        }
        let mut j = dots.partition_point(|&(p, _)| p < run_start);
        let hi = dots.partition_point(|&(p, _)| p < run_end);
        let mut s = run_start;
        while s < run_end {
            while j < hi && dots[j].0 <= s {
                j += 1;
            }
            if j >= hi {
                break;
            }
            // class 段最长 80 字节，且点必须还在同一个 run 里。
            let right = (s + 80).min(run_end - 1);
            let mut k = j;
            while k < hi && dots[k].0 <= right {
                k += 1;
            }
            if k > j {
                let (d, el) = dots[k - 1];
                let end = d + 1 + el;
                hits.push((s, end - s));
                s = end;
            } else {
                s += 1;
            }
        }
        i = run_end;
    }
    hits
}

// --------------------------------------------------------------------------
// 1) MINIFS 定长表
// --------------------------------------------------------------------------

/// `[size:4BE][offset:4BE][name:0x50]`，stride 0x58（name 字段长度可不同）。
fn minifs_fixed(data: &[u8], magic_off: usize) -> Option<Table> {
    let n = data.len();
    // (name 字段长度, stride)；第一条记录前 8 字节就是 size/offset。
    let layouts: [(usize, usize); 5] = [
        (0x50, 0x58),
        (0x48, 0x50),
        (0x40, 0x48),
        (0x30, 0x38),
        (0x60, 0x68),
    ];
    // (条目数, cum, entries, name_len, stride)
    let mut best: Option<(usize, f64, Vec<Entry>, usize, usize)> = None;
    for &(name_len, stride) in layouts.iter() {
        let lo = magic_off + 0x18;
        let hi = (magic_off + 0x34).min(n.saturating_sub(8));
        for p0 in lo..hi {
            let size0 = match be32(data, p0 - 8) {
                Some(v) => v,
                None => continue,
            };
            let off0 = match be32(data, p0 - 4) {
                Some(v) => v,
                None => continue,
            };
            if size0 > MAX_VALUE || off0 > MAX_VALUE {
                continue;
            }
            let stop = (p0 + name_len).min(n);
            let field = &data[p0..stop];
            let first = match memchr(0, field) {
                Some(i) => &field[..i],
                None => field,
            };
            if name_kind(first, None).is_none() {
                continue;
            }
            let mut ents = vec![Entry {
                name: decode_latin1(first),
                size: size0 as i64,
                offset: off0 as i64,
            }];
            let mut p = p0 + stride;
            while p + name_len <= n && ents.len() < MAX_ENTRIES {
                let field = &data[p..p + name_len];
                let nm = match memchr(0, field) {
                    Some(i) => &field[..i],
                    None => field,
                };
                if nm.is_empty() {
                    break;
                }
                // nm 一定以 NUL 结尾（否则它就等于整个字段），Python 里那句
                // `data[p+len(nm)] != 0` 永远不会触发，这里省略。
                if name_kind(nm, None).is_none() {
                    break;
                }
                let size = match be32(data, p - 8) {
                    Some(v) => v,
                    None => break,
                };
                let off = match be32(data, p - 4) {
                    Some(v) => v,
                    None => break,
                };
                if size > MAX_VALUE || off > MAX_VALUE {
                    break;
                }
                ents.push(Entry {
                    name: decode_latin1(nm),
                    size: size as i64,
                    offset: off as i64,
                });
                p += stride;
            }
            if ents.len() < 4 {
                continue;
            }
            let mut cum = cum_score(&ents);
            if cum < 0.75 {
                // 首条可能是证书这种「异类」。
                cum = cum_score(&ents[1..]);
            }
            if cum < 0.75 {
                continue;
            }
            let better = match &best {
                None => true,
                Some((bl, bc, _, _, _)) => (ents.len(), cum) > (*bl, *bc),
            };
            if better {
                best = Some((ents.len(), cum, ents, name_len, stride));
            }
        }
    }
    let (_cnt, cum, ents, _name_len, stride) = best?;
    let end = (magic_off + 0x1c + stride * ents.len()).min(n);
    let conf = (0.95 + 0.04 * cum + 0.01 * (ents.len() as f64 / 40.0).min(1.0)).min(1.0);
    Some(Table::new("minifs-fixed", ents, conf, magic_off, end))
}

// --------------------------------------------------------------------------
// 2) MINIFS 名字池 + 20 字节记录
// --------------------------------------------------------------------------

/// `[MINIFS][ver=3][count][data_size][name_block_len]` + 名字池 + 记录表。
fn minifs_pool(data: &[u8], magic_off: usize) -> Option<Table> {
    let n = data.len();
    if magic_off + 0x20 > n {
        return None;
    }
    let ver = be32(data, magic_off + 0x10)?;
    let count = be32(data, magic_off + 0x14)?;
    let _data_size = be32(data, magic_off + 0x18)?;
    if ver != 3 || !(4..=20000).contains(&count) {
        return None;
    }
    let name_len = be32(data, magic_off + 0x1c)? as usize;
    let ns = magic_off + 0x20;
    if name_len == 0 || name_len > n - ns {
        return None;
    }
    let pool = &data[ns..ns + name_len];
    // pool 内每个字符串的起始偏移 → 字符串（不含结尾 NUL）。
    let mut strings: HashMap<u32, &[u8]> = HashMap::new();
    let mut p = 0usize;
    while p < pool.len() && strings.len() < MAX_ENTRIES {
        let e = match memchr(0, &pool[p..]) {
            Some(i) => p + i,
            None => break,
        };
        strings.insert(p as u32, &pool[p..e]);
        p = e + 1;
    }
    if strings.len() < 4 {
        return None;
    }
    let tab = ns + name_len;
    let count_us = count as usize;
    if tab + count_us * 20 > n {
        return None;
    }
    let mut ents: Vec<Entry> = Vec::new();
    let mut resolved = 0usize;
    for i in 0..count_us {
        let base = tab + i * 20;
        let path_off = be32(data, base)?;
        let name_off = be32(data, base + 4)?;
        let _chunk = be32(data, base + 8)?;
        let _in_chunk = be32(data, base + 12)?;
        let size = be32(data, base + 16)?;
        let path = match strings.get(&path_off) {
            Some(v) => *v,
            None => continue,
        };
        let fname = match strings.get(&name_off) {
            Some(v) => *v,
            None => continue,
        };
        resolved += 1;
        if fname.is_empty() {
            continue;
        }
        // path.rstrip("/") + "/" + fname（path 为空时只用 fname）。
        let mut nb: Vec<u8> = Vec::with_capacity(path.len() + 1 + fname.len());
        if !path.is_empty() {
            let mut pl = path;
            while pl.last() == Some(&b'/') {
                pl = &pl[..pl.len() - 1];
            }
            nb.extend_from_slice(pl);
            nb.push(b'/');
        }
        nb.extend_from_slice(fname);
        if name_kind(&nb, None).is_none() {
            continue;
        }
        // 这种布局的数据按 chunk 压缩存放，raw 里没有可直接用的文件偏移；
        // offset 置 -1 表示「索引里有、但 raw 偏移未知」。
        ents.push(Entry {
            name: decode_latin1(&nb),
            size: size as i64,
            offset: -1,
        });
    }
    let need = std::cmp::max(4, (0.8 * count as f64) as usize);
    if resolved < need || ents.len() < 4 {
        return None;
    }
    if (ents.len() as f64) < 0.5 * count as f64 {
        return None;
    }
    let end = tab + count_us * 20;
    let ratio_resolved = resolved as f64 / count as f64;
    let conf = (0.88 + 0.06 * ratio_resolved + 0.05 * (ents.len() as f64 / 50.0).min(1.0)).min(0.99);
    Some(Table::new("minifs-pool", ents, conf, magic_off, end))
}

// --------------------------------------------------------------------------
// 3) 定长记录（自动推断 name 字段长度 / stride / 端序）
// --------------------------------------------------------------------------

/// 解析一串定长记录，返回 `(entries, cut_count)`。
fn parse_fixed(
    data: &[u8],
    start: usize,
    stride: usize,
    name_len: usize,
    endian: Endian,
) -> (Vec<Entry>, usize) {
    let n = data.len();
    let mut ents: Vec<Entry> = Vec::new();
    let mut cut = 0usize;
    let mut empty = 0usize;
    let mut p = start;
    while p + name_len + 8 <= n {
        let field = &data[p..p + name_len];
        let nm = match memchr(0, field) {
            Some(i) => &field[..i],
            None => field,
        };
        if nm.is_empty() {
            // 空槽（被删条目）允许跳过；非空非名说明解析已跑偏。
            if field.iter().filter(|&&c| c == 0).count() != name_len {
                break;
            }
            empty += 1;
            if empty > MAX_ENTRIES {
                break;
            }
            p += stride;
            continue;
        }
        let kind = match name_kind(nm, Some(name_len)) {
            Some(k) => k,
            None => break,
        };
        if kind == NameKind::Cut {
            cut += 1;
            // 允许截断名字，但别无限跑飞；真正的比例在 detect_fixed 里卡。
            if cut > 4 + (0.5 * ents.len() as f64) as usize {
                break;
            }
        }
        let size = match rd32(data, p + name_len, endian) {
            Some(v) => v,
            None => break,
        };
        let off = match rd32(data, p + name_len + 4, endian) {
            Some(v) => v,
            None => break,
        };
        if size > MAX_VALUE || off > MAX_VALUE {
            break;
        }
        ents.push(Entry {
            name: decode_latin1(nm),
            size: size as i64,
            offset: off as i64,
        });
        if ents.len() >= MAX_ENTRIES {
            break;
        }
        p += stride;
    }
    (ents, cut)
}

/// `[name:name_len][offset:4]`（没有 size；size 记 0）。
fn parse_offsets_only(
    data: &[u8],
    start: usize,
    stride: usize,
    name_len: usize,
    endian: Endian,
) -> Vec<Entry> {
    let n = data.len();
    let mut ents: Vec<Entry> = Vec::new();
    let mut p = start;
    while p + name_len + 4 <= n {
        let field = &data[p..p + name_len];
        let nm = match memchr(0, field) {
            Some(i) => &field[..i],
            None => field,
        };
        if nm.is_empty() {
            if field.iter().filter(|&&c| c == 0).count() != name_len {
                break;
            }
            p += stride;
            continue;
        }
        if name_kind(nm, Some(name_len)).is_none() {
            break;
        }
        let off = match rd32(data, p + name_len, endian) {
            Some(v) => v,
            None => break,
        };
        if off > MAX_VALUE {
            break;
        }
        ents.push(Entry {
            name: decode_latin1(nm),
            size: 0,
            offset: off as i64,
        });
        if ents.len() >= MAX_ENTRIES {
            break;
        }
        p += stride;
    }
    ents
}

/// 候选起点：run 里前 4 条名字的位置，以及按 stride 往前推出来的位置
/// （名字前面可能粘着表头 / 计数，regex 会把它们算进名字）。
fn candidate_starts(run: &[(usize, usize)], stride: usize) -> Vec<usize> {
    let mut set: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    let stride = stride as i64;
    for &(pos, _) in run.iter().take(4) {
        let pos = pos as i64;
        for d in -4i64..=4 {
            let q = pos + d;
            if q >= 0 {
                set.insert(q as usize);
            }
        }
        for k in 1i64..=3 {
            let q = pos - k * stride;
            for d in -2i64..=2 {
                let x = q + d;
                if x >= 0 {
                    set.insert(x as usize);
                }
            }
        }
    }
    set.into_iter().collect()
}

/// `[name + NUL pad + 4 字节 offset]`（只有偏移，没有 size）。
fn detect_offsets_only(data: &[u8], runs: &[Vec<(usize, usize)>]) -> Vec<Table> {
    let n = data.len();
    let mut out: Vec<Table> = Vec::new();
    for run in runs {
        if run.len() < 8 {
            continue;
        }
        let mut gaps: Vec<usize> = Vec::with_capacity(run.len() - 1);
        for i in 0..run.len() - 1 {
            gaps.push(run[i + 1].0 - run[i].0);
        }
        let mut strides: Vec<usize> = gaps
            .into_iter()
            .filter(|&g| (0x10..=0x400).contains(&g))
            .collect();
        strides.sort_unstable();
        strides.dedup();
        strides.truncate(6);
        for stride in strides {
            let name_len_i = stride as i64 - 4;
            if name_len_i < 4 {
                continue;
            }
            let name_len = name_len_i as usize;
            let starts = candidate_starts(run, stride);
            for endian in [Endian::Be, Endian::Le] {
                // (conf, len, s0, entries)
                let mut best: Option<(f64, usize, usize, Vec<Entry>)> = None;
                for &s0 in starts.iter() {
                    let ents = parse_offsets_only(data, s0, stride, name_len, endian);
                    if ents.len() < 8 {
                        continue;
                    }
                    let strict = ents
                        .windows(2)
                        .filter(|w| w[0].offset < w[1].offset)
                        .count() as f64
                        / (ents.len() - 1) as f64;
                    if strict < 0.95 {
                        continue;
                    }
                    if ents[0].offset > 0x10000 {
                        continue;
                    }
                    let cov = ents.len() as f64 / run.len() as f64;
                    if cov < 0.6 {
                        continue;
                    }
                    let conf = (0.50
                        + 0.20 * strict
                        + 0.12 * (ents.len() as f64 / 30.0).min(1.0)
                        + 0.08 * cov.min(1.0))
                    .min(0.87);
                    let key = (conf, ents.len());
                    let better = match &best {
                        None => true,
                        Some((bc, bl, _, _)) => key > (*bc, *bl),
                    };
                    if better {
                        best = Some((conf, ents.len(), s0, ents));
                    }
                }
                if let Some((conf, _, s0, ents)) = best {
                    let end = (s0 + stride * ents.len()).min(n);
                    out.push(Table::new("offsets-only", ents, conf, s0, end));
                }
            }
        }
    }
    out
}

/// 表头里可能写着条目数：看看 start 前 4 / 8 / 12 字节。
fn count_hint(data: &[u8], start: usize) -> Vec<u32> {
    let mut hints = Vec::new();
    for back in [4usize, 8, 12] {
        if start < back {
            continue;
        }
        let at = start - back;
        if at + 4 > data.len() {
            continue;
        }
        let raw = &data[at..at + 4];
        for endian in [Endian::Be, Endian::Le] {
            let v = match endian {
                Endian::Be => u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]),
                Endian::Le => u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]),
            };
            if v > 0 && v <= MAX_ENTRIES as u32 {
                hints.push(v);
            }
        }
    }
    hints
}

/// 定长记录表：stride / name 字段长度 / 端序全部推断。
fn detect_fixed(data: &[u8], runs: &[Vec<(usize, usize)>]) -> Vec<Table> {
    let n = data.len();
    let mut out: Vec<Table> = Vec::new();
    for run in runs {
        if run.len() < 6 {
            continue;
        }
        let mut gaps: Vec<usize> = Vec::with_capacity(run.len() - 1);
        for i in 0..run.len() - 1 {
            gaps.push(run[i + 1].0 - run[i].0);
        }
        let mut strides: Vec<usize> = gaps
            .into_iter()
            .filter(|&g| (0x10..=0x400).contains(&g))
            .collect();
        strides.sort_unstable();
        strides.dedup();
        strides.truncate(6);
        if strides.is_empty() {
            continue;
        }
        for stride in strides {
            for trailer in [8usize, 11, 12, 16, 6, 4] {
                let name_len_i = stride as i64 - trailer as i64;
                if name_len_i < 4 {
                    continue;
                }
                let name_len = name_len_i as usize;
                let starts = candidate_starts(run, stride);
                for endian in [Endian::Be, Endian::Le] {
                    // (conf, len, s0, entries, cut, cum)
                    let mut best: Option<(f64, usize, usize, Vec<Entry>, usize, f64)> = None;
                    for &s0 in starts.iter() {
                        let (ents, cut) = parse_fixed(data, s0, stride, name_len, endian);
                        if ents.len() < 6 {
                            continue;
                        }
                        if !ents.iter().any(|e| e.size != 0) {
                            // size 全是 0：其实是 [name][offset] 布局被按 8 字节
                            // trailer 误读了，交给 detect_offsets_only。
                            continue;
                        }
                        let cum = cum_score(&ents);
                        if cum < 0.85 {
                            continue;
                        }
                        let cov = ents.len() as f64 / run.len() as f64;
                        if cov < 0.6 {
                            continue;
                        }
                        let cut_ratio = cut as f64 / ents.len() as f64;
                        if cut_ratio > MAX_CUT_RATIO {
                            continue;
                        }
                        let conf = (0.45
                            + 0.30 * cum
                            + 0.20 * (ents.len() as f64 / 30.0).min(1.0)
                            + 0.05 * cov.min(1.0)
                            - 0.10 * cut_ratio)
                            .min(0.97);
                        let key = (conf, ents.len());
                        let better = match &best {
                            None => true,
                            Some((bc, bl, _, _, _, _)) => key > (*bc, *bl),
                        };
                        if better {
                            best = Some((conf, ents.len(), s0, ents, cut, cum));
                        }
                    }
                    let (mut conf, _, s0, ents, _cut, _cum) = match best {
                        Some(v) => v,
                        None => continue,
                    };
                    let hints = count_hint(data, s0);
                    if !hints.is_empty() {
                        let limit = (0.05 * ents.len() as f64).max(2.0);
                        let near = hints
                            .iter()
                            .any(|&h| ((h as i64 - ents.len() as i64).abs() as f64) <= limit);
                        if near {
                            conf = (conf + 0.08).min(0.97);
                        }
                    }
                    let end = (s0 + stride * ents.len()).min(n);
                    let conf = conf.max(0.0).min(1.0);
                    out.push(Table::new("fixed-stride", ents, conf, s0, end));
                }
            }
        }
    }
    out
}

// --------------------------------------------------------------------------
// 4) 名字块 + 偏移数组
// --------------------------------------------------------------------------

/// 从 start 起读连续 NUL 分隔的名字，返回 `(names, end_pos)`。
fn read_name_block(data: &[u8], start: usize, max_entries: usize) -> (Vec<String>, usize) {
    let n = data.len();
    let mut names: Vec<String> = Vec::new();
    let mut p = start;
    while p < n && names.len() < max_entries {
        let stop = n.min(p + 0x101);
        let end = match memchr(0, &data[p..stop]) {
            Some(i) => p + i,
            None => break,
        };
        let raw = &data[p..end];
        if name_kind(raw, None).is_none() {
            break;
        }
        names.push(decode_latin1(raw));
        p = end + 1;
    }
    (names, p)
}

/// 名字连续排，后面跟同样数量的 4 字节偏移；数量可能在前 4 / 12 字节。
fn detect_nameblock(data: &[u8], runs: &[Vec<(usize, usize)>]) -> Vec<Table> {
    let n = data.len();
    let mut out: Vec<Table> = Vec::new();
    for run in runs {
        if run.len() < 8 {
            continue;
        }
        for &(pos, _) in run.iter().take(2) {
            let (names, p) = read_name_block(data, pos, MAX_ENTRIES);
            if names.len() < 8 {
                continue;
            }
            let cnt = names.len();
            for align in 0..4usize {
                let q = p + align;
                if q + 4 * cnt > n {
                    continue;
                }
                for endian in [Endian::Be, Endian::Le] {
                    let mut offs: Vec<u32> = Vec::with_capacity(cnt);
                    let mut bad = false;
                    for i in 0..cnt {
                        match rd32(data, q + 4 * i, endian) {
                            Some(v) => {
                                if v > MAX_VALUE {
                                    bad = true;
                                    break;
                                }
                                offs.push(v);
                            }
                            None => {
                                bad = true;
                                break;
                            }
                        }
                    }
                    if bad {
                        continue;
                    }
                    let mono =
                        offs.windows(2).filter(|w| w[0] < w[1]).count() as f64 / (cnt - 1) as f64;
                    if mono < 0.9 {
                        continue;
                    }
                    if *offs.last().unwrap() == 0 {
                        continue;
                    }
                    let mut hint = false;
                    for back in [4usize, 12] {
                        if pos < back {
                            continue;
                        }
                        for endian2 in [Endian::Be, Endian::Le] {
                            if let Some(v) = rd32(data, pos - back, endian2) {
                                if v as usize == cnt {
                                    hint = true;
                                }
                            }
                        }
                    }
                    let mut conf = 0.42
                        + 0.22 * mono
                        + 0.18 * (cnt as f64 / 24.0).min(1.0)
                        + if hint { 0.18 } else { 0.0 };
                    if hint {
                        conf = (conf + 0.05).min(0.90);
                    }
                    let conf = conf.min(0.90).max(0.0).min(1.0);
                    let ents: Vec<Entry> = (0..cnt)
                        .map(|i| Entry {
                            name: names[i].clone(),
                            size: 0,
                            offset: offs[i] as i64,
                        })
                        .collect();
                    let end = (q + 4 * cnt).min(n);
                    out.push(Table::new("nameblock", ents, conf, pos, end));
                }
            }
        }
    }
    out
}

// --------------------------------------------------------------------------
// 5) 长度前缀记录
// --------------------------------------------------------------------------

/// 读长度前缀（1 或 2 字节）。
#[inline]
fn rd_ns_len(data: &[u8], p: usize, prefix_len: usize, endian: Endian) -> Option<usize> {
    if prefix_len == 1 {
        data.get(p).map(|&v| v as usize)
    } else {
        rd16(data, p, endian).map(|v| v as usize)
    }
}

/// 链式解析长度前缀记录（名字长度逐条读，可变）。返回 `(entries, 结束偏移)`。
///
/// `order == 0` 时 f1=size, f2=offset；`order == 1` 时反过来。
/// `prefix_len` 为 1 或 2。
#[allow(clippy::too_many_arguments)]
fn parse_lenprefix(
    data: &[u8],
    rec: usize,
    _first_len: usize,
    endian: Endian,
    order: usize,
    suffix: usize,
    mode: LenMode,
    prefix_len: usize,
) -> (Vec<Entry>, usize) {
    let n = data.len();
    let mut ents: Vec<Entry> = Vec::new();
    let mut p = rec;
    while ents.len() < MAX_ENTRIES {
        let a: u32;
        let b: u32;
        let nxt: usize;
        let ln: usize;
        let nstart: usize;
        match mode {
            LenMode::LenFirst => {
                if p + prefix_len > n {
                    break;
                }
                ln = match rd_ns_len(data, p, prefix_len, endian) {
                    Some(v) => v,
                    None => break,
                };
                if !(3..=0x80).contains(&ln) {
                    break;
                }
                nstart = p + prefix_len;
                if nstart + ln > n {
                    break;
                }
                let nm = &data[nstart..nstart + ln];
                if name_kind(nm, None).is_none() {
                    break;
                }
                let fpos = nstart + ln + suffix;
                if fpos + 8 > n {
                    break;
                }
                let x = match rd32(data, fpos, endian) {
                    Some(v) => v,
                    None => break,
                };
                let y = match rd32(data, fpos + 4, endian) {
                    Some(v) => v,
                    None => break,
                };
                a = x;
                b = y;
                nxt = fpos + 8;
            }
            LenMode::LenLast => {
                if p + 8 + prefix_len > n {
                    break;
                }
                let x = match rd32(data, p, endian) {
                    Some(v) => v,
                    None => break,
                };
                let y = match rd32(data, p + 4, endian) {
                    Some(v) => v,
                    None => break,
                };
                a = x;
                b = y;
                let lp = p + 8;
                ln = match rd_ns_len(data, lp, prefix_len, endian) {
                    Some(v) => v,
                    None => break,
                };
                if !(3..=0x80).contains(&ln) {
                    break;
                }
                nstart = lp + prefix_len;
                if nstart + ln > n {
                    break;
                }
                let nm = &data[nstart..nstart + ln];
                if name_kind(nm, None).is_none() {
                    break;
                }
                nxt = nstart + ln;
            }
        }
        let (size, off) = if order == 0 { (a, b) } else { (b, a) };
        if size > MAX_VALUE || off > MAX_VALUE {
            break;
        }
        ents.push(Entry {
            name: decode_latin1(&data[nstart..nstart + ln]),
            size: size as i64,
            offset: off as i64,
        });
        p = nxt;
    }
    (ents, p)
}

/// 长度前缀记录表。
fn detect_lenprefix(data: &[u8], runs: &[Vec<(usize, usize)>]) -> Vec<Table> {
    let mut out: Vec<Table> = Vec::new();
    for run in runs {
        if run.len() < 6 {
            continue;
        }
        for &(pos, name_len) in run.iter().take(3) {
            if !(3..=0x7f).contains(&name_len) {
                continue;
            }
            // 长度字节 / 字在名字前面：(prefix_len, 记录起点)
            let mut prefixes: Vec<(usize, usize)> = Vec::new();
            if pos >= 1 && data[pos - 1] as usize == name_len {
                prefixes.push((1, pos - 1));
            }
            if pos >= 2 {
                for endian2 in [Endian::Le, Endian::Be] {
                    if let Some(v) = rd16(data, pos - 2, endian2) {
                        // Python 原条件是 `v == name_len and name_len > 0xff`，
                        // 而上面已经把 name_len 卡在 <= 0x7f，所以永不成立。
                        // 保留原条件以保持行为一致。
                        if v as usize == name_len && name_len > 0xff {
                            prefixes.push((2, pos - 2));
                        }
                    }
                }
            }
            for &(prefix_len, rec) in prefixes.iter() {
                for endian in [Endian::Be, Endian::Le] {
                    for order in 0..2 {
                        for suffix in 0..4 {
                            let (ents, endp) = parse_lenprefix(
                                data,
                                rec,
                                name_len,
                                endian,
                                order,
                                suffix,
                                LenMode::LenFirst,
                                prefix_len,
                            );
                            if ents.len() < 6 {
                                continue;
                            }
                            let cum = cum_score(&ents);
                            if cum < 0.85 {
                                continue;
                            }
                            let conf = (0.55
                                + 0.3 * cum
                                + 0.15 * (ents.len() as f64 / 20.0).min(1.0))
                            .min(0.92);
                            out.push(Table::new("lenprefix", ents, conf.min(1.0), rec, endp));
                        }
                    }
                }
                // [size][offset][len][name]
                for endian in [Endian::Be, Endian::Le] {
                    for order in 0..2 {
                        if rec < 8 {
                            continue;
                        }
                        let rec2 = rec - 8;
                        let (ents, endp) = parse_lenprefix(
                            data,
                            rec2,
                            name_len,
                            endian,
                            order,
                            0,
                            LenMode::LenLast,
                            prefix_len,
                        );
                        if ents.len() < 6 {
                            continue;
                        }
                        let cum = cum_score(&ents);
                        if cum < 0.85 {
                            continue;
                        }
                        let conf = (0.55
                            + 0.3 * cum
                            + 0.15 * (ents.len() as f64 / 20.0).min(1.0))
                        .min(0.92);
                        out.push(Table::new("lenprefix", ents, conf.min(1.0), rec2, endp));
                    }
                }
            }
        }
    }
    out
}

// --------------------------------------------------------------------------
// 汇总
// --------------------------------------------------------------------------

/// `MINIFS` 魔数扫描（最多 32 次），先试名字池布局再试定长布局。
fn detect_minifs(data: &[u8]) -> Vec<Table> {
    let mut out: Vec<Table> = Vec::new();
    let mut start = 0usize;
    let mut guard = 0usize;
    while guard < 32 {
        guard += 1;
        let m = match memmem::find(&data[start..], b"MINIFS") {
            Some(x) => start + x,
            None => break,
        };
        start = m + 1;
        if let Some(t) = minifs_pool(data, m).or_else(|| minifs_fixed(data, m)) {
            out.push(t);
        }
    }
    out
}

/// 一张表的「名字集合」。
fn name_set(t: &Table) -> HashSet<&str> {
    t.entries.iter().map(|e| e.name.as_str()).collect()
}

/// 两张表的名字集合是否「基本是同一批名字」（互相包含率都 >= 0.9）。
fn set_overlap(a: &HashSet<&str>, b: &HashSet<&str>) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let inter = a.intersection(b).count();
    if inter == 0 {
        return false;
    }
    (inter as f64 / a.len() as f64) >= 0.9 && (inter as f64 / b.len() as f64) >= 0.9
}

/// 按 (confidence, 条目数) 降序；稳定排序，等价于 Python 的
/// `sorted(key=(conf, len), reverse=True)`。
fn cmp_table_desc(a: &Table, b: &Table) -> Ordering {
    match b.confidence.partial_cmp(&a.confidence) {
        Some(Ordering::Equal) | None => b.entries.len().cmp(&a.entries.len()),
        Some(o) => o,
    }
}

/// 去掉重复 / 被包含的表。
fn dedupe(mut tables: Vec<Table>) -> Vec<Table> {
    tables.sort_by(cmp_table_desc);
    let is_minifs = |l: &str| l == "minifs-fixed" || l == "minifs-pool";
    let mut kept: Vec<Table> = Vec::with_capacity(tables.len());
    for t in tables {
        let ns_t = name_set(&t);
        let mut dup = false;
        for k in kept.iter() {
            if t.layout == k.layout {
                let diff = if t.start >= k.start {
                    t.start - k.start
                } else {
                    k.start - t.start
                };
                if diff <= 0x20 {
                    if t.len() <= k.len() {
                        dup = true;
                    }
                    break;
                }
            }
            let ns_k = name_set(k);
            if ns_t.is_empty() || ns_k.is_empty() {
                continue;
            }
            let mut overlap = set_overlap(&ns_t, &ns_k);
            if !overlap {
                // 名字被切掉头 / 尾的「残片表」（例如从名字中间起读）用
                // basename 也能认出来。
                let bt: HashSet<&str> = t
                    .entries
                    .iter()
                    .map(|e| e.name.rsplit('/').next().unwrap_or(""))
                    .collect();
                let bk: HashSet<&str> = k
                    .entries
                    .iter()
                    .map(|e| e.name.rsplit('/').next().unwrap_or(""))
                    .collect();
                overlap = set_overlap(&bt, &bk);
            }
            if !overlap {
                continue;
            }
            // 同一批名字：MINIFS 布局（有魔数）永远压过靠推断的 fixed-stride，
            // 因为后者是把 [size][offset][name] 误当 [name][size][offset]，
            // 名字对、值错位一条。
            if is_minifs(&k.layout)
                && !is_minifs(&t.layout)
                && t.len() as f64 <= 1.5 * k.len() as f64
            {
                dup = true;
                break;
            }
            if t.len() <= k.len() {
                dup = true;
            }
            break;
        }
        if !dup {
            kept.push(t);
        }
    }
    kept.sort_by(cmp_table_desc);
    kept
}

/// 在 `data`（固件镜像原始字节）里找出所有可信的「文件名表」。
///
/// 返回按 (置信度, 条目数) 降序排列的表列表，语义与 Python `vxtable.find_tables`
/// 一致（默认 `min_entries=6`、`max_tables=48`）。
pub fn find_tables(data: &[u8]) -> Vec<Table> {
    const MIN_ENTRIES: usize = 6;
    const MAX_TABLES: usize = 48;
    if data.len() < 0x40 {
        return Vec::new();
    }
    let runs = name_runs(data);
    let mut cands: Vec<Table> = Vec::new();
    cands.extend(detect_minifs(data));
    if !runs.is_empty() {
        cands.extend(detect_fixed(data, &runs));
        cands.extend(detect_offsets_only(data, &runs));
        cands.extend(detect_nameblock(data, &runs));
        cands.extend(detect_lenprefix(data, &runs));
    }
    cands.retain(|t| t.len() >= MIN_ENTRIES);
    let mut kept = dedupe(cands);
    kept.sort_by(cmp_table_desc);
    kept.truncate(MAX_TABLES);
    kept
}

// --------------------------------------------------------------------------
// 单元测试（逐样本对齐见 tests/table_parity.rs）
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_kind_basics() {
        assert_eq!(name_kind(b"index.html", None), Some(NameKind::Full));
        assert_eq!(name_kind(b"a/b/c.css", None), Some(NameKind::Full));
        assert_eq!(name_kind(b"x.json", None), Some(NameKind::Full));
        assert_eq!(name_kind(b"abc", None), None);
        assert_eq!(name_kind(b"abcd.txt", Some(8)), Some(NameKind::Full));
        // 字段填满且扩展名被截断
        assert_eq!(name_kind(b"lastpassword.", Some(13)), Some(NameKind::Cut));
        // 没有 field_len 时不算
        assert_eq!(name_kind(b"lastpassword.", None), None);
    }

    #[test]
    fn name_hit_uses_ordered_alternation() {
        // Python 的 `htm|html` 先匹配 `htm`，所以 `index.html` 只吃到 `index.htm`（9 字节）
        assert_eq!(name_hits(b"index.html"), vec![(0, 9)]);
        // `js|json` 先匹配 `js`
        assert_eq!(name_hits(b"app.js"), vec![(0, 6)]);
        assert_eq!(name_hits(b"x.json"), vec![(0, 4)]);
        // 长 class 段 + 右端点：贪婪段先试最右边的点
        assert_eq!(name_hits(b"a/b/c.htm"), vec![(0, 9)]);
    }

    #[test]
    fn find_tables_empty() {
        assert!(find_tables(b"").is_empty());
        assert!(find_tables(&[0u8; 0x20]).is_empty());
    }

    /// 任意字节都不该 panic（固件里什么垃圾都有）。
    #[test]
    fn no_panic_on_random_and_structured() {
        let mut s: u64 = 0x1234_5678_9abc_def0;
        let mut next = move || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        for it in 0..300usize {
            let n = (next() % 6000) as usize;
            let mut d = vec![0u8; n];
            for b in d.iter_mut() {
                let r = next();
                *b = match r % 8 {
                    0 => b'.',
                    1 => b'/',
                    2 => b'0',
                    3 => 0,
                    4 => b'A' + (r % 26) as u8,
                    _ => (r >> 8) as u8,
                };
            }
            if n > 64 {
                let p = (next() as usize) % (n - 32);
                d[p..p + 4].copy_from_slice(b".htm");
                if it % 10 == 0 {
                    d[0..6].copy_from_slice(b"MINIFS");
                }
                if it % 7 == 0 {
                    d[p..p + 4].copy_from_slice(&[0, 0, 0, 3]);
                }
            }
            let ts = find_tables(&d);
            for t in &ts {
                assert!(!t.layout.is_empty());
                assert!((0.0..=1.0).contains(&t.confidence));
                assert!(t.entries.len() >= 6);
            }
        }
    }

    fn put_u32_be(d: &mut [u8], p: usize, v: u32) {
        d[p..p + 4].copy_from_slice(&v.to_be_bytes());
    }

    #[test]
    fn minifs_fixed_synthetic() {
        let stride = 0x58usize;
        let n = 0x100 + stride * 6 + 0x100;
        let mut d = vec![0u8; n];
        d[0x40..0x46].copy_from_slice(b"MINIFS");
        let base = 0x40;
        let p0 = base + 0x1c;
        for i in 0..6usize {
            let p = p0 + i * stride;
            let off = 0x1000 + i * 0x100;
            put_u32_be(&mut d, p - 8, 0x100);
            put_u32_be(&mut d, p - 4, off as u32);
            let nm = format!("file{}.htm", i);
            d[p..p + nm.len()].copy_from_slice(nm.as_bytes());
        }
        let ts = find_tables(&d);
        assert!(!ts.is_empty(), "应找到 minifs-fixed 表");
        let t = &ts[0];
        assert_eq!(t.layout, "minifs-fixed");
        assert_eq!(t.entries.len(), 6);
        assert_eq!(t.entries[0].name, "file0.htm");
        assert_eq!(t.entries[0].size, 0x100);
        assert_eq!(t.entries[0].offset, 0x1000);
        assert_eq!(t.entries[5].offset, 0x1000 + 5 * 0x100);
        assert_eq!(t.start, 0x40);
        assert_eq!(t.end, 0x40 + 0x1c + stride * 6);
    }
}
