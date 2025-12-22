use rand::{Rng, rngs::SmallRng};
use shakmaty::{Chess, Color, Position, Role, Square};

pub const MATE_SCORE: i32 = 100_000;

pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 320;
pub const BISHOP_VALUE: i32 = 330;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 20000;

#[rustfmt::skip]
const PAWN_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
   50, 50, 50, 50, 50, 50, 50, 50,
   10, 10, 20, 30, 30, 20, 10, 10,
    5,  5, 10, 25, 25, 10,  5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5, -5,-10,  0,  0,-10, -5,  5,
    5, 10, 10,-20,-20, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0
];

#[rustfmt::skip]
const KNIGHT_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];

#[rustfmt::skip]
const QUEEN_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

#[rustfmt::skip]
const KING_TABLE_MIDDLEGAME: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20
];

#[rustfmt::skip]
const KING_TABLE_ENDGAME: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50
];

const PIECE_TABLES: [[i32; 64]; 7] = [
    PAWN_TABLE,
    KNIGHT_TABLE,
    BISHOP_TABLE,
    ROOK_TABLE,
    QUEEN_TABLE,
    KING_TABLE_ENDGAME,
    KING_TABLE_MIDDLEGAME,
];

const PIECE_VALUES: [i32; 6] = [
    PAWN_VALUE,
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    KING_VALUE,
];

pub const MMV_LVA: [[i32; 6]; 6] = {
    let mut table = [[0; 6]; 6];

    let mut i = 0;
    while i < 6 {
        let mut j = 0;
        while j < 6 {
            let victim = PIECE_VALUES[i];
            let attacker = PIECE_VALUES[j];

            let score = victim * 10 - attacker;

            table[i][j] = score;

            j += 1;
        }

        i += 1;
    }

    table
};

pub fn evaluate(position: &Chess, rng: &mut SmallRng) -> i32 {
    let board = position.board();

    let mut white_score = 0;
    let mut black_score = 0;

    let mut game_phase = 0;

    let mut white_king_square = None;
    let mut black_king_square = None;

    for (square, piece) in board {
        let piece_value = PIECE_VALUES[piece.role as usize - 1];

        game_phase += match piece.role {
            Role::Bishop | Role::Knight => 3,
            Role::Rook => 5,
            Role::Queen => 10,
            _ => 0,
        };

        match piece.color {
            Color::White => {
                // we handle the king later
                if piece.role == Role::King {
                    white_king_square = Some(square);
                    continue;
                }

                // we flip for white because A1 = index 0, where as index 0 represents A8 in our piece tables
                let bonus = PIECE_TABLES[piece.role as usize - 1][square.flip_vertical() as usize];
                white_score += piece_value + bonus
            }
            Color::Black => {
                if piece.role == Role::King {
                    black_king_square = Some(square);
                    continue;
                }

                let bonus = PIECE_TABLES[piece.role as usize - 1][square as usize];
                black_score += piece_value + bonus
            }
        }
    }

    white_score += king_value(Color::White, white_king_square, game_phase);
    black_score += king_value(Color::Black, black_king_square, game_phase);

    let who2move = who2move_score(position.turn());

    ((white_score - black_score) * who2move) + rng.random_range(-10..=10)
}

fn king_value(color: Color, square: Option<Square>, game_phase: i32) -> i32 {
    let square = match color {
        Color::Black => square.unwrap() as usize,
        Color::White => square.unwrap().flip_vertical() as usize,
    };

    let square_bonus = (KING_TABLE_MIDDLEGAME[square] * game_phase
        + KING_TABLE_ENDGAME[square] * (64 - game_phase))
        / 64;

    KING_VALUE + square_bonus
}

fn who2move_score(color: shakmaty::Color) -> i32 {
    match color {
        Color::White => 1,
        Color::Black => -1,
    }
}
