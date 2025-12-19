use shakmaty::{ByColor, ByRole, Chess, Color, Position, Role};

pub const PAWN_VALUE: i32 = 100;
pub const BISHOP_VALUE: i32 = 300;
pub const KNIGHT_VALUE: i32 = 300;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 10000;

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
    let mut count: i32 = 0;

    count += pieces.pawn as i32 * PAWN_VALUE;
    count += pieces.bishop as i32 * BISHOP_VALUE;
    count += pieces.knight as i32 * KNIGHT_VALUE;
    count += pieces.rook as i32 * ROOK_VALUE;
    count += pieces.queen as i32 * QUEEN_VALUE;

    count
}

pub fn get_piece_value(piece: Role) -> i32 {
    match piece {
        Role::Pawn => PAWN_VALUE,
        Role::Bishop => BISHOP_VALUE,
        Role::Knight => KNIGHT_VALUE,
        Role::Rook => ROOK_VALUE,
        Role::Queen => QUEEN_VALUE,
        Role::King => KING_VALUE,
    }
}
