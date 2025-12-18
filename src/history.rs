use shakmaty::{Chess, zobrist::ZobristHash};

use crate::transposition_table::TTHash;

const HISTORY_SIZE: usize = 512;

#[derive(Clone, Copy, Debug)]
pub struct MoveHistory {
    pub history: [TTHash; HISTORY_SIZE],
    pub index: usize,
}

impl MoveHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_position(&mut self, pos: &Chess) {
        let hash = pos.zobrist_hash(shakmaty::EnPassantMode::Legal);
        self.push_hash(hash);
    }

    pub fn push_hash(&mut self, item: TTHash) {
        self.history[self.index] = item;
        self.index += 1;
    }

    pub fn pop(&mut self) {
        self.index -= 1;
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn count_item(&self, item: &TTHash) -> usize {
        let mut count = 0;
        for i in 0..self.index {
            if *item == self.history[i] {
                count += 1;
            }
        }

        count
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self {
            history: [0.into(); HISTORY_SIZE],
            index: 0,
        }
    }
}
