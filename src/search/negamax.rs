use std::time::{Duration, Instant};

use rand::SeedableRng;
use rand::rngs::SmallRng;
use shakmaty::zobrist::ZobristHash;
use shakmaty::{Chess, Move, MoveList, Position};

use crate::engine_options;
use crate::history::MoveHistory;
use crate::search::eval::MATE_SCORE;
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
    pub pv_line: Vec<Option<Move>>,
    transposition_table: Box<[TTEntry]>, // o
    table_length: usize,
    rng: SmallRng,

    deadline: Option<Instant>, // deadline for the search to stop at
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
            pv_line: vec![None; 100],
            rng: SmallRng::from_seed([0; 32]),

            deadline: None,
        }
    }

    pub fn set_time(&mut self, duration: u64) {
        if duration != 0 {
            self.deadline = Some(Instant::now() + Duration::from_millis(duration));
        }
    }

    #[allow(clippy::too_many_arguments)]
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

        if self.is_out_of_time() {
            return -alpha;
        }

        let original_alpha = alpha;
        let hash = position.zobrist_hash(shakmaty::EnPassantMode::Legal);

        let mut history = *history;
        history.push_hash(hash);

        // threefold detection
        let count = history.count_item(&hash);
        if count >= 2 && ply > 0 || position.is_insufficient_material() {
            return 0;
        }

        let is_check = position.is_check();
        let is_root = ply == 0;

        let tt_index = get_ttindex(hash, self.table_length);
        let tt_entry = self.transposition_table[tt_index];
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

        // whole node pruning
        if !is_check && !is_root {
            // reverse futility pruning
            let static_eval = self.static_eval(position);
            let margin = 80 * depth;

            if depth <= 4 && static_eval >= beta + margin as i32 {
                return static_eval;
            }

            // null move pruning
            if depth >= 3 {
                // Make sure it's not king and pawn endgame
                let board = position.board();
                let non_pawns = (board.occupied() & (!board.pawns())).count();
                if non_pawns > 2 {
                    let null_position = position.clone().swap_turn().unwrap();
                    let reduction = 3 + depth / 3;
                    let null_score = -self.negamax(
                        // search with zero window
                        &null_position,
                        &history,
                        depth.saturating_sub(reduction),
                        ply + 1,
                        -beta,
                        -beta + 1,
                    );

                    // Make sure it's not a mate score
                    if null_score >= beta && null_score.abs() < 69000 {
                        return null_score;
                    }
                }
            }
        }

        let mut max = -MATE_SCORE;
        let mut moves = position.legal_moves();
        self.sort_moves(&mut moves, &tt_entry.best_move);

        if moves.is_empty() {
            if is_check {
                // checkmate
                return -MATE_SCORE + ply as i32;
            } else {
                // stalemate
                return 0;
            }
        }

        let mut best_move = None;
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
                best_move = Some(mov);

                if is_root {
                    self.pv_line[0] = Some(mov);
                }

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
                best_move,
                score: max,
            };
        }

        max
    }

    #[inline(always)]
    fn static_eval(&mut self, position: &Chess) -> i32 {
        super::eval::evaluate(position, &mut self.rng)
    }

    pub fn is_out_of_time(&self) -> bool {
        if let Some(deadline) = self.deadline
            && Instant::now() > deadline
        {
            return true;
        }

        false
    }

    fn quiescence(&mut self, position: &Chess, alpha: &mut i32, beta: i32) -> i32 {
        let static_eval = self.static_eval(position);

        if self.is_out_of_time() {
            return static_eval;
        }

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
        if moves.is_empty() {
            return;
        }

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
                -super::eval::MMV_LVA[victim as usize - 1][m.role() as usize - 1]
            } else {
                100000 // aka show all normal moves last
            }
        });
    }
}
