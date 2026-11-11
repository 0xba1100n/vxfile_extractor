//! 熵扫描：把连续的高熵窗口合并成地址区间，用于标注疑似加密/压缩区。

pub fn shannon(b: &[u8]) -> f64 {
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

#[derive(Debug, Clone)]
pub struct Region {
    pub start: usize,
    pub end: usize,
    pub ent_norm: f64, // 平均熵 / 8，0..1
}

/// 扫描 data，返回归一化熵 > thr 的连续区间（默认窗口 4 KiB）
pub fn ranges(data: &[u8], win: usize, thr: f64) -> Vec<Region> {
    let mut out: Vec<Region> = Vec::new();
    let mut cur: Option<(usize, usize, f64, usize)> = None; // start,end,sum,count
    for (i, w) in data.chunks(win).enumerate() {
        let e = shannon(w) / 8.0;
        let s = i * win;
        let end = (s + w.len()).min(data.len());
        if e > thr {
            match cur {
                Some((cs, _ce, sum, cnt)) => cur = Some((cs, end, sum + e, cnt + 1)),
                None => cur = Some((s, end, e, 1)),
            }
        } else if let Some((cs, ce, sum, cnt)) = cur.take() {
            out.push(Region { start: cs, end: ce, ent_norm: sum / cnt as f64 });
        }
    }
    if let Some((cs, ce, sum, cnt)) = cur {
        out.push(Region { start: cs, end: ce, ent_norm: sum / cnt as f64 });
    }
    out
}

/// 只保留达到最小体积的区间，按体积降序
pub fn big_ranges(data: &[u8], win: usize, thr: f64, min_bytes: usize) -> Vec<Region> {
    let mut v: Vec<Region> = ranges(data, win, thr)
        .into_iter()
        .filter(|r| r.end - r.start >= min_bytes)
        .collect();
    v.sort_by(|a, b| (b.end - b.start).cmp(&(a.end - a.start)));
    v
}