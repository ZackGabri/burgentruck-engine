use crate::{history::MoveHistory, search::negamax::PVariation, transposition_table};
use negamax::Negamax;

use shakmaty::{Chess, Color, Move, Position};
use std::time::Duration;

pub mod eval;
pub mod negamax;

pub const MATE_SCORE: i32 = 100_000;
pub const MATE_THRESHOLD: i32 = MATE_SCORE - 1000;
pub const MAX_PLY: usize = 128;

// Helper function to print search info
fn print_info(
    depth: usize,
    best_score: i32,
    duration: Duration,
    negamax: &Negamax,
    pv: &PVariation,
) {
    let Negamax { node_count, .. } = negamax;

    let hashfull = transposition_table::hashfull();
    let nps = (*node_count as f64 / duration.as_secs_f64()) as usize;
    let time_ms = duration.as_millis();

    let score_string = if best_score.abs() >= MATE_THRESHOLD {
        // safe mate detection range
        let mate_in = if best_score > 0 {
            (MATE_SCORE - best_score + 1) / 2
        } else {
            -(MATE_SCORE + best_score + 1) / 2
        };
        format!("mate {mate_in}")
    } else {
        format!("cp {best_score}")
    };

    println!(
        "info depth {depth} score {score_string} nodes {node_count} nps {nps} hashfull {hashfull} time {time_ms} pv {pv}"
    );
}

#[derive(Default)]
pub struct SearchOptions {
    pub max_depth: Option<usize>,
    pub max_nodes: Option<usize>,
    pub deadline: Option<minstant::Instant>,
    pub bench: bool,
}

pub fn search(
    position: &Chess,
    history: Option<&MoveHistory>,
    search_options: SearchOptions,
) -> (Option<Move>, usize) {
    let SearchOptions {
        max_nodes,
        max_depth,
        deadline,
        bench,
    } = search_options;

    let mut negamax = Negamax::new();
    negamax.deadline = deadline;

    let max_depth = max_depth.unwrap_or_else(|| {
        if max_nodes.is_some() || deadline.is_some() {
            100
        } else {
            crate::engine_options().get_number("Default Depth")
        }
    });

    let max_nodes = max_nodes.unwrap_or_default();
    let default_history = MoveHistory::default();
    let history = history.unwrap_or(&default_history);

    let start = minstant::Instant::now();

    // aspiration window variables
    let mut alpha = -MATE_SCORE;
    // let mut alpha_window = 50;
    let mut beta = MATE_SCORE;
    // let mut beta_window = 50;
    let window_size = 50;
    let mut delta = window_size;

    let mut depth = 1;

    let mut pv = PVariation::default();
    while depth <= max_depth {
        if negamax.is_out_of_time() && depth > 1 {
            break;
        }

        let score = negamax.negamax(position, history, depth, 0, alpha, beta, &mut pv, true);

        if depth > 3 {
            if score <= alpha {
                // alpha -= alpha_window;
                // alpha_window *= 2;
                beta = (alpha + beta) / 2;
                alpha = std::cmp::max(-MATE_SCORE, alpha - delta);
                delta += delta / 2;
                continue;
            }
            if score >= beta {
                // beta += beta_window;
                // beta_window *= 2;
                beta = std::cmp::min(MATE_SCORE, beta + delta);
                delta += delta / 2;
                continue;
            }

            delta = window_size;
            alpha = std::cmp::max(-MATE_SCORE, score - delta);
            beta = std::cmp::min(score + delta, MATE_SCORE);
        }

        if !bench {
            let duration = start.elapsed();
            print_info(depth, score, duration, &negamax, &pv);
        }

        if negamax.is_out_of_time() {
            break;
        }
        if max_nodes > 0 && negamax.node_count >= max_nodes {
            break;
        }

        depth += 1;
    }

    // if negamax failed for whatever reason, then just play the first move in the position
    if pv.line[0].is_none() {
        pv.line[0] = position.legal_moves().first().copied();
        // just print invalid pv so engines can warn or smth
        println!("info depth 0 score cp 0 nodes 0 nps 0 hashfull -1 time -1 pv e2e2 e2e1 e1e2");
    }

    (pv.line[0], negamax.node_count)
}

#[derive(Default, Debug)]
pub struct TimeControl {
    pub w_time: u64,
    pub b_time: u64,
    pub w_inc: u64,
    pub b_inc: u64,
}

impl TimeControl {
    pub fn into_deadline(
        self,
        start_time: minstant::Instant,
        turn: Color,
        played_moves: u64,
    ) -> Option<minstant::Instant> {
        let divisor = 30.max(60_u64.saturating_sub(played_moves));

        let duration = match turn {
            Color::Black if self.b_time > 0 => (self.b_time / divisor) + (self.b_inc / 2),
            Color::White if self.w_time > 0 => (self.w_time / divisor) + (self.w_inc / 2),
            _ => 0,
        };

        if duration > 0 {
            Some(start_time + Duration::from_millis(duration))
        } else {
            None
        }
    }
}
