use shakmaty::zobrist::Zobrist64;

pub const TABLE_SIZE: usize = 1_usize << 22;
pub type TTHash = Zobrist64;

#[derive(Clone)]
pub enum TTBound {
    Exact,
    Upper,
    Lower,
}

#[derive(Clone)]
pub struct TTEntry {
    pub hash: TTHash,
    pub bound: TTBound,
    pub depth: usize,
    pub score: i32,
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            hash: Default::default(),
            depth: Default::default(),
            bound: TTBound::Exact,
            score: -69420,
        }
    }
}

pub fn get_ttindex(hash: TTHash) -> usize {
    (hash.0 as usize) & (TABLE_SIZE - 1)
}
