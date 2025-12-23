use crate::history::MoveHistory;
use negamax::{Negamax, TimeControl};

use shakmaty::{Chess, Color, Move, Position};
use std::time::Instant;

pub mod eval;
pub mod negamax;

pub const MATE_SCORE: i32 = 100_000;

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

        let score = negamax.negamax(position, history, depth, 0, -MATE_SCORE, MATE_SCORE);

        let duration = start.elapsed();

        // bench has it's own printing so we don't want it getting cluttered by info prints
        if !bench {
            println!(
                "info depth {depth} score cp {score} nodes {} nps {} hashfull {} time {} pv {}",
                negamax.node_count,
                (negamax.node_count as f64 / duration.as_secs_f64()) as usize,
                negamax.hashfull(),
                duration.as_millis(),
                negamax
                    .pv_line
                    .iter()
                    .filter_map(|x| x.map(|x| x.to_uci(position.castles().mode()).to_string()))
                    .collect::<Vec<String>>()
                    .join(" ")
            );
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
