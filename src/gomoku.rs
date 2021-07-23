// for shuffling
use rand::{prelude::ThreadRng, seq::SliceRandom, thread_rng};

pub mod board;
use board::{Board, Tile, TilePointer};

type Score = i64;

#[derive(Debug)]
pub struct Stats {
  pub boards_evaluated: u32,
  pub cached_boards_used: u32,
  pub pruned: u32,
}
impl Stats {
  pub fn new() -> Stats {
    Stats {
      boards_evaluated: 0,
      cached_boards_used: 0,
      pruned: 0,
    }
  }
}

use std::collections::HashMap;
pub struct Cache {
  pub boards: HashMap<u128, (Score, bool)>,
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
  cached_boards: &mut HashMap<u128, (Score, bool)>,
  current_player: bool,
) -> Score {
  stats.boards_evaluated += 1;

  let board_hash = board.hash();

  if let Some(&(cached_score, owner)) = cached_boards.get(&board_hash) {
    stats.cached_boards_used += 1;

    let score = if current_player == owner {
      cached_score
    } else {
      -cached_score
    };

    return score;
  }

  let score = board
    .get_all_tile_sequences()
    .iter()
    .fold(0, |total, sequence| {
      // total + player - opponent
      total + eval_sequence(&sequence, current_player, false)
        - eval_sequence(&sequence, next_player(current_player), true)
    });

  cached_boards.insert(board_hash, (score, current_player));

  score
}

fn eval_sequence(sequence: &[&Tile], evaluate_for: bool, is_on_turn: bool) -> Score {
  let mut score = 0;
  let mut consecutive = 0;
  let mut open_ends = 0;
  let mut has_hole = false;

  for (index, tile) in sequence.iter().enumerate() {
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

fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool, is_on_turn: bool) -> Score {
  if consecutive == 0 || open_ends == 0 {
    return 0;
  }

  if has_hole {
    if !is_on_turn {
      return 0;
    }

    if consecutive == 5 {
      40_000
    } else if consecutive == 4 && open_ends == 2 {
      20_000
    } else {
      0
    }
  } else {
    match consecutive {
      5 => 500_000,
      4 => match open_ends {
        2 => {
          if is_on_turn {
            50_000
          } else {
            40_000
          }
        }
        1 => {
          if is_on_turn {
            40_000
          } else {
            50
          }
        }
        _ => 0,
      },
      3 => match open_ends {
        2 => {
          if is_on_turn {
            5_000
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
}

struct AlphaBeta(Score, Score);

#[derive(Debug)]
pub struct Move {
  pub tile: TilePointer,
  pub score: Score,
}

fn minimax(
  board: &mut Board,
  stats: &mut Stats,
  cache: &mut Cache,
  rng: &mut ThreadRng,
  current_player: bool,
  remaining_depth: u8,
  alpha_beta: &mut AlphaBeta,
) -> Move {
  let mut available_moves = board.get_empty_tiles();

  if available_moves.is_empty() {
    return Move {
      tile: TilePointer { x: 0, y: 0 },
      score: -1_000_000_000_000,
    };
  }

  available_moves.shuffle(rng);

  let board_size = board.get_size();

  #[allow(clippy::cast_precision_loss)]
  let middle = (board_size - 1) as f64 / 2.0;

  #[allow(clippy::cast_possible_truncation)]
  #[allow(clippy::cast_precision_loss)]
  let dist = |p1: TilePointer| {
    let raw_dist = (p1.x as f64 - middle).powi(2) + (p1.y as f64 - middle).powi(2);
    raw_dist.round() as Score
  };

  let tiles_to_consider = if remaining_depth > 0 {
    // eval each move to depth 0,
    // sort them based on the result and
    // the distance from middle of the board,
    // then return 10 best of them
    let mut move_results: Vec<Move> = available_moves
      .iter()
      .map(|tile| {
        board.set_tile(tile, Some(current_player));
        let analysis = evaluate_board(board, stats, &mut cache.boards, current_player);
        board.set_tile(tile, None);

        Move {
          tile: *tile,
          score: analysis - dist(*tile),
        }
      })
      .collect();

    move_results.sort_unstable_by_key(|move_result| move_result.score);
    move_results.reverse(); // descending order

    move_results
      .iter()
      .take(10)
      .map(|result| result.tile)
      .collect()
  } else {
    available_moves
  };

  let mut best_tile = tiles_to_consider[0];
  let AlphaBeta(mut alpha, beta) = alpha_beta;

  for tile in &tiles_to_consider {
    board.set_tile(tile, Some(current_player));

    let score = if remaining_depth > 0 {
      minimax(
        board,
        stats,
        cache,
        rng,
        next_player(current_player),
        remaining_depth - 1,
        &mut AlphaBeta(-*beta, -alpha),
      )
      .score
    } else {
      evaluate_board(board, stats, &mut cache.boards, current_player)
    };

    board.set_tile(tile, None);

    if score > *beta {
      stats.pruned += 1;
      return Move {
        tile: *tile,
        score: -score,
      };
    }

    if score > alpha {
      alpha = score;
      best_tile = *tile;
    }
  }

  Move {
    tile: best_tile,
    score: -alpha,
  }
}

pub fn decide(board: &Board, player: bool, analysis_depth: u8) -> (Board, Move, Stats) {
  let mut cache = Cache::new();

  let result = decide_with_cache(board, player, analysis_depth, &mut cache);

  println!("cache: boards {:?}", cache.boards.len());

  result
}

pub fn decide_with_cache(
  board: &Board,
  player: bool,
  analysis_depth: u8,
  cache: &mut Cache,
) -> (Board, Move, Stats) {
  let mut board = board.clone();
  let mut stats = Stats::new();

  let mut alpha_beta = AlphaBeta(-1_000_000_000_000, 1_000_000_000_000);

  let move_ = minimax(
    &mut board,
    &mut stats,
    cache,
    &mut thread_rng(),
    player,
    analysis_depth,
    &mut alpha_beta,
  );

  board.set_tile(&move_.tile, Some(player));

  (board, move_, stats)
}
