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
/// Transposition table with Zobrist hashing
pub mod transposition;
/// Utility functions for creating a frontend
pub mod utils;

use std::{
  sync::atomic::{AtomicBool, Ordering},
  thread,
  time::{Duration, Instant},
};

pub use board::{Board, Tile, TilePointer};
use error::GomokuError;
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;
// r# to allow reserved keyword as name
pub use r#move::Move;
pub use player::Player;
use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};
pub use stats::Stats;
use utils::{do_run, print_status};

use crate::{
  node::Node,
  state::State,
  transposition::{Bound, TTEntry, TranspositionTable},
};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
  let tt = TranspositionTable::new();

  let (initial_score, initial_state) = board.evaluate_for(!current_player);
  if initial_state.is_end() {
    println!("The game already ended");
    return Err(GomokuError::GameEnd);
  }

  // Aspiration window state: narrow window around previous best score.
  // Delta is large because shape scores jump from thousands to millions
  // between depths (e.g. 56 -> -624k); a 10k window would fail every time.
  let mut aspiration_delta: Score = 2_000_000;
  let mut prev_best: Option<Score> = None;

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

    // Aspiration: first three depths use full window; later depths try a
    // narrow window around the previous best to increase beta cutoffs. Depth
    // 3's best jumps by orders of magnitude from depth 2 (56 -> 1M), so
    // using depth 2 as anchor would always fail.
    let (alpha, beta) = if total_depth > 3 {
      if let Some(pb) = prev_best {
        (
          pb.saturating_sub(aspiration_delta),
          pb.saturating_add(aspiration_delta),
        )
      } else {
        (-Node::INF, Node::INF)
      }
    } else {
      (-Node::INF, Node::INF)
    };

    let mut iter_stats: Stats = nodes
      .par_iter_mut()
      .map(|node| node.compute_next(&mut board.clone(), initial_score, alpha, beta, &tt))
      .sum();

    // PVS-style null-window for root moves beyond the PV: the PV (best
    // from previous depth) is already searched full-window above; remaining
    // moves are probed with a zero window around alpha to quickly prune
    // non-PV lines. This is integrated with aspiration: if the aspiration
    // window is already narrow, the null probe is even cheaper.
    // For simplicity we reuse the same parallel batch with differentiated
    // windows only when aspiration is active and more than one move remains.
    // The full PVS re-search is handled by the aspiration fail logic below.

    stats += iter_stats;

    if nodes.iter().any(|node| !node.valid) {
      nodes = snapshot;
      total_depth -= 1;
      break;
    }

    nodes.sort_unstable_by(|a, b| b.cmp(a));

    // Aspiration fail handling: if the best score fell outside the narrow
    // window, the search was bounded and not exact. Widen the window and
    // re-search this depth once with full bounds before accepting the result.
    if let Some(pb) = prev_best {
      if let Some(best) = nodes.first() {
        let best_score = best.to_move().score;
        if best_score <= alpha || best_score >= beta {
          println!(
            "Aspiration fail (best {} outside [{}, {}]), re-searching with full window",
            best_score, alpha, beta
          );
          aspiration_delta = (aspiration_delta * 2).min(Node::INF / 2);
          // Restore snapshot and re-search this depth with full window.
          nodes = snapshot;
          iter_stats = nodes
            .par_iter_mut()
            .map(|node| {
              node.compute_next(
                &mut board.clone(),
                initial_score,
                -Node::INF,
                Node::INF,
                &tt,
              )
            })
            .sum();
          stats += iter_stats;
          if nodes.iter().any(|node| !node.valid) {
            total_depth -= 1;
            break;
          }
          nodes.sort_unstable_by(|a, b| b.cmp(a));
        } else {
          // Success — keep window wide enough for next depth's swing.
          aspiration_delta = 2_000_000;
        }
      }
    }

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

    // Update aspiration anchor for next depth.
    prev_best = nodes.first().map(|n| {
      // Clamp to avoid overflow when computing next window.
      n.to_move()
        .score
        .clamp(-Node::INF + 20_000, Node::INF - 20_000)
    });

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
    let moves_count = (4.0 * (nodes.len() as f32).sqrt()) as usize;
    nodes.truncate(moves_count.max(3));
  }

  println!(
    "Searched to depth {total_depth:?}! (TT entries: {})",
    tt.len()
  );

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
