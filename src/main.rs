use std::collections::VecDeque;

use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::{Chess, Position};

fn main() -> Result<(), anyhow::Error> {
    let mut pos = Chess::default();

    let stdin = std::io::stdin();
    loop {
        let mut raw_input = String::new();
        stdin.read_line(&mut raw_input).unwrap();

        let mut args: VecDeque<&str> = raw_input.split_ascii_whitespace().collect();

        match args.pop_front().unwrap_or("") {
            "uci" => {
                println!("id name Bürgentruck");
                println!("id author ZackGabri & TampliteSiphronKents");
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "setoption" => {}
            "ucinewgame" => {}

            "position" => match args.pop_front().unwrap_or("") {
                "startpos" => {
                    pos = Chess::default();

                    if args.pop_front() == Some("moves") {
                        let moves = Vec::from(args).into_iter().map(|x| x.trim());

                        for m in moves {
                            let uci: UciMove = m.parse()?;
                            let m = uci.to_move(&pos).unwrap();

                            pos.play_unchecked(m);
                        }
                    }
                }
                "fen" => {
                    let args: String = Vec::from(args).join(" ");
                    let mut split: VecDeque<&str> = args.split("moves").collect();

                    let fen: Fen = split.pop_front().unwrap_or_default().parse()?;
                    pos = fen.into_position(pos.castles().mode())?;

                    let moves = split
                        .pop_front()
                        .map(|moves| moves.split_ascii_whitespace().map(|x| x.trim()));

                    if let Some(moves) = moves {
                        for m in moves {
                            let uci: UciMove = m.parse()?;
                            let m = uci.to_move(&pos)?;

                            pos.play_unchecked(m);
                        }
                    }
                }
                _ => {}
            },

            "go" => {
                let moves = pos.legal_moves();
                let best_move = moves.first();

                if let Some(best_move) = best_move {
                    println!("bestmove {}", best_move.to_uci(pos.castles().mode()));
                }
            }
            "stop" => {}

            "d" | "display" => {
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

            "" => {}
            "quit" => break,
            _ => println!("Unkown Command: {raw_input}"),
        }
    }

    Ok(())
}
