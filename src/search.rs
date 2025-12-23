use crate::history::MoveHistory;
use negamax::{Negamax, TimeControl};

use shakmaty::{Chess, Color, Move, Position};
use std::time::{Duration, Instant};

pub mod eval;
pub mod negamax;

pub const MATE_SCORE: i32 = 100_000;
pub const MATE_THRESHOLD: i32 = MATE_SCORE - 1000;

// Helper function to print search info
fn print_info(
    depth: usize,
    best_score: i32,
    duration: Duration,
    position: &Chess,
    negamax: &Negamax,
) {
    let Negamax {
        node_count,
        pv_line,
        ..
    } = negamax;

    let hashfull = negamax.hashfull();
    let nps = (*node_count as f64 / duration.as_secs_f64()) as usize;
    let time_ms = duration.as_millis();

    let pv = pv_line
        .iter()
        .flatten()
        .map(|m| m.to_uci(position.castles().mode()).to_string())
        .collect::<Vec<_>>()
        .join(" ");

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
    pub max_time: Option<u64>,
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
        max_time,
        bench,
    } = search_options;

    let max_depth = max_depth.unwrap_or_else(|| {
        if max_nodes.is_some() || max_time.is_some() {
            100
        } else {
            crate::engine_options().get_number("Default Depth")
        }
    });

    let max_nodes = max_nodes.unwrap_or_default();
    let default_history = MoveHistory::default();
    let history = history.unwrap_or(&default_history);

    let mut negamax = Negamax::new();
    let start = Instant::now();

    if let Some(time) = max_time {
        negamax.set_time(time);
    }

    for depth in 1..=max_depth {
        if negamax.is_out_of_time() {
            break;
        }

        let score = negamax.negamax(position, history, depth, 0, -MATE_SCORE, MATE_SCORE, true);

        let duration = start.elapsed();

        if !bench {
            print_info(depth, score, duration, position, &negamax);
        }

        if negamax.is_out_of_time() {
            break;
        }
        if max_nodes > 0 && negamax.node_count >= max_nodes {
            break;
        }
    }

    (negamax.pv_line[0], negamax.node_count)
}

pub fn allocate_time(position: &Chess, time: &TimeControl, played_moves: u64) -> Option<u64> {
    let divisor = 30.max(60_u64.saturating_sub(played_moves));

    match position.turn() {
        Color::Black if time.b_time > 0 => Some((time.b_time / divisor) + (time.b_inc / 2)),
        Color::White if time.w_time > 0 => Some((time.w_time / divisor) + (time.w_inc / 2)),
        _ => None,
    }
}
