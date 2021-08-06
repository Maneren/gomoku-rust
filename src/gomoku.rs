mod board;
mod cache;
mod functions;
mod r#move; // r# to allow reserved keyword as name
mod node;
mod stats;

pub use board::{Board, Player, Tile, TilePointer};
pub use cache::Cache;
pub use functions::{evaluate_board, get_dist_fn, time_remaining};
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

fn nodes_sorted_by_shallow_eval(
  moves: &[TilePointer],
  board: &mut Board,
  stats_arc: &Arc<Mutex<Stats>>,
  cache_arc: &Arc<Mutex<Cache>>,
  current_player: Player,
  end_time: Instant,
) -> Vec<Node> {
  let dist = get_dist_fn(board.get_size());
  let mut nodes: Vec<_> = moves
    .iter()
    .map(|&tile| {
      board.set_tile(tile, Some(current_player));
      let (analysis, is_game_end) = evaluate_board(board, stats_arc, cache_arc, current_player);
      board.set_tile(tile, None);

      Node::new(
        tile,
        current_player,
        analysis - dist(tile),
        is_game_end,
        end_time,
      )
    })
    .collect();

  nodes.sort_unstable_by_key(|node| -node.score);

  nodes
}

fn minimax_top_level(
  board: &mut Board,
  cache_ref: &mut Cache,
  stats_ref: &mut Stats,
  current_player: Player,
  end_time: Instant,
) -> Result<Move, board::Error> {
  let available_moves = board.get_empty_tiles()?;

  let cache = cache_ref.clone();
  let stats = stats_ref.clone();
  let cache_arc = Arc::new(Mutex::new(cache));
  let stats_arc = Arc::new(Mutex::new(stats));

  let presorted_nodes = nodes_sorted_by_shallow_eval(
    &available_moves,
    board,
    &stats_arc,
    &cache_arc,
    current_player,
    end_time,
  );

  // if there is winning move, return it
  let best_winning_node = presorted_nodes
    .iter()
    .filter(|Node { is_end, .. }| *is_end)
    .max();

  if let Some(node) = best_winning_node {
    *stats_ref = stats_arc.lock().unwrap().to_owned();
    *cache_ref = cache_arc.lock().unwrap().to_owned();
    return Ok(node.to_move());
  }

  let print_status = |msg: &str, end_time: Instant| {
    println!(
      "{} ({:?} remaining)",
      msg,
      end_time
        .checked_duration_since(Instant::now())
        .unwrap_or_else(|| Duration::from_millis(0))
    );
  };

  print_status("computing depth 1", end_time);

  let moves_count = 30;

  let presorted_nodes: Vec<Node> = presorted_nodes.into_iter().take(moves_count).collect();

  let mut nodes_generations = vec![presorted_nodes];

  let cores = num_cpus::get();
  let pool = ThreadPool::with_name(String::from("node"), cores);

  let mut nodes;
  let mut nodes_arc = Arc::new(Mutex::new(Vec::new()));
  let mut done = false;

  let mut i = 2;

  while time_remaining(end_time) && !done {
    print_status(&format!("computing depth {}", i), end_time);
    i += 1;

    #[allow(clippy::explicit_into_iter_loop)]
    for node in nodes_generations.last_mut().unwrap() {
      let mut node = node.clone();
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
    if pool.panic_count() > 0 {
      panic!("{} subthreads panicked", pool.panic_count());
    }

    let nodes_mutex = Arc::try_unwrap(nodes_arc).unwrap();
    nodes_arc = Arc::new(Mutex::new(Vec::new()));
    nodes = nodes_mutex.into_inner().unwrap();

    if nodes.iter().all(|node| node.is_end) {
      done = true;
    };

    nodes.sort_unstable_by_key(|node| -node.score);
    nodes_generations.push(nodes);
  }

  // find latest usable generation
  while !nodes_generations.is_empty()
    && nodes_generations
      .last()
      .unwrap()
      .iter()
      .any(|node| !node.valid)
  {
    nodes_generations.pop();
  }

  if nodes_generations.is_empty() {
    panic!("no generation computed, try increasing time limit")
  }

  let last_generation = nodes_generations.last().unwrap();

  let best_node = last_generation.iter().max().unwrap().clone();

  println!();

  println!("searched to depth {:?}", nodes_generations.len());

  let Node { tile, score, .. } = best_node;

  *stats_ref = stats_arc.lock().unwrap().to_owned();
  *cache_ref = cache_arc.lock().unwrap().to_owned();

  Ok(Move { tile, score })
}

pub fn decide(
  board: &Board,
  player: Player,
  max_time: u64,
) -> Result<(Board, Move, Stats), board::Error> {
  let mut cache = Cache::new(board.get_size());

  let result = decide_with_cache(board, player, max_time, &mut cache)?;

  println!("cache: {:?}", cache.stats);

  Ok(result)
}

pub fn decide_with_cache(
  board: &Board,
  player: Player,
  max_time: u64,
  cache_ref: &mut Cache,
) -> Result<(Board, Move, Stats), board::Error> {
  let mut board = board.clone();
  let mut stats = Stats::new();

  let max_time = Duration::from_millis(max_time);

  let end = Instant::now().checked_add(max_time).unwrap();

  let move_ = minimax_top_level(&mut board, cache_ref, &mut stats, player, end)?;

  board.set_tile(move_.tile, Some(player));

  Ok((board, move_, stats))
}
