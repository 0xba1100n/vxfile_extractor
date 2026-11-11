//! Cross-language parity tests: `vxcore::symbol` against the Python reference
//! (`tools/vxsym.py`).
//!
//! The snapshots under `work/parity/*_sym.json` are produced by
//! `work/parity/gen_snapshots.py` (run from the repository root).  Each one
//! records
//!
//! * `hits` -- `find_symbols()` metadata in order: detector, offset, size,
//!   names_count, has_table, plus the evidence `names` whenever Python does not
//!   truncate them (the reference caps the evidence list at 48 entries), and
//! * `recover` -- `recover_symbol_table()` metadata plus the *complete* carved
//!   name list taken from the returned blob (not the 48-name evidence list).
//!
//! Every test re-runs the Rust detector on the same firmware image and asserts
//! identical evidence; for the recovered tables it also asserts the name set and
//! that the carved blob is exactly the NUL separated table described by those
//! names.
//!
//! Covered samples:
//!
//! | snapshot          | detector | names | what it exercises                    |
//! |-------------------|----------|-------|--------------------------------------|
//! | `tds744a_sym`     | B        | 5391  | uncompressed table, start trimming   |
//! | `mac1200re_sym`   | A        | 6188  | LZMA-alone carve inside a container  |
//! | `archerc80_sym`   | A        | 10999 | large LZMA-alone carve               |
//! | `chk_raw_sym`     | A        | 6     | header-less raw LZMA1 (Netgear)      |
//! | `intellistar_sym` | B        | 2586  | application table, B suffix rule     |
//! | `linksys_sym`     | B        | 684   | small uncompressed table             |
//! | `synth_c_sym`     | C        | 16    | 64 KiB API-density window            |

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use vxcore::symbol::{find_symbols, recover_symbol_table, SymHit};

/// Walk up from `CARGO_MANIFEST_DIR` until the directory that holds
/// `work/parity` is found.
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
        "repository root not found (no work/parity above CARGO_MANIFEST_DIR={})",
        env!("CARGO_MANIFEST_DIR")
    );
}

fn load_snapshot(name: &str) -> (Vec<u8>, Value) {
    let snap = repo_root()
        .join("work")
        .join("parity")
        .join(format!("{}.json", name));
    let txt = fs::read_to_string(&snap)
        .unwrap_or_else(|e| panic!("read snapshot {}: {}", snap.display(), e));
    let doc: Value = serde_json::from_str(&txt)
        .unwrap_or_else(|e| panic!("parse snapshot {}: {}", snap.display(), e));
    let rel = doc["path"]
        .as_str()
        .unwrap_or_else(|| panic!("{}: missing path", name));
    let sample = repo_root().join(rel);
    let data =
        fs::read(&sample).unwrap_or_else(|e| panic!("read sample {}: {}", sample.display(), e));
    (data, doc)
}

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names
}

fn as_strings(values: &[Value]) -> Vec<String> {
    values
        .iter()
        .map(|v| v.as_str().expect("name string").to_string())
        .collect()
}

/// Names recovered from a carved table blob (the blob must be exactly
/// `name\0name\0...`).
fn names_in_blob(blob: &[u8]) -> Vec<String> {
    blob.split(|&b| b == 0)
        .filter(|t| !t.is_empty())
        .map(|t| String::from_utf8_lossy(t).into_owned())
        .collect()
}

fn check_hits(got: &[SymHit], exp: &Value, sample: &str) {
    let exp = exp
        .as_array()
        .unwrap_or_else(|| panic!("{}: hits is not an array", sample));
    assert_eq!(got.len(), exp.len(), "{}: number of hits", sample);
    for (i, (got, exp)) in got.iter().zip(exp.iter()).enumerate() {
        let detector = exp["detector"]
            .as_str()
            .and_then(|s| s.chars().next())
            .unwrap_or_else(|| panic!("{}: hit {} detector", sample, i));
        assert_eq!(got.detector, detector, "{}: hit {} detector", sample, i);
        assert_eq!(
            got.offset,
            exp["offset"].as_u64().unwrap() as usize,
            "{}: hit {} offset",
            sample,
            i
        );
        assert_eq!(
            got.names_count,
            exp["names_count"].as_u64().unwrap() as usize,
            "{}: hit {} names_count",
            sample,
            i
        );
        assert_eq!(
            got.has_table,
            exp["has_table"].as_bool().unwrap(),
            "{}: hit {} has_table",
            sample,
            i
        );
        assert_eq!(
            got.names.len(),
            got.names_count,
            "{}: hit {} names list is incomplete",
            sample,
            i
        );
        assert!(!got.label.is_empty(), "{}: hit {} label", sample, i);
        if got.has_table {
            assert_eq!(
                got.blob.len(),
                exp["size"].as_u64().unwrap() as usize,
                "{}: hit {} table size",
                sample,
                i
            );
            assert_eq!(
                names_in_blob(&got.blob),
                got.names,
                "{}: hit {} blob/name mismatch",
                sample,
                i
            );
        }
        // Python's evidence list is un-truncated up to 48 names.
        if let Some(names) = exp.get("names").and_then(|v| v.as_array()) {
            let want = as_strings(names);
            assert_eq!(got.names, want, "{}: hit {} evidence names", sample, i);
        }
    }
}

fn check_recover(data: &[u8], exp: &Value, sample: &str) {
    let got = recover_symbol_table(data);
    match exp {
        Value::Null => assert!(
            got.is_none(),
            "{}: expected no recovered table, got {:?}",
            sample,
            got.map(|h| (h.detector, h.offset, h.names_count))
        ),
        _ => {
            let got = got.unwrap_or_else(|| panic!("{}: expected a recovered table", sample));
            let detector = exp["detector"]
                .as_str()
                .and_then(|s| s.chars().next())
                .unwrap();
            assert_eq!(got.detector, detector, "{}: recover detector", sample);
            assert_eq!(
                got.offset,
                exp["offset"].as_u64().unwrap() as usize,
                "{}: recover offset",
                sample
            );
            assert_eq!(
                got.names_count,
                exp["names_count"].as_u64().unwrap() as usize,
                "{}: recover names_count",
                sample
            );
            assert!(got.has_table, "{}: recovered hit must carry a table", sample);
            assert_eq!(
                got.blob.len(),
                exp["size"].as_u64().unwrap() as usize,
                "{}: recover table size",
                sample
            );
            let want = as_strings(exp["names"].as_array().expect("recover names"));
            assert_eq!(want.len(), got.names_count, "{}: snapshot name count", sample);
            // The tier-1 contract: the recovered name *set* is identical.
            assert_eq!(
                sorted(got.names.clone()),
                sorted(want.clone()),
                "{}: recovered name set",
                sample
            );
            // ... and the blob is exactly those names, in order.
            assert_eq!(
                names_in_blob(&got.blob),
                want,
                "{}: recovered blob names",
                sample
            );
        }
    }
}

fn check(name: &str) {
    let (data, doc) = load_snapshot(name);
    let hits = find_symbols(&data);
    check_hits(&hits, &doc["hits"], name);
    check_recover(&data, &doc["recover"], name);
}

#[test]
fn parity_tds744a_uncompressed_table() {
    // Detector B, 5391 names, `samples/expand-d/tektronix_TDS744A_4.2.1e_fw.bin`.
    check("tds744a_sym");
}

#[test]
fn parity_mac1200re_compressed_table() {
    // Detector A (LZMA-alone carve), 6188 names,
    // `samples/mercury/MAC1200RE_V2.0.zip.inner.bin`.
    check("mac1200re_sym");
}

#[test]
fn parity_archer_c80_compressed_table() {
    // Detector A (LZMA-alone carve), 10999 names,
    // `samples/tp-link/Archer_C80_US_V1_191122.zip.inner.bin`.
    check("archerc80_sym");
}

#[test]
fn parity_netgear_chk_raw_lzma() {
    // Detector A, header-less raw LZMA1 stream at 0x56, marker-only (6 API names,
    // no carved table): `samples/_x/wgr614v5_1_0_9_1_0_6.chk`.
    let (data, doc) = load_snapshot("chk_raw_sym");
    let hits = find_symbols(&data);
    check_hits(&hits, &doc["hits"], "chk_raw_sym");
    assert_eq!(hits.len(), 1, "chk_raw_sym: single A hit");
    assert_eq!(hits[0].detector, 'A');
    assert_eq!(hits[0].offset, 0x56);
    assert!(!hits[0].has_table);
    assert!(hits[0].blob.is_empty());
    check_recover(&data, &doc["recover"], "chk_raw_sym");
}

#[test]
fn parity_intellistar_uncompressed_table() {
    // Detector B, 2586 names, application table with VxWorks suffixes.
    check("intellistar_sym");
}

#[test]
fn parity_linksys_uncompressed_table() {
    // Detector B, 684 names, small uncompressed table.
    check("linksys_sym");
}

#[test]
fn parity_synthetic_density_window() {
    // Synthetic: 16 distinct API names in one 64 KiB window, no name table.
    let (data, doc) = load_snapshot("synth_c_sym");
    let hits = find_symbols(&data);
    check_hits(&hits, &doc["hits"], "synth_c_sym");
    assert_eq!(hits.len(), 1, "synth_c_sym: single C hit");
    assert_eq!(hits[0].detector, 'C');
    assert!(!hits[0].has_table);
    assert_eq!(hits[0].names_count, 16);
    assert_eq!(hits[0].offset, 1000);
    assert!(recover_symbol_table(&data).is_none());
}

#[test]
fn parity_synthetic_multi_hit_ordering() {
    // Synthetic: an uncompressed table on the raw layer (B, 64 names) plus an
    // embedded LZMA-alone layer that carries its own table (A, 84 names).
    // Exercises layer expansion, B-on-raw / A-on-lzma and the strength order
    // (`has_table`, names_count, size, detector) used by find_symbols.
    let (data, doc) = load_snapshot("synth_multi_sym");
    let hits = find_symbols(&data);
    check_hits(&hits, &doc["hits"], "synth_multi_sym");
    assert_eq!(hits.len(), 2, "synth_multi_sym: A and B hits");
    assert_eq!(hits[0].detector, 'A');
    assert_eq!(hits[0].names_count, 84);
    assert!(hits[0].has_table);
    assert_eq!(hits[1].detector, 'B');
    assert_eq!(hits[1].offset, 64);
    assert_eq!(hits[1].names_count, 64);
    assert!(hits[1].has_table);
    let rec = recover_symbol_table(&data).expect("recovered table");
    assert_eq!(rec.detector, 'A');
    assert_eq!(rec.names_count, 84);
}

/// Guard-rail: a real table hit must expose a complete, NUL separated name list
/// and recovery must be the strongest table-carrying hit.
#[test]
fn recover_returns_strongest_table() {
    let (data, _) = load_snapshot("archerc80_sym");
    let hits = find_symbols(&data);
    let rec = recover_symbol_table(&data).expect("recovered table");
    let first_table = hits
        .iter()
        .find(|h| h.has_table && !h.blob.is_empty())
        .expect("table hit");
    assert_eq!(rec.detector, first_table.detector);
    assert_eq!(rec.offset, first_table.offset);
    assert_eq!(rec.names_count, first_table.names_count);
    assert_eq!(rec.blob, first_table.blob);
}
