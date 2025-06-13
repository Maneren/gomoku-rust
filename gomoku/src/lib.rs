//! Gomoku engine

#![warn(clippy::pedantic)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::similar_names)]
#![allow(clippy::must_use_candidate)]
#![allow(dead_code)]
#![allow(missing_docs)]

mod board;
mod cache;
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
  time::Duration,
};

pub use board::{Board, Tile, TilePointer};
use error::GomokuError;
#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
use jemallocator::Jemalloc;
pub use player::Player;
// r# to allow reserved keyword as name
pub use r#move::Move;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
pub use stats::Stats;

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
  // let end_time = Instant::now() + time_limit;

  END.store(false, Ordering::Relaxed);

  thread::spawn(move || {
    thread::sleep(time_limit * 99 / 100);
    END.store(true, Ordering::Release);
  });

  let empty_tiles = board.pointers_to_empty_tiles().collect::<Vec<_>>();
  if empty_tiles.is_empty() {
    return Err(GomokuError::NoEmptyTiles);
  }

  let mut depth = 2;
  let cache = cache::Cache::with_capacity(16 * 1024 * 1024);

  let (best_move, stats) = loop {
    println!("Depth: {depth:?}");
    let moves: Vec<_> = empty_tiles
      .par_iter()
      .map(|&tile| {
        let mut board = board.clone();
        let mut stats = Stats::new();
        board.set_tile(tile, Some(current_player));
        let score = node::alpha_beta_negamax(
          &mut board,
          !current_player,
          depth,
          -Score::MAX,
          Score::MAX,
          &mut stats,
          &cache,
        );
        board.set_tile(tile, None);

        (Move { tile, score }, stats)
      })
      .collect();

    let &(best_move, _) = moves.iter().min_by_key(|(move_, _)| move_.score).unwrap();

    let stats = moves.iter().map(|(_, stats)| *stats).sum::<Stats>();

    if !utils::do_run() {
      break (best_move, stats);
    }

    depth += 1;
  };

  println!("Searched to depth {depth:?}!");
  println!("Stats: {stats:#?}");
  println!("Best move sequence: {best_move:#?}");
  println!("Cache size: {}", cache.size());

  Ok((best_move, stats))
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
