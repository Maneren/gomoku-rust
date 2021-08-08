mod board;
mod cache;
mod functions;
mod r#move; // r# to allow reserved keyword as name
mod node;
mod stats;

pub use board::{Board, Player, Tile, TilePointer};
pub use cache::Cache;
pub use r#move::Move; // r# to allow reserved keyword as name

use functions::{
  evaluate_board, get_dist_fn, nodes_sorted_by_shallow_eval, print_status, shallow_clone_nodes,
  time_remaining,
};
use node::Node;
use stats::Stats;

use std::{
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use threadpool::ThreadPool;

type Score = i32;

fn minimax_top_level(
  board: &mut Board,
  cache_ref: &mut Cache,
  stats_ref: &mut Stats,
  current_player: Player,
  end_time: &Arc<Instant>,
) -> Result<Move, board::Error> {
  let cache_arc = Arc::new(Mutex::new(cache_ref.clone()));
  let stats_arc = Arc::new(Mutex::new(stats_ref.clone()));

  print_status("computing depth 1", **end_time);

  let presorted_nodes =
    nodes_sorted_by_shallow_eval(board, &stats_arc, &cache_arc, current_player, end_time)?;

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

  let moves_count = presorted_nodes.len() / 10;

  let presorted_nodes: Vec<Node> = presorted_nodes.into_iter().take(moves_count).collect();

  let cores = num_cpus::get();
  let pool = ThreadPool::with_name(String::from("node"), cores);

  let mut nodes = presorted_nodes;
  let mut nodes_generations: Vec<Vec<Node>> = vec![shallow_clone_nodes(&nodes)];

  let mut i = 1;

  let is_generation_valid = |generation: &[Node]| generation.iter().all(|node| node.valid);

  while time_remaining(end_time) {
    i += 1;
    print_status(&format!("computing depth {}", i), **end_time);

    let nodes_arc = Arc::new(Mutex::new(Vec::new()));

    for mut node in nodes {
      let mut board_clone = board.clone();

      let nodes_arc_clone = nodes_arc.clone();

      pool.execute(move || {
        node.compute_next(&mut board_clone);
        nodes_arc_clone.lock().unwrap().push(node);
      });
    }

    pool.join();
    if pool.panic_count() > 0 {
      panic!("{} node threads panicked", pool.panic_count());
    }

    // get the value from the arc-mutex
    let nodes_mutex = Arc::try_unwrap(nodes_arc).unwrap();
    nodes = nodes_mutex.into_inner().unwrap();

    nodes.sort_unstable_by(|a, b| a.cmp(b).reverse());

    if !is_generation_valid(&nodes) {
      break;
    }

    nodes_generations.push(shallow_clone_nodes(&nodes));

    if nodes.get(0).unwrap().is_end {
      break;
    };
  }

  println!("nodes_generations: {:#?}", nodes_generations);

  let last_generation = nodes_generations.last().unwrap();

  let best_node = last_generation.iter().max().unwrap().clone();

  println!();

  println!("searched to depth {:?}!", nodes_generations.len());

  *stats_ref = stats_arc.lock().unwrap().to_owned();
  *cache_ref = cache_arc.lock().unwrap().to_owned();

  Ok(best_node.to_move())
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

  let end = Arc::new(Instant::now().checked_add(max_time).unwrap());

  let move_ = minimax_top_level(&mut board, cache_ref, &mut stats, player, &end)?;

  board.set_tile(move_.tile, Some(player));

  Ok((board, move_, stats))
}
