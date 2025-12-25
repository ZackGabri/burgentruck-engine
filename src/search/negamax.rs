use std::sync::OnceLock;
use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use shakmaty::zobrist::ZobristHash;
use shakmaty::{Chess, Move, MoveList, Position};

use crate::engine_options;
use crate::history::MoveHistory;
use crate::search::{MATE_SCORE, MATE_THRESHOLD, MAX_PLY, MAX_PSUEDO_MOVES};
use crate::transposition_table::{TTBound, TTEntry, get_ttindex};

pub struct PVariation {
    pub length: usize,
    pub line: [Option<Move>; MAX_PLY],
}

impl Default for PVariation {
    fn default() -> Self {
        Self {
            length: 0,
            line: [None; MAX_PLY],
        }
    }
}

impl std::fmt::Display for PVariation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        for i in 0..self.length {
            if let Some(m) = self.line[i] {
                output += &m.to_uci(shakmaty::CastlingMode::Standard).to_string();

                // add spaces between unless it's last move
                if i != self.length - 1 {
                    output += " ";
                }
            }
        }

        write!(f, "{output}")
    }
}

// Late Move Reductions table
static LMR_TABLE: OnceLock<[[usize; MAX_PSUEDO_MOVES]; MAX_PLY]> = OnceLock::new();
fn init_lmr_table() -> [[usize; MAX_PSUEDO_MOVES]; MAX_PLY] {
    let mut table = [[1; MAX_PSUEDO_MOVES]; MAX_PLY];

    let mut depth = 3;
    while depth < MAX_PLY {
        let mut move_num = 4;
        while move_num < MAX_PSUEDO_MOVES {
            table[depth][move_num] =
                (0.50 + (depth as f64).ln() * (move_num as f64).ln() / 3.0) as usize;

            move_num += 1;
        }
        depth += 1;
    }

    table
}

// struct for shared data between every negamax call
pub struct Negamax {
    pub node_count: usize,
    transposition_table: Box<[TTEntry]>,
    ttable_length: usize,
    ttable_entries: usize,

    history_table: [[[i32; 64]; 64]; 2],
    lmr_table: &'static [[usize; MAX_PSUEDO_MOVES]],

    rng: SmallRng,
    pub deadline: Option<Instant>, // deadline for the search to stop at
}

impl Negamax {
    pub fn new() -> Self {
        // convert from megabytes to bytes
        let desired_size = engine_options().get_number("Hash") * 1024 * 1024;
        let tt_entry_size = size_of::<TTEntry>();

        // calculate how many entries will fit in that memory size
        let tt_table_length = desired_size / tt_entry_size;

        Self {
            node_count: 0,
            transposition_table: vec![TTEntry::default(); tt_table_length].into_boxed_slice(),
            ttable_length: tt_table_length,
            ttable_entries: 0,

            history_table: [[[0; 64]; 64]; 2],
            lmr_table: LMR_TABLE.get_or_init(init_lmr_table),

            rng: SmallRng::from_seed([0; 32]),

            deadline: None,
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
        pv: &mut PVariation,
        pv_node: bool,
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

        let tt_index = get_ttindex(hash, self.ttable_length);
        let tt_entry = self.transposition_table[tt_index];
        let replace_tt = tt_entry.depth < depth || tt_entry.hash != hash;

        // tt probing and cutoffs
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
                        pv,
                        false,
                    );

                    // Make sure it's not a mate score
                    if null_score >= beta && null_score.abs() < MATE_THRESHOLD {
                        return null_score;
                    }
                }
            }
        }

        let mut max = -MATE_SCORE;
        let moves = self.get_sorted_moves(position, &tt_entry.best_move);

        if moves.is_empty() {
            if is_check {
                // checkmate
                return -MATE_SCORE + ply as i32;
            } else {
                // stalemate
                return 0;
            }
        }

        pv.length = 0;
        let mut best_move = None;
        for (move_index, mov) in moves.into_iter().enumerate() {
            let position = position.clone().play(mov).unwrap();
            self.node_count += 1;

            let mut score = -MATE_SCORE;
            let mut child_pv = PVariation::default();

            if depth >= 3 && move_index >= 4 && max.abs() < MATE_THRESHOLD {
                // Late Move Reductions (LMR)
                let depth_reduction = self.lmr_table[depth][move_index];
                let reduced_depth = depth.saturating_sub(1 + depth_reduction); // depth - 1 - r

                // do reduced search with a null window
                score = -self.negamax(
                    &position,
                    &history.clone(),
                    reduced_depth,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    &mut child_pv,
                    false,
                );

                // if it fails then do a full research instead
                if score > alpha {
                    score = -self.negamax(
                        &position,
                        &history.clone(),
                        depth - 1,
                        ply + 1,
                        -alpha - 1,
                        -alpha,
                        &mut child_pv,
                        false,
                    );
                }
            }
            // Principal variation search (PVS)
            // If we are in a non-PV node, OR we are in a PV-node examining moves after the 1st legal move
            else if !pv_node || move_index > 0 {
                // Perform zero-window search (ZWS) on non-PV nodes
                score = -self.negamax(
                    &position,
                    &history.clone(),
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    &mut child_pv,
                    false,
                );
            }

            // We are in a PV node and either it's the first legal move, OR the ZWS failed high
            if pv_node && (move_index == 0 || score > alpha) {
                score = -self.negamax(
                    &position,
                    &history.clone(),
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    &mut child_pv,
                    true,
                );
            }

            if score > max {
                max = score;
                best_move = Some(mov);

                if score > alpha {
                    pv.length = 1 + child_pv.length;
                    pv.line[0] = Some(mov);
                    pv.line[1..pv.length + 1].copy_from_slice(&child_pv.line[0..pv.length]);

                    alpha = score;
                }
            }

            if score >= beta {
                if mov.capture().is_none() {
                    let from = match mov.from() {
                        Some(v) => v,
                        None => break,
                    } as usize;
                    let to = mov.to() as usize;
                    let turn = position.turn() as usize;

                    self.history_table[turn][from][to] += (depth * depth) as i32;
                }

                break;
            }
        }

        if replace_tt {
            if tt_entry.hash == 0.into() {
                self.ttable_entries += 1;
            }

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

    fn get_sorted_moves(&self, position: &Chess, hash_move: &Option<Move>) -> MoveList {
        let mut move_list = position.legal_moves();
        let turn = position.turn() as usize;

        move_list.sort_by_key(|m| {
            if let Some(hash_move) = hash_move
                && m == hash_move
            {
                return -9999999; // hash moves always first
            }

            // if the move is a capture then sort it based on the MMV-LVA table
            if let Some(victim) = m.capture() {
                // +50000 so it's ahead of the normal moves
                return -(super::eval::MMV_LVA[victim as usize - 1][m.role() as usize - 1] + 50000);
            };

            // otherwise sort it based on the history table
            let from = match m.from() {
                Some(v) => v,
                None => return 0,
            } as usize;
            let to = m.to() as usize;
            -self.history_table[turn][from][to]
        });

        move_list
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

    pub fn hashfull(&self) -> usize {
        (self.ttable_entries * 1000) / self.ttable_length
    }
}
