use crate::history::MoveHistory;
use crate::search::eval::MATE_SCORE;
use crate::search::eval::MATE_THRESHOLD;

use negamax::{Negamax, TimeControl};

use shakmaty::{Chess, Color, Move, Position};
use std::time::{Duration, Instant};

pub mod eval;
pub mod negamax;

pub const DEFAULT_ALPHA: i32 = -MATE_SCORE;
pub const DEFAULT_BETA: i32 = MATE_SCORE;

// Helper function to print search info
fn print_info(
    depth: usize,
    best_score: i32,
    node_count: usize,
    duration: Duration,
    pv_line: &[Option<Move>],
    position: &Chess,
) {
    let nps = (node_count as f64 / duration.as_secs_f64()) as usize;
    let time_ms = duration.as_millis();

    let pv = pv_line
        .iter()
        .flatten()
        .map(|m| m.to_uci(position.castles().mode()).to_string())
        .collect::<Vec<_>>()
        .join(" ");

    if best_score.abs() >= MATE_THRESHOLD {  // safe mate detection range
        let mate_in = if best_score > 0 {
            (MATE_SCORE - best_score + 1) / 2
        } else {
            -(MATE_SCORE + best_score + 1) / 2
        };
        println!(
            "info depth {depth} score mate {mate_in} nodes {node_count} nps {nps} time {time_ms} pv {pv}"
        );
    } else {
        println!(
            "info depth {depth} score cp {best_score} nodes {node_count} nps {nps} time {time_ms} pv {pv}"
        );
    }
}

pub fn search(
    position: &Chess,
    history: Option<&MoveHistory>,
    max_depth: Option<usize>,
    max_nodes: Option<usize>,
    time: Option<u64>,
) -> Option<Move> {
    let max_depth = if max_nodes.is_some() || time.is_some() {
        100 // just a really high depth so we hit the other limits before it
    } else {
        max_depth.unwrap_or(crate::engine_options().get_number("Default Depth"))
    };
    let max_nodes = max_nodes.unwrap_or_default();
    let default_history = MoveHistory::default();
    let history = history.unwrap_or(&default_history);

    let mut negamax = Negamax::new();

    let start = Instant::now();

    if let Some(time) = time {
        negamax.set_time(time);
    }

    for depth in 1..=max_depth {
        let best_score = negamax.negamax(position, history, depth, 0, DEFAULT_ALPHA, DEFAULT_BETA, true);
        let duration = start.elapsed();

        print_info(
            depth,
            best_score,
            negamax.node_count,
            duration,
            &negamax.pv_line,
            position,
        );      

        if negamax.is_out_of_time() {
            break;
        }
        if max_nodes > 0 && negamax.node_count >= max_nodes {
            break;
        }
    }

    negamax.pv_line[0]
}

pub fn allocate_time(position: &Chess, time: &TimeControl, played_moves: u64) -> Option<u64> {
    let divisor = 30.max(60_u64.saturating_sub(played_moves));

    match position.turn() {
        Color::Black if time.b_time > 0 => Some((time.b_time / divisor) + (time.b_inc / 2)),
        Color::White if time.w_time > 0 => Some((time.w_time / divisor) + (time.w_inc / 2)),
        _ => None,
    }
}
