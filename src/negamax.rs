use shakmaty::{ByColor, ByRole, Chess, Color, Move, Position};

const DEFAULT_SEARCH_DEPTH: usize = 3;

pub fn search(position: &Chess, depth: Option<usize>) -> Option<Move> {
    let depth = depth.unwrap_or(DEFAULT_SEARCH_DEPTH);

    let mut best_move: Option<Move> = None;
    let best_score = negamax(position, depth, 0, &mut best_move);

    dbg!(best_score);
    dbg!(best_move)
}

pub fn negamax(position: &Chess, depth: usize, ply: usize, best_move: &mut Option<Move>) -> i32 {
    if depth == 0 {
        return evaluate(position);
    }

    let is_root = ply == 0;
    let mut max = -69420;

    let moves = position.legal_moves();
    for mov in moves.into_iter() {
        let position = position.clone().play(mov).unwrap();
        let score = -negamax(&position, depth - 1, ply + 1, best_move);

        if score > max {
            max = score;
            if is_root {
                let _ = best_move.insert(mov);
            }
        }
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
    let mut count = 0;

    count += pieces.pawn;
    count += pieces.bishop * 3;
    count += pieces.knight * 3;
    count += pieces.rook * 5;
    count += pieces.queen * 9;

    count as _
}
