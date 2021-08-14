mod board;
mod functions;
mod r#move; // r# to allow reserved keyword as name
mod node;
mod stats;

pub use board::{Board, Player, Tile, TilePointer};
pub use r#move::Move; // r# to allow reserved keyword as name

use functions::{
  evaluate_board, get_dist_fn, nodes_sorted_by_shallow_eval, print_status, time_remaining,
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
  stats_ref: &mut Stats,
  current_player: Player,
  end_time: &Arc<Instant>,
) -> Result<Move, board::Error> {
  let stats_arc = Arc::new(Mutex::new(stats_ref.clone()));

  let empty_tiles = board.get_empty_tiles()?;

  print_status(
    &format!("computing depth 1 for {} nodes", empty_tiles.len()),
    **end_time,
  );

  let presorted_nodes =
    nodes_sorted_by_shallow_eval(board, empty_tiles, &stats_arc, current_player, end_time)?;

  // if there is winning move, return it
  let best_winning_node = presorted_nodes
    .iter()
    .filter(|Node { state, .. }| state.is_win())
    .max();

  if let Some(node) = best_winning_node {
    *stats_ref = stats_arc.lock().unwrap().to_owned();
    return Ok(node.to_move());
  }

  #[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
  )]
  let moves_count = (1.5 * (presorted_nodes.len() as f32).sqrt()) as usize;

  let presorted_nodes: Vec<Node> = presorted_nodes.into_iter().take(moves_count).collect();

  let cores = num_cpus::get();
  let pool = ThreadPool::with_name(String::from("node"), cores);

  let mut nodes = presorted_nodes;
  let mut nodes_generations = vec![nodes.clone()];
  let nodes_arc = Arc::new(Mutex::new(Vec::new()));

  let mut i = 1;

  while time_remaining(end_time) && !nodes.is_empty() {
    i += 1;
    print_status(
      &format!("computing depth {} for {} nodes", i, nodes.len()),
      **end_time,
    );

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
    };

    // get the value from the arc-mutex
    nodes = nodes_arc.lock().unwrap().drain(..).collect();

    nodes.sort_unstable_by(|a, b| b.cmp(a));

    if nodes.iter().any(|node| !node.valid) {
      break;
    }

    nodes_generations.push(nodes.clone());

    if nodes[0].state.is_win() || nodes.iter().all(|node| node.state.is_lose()) {
      break;
    };

    nodes.retain(|child| !child.state.is_lose());
  }

  println!();
  println!("searched to depth {:?}!", nodes_generations.len());

  *stats_ref = stats_arc.lock().unwrap().to_owned();

  let last_generation = nodes_generations.last().unwrap();
  let best_node = last_generation.iter().max().unwrap();

  println!("Best moves: {:#?}", best_node);

  Ok(best_node.to_move())
}

pub fn decide(
  board: &mut Board,
  player: Player,
  time_limit: u64,
) -> Result<(Move, Stats), board::Error> {
  let mut stats = Stats::new();

  let time_limit = Duration::from_millis(time_limit);
  let end = Arc::new(Instant::now().checked_add(time_limit).unwrap());

  let move_ = minimax_top_level(board, &mut stats, player, &end)?;

  board.set_tile(move_.tile, Some(player));

  Ok((move_, stats))
}
