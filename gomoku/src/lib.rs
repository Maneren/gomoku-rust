mod board;
mod functions;
mod r#move; // r# to allow reserved keyword as name
mod node;
mod player;
mod state;
mod stats;
pub mod utils;

use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
  thread::{sleep, spawn},
  time::{Duration, Instant},
};

pub use board::{Board, TilePointer};
use functions::{check_winning, evaluate_board, nodes_sorted_by_shallow_eval};
pub use player::Player;
pub use r#move::Move; // r# to allow reserved keyword as name
use stats::Stats;
use threadpool::ThreadPool;
use utils::{do_run, format_number, print_status};

type Tile = Option<Player>;
type Score = i32;

fn minimax_top_level(
  board: &mut Board,
  current_player: Player,
  time_limit: Duration,
  threads: usize,
) -> Result<(Move, Stats), board::Error> {
  let mut stats = Stats::new();
  let end_time = Instant::now().checked_add(time_limit).unwrap();

  let end = Arc::new(AtomicBool::new(false));

  {
    let end = end.clone();
    spawn(move || {
      sleep(time_limit);
      end.store(true, Ordering::Relaxed);
    });
  }

  let empty_tiles = board.get_empty_tiles()?;
  print_status(&format!("computing depth 1"), &end_time);
  let presorted_nodes = nodes_sorted_by_shallow_eval(
    board,
    empty_tiles,
    &mut stats,
    current_player,
    &end,
    threads,
  );

  // if there is winning move, return it
  if let Some(winning_move) = check_winning(&presorted_nodes) {
    return Ok((winning_move, stats));
  }

  #[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
  )]
  let mut nodes = presorted_nodes;
  let moves_count = (1.5 * (nodes.len() as f32).sqrt()) as usize;
  nodes.truncate(moves_count);

  let pool = ThreadPool::with_name(String::from("node"), threads);

  let mut generation_number = 1;
  let mut last_generation = nodes.clone();
  let arc = Arc::new(Mutex::new((Vec::new(), stats)));

  while do_run(&end) {
    generation_number += 1;

    print_status(&format!("computing depth {generation_number}"), &end_time);

    for mut node in nodes {
      let mut board_clone = board.clone();
      let arc_clone = arc.clone();

      pool.execute(move || {
        let mut stats = Stats::new();

        node.compute_next(&mut board_clone, &mut stats);

        let (nodes, total_stats) = &mut *arc_clone.lock().unwrap();

        nodes.push(node);
        *total_stats += stats;
      });
    }

    pool.join();

    assert!(pool.panic_count() == 0, "node threads panicked");

    let (new_nodes, ..) = &mut *arc.lock().unwrap();

    nodes = new_nodes.drain(..).collect();

    if nodes.iter().any(|node| !node.valid) {
      break;
    }

    nodes.sort_unstable_by(|a, b| b.cmp(a));

    last_generation = nodes.clone();

    if nodes.iter().any(|node| node.state.is_win()) || nodes.iter().all(|node| node.state.is_lose())
    {
      break;
    }

    nodes.retain(|child| !child.state.is_lose());

    if nodes.len() <= 1 {
      break;
    }

    if generation_number >= 4 {
      nodes.truncate(threads);
    }
  }

  println!();

  if nodes.iter().any(|node| node.state.is_win()) {
    println!("Winning move found!",);
  } else if nodes.iter().all(|node| node.state.is_lose()) {
    println!("All moves are losing :(");
  }

  println!("Searched to depth {:?}!", generation_number - 1);

  println!();

  let (_, stats) = &mut *arc.lock().unwrap();
  let stats = stats.clone();

  let best_node = last_generation.iter().max().unwrap();

  println!("Best moves: {best_node:#?}");
  // {
  //   let mut best_board = board.clone();

  //   let mut current = best_node.best_moves.clone();

  //   best_board.set_tile(current.tile, Some(current.player));
  //   while current.next.is_some() {
  //     current = *current.next.unwrap();
  //     best_board.set_tile(current.tile, Some(current.player));
  //   }
  //   println!("Best board: \n{}", best_board);
  // }

  Ok((best_node.to_move(), stats))
}

pub fn decide(
  board: &mut Board,
  player: Player,
  time_limit: u64,
  threads: usize,
) -> Result<(Move, Stats), board::Error> {
  let time_limit = Duration::from_millis(time_limit);

  let (move_, stats) = minimax_top_level(board, player, time_limit, threads)?;

  board.set_tile(move_.tile, Some(player));

  Ok((move_, stats))
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn perf(time_limit: u64, threads: usize, board_size: u8) {
  let time_limit = Duration::from_secs(time_limit);
  let end = Arc::new(AtomicBool::new(false));

  {
    let end = end.clone();
    spawn(move || {
      sleep(time_limit);
      end.store(true, Ordering::Relaxed);
    });
  }

  let board = Board::get_empty_board(board_size);
  let counter_arc = Arc::new(Mutex::new(0));
  let tile = TilePointer {
    x: board_size / 2,
    y: board_size / 2,
  };

  let start = Instant::now();
  let pool = ThreadPool::with_name(String::from("node"), threads);
  for _ in 0..threads {
    let mut board_clone = board.clone();
    let counter_arc_clone = counter_arc.clone();
    let end_clone = end.clone();

    pool.execute(move || {
      let mut i = 0;
      while do_run(&end_clone) {
        board_clone.set_tile(tile, Some(Player::X));
        let (..) = evaluate_board(&board_clone, Player::O);
        board_clone.set_tile(tile, None);
        i += 1;
      }
      *counter_arc_clone.lock().unwrap() += i;
    });
  }

  pool.join();
  assert!(
    pool.panic_count() == 0,
    "{} node threads panicked",
    pool.panic_count()
  );

  let elapsed = start.elapsed().as_millis() as u64;

  let counter = *counter_arc.lock().unwrap();
  let per_second = counter * 1000 / elapsed; // * 1000 to account for milliseconds
  println!(
    "total evals = {} ({})",
    counter,
    format_number(counter as f32)
  );
  println!(
    "evals/s = {} ({})",
    per_second,
    format_number(per_second as f32),
  );
}
