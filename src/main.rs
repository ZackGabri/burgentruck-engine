use std::io::{self, Write};
use std::sync::OnceLock;
use std::time::Duration;

use shakmaty::fen::Fen;
use shakmaty::uci::UciMove;
use shakmaty::{Chess, Position};

use crate::history::MoveHistory;
use crate::options::EngineOptions;
use crate::search::SearchOptions;

mod bench;
mod history;
mod options;
mod search;
mod transposition_table;

pub fn engine_options() -> &'static EngineOptions {
    static ENGINE_OPTIONS: OnceLock<EngineOptions> = OnceLock::new();

    ENGINE_OPTIONS.get_or_init(EngineOptions::default)
}

fn main() -> Result<(), anyhow::Error> {
    let mut pos = Chess::default();

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut history = MoveHistory::new();

    // bench
    if let Some(arg) = std::env::args().nth(1)
        && arg == "bench"
    {
        bench::bench()?;
        std::process::exit(0);
    }

    loop {
        let mut line = String::new();
        stdin.read_line(&mut line).unwrap();

        let start_time = minstant::Instant::now();

        let tokens = line.split_ascii_whitespace().collect::<Vec<_>>();

        match tokens.as_slice() {
            ["uci", ..] => {
                println!("id name Bürgentruck {}", env!("CARGO_PKG_VERSION"));
                println!("id author ZackGabri & TampliteSiphronKents");
                println!();
                EngineOptions::print_defaults();

                println!("uciok");
            }
            ["setoption", args @ ..] => {
                if let ["name", args @ ..] = args {
                    let mut split = args.split(|x| *x == "value");
                    let name = split.next().unwrap_or(&[""]).join(" ");
                    let value = split.next().unwrap_or(&[""]).join(" ");

                    let result = engine_options().set(name, value);
                    if let Err(err) = result {
                        println!("{err:?}")
                    }
                };
            }

            ["isready", ..] => println!("readyok"),
            ["ucinewgame", ..] => {}

            ["position"] => {}
            ["position", pos_type, args @ ..] => match *pos_type {
                "startpos" => {
                    history.reset();
                    pos = Chess::default();

                    if args.first().copied() == Some("moves") {
                        let moves = args.get(1..).unwrap_or_default();

                        for m in moves {
                            let uci: UciMove = m.trim().parse()?;
                            let m = uci.to_move(&pos).unwrap();

                            pos.play_unchecked(m);
                            history.push_position(&pos);
                        }
                        if !moves.is_empty() {
                            history.pop(); // remove last position to avoid doubling it when running negamax
                        }
                    }
                }
                "fen" => {
                    history.reset();

                    let args: String = args.join(" ");
                    let split: Vec<&str> = args.split("moves").collect();

                    let fen: Fen = split.first().copied().unwrap_or_default().parse()?;
                    pos = fen.into_position(pos.castles().mode())?;
                    history.push_position(&pos);

                    let moves = split.get(1..).unwrap_or_default();
                    for m in moves {
                        let uci: UciMove = m.trim().parse()?;
                        let m = uci.to_move(&pos)?;

                        pos.play_unchecked(m);
                        history.push_position(&pos);
                    }
                    if !moves.is_empty() {
                        history.pop(); // remove last position to avoid doubling it when running negamax
                    }
                }
                _ => {}
            },

            ["go", "movetime", val] => {
                let search_options = SearchOptions {
                    deadline: val
                        .parse()
                        .map(|val| minstant::Instant::now() + Duration::from_millis(val))
                        .ok(),
                    ..Default::default()
                };
                let (m, _) = search::search(&pos, Some(&history), search_options);

                if let Some(best_move) = m {
                    println!("bestmove {}", best_move.to_uci(pos.castles().mode()));
                }
            }
            ["go", args @ ..] => {
                let search_args = args.chunks(2);

                let mut time = search::TimeControl::default();
                let mut max_depth = None;
                let mut max_nodes = None;

                for arg in search_args {
                    match arg {
                        ["nodes", nodes] => max_nodes = nodes.parse().ok(),
                        ["depth", depth] => max_depth = depth.parse().ok(),

                        ["wtime", wtime] => time.w_time = wtime.parse().unwrap_or_default(),
                        ["btime", btime] => time.b_time = btime.parse().unwrap_or_default(),
                        ["winc", inc] => time.w_inc = inc.parse().unwrap_or_default(),
                        ["binc", inc] => time.b_inc = inc.parse().unwrap_or_default(),
                        _ => {}
                    }
                }

                let search_options = SearchOptions {
                    deadline: time.into_deadline(
                        start_time,
                        pos.turn(),
                        pos.fullmoves().get() as u64,
                    ),
                    max_depth,
                    max_nodes,
                    ..Default::default()
                };

                let (m, _) = search::search(&pos, Some(&history), search_options);

                if let Some(best_move) = m {
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
            ["bench", ..] => {
                if let Err(err) = bench::bench() {
                    println!("Bench Failed! {:?}", err);
                }
            }

            [] => {}
            ["quit", ..] => break,
            _ => println!("Unkown Command: {line}"),
        }

        stdout.flush()?;
    }

    Ok(())
}
