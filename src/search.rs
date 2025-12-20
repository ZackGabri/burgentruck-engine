use crate::{history::MoveHistory, search::negamax::TimeControl};
use negamax::Negamax;

use shakmaty::{Chess, Color, Move, Position};
use std::time::Instant;

pub mod eval;
pub mod negamax;

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
        let best_score = negamax.negamax(position, history, depth, 0, -69420, 69420);
        let duration = start.elapsed();

        println!(
            "info depth {depth} score cp {best_score} nodes {} nps {} time {} pv {}",
            negamax.node_count,
            (negamax.node_count as f64 / duration.as_secs_f64()) as usize,
            duration.as_millis(),
            negamax
                .pv_line
                .iter()
                .filter_map(|x| x.map(|x| x.to_uci(position.castles().mode()).to_string()))
                .collect::<Vec<String>>()
                .join(" ")
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
    let dividor = 30.max(60 - played_moves);

    match position.turn() {
        Color::Black if time.b_time > 0 => Some((time.b_time / dividor) + (time.b_inc / 2)),
        Color::White if time.w_time > 0 => Some((time.w_time / dividor) + (time.w_inc / 2)),
        _ => None,
    }
}
