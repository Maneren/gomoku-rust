// for shuffling
use rand::seq::SliceRandom;
use rand::thread_rng;

pub mod board;
use board::{Board, TilePointer};

#[derive(Debug)]
pub struct Stats {
  pub boards_evaluated: u32,
  pub pruned: u32,
  pub cached_boards_used: u32,
}
impl Stats {
  pub fn new() -> Stats {
    Stats {
      boards_evaluated: 0,
      pruned: 0,
      cached_boards_used: 0,
      // eval_times: EvalTimes { board: Vec::new() },
    }
  }
}

use std::collections::HashMap;
pub struct Cache {
  pub boards: HashMap<(u128, bool), i128>,
}
impl Cache {
  pub fn new() -> Cache {
    Cache {
      boards: HashMap::new(),
    }
  }
}

fn next_player(current: bool) -> bool {
  !current
}

fn evaluate_board(
  board: &mut Board,
  stats: &mut Stats,
  cached_boards: &mut HashMap<(u128, bool), i128>,
  current_player: bool,
) -> i128 {
  stats.boards_evaluated += 1;

  // TODO: cache only based on hash and add sign to the result based on current player
  let board_hash = (board.hash(), current_player);
  if cached_boards.contains_key(&board_hash) {
    stats.cached_boards_used += 1;
    return cached_boards[&board_hash];
  }

  let score = board
    .get_all_tile_sequences()
    .iter()
    .fold(0, |total, sequence| {
      // total + player - opponent
      total + eval_sequence(&sequence, current_player, false)
        - eval_sequence(&sequence, next_player(current_player), true)
    });

  cached_boards.insert(board_hash, score);

  score
}

fn eval_sequence(sequence: &[&Option<bool>], evaluate_for: bool, is_on_turn: bool) -> i128 {
  let mut score = 0;
  let mut consecutive = 0;
  let mut open_ends = 0;
  let mut has_hole = false;

  for (index, tile) in sequence.iter().enumerate() {
    let tile = *tile;

    if let Some(player) = tile {
      if *player == evaluate_for {
        consecutive += 1
      } else {
        score += shape_score(consecutive, open_ends, has_hole, is_on_turn);
        consecutive = 0;
        open_ends = 0;
      }
    } else if consecutive == 0 {
      open_ends = 1
    } else {
      if !has_hole && index + 1 < sequence.len() && sequence[index + 1] == &Some(evaluate_for) {
        has_hole = true;
        consecutive += 1;
        continue;
      }

      open_ends += 1;

      score += shape_score(consecutive, open_ends, has_hole, is_on_turn);

      consecutive = 0;
      open_ends = 1;
      has_hole = false;
    }
  }

  score += shape_score(consecutive, open_ends, has_hole, is_on_turn);

  score
}

fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool, is_on_turn: bool) -> i128 {
  if consecutive == 0 || open_ends == 0 {
    return 0;
  }

  if has_hole {
    if !is_on_turn {
      return 0;
    }
    return if consecutive == 5 {
      50_000
    } else if consecutive == 4 && open_ends == 2 {
      20_000
    } else {
      0
    };
  }

  match consecutive {
    5 => 100_000,
    4 => match open_ends {
      2 => 50_000,
      1 => {
        if is_on_turn {
          10_000
        } else {
          500
        }
      }
      _ => 0,
    },
    3 => match open_ends {
      2 => {
        if is_on_turn {
          500
        } else {
          50
        }
      }
      1 => 10,
      _ => 0,
    },
    2 => match open_ends {
      2 => 5,
      _ => 0,
    },
    _ => 0,
  }
}

pub type Move = (TilePointer, i128);
fn minimax(
  board: &mut Board,
  stats: &mut Stats,
  cache: &mut Cache,
  current_player: bool,
  remaining_depth: u8,
  alpha: i128,
  beta: i128,
) -> Move {
  let mut available_moves = board.get_empty_tiles();
  available_moves.shuffle(&mut thread_rng());

  let moves_to_consider: Vec<(usize, usize)>;

  if remaining_depth > 0 {
    let mut move_results: Vec<Move> = vec![];

    for move_ in &available_moves {
      board.set_tile(move_, Some(current_player));
      let analysis = evaluate_board(board, stats, &mut cache.boards, current_player);
      board.set_tile(move_, None);

      move_results.push((*move_, analysis));
    }

    move_results.sort_unstable_by_key(|move_result| move_result.1);
    move_results.reverse(); // descending order

    moves_to_consider = move_results[0..5].iter().map(|result| result.0).collect();
  } else {
    moves_to_consider = available_moves;
  }

  let mut best_move = moves_to_consider[0];
  let mut alpha = alpha;

  for move_ in &moves_to_consider {
    board.set_tile(move_, Some(current_player));

    let score: i128 = if remaining_depth > 0 {
      minimax(
        board,
        stats,
        cache,
        next_player(current_player),
        remaining_depth - 1,
        -beta,
        -alpha,
      )
      .1
    } else {
      evaluate_board(board, stats, &mut cache.boards, current_player)
    };

    board.set_tile(move_, None);

    if score > beta {
      stats.pruned += 1;
      return (best_move, beta);
    }
    if score > alpha {
      alpha = score;
      best_move = *move_;
    }
  }

  (best_move, -alpha)
}

pub fn decide(board: &Board, player: bool, analysis_depth: u8) -> (Board, Move, Stats) {
  let mut board = board.clone();
  let mut stats = Stats::new();
  let mut cache = Cache::new();

  let alpha = -(10_i128.pow(10));
  let beta = 10_i128.pow(10);

  let move_ = minimax(
    &mut board,
    &mut stats,
    &mut cache,
    player,
    analysis_depth,
    alpha,
    beta,
  );

  board.set_tile(&move_.0, Some(player));

  println!("cache: boards {:?}", cache.boards.len());

  (board, move_, stats)
}

pub fn decide_with_cache(
  board: &Board,
  player: bool,
  analysis_depth: u8,
  cache: &mut Cache,
) -> (Board, Move, Stats) {
  let mut board = board.clone();
  let mut stats = Stats::new();

  let alpha = -(10_i128.pow(10));
  let beta = 10_i128.pow(10);

  let move_ = minimax(
    &mut board,
    &mut stats,
    cache,
    player,
    analysis_depth,
    alpha,
    beta,
  );

  board.set_tile(&move_.0, Some(player));

  (board, move_, stats)
}
