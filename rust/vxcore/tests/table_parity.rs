//! 与 Python 侧 `tools/vxtable.py` 的逐条对齐测试。
//!
//! 基准快照由 Python 生成（`work/parity/gen.py` / `gen_synth.py` / `gen_xcheck.py`）：
//!
//! * `work/parity/<sample>.bin`        —— 参与扫描的镜像本体（zip 已取最大成员）
//! * `work/parity/<sample>_table.json` —— Python `find_tables` 选出的最佳表
//!   （`layout` / `start` / `end` / `confidence` / `tables` / `entries`）
//! * `work/parity/synth/*`             —— 覆盖 3 个真实样本之外的布局
//! * `work/parity/xcheck/*`            —— 6 个必须先用 7-Zip/zip 解包的真实样本
//!
//! 断言粒度是「条目逐条一致」：名字、size、offset 全部相等，另外还比对
//! layout / start / end / confidence / 表数量。
//!
//! 快照缺失或过期时先跑：
//! ```text
//! python3 work/parity/gen.py && python3 work/parity/gen_synth.py
//! python work/parity/gen_xcheck.py        # 需要 Windows 侧 7-Zip（仅 rar）
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use vxcore::table::{find_tables, Table};

/// 从 `CARGO_MANIFEST_DIR` 往上找到同时有 `work/parity` 的仓库根。
fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..8 {
        if p.join("work").join("parity").is_dir() {
            return p;
        }
        if !p.pop() {
            break;
        }
    }
    panic!(
        "找不到仓库根（含 work/parity 的目录），CARGO_MANIFEST_DIR={}",
        env!("CARGO_MANIFEST_DIR")
    );
}

/// Python 侧 `max(tables, key=lambda t: (len(t), t.confidence))`：并列时取第一个。
fn best_table(tables: &[Table]) -> Option<&Table> {
    let mut it = tables.iter();
    let mut best = it.next()?;
    for t in it {
        if (t.entries.len(), t.confidence) > (best.entries.len(), best.confidence) {
            best = t;
        }
    }
    Some(best)
}

/// 比对单个快照，返回所有不一致项（空表示通过）。
fn check_snapshot(json_path: &Path) -> Vec<String> {
    let mut errs: Vec<String> = Vec::new();
    let name = json_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("<unknown>")
        .trim_end_matches("_table")
        .to_string();

    let raw = match fs::read(json_path) {
        Ok(v) => v,
        Err(e) => return vec![format!("[{name}] 读不到快照 {}: {e}", json_path.display())],
    };
    let snap: Value = match serde_json::from_slice(&raw) {
        Ok(v) => v,
        Err(e) => return vec![format!("[{name}] 快照不是合法 json: {e}")],
    };

    let bin_path = json_path.with_file_name(format!("{name}.bin"));
    let data = match fs::read(&bin_path) {
        Ok(v) => v,
        Err(e) => {
            return vec![format!(
                "[{name}] 读不到镜像 {}: {e}（先跑 python3 work/parity/gen*.py）",
                bin_path.display()
            )]
        }
    };

    // 快照与镜像必须是同一份数据。
    if let Some(sz) = snap["size"].as_u64() {
        if sz as usize != data.len() {
            errs.push(format!(
                "[{name}] 镜像长度不符: 快照 {sz}, 实际 {}",
                data.len()
            ));
        }
    }

    let want_layout = snap["layout"].as_str().unwrap_or("");
    let want_entries = snap["entries"].as_array().cloned().unwrap_or_default();
    let want_start = snap["start"].as_u64().map(|v| v as usize);
    let want_end = snap["end"].as_u64().map(|v| v as usize);
    let want_conf = snap["confidence"].as_f64();
    let want_tables = snap["tables"].as_u64().map(|v| v as usize);

    let tables = find_tables(&data);
    let best = match best_table(&tables) {
        Some(t) => t,
        None => {
            errs.push(format!(
                "[{name}] find_tables 返回空，Python 侧是 {} 条（{}）",
                want_entries.len(),
                want_layout
            ));
            return errs;
        }
    };

    if let Some(w) = want_tables {
        if tables.len() != w {
            errs.push(format!(
                "[{name}] 表数量不符: 快照 {w}, 实际 {}（{:?}）",
                tables.len(),
                tables.iter().map(|t| t.layout.as_str()).collect::<Vec<_>>()
            ));
        }
    }
    if best.layout != want_layout {
        errs.push(format!(
            "[{name}] layout 不符: 快照 {want_layout}, 实际 {}",
            best.layout
        ));
    }
    if best.entries.len() != want_entries.len() {
        errs.push(format!(
            "[{name}] 条目数不符: 快照 {}, 实际 {}",
            want_entries.len(),
            best.entries.len()
        ));
    }
    if let Some(w) = want_start {
        if best.start != w {
            errs.push(format!("[{name}] start 不符: 快照 0x{w:X}, 实际 0x{:X}", best.start));
        }
    }
    if let Some(w) = want_end {
        if best.end != w {
            errs.push(format!("[{name}] end 不符: 快照 0x{w:X}, 实际 0x{:X}", best.end));
        }
    }
    if let Some(w) = want_conf {
        // Python 与 Rust 用同样的 f64 运算顺序，允许极小误差即可。
        if (best.confidence - w).abs() > 1e-9 {
            errs.push(format!(
                "[{name}] confidence 不符: 快照 {w:.12}, 实际 {:.12}",
                best.confidence
            ));
        }
    }

    // 条目逐条比对（名字 / size / offset 全等）。
    let n = want_entries.len().min(best.entries.len());
    let mut bad = 0usize;
    for i in 0..n {
        let e = &want_entries[i];
        let got = &best.entries[i];
        let wname = e.get(0).and_then(|v| v.as_str()).unwrap_or("");
        let wsize = e.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
        let woff = e.get(2).and_then(|v| v.as_i64()).unwrap_or(0);
        if got.name != wname || got.size != wsize || got.offset != woff {
            bad += 1;
            if bad <= 8 {
                errs.push(format!(
                    "[{name}] 第 {i} 条不符: Python ({wname:?}, {wsize}, {woff}), Rust ({:?}, {}, {})",
                    got.name, got.size, got.offset
                ));
            }
        }
    }
    if bad > 8 {
        errs.push(format!("[{name}] 共 {bad} 条不一致（只显示前 8 条）"));
    }

    if errs.is_empty() {
        println!(
            "[{name}] OK  {} n={} conf={:.4} tables={}",
            best.layout,
            best.entries.len(),
            best.confidence,
            tables.len()
        );
    }
    errs
}

fn check_named(sample: &str) {
    let path = repo_root()
        .join("work")
        .join("parity")
        .join(format!("{sample}_table.json"));
    assert!(path.is_file(), "缺少快照 {}", path.display());
    let errs = check_snapshot(&path);
    assert!(errs.is_empty(), "对齐失败:\n{}", errs.join("\n"));
}

/// 3 个必须逐条一致的样本之一：Mercury MW313R v4，`minifs-fixed` 124 条。
#[test]
fn parity_mw313r_minifs_fixed() {
    check_named("mw313r");
}

/// 3 个必须逐条一致的样本之一：TP-Link Archer C80 v1，`minifs-pool` 297 条。
#[test]
fn parity_archer_c80_minifs_pool() {
    check_named("archer_c80");
}

/// 3 个必须逐条一致的样本之一：Linksys WRT54G v5/v6，`fixed-stride` 93 条（TROC）。
#[test]
fn parity_wrt54g_fixed_stride() {
    check_named("wrt54g");
}

/// 合成样本：覆盖 minifs-fixed 变体 / offsets-only / nameblock / lenprefix /
/// fixed-stride BE 等 3 个真实样本没覆盖到的分支。
#[test]
fn parity_synthetic_layouts() {
    let dir = repo_root().join("work").join("parity").join("synth");
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("读不到 {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.ends_with("_table.json"))
                .unwrap_or(false)
        })
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "{} 里没有 *_table.json", dir.display());
    let mut errs = Vec::new();
    for p in &paths {
        errs.extend(check_snapshot(p));
    }
    assert!(errs.is_empty(), "对齐失败:\n{}", errs.join("\n"));
}

/// 一次性把所有快照（真实样本 + 合成样本 + 解包样本）跑一遍。
#[test]
fn parity_all_snapshots() {
    let root = repo_root().join("work").join("parity");
    let dirs = vec![root.clone(), root.join("synth"), root.join("xcheck")];
    let mut paths: Vec<PathBuf> = Vec::new();
    for d in dirs.iter() {
        if !d.is_dir() {
            continue;
        }
        for e in fs::read_dir(d).unwrap().filter_map(|e| e.ok()) {
            let p = e.path();
            if p.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.ends_with("_table.json"))
                .unwrap_or(false)
            {
                paths.push(p);
            }
        }
    }
    paths.sort();
    assert!(!paths.is_empty(), "没有任何快照（先跑 python3 work/parity/gen*.py）");
    let mut errs = Vec::new();
    for p in &paths {
        errs.extend(check_snapshot(p));
    }
    assert!(errs.is_empty(), "对齐失败:\n{}", errs.join("\n"));
}
