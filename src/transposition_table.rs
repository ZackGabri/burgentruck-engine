use shakmaty::zobrist::Zobrist64;

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

pub fn get_ttindex(hash: TTHash, length: usize) -> usize {
    (hash.0 as usize) & (length - 1)
}
