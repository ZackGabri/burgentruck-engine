use shakmaty::{ByColor, ByRole, Chess, Color, Position};

pub fn evaluate(position: &Chess) -> i32 {
    let ByColor { white, black } = position.board().material();
    let who2move = who2move_score(position.turn());

    ((count_material(white) - count_material(black)) * who2move) + rand::random_range(-10..=10)
}

fn who2move_score(color: shakmaty::Color) -> i32 {
    match color {
        Color::White => 1,
        Color::Black => -1,
    }
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
