use std::time::{Duration, Instant};

use shakmaty::zobrist::ZobristHash;
use shakmaty::{Chess, Move, MoveList, Position};

use crate::engine_options;
use crate::history::MoveHistory;
use crate::search::eval::get_piece_value;
use crate::transposition_table::{TTBound, TTEntry, get_ttindex};

#[derive(Default, Debug)]
pub struct TimeControl {
    pub w_time: u64,
    pub b_time: u64,
    pub w_inc: u64,
    pub b_inc: u64,
}

// struct for shared data between every negamax call
pub struct Negamax {
    pub node_count: usize,
    transposition_table: Box<[TTEntry]>,
    table_length: usize,

    pub start_time: Option<Instant>,      // start time in millis
    pub allocated_time: Option<Duration>, // total allocated time to spend searching
}

impl Negamax {
    pub fn new() -> Self {
        // convert from megabytes to bytes
        let desired_size = engine_options().get_number("Hash") * 1024 * 1024;
        let tt_entry_size = size_of::<TTEntry>();

        // calculate how many entries will fit in that memory size
        let table_length = desired_size / tt_entry_size;

        Self {
            node_count: 0,
            transposition_table: vec![TTEntry::default(); table_length].into_boxed_slice(),
            table_length,

            start_time: None,
            allocated_time: None,
        }
    }

    pub fn set_time(&mut self, duration: u64) {
        if duration != 0 {
            self.allocated_time = Some(Duration::from_millis(duration));
            self.start_time = Some(Instant::now());
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn negamax(
        &mut self,
        position: &Chess,
        history: &MoveHistory,
        pv_line: &mut [Option<Move>],
        depth: usize,
        ply: usize,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        if depth == 0 {
            return self.quiescence(position, &mut alpha, beta);
        }

        if position.is_insufficient_material() {
            return 0;
        }

        let original_alpha = alpha;
        let hash = position.zobrist_hash(shakmaty::EnPassantMode::Legal);

        let mut history = *history;
        history.push_hash(hash);

        // threefold detection
        let count = history.count_item(&hash);
        if count >= 2 && ply > 0 {
            return 0;
        }

        let tt_index = get_ttindex(hash, self.table_length);
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
        let mut moves = position.legal_moves();

        if moves.is_empty() {
            if position.is_check() {
                // checkmate
                return -69420 + ply as i32;
            } else {
                // stalemate
                return 0;
            }
        }

        self.sort_moves(&mut moves, &tt_entry.best_move);
        for mov in moves.into_iter() {
            if let Some(allocated_time) = self.allocated_time
                && let Some(start_time) = self.start_time
            {
                let time_elapsed = Instant::now().duration_since(start_time);

                if time_elapsed >= allocated_time {
                    return alpha;
                }
            }

            let position = position.clone().play(mov).unwrap();
            let mut child_pv = vec![None; depth];
            let score = -self.negamax(
                &position,
                &history.clone(),
                &mut child_pv,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
            );
            self.node_count += 1;

            if score > max {
                max = score;

                pv_line[0] = Some(mov);
                pv_line[1..(child_pv.len() + 1)].copy_from_slice(&child_pv[..]);

                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                break;
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
                best_move: pv_line[0],
                score: max,
            };
        }

        max
    }

    fn quiescence(&mut self, position: &Chess, alpha: &mut i32, beta: i32) -> i32 {
        let static_eval = super::eval::evaluate(position);

        let mut best_value = static_eval;
        if best_value >= beta {
            return best_value;
        }
        if best_value > *alpha {
            *alpha = best_value;
        }

        let mut captures = position.capture_moves();
        self.sort_captures(&mut captures);

        for capture in captures {
            let position = position.clone().play(capture).unwrap();
            let score = -self.quiescence(&position, &mut -beta, -*alpha);
            self.node_count += 1;

            if score >= beta {
                return score;
            }
            if score > best_value {
                best_value = score;
            }
            if score > *alpha {
                *alpha = score;
            }
        }

        best_value
    }

    fn sort_moves(&self, moves: &mut MoveList, hash_move: &Option<Move>) {
        self.sort_captures(moves);

        if let Some(best) = hash_move
            && let Some(best_move_index) = moves.iter().position(|m| m == best)
        {
            moves.swap(0, best_move_index);
        }
    }

    fn sort_captures(&self, moves: &mut MoveList) {
        moves.sort_by_key(|m| {
            if let Some(victim) = m.capture() {
                let attacker = get_piece_value(m.role());
                let victim = get_piece_value(victim);
                -(victim * 10 - attacker)
            } else {
                100000 // aka show all normal moves last
            }
        });
    }
}
