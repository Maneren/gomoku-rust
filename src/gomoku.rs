mod board;
mod cache;
mod functions;
mod r#move; // r# to allow reserved keyword as name
mod node;
mod stats;

pub use board::{Board, Tile, TilePointer};
pub use cache::Cache;
use functions::{evaluate_board, get_dist_fn, next_player};
use node::Node;
pub use r#move::{Move, MoveWithEnd}; // r# to allow reserved keyword as name
use stats::Stats;

use std::{
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use threadpool::ThreadPool;

type Score = i32;
const ALPHA_DEFAULT: Score = -1_000_000_000;
const BETA_DEFAULT: Score = 1_000_000_000;

fn moves_sorted_by_shallow_eval(
  moves: &[TilePointer],
  board: &mut Board,
  stats_arc: &Arc<Mutex<Stats>>,
  cache_arc: &Arc<Mutex<Cache>>,
  current_player: bool,
) -> Vec<MoveWithEnd> {
  let dist = get_dist_fn(board.get_size());
  let mut moves: Vec<MoveWithEnd> = moves
    .iter()
    .map(|&tile| {
      board.set_tile(tile, Some(current_player));
      let (analysis, is_game_end) = evaluate_board(board, stats_arc, cache_arc, current_player);
      board.set_tile(tile, None);

      MoveWithEnd {
        tile,
        score: -(analysis - dist(tile)),
        is_end: is_game_end,
      }
    })
    .collect();

  moves.sort_unstable();

  moves
}

fn minimax_top_level(
  board: &mut Board,
  cache_ref: &mut Cache,
  stats_ref: &mut Stats,
  current_player: bool,
  end_time: Instant,
) -> Result<Move, board::Error> {
  let available_moves = board.get_empty_tiles()?;

  let cache = cache_ref.clone();
  let stats = stats_ref.clone();
  let cache_arc = Arc::new(Mutex::new(cache));
  let stats_arc = Arc::new(Mutex::new(stats));

  let presorted_moves = moves_sorted_by_shallow_eval(
    &available_moves,
    board,
    &stats_arc,
    &cache_arc,
    current_player,
  );

  let moves_count = 25;

  let nodes = Vec::with_capacity(moves_count);
  let nodes_arc = Arc::new(Mutex::new(nodes));

  let cores = num_cpus::get();
  let pool = ThreadPool::new(cores);

  // if there is winning move, return it
  let best_winning_move = presorted_moves
    .iter()
    .take(moves_count)
    .filter(|MoveWithEnd { is_end, .. }| *is_end)
    .max();

  if let Some(move_) = best_winning_move {
    *stats_ref = stats_arc.lock().unwrap().to_owned();
    *cache_ref = cache_arc.lock().unwrap().to_owned();
    return Ok(move_.into());
  }

  presorted_moves.into_iter().take(moves_count).for_each(
    |MoveWithEnd {
       tile,
       score,
       is_end,
     }| {
      let mut board_clone = board.clone();
      board_clone.set_tile(tile, Some(current_player));

      let cache_arc_clone = cache_arc.clone();
      let stats_arc_clone = stats_arc.clone();
      let nodes_arc_clone = nodes_arc.clone();

      pool.execute(move || {
        let mut node = Node::new(
          tile,
          next_player(current_player),
          -score,
          is_end,
          end_time,
          1,
        );

        node.eval(
          &mut board_clone,
          &stats_arc_clone,
          &cache_arc_clone,
          BETA_DEFAULT,
        );

        nodes_arc_clone.lock().unwrap().push(node);
      });
    },
  );

  pool.join();

  // get the value from the Arc
  let mut nodes = Arc::try_unwrap(nodes_arc).unwrap().into_inner().unwrap();
  let mut nodes_arc = Arc::new(Mutex::new(Vec::new()));

  while check_time(end_time) {
    println!(
      "deepening {:?} remaining",
      end_time
        .checked_duration_since(Instant::now())
        .unwrap_or_else(|| Duration::from_millis(0))
    );

    #[allow(clippy::explicit_into_iter_loop)]
    for mut node in nodes.into_iter() {
      let mut board_clone = board.clone();

      let cache_arc_clone = cache_arc.clone();
      let stats_arc_clone = stats_arc.clone();
      let nodes_arc_clone = nodes_arc.clone();

      pool.execute(move || {
        node.compute_next(
          &mut board_clone,
          &stats_arc_clone,
          &cache_arc_clone,
          BETA_DEFAULT,
        );
        nodes_arc_clone.lock().unwrap().push(node);
      });
    }

    pool.join();

    let nodes_mutex = Arc::try_unwrap(nodes_arc).unwrap();
    nodes = nodes_mutex.into_inner().unwrap();
    nodes_arc = Arc::new(Mutex::new(Vec::new()));
  }

  let best_node = nodes.into_iter().max().unwrap();

  println!();

  let Node { tile, score, .. } = best_node;

  *stats_ref = stats_arc.lock().unwrap().to_owned();
  *cache_ref = cache_arc.lock().unwrap().to_owned();

  Ok(Move { tile, score })
}

fn check_time(end_time: Instant) -> bool {
  Instant::now().checked_duration_since(end_time).is_none()
}

pub fn decide(
  board: &Board,
  player: bool,
  max_time: u64,
) -> Result<(Board, Move, Stats), board::Error> {
  let mut cache = Cache::new(board.get_size());

  let result = decide_with_cache(board, player, max_time, &mut cache)?;

  println!("cache: {:?}", cache.stats);

  Ok(result)
}

pub fn decide_with_cache(
  board: &Board,
  player: bool,
  max_time: u64,
  cache_ref: &mut Cache,
) -> Result<(Board, Move, Stats), board::Error> {
  let mut board = board.clone();
  let mut stats = Stats::new();

  let max_time = Duration::from_millis(max_time);

  // TODO: handle the error
  let end = Instant::now().checked_add(max_time).unwrap();

  let move_ = minimax_top_level(&mut board, cache_ref, &mut stats, player, end)?;

  board.set_tile(move_.tile, Some(player));

  Ok((board, move_, stats))
}
