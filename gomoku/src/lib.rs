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
use node::Node;
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
  print_status(
    &format!("computing depth 1 for {} nodes", empty_tiles.len()),
    &end_time,
  );
  let presorted_nodes = nodes_sorted_by_shallow_eval(
    board,
    empty_tiles,
    &mut stats,
    current_player,
    &end,
    threads,
  );

  // if there is winning move, return it
  if let Some(winning_move) = check_winning(&presorted_nodes, stats) {
    return Ok(winning_move);
  }

  #[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
  )]
  let moves_count = (1.5 * (presorted_nodes.len() as f32).sqrt()) as usize;
  let presorted_nodes: Vec<_> = presorted_nodes.into_iter().take(moves_count).collect();

  let pool = ThreadPool::with_name(String::from("node"), threads);

  let mut nodes = presorted_nodes;
  let mut generation_number = 1;
  let mut last_generation = nodes.clone();
  let nodes_arc = Arc::new(Mutex::new(Vec::new()));
  let stats_arc = Arc::new(Mutex::new(Stats::new()));

  while do_run(&end) {
    generation_number += 1;

    let node_count = nodes.len() + nodes.iter().map(Node::node_count).sum::<usize>();

    print_status(
      &format!(
        "computing depth {} for {} nodes",
        generation_number, node_count
      ),
      &end_time,
    );

    for mut node in nodes {
      let mut board_clone = board.clone();
      let nodes_arc_clone = nodes_arc.clone();
      let stats_arc_clone = stats_arc.clone();

      pool.execute(move || {
        let mut stats = Stats::new();

        node.compute_next(&mut board_clone, &mut stats);

        nodes_arc_clone.lock().unwrap().push(node);
        *stats_arc_clone.lock().unwrap() += stats;
      });
    }

    pool.join();

    assert!(pool.panic_count() == 0, "node threads panicked");

    // HACK: get the nodes from the arc-mutex
    nodes = nodes_arc.lock().unwrap().drain(..).collect();

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

  println!("Searched to depth {:?}!", generation_number);

  println!();

  let stats = stats_arc.lock().unwrap().clone();

  let best_node = last_generation.iter().max().unwrap();

  println!("Best moves: {:#?}", best_node);
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
