use shakmaty::{Chess, Position};

use crate::transposition_table::TTHash;

const HISTORY_SIZE: usize = 2048;

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

    pub fn hash_exists(&self, item: &TTHash) -> bool {
        for i in 0..self.index {
            if *item == self.history[i] {
                return true;
            }
        }

        false
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
