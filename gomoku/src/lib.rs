//! Gomoku engine

#![warn(clippy::pedantic)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::similar_names)]
#![allow(clippy::must_use_candidate)]
#![warn(missing_docs)]

mod board;
mod error;
mod r#move; // r# to allow reserved keyword as name
mod node;
mod player;
mod state;
mod stats;
/// Utility functions for creating a frontend
pub mod utils;

use std::{
  sync::atomic::{AtomicBool, Ordering},
  thread,
  time::{Duration, Instant},
};

pub use board::{Board, Tile, TilePointer};
use error::GomokuError;
#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use jemallocator::Jemalloc;
pub use player::Player;
// r# to allow reserved keyword as name
pub use r#move::Move;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
pub use stats::Stats;
use utils::{do_run, print_status};

use crate::{node::Node, state::State};

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

static END: AtomicBool = AtomicBool::new(false);

type Score = i32;

fn minimax(
  board: &mut Board,
  current_player: Player,
  time_limit: Duration,
) -> Result<(Move, Stats), GomokuError> {
  let end_time = Instant::now() + time_limit;

  END.store(false, Ordering::Relaxed);

  thread::spawn(move || {
    thread::sleep(time_limit * 99 / 100);
    END.store(true, Ordering::Release);
  });

  let mut nodes = board
    .pointers_to_empty_tiles()
    .map(|tile| Node::new(tile, current_player, State::NotEnd))
    .collect::<Vec<_>>();

  if nodes.is_empty() {
    return Err(GomokuError::NoEmptyTiles);
  }

  let mut total_depth = 0;
  let mut stats = Stats::new();

  let (initial_score, initial_state) = board.evaluate_for(!current_player);
  if initial_state.is_end() {
    println!("The game already ended");
    return Err(GomokuError::GameEnd);
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

    if nodes.iter().all(|node| node.state == State::Draw) {
      println!("All moves are draws.");
      break;
    }

    nodes.retain(|child| child.state == State::NotEnd);

    if nodes.len() <= 1 {
      println!("Only one viable move left");
      break;
    }

    #[allow(
      clippy::cast_precision_loss,
      clippy::cast_possible_truncation,
      clippy::cast_sign_loss
    )]
    let moves_count = (2.0 * (nodes.len() as f32).sqrt()) as usize;
    nodes.truncate(moves_count.max(3));
  }

  println!("Searched to depth {total_depth:?}!");

  println!();

  let best_node = nodes.iter().max().expect("we never remove all nodes");

  println!("Best move sequence: {best_node:#?}");

  Ok((best_node.to_move(), stats))
}

/// Sets the thread count for the rayon threadpool
///
/// # Errors
/// Returns an error if the thread count is already set.
pub fn set_thread_count(threads: usize) -> Result<(), Box<dyn std::error::Error>> {
  rayon::ThreadPoolBuilder::new()
    .num_threads(threads)
    .build_global()
    .map_err(|_| "Thread count already set".into())
}

/// Returns the best move and stats for the given board.
///
/// # Errors
/// Returns an error if the engine failed to find a move. See [`GomokuError`]
/// for possible errors.
pub fn decide(
  board: &mut Board,
  player: Player,
  time_limit: u64,
) -> Result<(Move, Stats), GomokuError> {
  let time_limit = Duration::from_millis(time_limit);

  let (move_, stats) = minimax(board, player, time_limit)?;

  board.set_tile(move_.tile, Some(player));

  Ok((move_, stats))
}
