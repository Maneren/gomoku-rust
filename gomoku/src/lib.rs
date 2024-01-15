#![warn(clippy::pedantic)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::must_use_candidate)]

mod board;
mod functions;
mod r#move; // r# to allow reserved keyword as name
mod node;
mod player;
mod state;
mod stats;
pub mod utils;

use std::{
  sync::atomic::{AtomicBool, Ordering},
  thread::{sleep, spawn},
  time::{Duration, Instant},
};

pub use board::{Board, Tile, TilePointer};
use functions::evaluate_board;
#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use jemallocator::Jemalloc;
pub use player::Player;
// r# to allow reserved keyword as name
pub use r#move::Move;
use rayon::prelude::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
pub use stats::Stats;
use utils::{do_run, format_number, print_status};

use crate::{node::Node, state::State};

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

static END: AtomicBool = AtomicBool::new(false);

type Score = i32;

fn minimax_top_level(
  board: &mut Board,
  current_player: Player,
  time_limit: Duration,
) -> Result<(Move, Stats), board::Error> {
  let mut stats = Stats::new();
  let end_time = Instant::now().checked_add(time_limit).unwrap();

  END.store(false, Ordering::Relaxed);

  spawn(move || {
    sleep(time_limit);
    END.store(true, Ordering::Release);
  });

  let empty_tiles = board.get_empty_tiles();

  let Some(empty_tiles) = empty_tiles else {
    return Err(GomokuError::NoEmptyTiles);
  };

  let mut nodes = empty_tiles
    .into_iter()
    .map(|tile| Node::new(tile, current_player, State::NotEnd, &mut stats))
    .collect::<Vec<_>>();

  let mut total_depth = 0;
  let mut stats = Stats::new();

  let (initial_score, initial_state) = evaluate_board(board, !current_player);
  if initial_state.is_end() {
    println!("The game already ended");
  }

  while do_run() {
    total_depth += 1;

    print_status(
      &format!(
        "computing depth {total_depth} for {} nodes",
        nodes.iter().map(Node::node_count).sum::<usize>()
      ),
      &end_time,
    );

    let snapshot = nodes.clone();

    stats += nodes
      .par_iter_mut()
      .map(|node| node.compute_next(&mut board.clone(), initial_score))
      .sum();

    if nodes.iter().any(|node| !node.valid) {
      nodes = snapshot;
      total_depth -= 1;
      break;
    }

    nodes.sort_unstable_by(|a, b| b.cmp(a));

    if nodes.iter().any(|node| node.state.is_win()) {
      println!("Winning move found!");
      break;
    }

    if nodes.iter().all(|node| node.state.is_lose()) {
      println!("All moves are losing :(");
      break;
    }

    nodes.retain(|child| !child.state.is_lose());

    if nodes.len() <= 1 {
      println!("Only one viable move left");
      break;
    }

    #[allow(
      clippy::cast_precision_loss,
      clippy::cast_possible_truncation,
      clippy::cast_sign_loss
    )]
    let moves_count = (1.5 * (nodes.len() as f32).sqrt()) as usize;
    nodes.truncate(moves_count.max(3));
  }

  println!("Searched to depth {total_depth:?}!");

  println!();

  let best_node = nodes.iter().max().unwrap();

  println!("Best moves: {best_node:#?}");

  Ok((best_node.to_move(), stats))
}

pub fn set_thread_count(threads: usize) -> Result<(), Box<dyn std::error::Error>> {
  rayon::ThreadPoolBuilder::new()
    .num_threads(threads)
    .build_global()
    .map_err(|_| "Thread count already set".into())
}

pub fn decide(
  board: &mut Board,
  player: Player,
  time_limit: u64,
) -> Result<(Move, Stats), board::Error> {
  let time_limit = Duration::from_millis(time_limit);

  let (move_, stats) = minimax_top_level(board, player, time_limit)?;

  board.set_tile(move_.tile, Some(player));

  Ok((move_, stats))
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn perf(time_limit: u64, threads: usize, board_size: u8) {
  let time_limit = Duration::from_secs(time_limit);

  END.store(false, Ordering::Relaxed);

  set_thread_count(threads).unwrap();

  spawn(move || {
    sleep(time_limit);
    END.store(true, Ordering::Relaxed);
  });

  let board = Board::get_empty_board(board_size);
  let tile = TilePointer {
    x: board_size / 2,
    y: board_size / 2,
  };

  let start = Instant::now();
  let counter: u64 = (0..threads)
    .into_par_iter()
    .map(|_| {
      let mut board_clone = board.clone();

      let mut i = 0;
      while do_run() {
        board_clone.set_tile(tile, Some(Player::X));
        let (..) = evaluate_board(&board_clone, Player::O);
        board_clone.set_tile(tile, None);
        i += 1;
      }
      i
    })
    .sum();

  let elapsed = start.elapsed().as_millis() as u64;
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
