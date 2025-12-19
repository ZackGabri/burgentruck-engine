use shakmaty::zobrist::ZobristHash;
use shakmaty::{Chess, Move, MoveList, Position};

use crate::engine_options;
use crate::history::MoveHistory;
use crate::search::eval::get_piece_value;
use crate::transposition_table::{TTBound, TTEntry, get_ttindex};

// struct for shared data between every negamax call
pub struct Negamax {
    pub node_count: usize,
    pub best_line: Vec<Option<Move>>,
    transposition_table: Box<[TTEntry]>,
    table_length: usize,
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
            best_line: vec![None; 100],
            transposition_table: vec![TTEntry::default(); table_length].into_boxed_slice(),
            table_length,
        }
    }

    pub fn negamax(
        &mut self,
        position: &Chess,
        history: &MoveHistory,
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

        self.sort_moves(&mut moves, ply);

        for mov in moves.into_iter() {
            let position = position.clone().play(mov).unwrap();
            let score = -self.negamax(
                &position,
                &history.clone(),
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
            );
            self.node_count += 1;

            if score > max {
                max = score;

                self.best_line[ply] = Some(mov);

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

    fn sort_moves(&self, moves: &mut MoveList, ply: usize) {
        self.sort_captures(moves);

        if let Some(best) = self.best_line[ply]
            && let Some(best_move_index) = moves.iter().position(|m| *m == best)
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
