use shakmaty::{ByColor, ByRole, Chess, Color, Move, Position};

const DEFAULT_SEARCH_DEPTH: usize = 3;

pub fn search(position: &Chess, depth: Option<usize>) -> Option<Move> {
    let moves = position.legal_moves();

    let mut best_score = -69420;
    let mut best_move: Option<Move> = None;

    for mov in moves.into_iter() {
        let position = position.clone().play(mov).unwrap();
        let score = negamax(depth.unwrap_or(DEFAULT_SEARCH_DEPTH), position);

        if score > best_score {
            best_score = score;
            let _ = best_move.insert(mov);
        }
    }

    dbg!(best_move)
}

pub fn negamax(depth: usize, position: Chess) -> i32 {
    if depth == 0 {
        return evaluate(&position);
    }

    let mut max = -1;

    let moves = position.legal_moves();
    for mov in moves.into_iter() {
        let position = position.clone().play(mov).unwrap();
        let score = -negamax(depth - 1, position);

        if score > max {
            max = score;
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

    count_material(white) - count_material(black)
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
