//! Parity harness: compare `vxcore::scan_all` against the layer list produced
//! by the Python scanner for real firmware samples.
//!
//! The expected JSON files are generated with:
//!
//! ```text
//! python3 work/parity/make_expected.py
//! ```
//!
//! `make_expected.py` runs `tools/vxbinwalk.scan_all`, classifies each row into
//! the Rust `Layer` kind vocabulary and drops the image/HTML/JS carving rows
//! (there is no `Layer` kind for them; the counts are recorded under
//! `carving_skipped` in each JSON file).
//!
//! Two fixture families are checked:
//!
//! * extracted firmware members (what the extractor normally feeds the
//!   scanner):
//!   - `mw313r_v4`  = samples/mercury/MW313R_V4.0_160602.zip -> mw313rv4.bin
//!   - `archer_c80` = samples/tp-link/Archer_C80_US_V1_191122.zip -> c80v1...bin
//!   - `wgr614v3`   = samples/netgear/WGR614v3_FW2.00.zip -> wgr614v2_v_2_00.chk
//! * the original ZIP archives themselves, to exercise the built-in ZIP reader.
//!
//! `offset`, `kind` and `size` must match Python exactly, in the same order.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn workspace() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures() -> PathBuf {
    workspace().join("work/parity")
}

fn compare_layers(name: &str, data: &[u8], expected: &Value) {
    let exp = expected["layers"].as_array().expect("layers array");
    let got = vxcore::scan_all(data);
    assert_eq!(
        got.len(),
        exp.len(),
        "{name}: layer count differs (rust {} vs python {})",
        got.len(),
        exp.len()
    );
    for (i, (g, e)) in got.iter().zip(exp.iter()).enumerate() {
        let eo = e["offset"].as_u64().expect("offset") as usize;
        let es = e["size"].as_u64().expect("size") as usize;
        let ek = e["kind"].as_str().expect("kind");
        assert_eq!(
            g.offset, eo,
            "{name}: layer {i} offset differs (kind {ek}, size {es})"
        );
        assert_eq!(
            g.data.len(),
            es,
            "{name}: layer {i} size differs at offset {eo} (kind {ek}, rust kind {})",
            g.kind
        );
        assert_eq!(g.kind, ek, "{name}: layer {i} kind differs at offset {eo}");
    }
    eprintln!(
        "{name}: {} layers, offsets/sizes/kinds match Python ({} python rows total)",
        got.len(),
        expected["python_rows"].as_u64().unwrap_or(0)
    );
}

fn load_fixture(name: &str) -> Option<Value> {
    let json = fixtures().join(format!("{}.json", name));
    if !json.exists() {
        eprintln!("SKIP {name}: missing {}", json.display());
        return None;
    }
    Some(serde_json::from_slice(&fs::read(&json).expect("read json")).expect("parse json"))
}

#[test]
fn python_parity_three_samples() {
    let samples = ["mw313r_v4", "archer_c80", "wgr614v3"];
    let mut compared = 0usize;
    for name in samples {
        let bin = fixtures().join(format!("{}.inner.bin", name));
        let Some(expected) = load_fixture(name) else {
            continue;
        };
        if !bin.exists() {
            eprintln!("SKIP {name}: missing {}", bin.display());
            continue;
        }
        let data = fs::read(&bin).expect("read sample");
        compare_layers(name, &data, &expected);
        compared += 1;
    }
    assert!(
        compared >= 3,
        "only {compared}/3 member parity samples ran; fixtures missing under {}",
        fixtures().display()
    );
}

#[test]
fn python_parity_zip_archives() {
    let samples = [
        ("mw313r_v4_zip", "samples/mercury/MW313R_V4.0_160602.zip"),
        ("archer_c80_zip", "samples/tp-link/Archer_C80_US_V1_191122.zip"),
        ("wgr614v3_zip", "samples/netgear/WGR614v3_FW2.00.zip"),
    ];
    let mut compared = 0usize;
    for (name, rel) in samples {
        let bin = workspace().join(rel);
        let Some(expected) = load_fixture(name) else {
            continue;
        };
        if !bin.exists() {
            eprintln!("SKIP {name}: missing {}", bin.display());
            continue;
        }
        let data = fs::read(&bin).expect("read zip");
        compare_layers(name, &data, &expected);
        compared += 1;
    }
    assert!(
        compared >= 3,
        "only {compared}/3 zip parity samples ran; fixtures missing under {}",
        fixtures().display()
    );
}
