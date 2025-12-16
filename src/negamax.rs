use shakmaty::{Chess, Move, Position};
use std::time::Instant;

const DEFAULT_SEARCH_DEPTH: usize = 5;

pub fn search(position: &Chess, depth: Option<usize>) -> Option<Move> {
    let depth = depth.unwrap_or(DEFAULT_SEARCH_DEPTH);

    let mut negamax = Negamax::new();

    let start = Instant::now();
    let best_score = negamax.negamax(position, depth, 0);
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

    dbg!(negamax.best_line[0])
}

// struct for shared data between every negamax run
pub struct Negamax {
    node_count: usize,
    best_line: Vec<Option<Move>>,
}

impl Negamax {
    fn new() -> Self {
        Self {
            node_count: 0,
            best_line: Vec::new(),
        }
    }

    fn negamax(&mut self, position: &Chess, depth: usize, ply: usize) -> i32 {
        if depth == 0 {
            return crate::eval::evaluate(position);
        }

        if position.is_insufficient_material() {
            return 0;
        }

        let mut max = -69420;
        let moves = position.legal_moves();

        if moves.is_empty() {
            if position.is_check() {
                // checkmate
                return -69420 + ply as i32;
            } else {
                // stalemate
                return 0;
            }
        }

        for mov in moves.into_iter() {
            let position = position.clone().play(mov).unwrap();
            let score = self.negamax(&position, depth - 1, ply + 1);

            if score > max {
                max = score;
                let _ = self.best_line[ply].insert(mov);
            }

            self.node_count += 1;
        }

        max
    }
}
