use std::{cell::UnsafeCell, sync::OnceLock};

use shakmaty::zobrist::Zobrist64;

use crate::search::MATE_SCORE;

// SAFETY:
// We allow benign data races:
// - entries are copied in/out
// - no references escape
// - overwrites are allowed
#[derive(Debug)]
pub struct GlobalTT(UnsafeCell<TTable>);
unsafe impl Sync for GlobalTT {}

static TT: OnceLock<GlobalTT> = OnceLock::new();

#[inline(always)]
fn tt() -> &'static GlobalTT {
    TT.get_or_init(|| GlobalTT(UnsafeCell::new(TTable::new())))
}

#[inline(always)]
pub fn get(hash: TTHash) -> TTEntry {
    unsafe { (*tt().0.get()).get(hash) }
}

#[inline(always)]
pub fn put(hash: TTHash, entry: TTEntry) {
    unsafe { (*tt().0.get()).put(hash, entry) }
}

#[inline(always)]
pub fn hashfull() -> u32 {
    unsafe { (*tt().0.get()).hashfull() }
}

#[inline(always)]
pub fn clear() {
    unsafe { (*tt().0.get()).clear() }
}

#[inline(always)]
pub fn resize() {
    let new_size = crate::engine_options().get_number("Hash");
    unsafe {
        // SAFETY:
        // - Search threads are stopped
        // - No concurrent reads/writes
        (*tt().0.get()).resize(new_size);
    }
}

pub type TTHash = Zobrist64;
pub struct TTable {
    table: Box<[TTEntry]>,
    length: usize,
}

impl TTable {
    pub fn new() -> Self {
        let bytes = crate::engine_options().get_number("Hash") * 1024 * 1024;

        // calculate how many entries will fit in that memory size
        let table_length = bytes / size_of::<TTEntry>();
        Self {
            table: vec![TTEntry::default(); table_length].into_boxed_slice(),
            length: table_length,
        }
    }

    #[inline(always)]
    pub fn get(&self, hash: TTHash) -> TTEntry {
        self.table[self.get_index(&hash)]
    }

    #[inline(always)]
    pub fn put(&mut self, hash: TTHash, entry: TTEntry) {
        self.table[self.get_index(&hash)] = entry;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.table.fill(TTEntry::default());
    }

    #[inline(always)]
    fn get_index(&self, hash: &TTHash) -> usize {
        (hash.0 as usize) & (self.length - 1)
    }

    pub fn resize(&mut self, megabytes: usize) {
        let bytes = megabytes * 1024 * 1024;
        let table_length = bytes / size_of::<TTEntry>();

        self.table = vec![TTEntry::default(); table_length].into_boxed_slice();
        self.length = table_length;
    }

    // estimates how full the hash table is by sampling a few hashes
    pub fn hashfull(&self) -> u32 {
        const SAMPLE: usize = 1000;
        let mut used = 0;

        let stride = self.length / SAMPLE;
        for i in (0..self.length).step_by(stride) {
            if self.table[i].hash.0 != 0 {
                used += 1;
            }
        }

        // return 0-1000 like UCI expects
        (used * 1000 / SAMPLE) as u32
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TTBound {
    Exact,
    Upper,
    Lower,
}

#[derive(Clone, Copy, Debug)]
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
            score: -MATE_SCORE,
        }
    }
}
