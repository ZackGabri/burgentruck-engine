use shakmaty::zobrist::Zobrist64;

pub const TABLE_SIZE: usize = 1_usize << 22;
type TTEntryHashSize = Zobrist64;

#[derive(Clone)]
pub struct TTEntry {
    pub hash: TTEntryHashSize,
    pub depth: usize,
    pub score: i32,
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            hash: Default::default(),
            depth: Default::default(),
            score: -69420,
        }
    }
}

pub fn get_ttindex(hash: TTEntryHashSize) -> usize {
    (hash.0 as usize) & (TABLE_SIZE - 1)
}
