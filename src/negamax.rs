use shakmaty::zobrist::ZobristHash;
use shakmaty::{Chess, Move, Position};
use std::time::Instant;

use crate::transposition_table::{TABLE_SIZE, TTBound, TTEntry, get_ttindex};

const DEFAULT_SEARCH_DEPTH: usize = 6;

pub fn search(position: &Chess, depth: Option<usize>) -> Option<Move> {
    let depth = depth.unwrap_or(DEFAULT_SEARCH_DEPTH);

    let mut negamax = Negamax::new();

    let start = Instant::now();
    let best_score = negamax.negamax(position, depth, 0, -69420, 69420);
    let duration = start.elapsed();

    println!(
        "info depth {depth} score cp {best_score} nodes {} nps {} time {} pv {}",
        negamax.node_count,
        (negamax.node_count as f64 / duration.as_secs_f64()) as usize,
        duration.as_millis(),
        negamax
            .best_line
            .iter()
            .filter_map(|x| x.map(|x| x.to_uci(position.castles().mode()).to_string()))
            .collect::<Vec<String>>()
            .join(" ")
    );

    dbg!(negamax.best_line[0])
}

// struct for shared data between every negamax call
pub struct Negamax {
    node_count: usize,
    best_line: Vec<Option<Move>>,
    transposition_table: Box<[TTEntry]>,
}

impl Negamax {
    fn new() -> Self {
        Self {
            node_count: 0,
            best_line: vec![None; 100],
            transposition_table: vec![TTEntry::default(); TABLE_SIZE].into_boxed_slice(),
        }
    }

    fn negamax(
        &mut self,
        position: &Chess,
        depth: usize,
        ply: usize,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        if depth == 0 {
            return crate::eval::evaluate(position);
        }

        if position.is_insufficient_material() {
            return 0;
        }

        let original_alpha = alpha;
        let hash = position.zobrist_hash(shakmaty::EnPassantMode::Legal);
        let tt_index = get_ttindex(hash);
        let tt_entry = &self.transposition_table[tt_index];
        let replace_tt = tt_entry.depth < depth || tt_entry.hash != hash;

        if tt_entry.depth >= depth && tt_entry.hash == hash {
            match tt_entry.bound {
                TTBound::Exact => return tt_entry.score,
                TTBound::Lower => {
                    if tt_entry.score >= beta {
                        return tt_entry.score;
                    }
                }
                TTBound::Upper => {
                    if tt_entry.score < alpha {
                        return tt_entry.score;
                    }
                }
            }
        }

        let mut max = -69420;
        let moves = position.legal_moves();

        if moves.is_empty() {
            if position.is_check() {
                // checkmate
                return -69420 + ply as i32;
            } else {
                // stalemate
                return 0;
            }
        }

        for mov in moves.into_iter() {
            let position = position.clone().play(mov).unwrap();
            let score = -self.negamax(&position, depth - 1, ply + 1, -beta, -alpha);
            self.node_count += 1;

            if score > max {
                max = score;
                let _ = self.best_line[ply].insert(mov);

                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                return max;
            }
        }

        if replace_tt {
            let bound = if max <= original_alpha {
                TTBound::Upper
            } else if max >= beta {
                TTBound::Lower
            } else {
                TTBound::Exact
            };

            self.transposition_table[tt_index] = TTEntry {
                hash,
                depth,
                bound,
                score: max,
            };
        }

        max
    }
}
