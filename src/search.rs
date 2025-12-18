use crate::history::MoveHistory;
use negamax::Negamax;

use shakmaty::{Chess, Move, Position};
use std::time::Instant;

mod eval;
mod negamax;

const DEFAULT_SEARCH_DEPTH: usize = 5;

pub fn search(
    position: &Chess,
    history: Option<&MoveHistory>,
    max_depth: Option<usize>,
    max_nodes: Option<usize>,
) -> Option<Move> {
    let max_depth = max_depth.unwrap_or(DEFAULT_SEARCH_DEPTH);
    let max_nodes = max_nodes.unwrap_or_default();

    let default_history = MoveHistory::default();
    let history = history.unwrap_or(&default_history);

    let mut negamax = Negamax::new();
    println!("info history size {:?}", history.index);

    let start = Instant::now();
    for depth in 0..=max_depth {
        let best_score = negamax.negamax(position, history, depth, 0, -69420, 69420);
        let duration = start.elapsed();

        println!(
            "info depth {depth} score cp {best_score} nodes {} nps {} time {} pv {}",
            negamax.node_count,
            (negamax.node_count as f64 / duration.as_secs_f64()) as usize,
            duration.as_millis(),
            negamax
                .best_line
                .iter()
                .filter_map(|x| x.map(|x| x.to_uci(position.castles().mode()).to_string()))
                .collect::<Vec<String>>()
                .join(" ")
        );

        if max_nodes > 0 && negamax.node_count >= max_nodes {
            break;
        }
    }

    dbg!(negamax.best_line[0])
}
