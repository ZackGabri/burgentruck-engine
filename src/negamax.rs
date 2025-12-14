use shakmaty::{ByColor, ByRole, Chess, Color, Move, Position};
use std::time::Instant;

const DEFAULT_SEARCH_DEPTH: usize = 5;

pub fn search(position: &Chess, depth: Option<usize>) -> Option<Move> {
    let depth = depth.unwrap_or(DEFAULT_SEARCH_DEPTH);

    let mut best_line: Vec<Option<Move>> = vec![None; depth];
    let mut node_count = 0;

    let start = Instant::now();
    let best_score = negamax(position, depth, 0, &mut best_line, &mut node_count);
    let duration = start.elapsed();

    println!(
        "info depth {depth} score cp {best_score} nodes {node_count} nps {} time {} pv {}",
        (node_count as f64 / duration.as_secs_f64()) as usize,
        duration.as_millis(),
        best_line
            .iter()
            .filter_map(|x| x.map(|x| x.to_uci(position.castles().mode()).to_string()))
            .collect::<Vec<String>>()
            .join(" ")
    );

    dbg!(best_line[0])
}

pub fn negamax(
    position: &Chess,
    depth: usize,
    ply: usize,
    best_line: &mut Vec<Option<Move>>,
    node_count: &mut usize,
) -> i32 {
    if depth == 0 {
        return evaluate(position);
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
        let score = -negamax(&position, depth - 1, ply + 1, best_line, node_count);

        if score > max {
            max = score;
            let _ = best_line[ply].insert(mov);
        }

        *node_count += 1;
    }

    max
}

fn who2move_score(color: shakmaty::Color) -> i32 {
    match color {
        Color::White => 1,
        Color::Black => -1,
    }
}

pub fn evaluate(position: &Chess) -> i32 {
    let ByColor { white, black } = position.board().material();

    (count_material(white) - count_material(black)) * who2move_score(position.turn())
}

fn count_material(pieces: ByRole<u8>) -> i32 {
    let mut count: u32 = 0;

    count += pieces.pawn as u32 * 100;
    count += pieces.bishop as u32 * 300;
    count += pieces.knight as u32 * 300;
    count += pieces.rook as u32 * 500;
    count += pieces.queen as u32 * 900;

    count as _
}
