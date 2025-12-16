use std::io::{self, Write};

use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::{Chess, Position};

mod eval;
mod negamax;
mod transposition_table;

fn main() -> Result<(), anyhow::Error> {
    let mut pos = Chess::default();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        let tokens = line.split_ascii_whitespace().collect::<Vec<_>>();

        match tokens.as_slice() {
            ["uci", ..] => {
                println!("id name Bürgentruck");
                println!("id author ZackGabri & TampliteSiphronKents");
                println!("uciok");
            }
            ["isready", ..] => println!("readyok"),
            ["setoption", _args @ ..] => {}
            ["ucinewgame", ..] => {}

            ["position"] => {}
            ["position", pos_type, args @ ..] => match *pos_type {
                "startpos" => {
                    pos = Chess::default();

                    if args.first().copied() == Some("moves") {
                        let moves = args.get(1..).unwrap_or_default();

                        dbg!(moves);
                        for m in moves {
                            let uci: UciMove = m.trim().parse()?;
                            let m = uci.to_move(&pos).unwrap();

                            pos.play_unchecked(m);
                        }
                    }
                }
                "fen" => {
                    let args: String = args.join(" ");
                    let split: Vec<&str> = args.split("moves").collect();

                    let fen: Fen = split.first().copied().unwrap_or_default().parse()?;
                    pos = fen.into_position(pos.castles().mode())?;

                    let moves = split.get(1..).unwrap_or_default();
                    for m in moves {
                        let uci: UciMove = m.trim().parse()?;
                        let m = uci.to_move(&pos)?;

                        pos.play_unchecked(m);
                    }
                }
                _ => {}
            },

            ["go", _args @ ..] => {
                // let moves = pos.legal_moves();
                // let best_move = moves.first();

                let best_move = negamax::search(&pos, None);

                if let Some(best_move) = best_move {
                    println!("bestmove {}", best_move.to_uci(pos.castles().mode()));
                }
            }
            ["stop", ..] => {}

            ["d" | "display", ..] => {
                let mut board: [[char; 8]; 8] = [['.'; 8]; 8];
                pos.board().iter().for_each(|(square, piece)| {
                    let (file, rank) = square.coords();
                    board[rank as usize][file as usize] = piece.char();
                });

                board.reverse();

                board.iter().for_each(|chunk| {
                    println!(
                        "{}",
                        chunk
                            .iter()
                            .map(|c| format!("{c}"))
                            .collect::<Vec<String>>()
                            .join(" ")
                    );
                });
                println!("Fen: {}", pos.board().board_fen());
            }

            [] => {}
            ["quit", ..] => break,
            _ => println!("Unkown Command: {line}"),
        }

        stdout.flush()?;
    }

    Ok(())
}
