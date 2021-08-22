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
use stats::Stats;

use std::{
  ops::Add,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use threadpool::ThreadPool;

type Score = i32;

fn minimax_top_level(
  board: &mut Board,
  current_player: Player,
  end_time: &Arc<Instant>,
  threads: usize,
) -> Result<(Move, Stats), board::Error> {
  let mut stats = Stats::new();

  let empty_tiles = board.get_empty_tiles()?;
  print_status(
    &format!("computing depth 1 for {} nodes", empty_tiles.len()),
    **end_time,
  );
  let presorted_nodes =
    nodes_sorted_by_shallow_eval(board, empty_tiles, &mut stats, current_player, end_time);

  // if there is winning move, return it
  let winning_node = presorted_nodes
    .iter()
    .filter(|node| node.state.is_win())
    .max();

  if let Some(node) = winning_node {
    return Ok((node.to_move(), stats));
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
  let mut nodes_generations = vec![nodes.clone()];
  let nodes_arc = Arc::new(Mutex::new(Vec::new()));
  let stats_arc = Arc::new(Mutex::new(Vec::new()));

  let mut i = 1;

  while time_remaining(end_time) {
    i += 1;
    print_status(
      &format!("computing depth {} for {} nodes", i, nodes.len()),
      **end_time,
    );

    for mut node in nodes {
      let mut board_clone = board.clone();
      let mut stats_clone = Stats::new();
      let nodes_arc_clone = nodes_arc.clone();
      let stats_arc_clone = stats_arc.clone();

      pool.execute(move || {
        node.compute_next(&mut board_clone, &mut stats_clone);
        nodes_arc_clone.lock().unwrap().push(node);
        stats_arc_clone.lock().unwrap().push(stats_clone);
      });
    }

    pool.join();
    if pool.panic_count() > 0 {
      panic!("{} node threads panicked", pool.panic_count());
    };

    // HACK: get the nodes from the arc-mutex
    nodes = nodes_arc.lock().unwrap().drain(..).collect();

    if nodes.iter().any(|node| !node.valid) {
      break;
    }

    nodes.sort_unstable_by(|a, b| b.cmp(a));
    nodes_generations.push(nodes.clone());

    if nodes.iter().any(|node| node.state.is_win()) || nodes.iter().all(|node| node.state.is_lose())
    {
      break;
    }

    nodes.retain(|child| !child.state.is_lose());
  }

  println!();

  if nodes.iter().any(|node| node.state.is_win()) {
    println!("Winning move found!",);
  } else if nodes.iter().all(|node| node.state.is_lose()) {
    println!("All moves are losing :(");
  }

  println!("Searched to depth {:?}!", nodes_generations.len());

  println!();

  let stats = stats_arc
    .lock()
    .unwrap()
    .iter()
    .fold(Stats::new(), |total, stats| total.add(*stats));

  let last_generation = nodes_generations.last().unwrap();
  let best_node = last_generation.iter().max().unwrap();

  println!("Best moves: {:#?}", best_node);

  Ok((best_node.to_move(), stats))
}

pub fn decide(
  board: &mut Board,
  player: Player,
  time_limit: u64,
  threads: usize,
) -> Result<(Move, Stats), board::Error> {
  let time_limit = Duration::from_millis(time_limit);
  let end = Arc::new(Instant::now().checked_add(time_limit).unwrap());

  let (move_, stats) = minimax_top_level(board, player, &end, threads)?;

  board.set_tile(move_.tile, Some(player));

  Ok((move_, stats))
}
