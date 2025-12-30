use std::sync::OnceLock;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use shakmaty::zobrist::ZobristHash;
use shakmaty::{Chess, Move, MoveList, Position};

use crate::history::MoveHistory;
use crate::search::{MATE_SCORE, MAX_PLY};
use crate::transposition_table::{self, TTBound, TTEntry};

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

#[derive(Default, Clone, Copy)]
struct KillerMoves {
    moves: [Option<Move>; 2],
}

impl KillerMoves {
    #[inline(always)]
    fn insert_move(&mut self, m: Move) {
        if self.moves[0] == Some(m) {
            return;
        }
        self.moves[1] = self.moves[0];
        self.moves[0] = Some(m);
    }

    #[inline(always)]
    fn contains_move(&self, m: Move) -> bool {
        self.moves[0] == Some(m) || self.moves[1] == Some(m)
    }
}

pub const MAX_PSUEDO_MOVES: usize = 280;

// Late Move Reductions table
static LMR_TABLE: OnceLock<[[usize; MAX_PSUEDO_MOVES]; MAX_PLY]> = OnceLock::new();
fn get_lmr_table() -> &'static [[usize; MAX_PSUEDO_MOVES]; MAX_PLY] {
    LMR_TABLE.get_or_init(|| {
        let mut table = [[0; MAX_PSUEDO_MOVES]; MAX_PLY];

        let mut depth = 1;
        while depth < MAX_PLY {
            let mut move_num = 1;
            while move_num < MAX_PSUEDO_MOVES {
                table[depth][move_num] =
                    (0.50 + (depth as f64).ln() * (move_num as f64).ln() / 3.00) as usize;

                move_num += 1;
            }
            depth += 1;
        }

        table
    })
}

// struct for shared data between every negamax call
pub struct Negamax {
    pub node_count: usize,
    history_table: [[[i32; 64]; 64]; 2],
    killer_move_table: [KillerMoves; MAX_PLY],

    #[allow(unused)]
    rng: SmallRng,

    pub deadline: Option<minstant::Instant>, // deadline for the search to stop at
}

impl Negamax {
    pub fn new() -> Self {
        Self {
            node_count: 0,
            history_table: [[[0; 64]; 64]; 2],
            killer_move_table: [KillerMoves::default(); MAX_PLY],

            rng: SmallRng::from_seed([0; 32]),

            deadline: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn negamax(
        &mut self,
        position: &Chess,
        history: &MoveHistory,
        mut depth: usize,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        pv: &mut PVariation,
        pv_node: bool,
    ) -> i32 {
        if depth == 0 {
            return self.quiescence(position, &mut alpha, beta);
        }

        if depth > 1 && self.is_out_of_time() {
            return -alpha;
        }

        let original_alpha = alpha;
        let hash = position.zobrist_hash(shakmaty::EnPassantMode::Legal);

        let mut history = *history;
        history.push_hash(hash);

        // threefold detection
        let count = history.count_item(&hash);
        if count >= 2 && ply > 0
            || position.is_insufficient_material()
            || position.halfmoves() >= 100
        {
            return 0;
        }

        let is_check = position.is_check();
        let is_root = ply == 0;

        let tt_entry = transposition_table::get(hash);
        let tt_hit = tt_entry.hash == hash;
        let replace_tt = tt_entry.depth < depth || !tt_hit;

        pv.length = 0; // ensure fresh pv
        // tt probing and cutoffs
        if tt_entry.depth >= depth && tt_entry.hash == hash {
            match tt_entry.bound {
                TTBound::Exact => {
                    pv.line[pv.length] = tt_entry.best_move;
                    pv.length += 1;
                    return tt_entry.score;
                }
                TTBound::Lower => {
                    if tt_entry.score >= beta {
                        pv.line[pv.length] = tt_entry.best_move;
                        pv.length += 1;
                        return tt_entry.score;
                    }
                }
                TTBound::Upper => {
                    if tt_entry.score < alpha {
                        pv.line[pv.length] = tt_entry.best_move;
                        pv.length += 1;
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
                    if null_score >= beta && null_score.abs() < 69000 {
                        return null_score;
                    }
                }
            }
        }

        // Internal iterative reductions (IIR)
        // If the position has not been searched yet (i.e. no hash move), we try searching with reduced
        // depth to record a move that we can later re-use.
        if depth >= 6 
            && !is_check 
            && pv_node
            && (!tt_hit || tt_entry.depth <= depth - 5)
        {
            depth -= 1;
        }

        let mut max = -MATE_SCORE;
        let moves = self.get_sorted_moves(position, ply, &tt_entry.best_move);

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
        for (move_index, (_, mov)) in moves.into_iter().enumerate() {
            if depth > 1 && self.is_out_of_time() {
                return alpha;
            }

            let is_quiet = !mov.is_capture() && !mov.is_promotion();

            // Move loop pruning
            if !is_root && !pv_node && is_quiet && max.abs() < crate::search::MATE_THRESHOLD {
                // Futility pruning
                if depth <= 3 && move_index >= 4 {
                    let static_eval = self.static_eval(position);

                    // Discard moves with no potential to raise alpha
                    if static_eval + 300 * depth as i32 <= alpha {
                        continue;
                    }
                }
            }

            let mut position = position.clone();
            position.play_unchecked(mov);

            self.node_count += 1;

            let mut score = -MATE_SCORE;
            let mut child_pv = PVariation::default();

            // Principal variation search (PVS)
            // If we are in a non-PV node, OR we are in a PV-node examining moves after the 1st legal move
            if !pv_node || move_index > 0 {
                let lmr_conditions = !is_check
                    && depth >= 3
                    && move_index >= 3
                    && is_quiet
                    && !self.killer_move_table[ply].contains_move(mov);

                let lmr_reduction = if lmr_conditions {
                    get_lmr_table()[depth][move_index]
                } else {
                    0
                };

                // Perform zero-window search (ZWS) on non-PV nodes
                score = -self.negamax(
                    &position,
                    &history.clone(),
                    depth - lmr_reduction - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    &mut child_pv,
                    false,
                );

                // if LMR fails high then do a full search instead
                if lmr_reduction > 0 && score > alpha {
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
                // store quiet moves
                if is_quiet {
                    let from = match mov.from() {
                        Some(v) => v,
                        None => break,
                    } as usize;
                    let to = mov.to() as usize;
                    let turn = position.turn() as usize;

                    self.history_table[turn][from][to] += (depth * depth) as i32;
                    self.store_killer_move(ply, mov);
                }

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

            transposition_table::put(
                hash,
                TTEntry {
                    hash,
                    depth,
                    bound,
                    best_move,
                    score: max,
                },
            );
        }

        max
    }

    #[inline(always)]
    fn static_eval(&mut self, position: &Chess) -> i32 {
        super::eval::evaluate(position)
    }

    pub fn is_out_of_time(&self) -> bool {
        if let Some(deadline) = self.deadline
            && minstant::Instant::now() > deadline
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
            let mut position = position.clone();
            position.play_unchecked(capture);

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

    fn get_sorted_moves(
        &self,
        position: &Chess,
        ply: usize,
        hash_move: &Option<Move>,
    ) -> Vec<(i32, Move)> {
        let move_list = position.legal_moves();
        let turn = position.turn() as usize;
        let killers = self.killer_move_table[ply];

        let mut scored: Vec<(i32, Move)> = Vec::with_capacity(move_list.len());
        for m in move_list {
            let score = if Some(m) == *hash_move {
                2_000_000_000 // Hash move is highest priority
            } else if let Some(victim) = m.capture() {
                1_000_000_000 + super::eval::MVV_LVA[victim as usize - 1][m.role() as usize - 1]
            } else if killers.contains_move(m) {
                1_000_000_000 // Same value as MVV-LVA so it's above bad captures but below good ones
            } else {
                // History moves
                self.history_table[turn][m.from().unwrap() as usize][m.to() as usize]
            };

            scored.push((score, m));
        }

        // sort the scores
        scored.sort_unstable_by_key(|&(s, _)| -s);
        scored
    }

    fn sort_captures(&self, moves: &mut MoveList) {
        moves.sort_by_key(|m| {
            if let Some(victim) = m.capture() {
                -super::eval::MVV_LVA[victim as usize - 1][m.role() as usize - 1]
            } else {
                100000 // aka show all normal moves last
            }
        });
    }

    fn store_killer_move(&mut self, ply: usize, m: Move) {
        if !self.killer_move_table[ply].contains_move(m) {
            self.killer_move_table[ply].insert_move(m);
        }
    }
}
