//! VxWorks firmware extraction engine.
pub mod carve;
pub mod entropy;
pub mod lzcarve;
pub mod minifs;
pub mod minifs_v2;
pub mod pk11;
pub mod profile;
pub mod rawkernel;
pub mod scan;
pub mod table;
pub mod symbol;
pub mod verdict;

pub use carve::{carve, Carved};
pub use minifs::Entry as MinifsEntry;
pub use scan::{scan_all, Layer};
pub use table::{find_tables, Entry, Table};
pub use symbol::{find_symbols, recover_symbol_table, SymHit};
pub use verdict::{judge, judge_image, FileVerdict, Verdict};

#[derive(Debug, Clone, Default)]
pub struct Info {
    pub layers: Vec<Layer>,
    pub tables: Vec<Table>,
    pub symbols: Vec<SymHit>,
}