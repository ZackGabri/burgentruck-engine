use shakmaty::zobrist::Zobrist64;

pub type TTHash = Zobrist64;

#[derive(Clone, Copy)]
pub enum TTBound {
    Exact,
    Upper,
    Lower,
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: TTHash,
    pub bound: TTBound,
    pub depth: usize,
    pub score: i32,
    pub best_move: Option<shakmaty::Move>,
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            hash: Default::default(),
            depth: Default::default(),
            bound: TTBound::Exact,
            best_move: None,
            score: -69420,
        }
    }
}

pub fn get_ttindex(hash: TTHash, length: usize) -> usize {
    (hash.0 as usize) & (length - 1)
}
