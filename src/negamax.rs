use shakmaty::{Chess, Move, Position};

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::time::Instant;

const DEFAULT_SEARCH_DEPTH: usize = 5;
const THREAD_COUNT: usize = 1;

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

    // if we're in root then do multithreading, otherwise run normal to avoid recursive threads
    if ply == 0 && THREAD_COUNT > 1 {
        let mut threads = Vec::new();

        let moves = Arc::new(moves);
        let next = Arc::new(AtomicUsize::new(0));
        let shared_node_count = Arc::new(AtomicUsize::new(0));
        let shared_best = Arc::new(AtomicUsize::new(0));
        let shared_max = Arc::new(AtomicI32::new(-69420));

        for _ in 0..THREAD_COUNT {
            let moves_clone = moves.clone();
            let next_clone = next.clone();
            let position_clone = position.clone();
            let node_count_clone = shared_node_count.clone();
            let max_clone = shared_max.clone();
            let best_clone = shared_best.clone();

            threads.push(std::thread::spawn(move || {
                let mut thread_nodes = 0;

                // (index, score)
                let mut thread_best: (usize, i32) = (0, -69420);
                loop {
                    let i = next_clone.clone().fetch_add(1, Ordering::Relaxed);
                    if i >= moves_clone.len() {
                        break;
                    }

                    let move_to_search = moves_clone[i];
                    let position = position_clone.clone().play(move_to_search).unwrap();

                    let mut best_line: Vec<Option<Move>> = vec![None; depth];

                    let score = -negamax(
                        &position,
                        depth - 1,
                        ply + 1,
                        &mut best_line,
                        &mut thread_nodes,
                    );

                    if score > thread_best.1 {
                        thread_best = (i, score);
                    }

                    thread_nodes += 1;
                }

                node_count_clone.fetch_add(thread_nodes, Ordering::Relaxed);
                if thread_best.1 > (*max_clone).load(Ordering::Relaxed) {
                    best_clone.store(thread_best.0, Ordering::Relaxed);
                    max_clone.store(thread_best.1, Ordering::Relaxed);
                }
            }));
        }

        // wait for all threads to finish
        for thread in threads {
            thread.join().unwrap();
        }

        max = shared_max.load(Ordering::Relaxed);
        *node_count = shared_node_count.load(Ordering::Relaxed);
        best_line[0] = Some(moves[shared_best.load(Ordering::Relaxed)]);
    } else {
        for mov in moves.into_iter() {
            let position = position.clone().play(mov).unwrap();
            let score = -negamax(&position, depth - 1, ply + 1, best_line, node_count);

            if score > max {
                max = score;
                let _ = best_line[ply].insert(mov);
            }

            *node_count += 1;
        }
    }

    max
}
