mod board;
mod cache;
mod r#move; // r# to allow reserved keyword as name
mod stats;

pub use board::{Board, Tile, TilePointer};
pub use cache::Cache;
pub use r#move::{Move, MoveWithEnd}; // r# to allow reserved keyword as name
pub use stats::Stats;

use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

type Score = i32;
const ALPHA_DEFAULT: Score = -1_000_000_000;
const BETA_DEFAULT: Score = 1_000_000_000;

fn next_player(current: bool) -> bool {
  !current
}

fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool, is_on_turn: bool) -> (Score, bool) {
  if consecutive == 0 || open_ends == 0 {
    return (0, false);
  }

  if has_hole {
    if !is_on_turn {
      return (0, false);
    }

    if consecutive == 5 {
      (1_000_000, false)
    } else if consecutive == 4 {
      match open_ends {
        2 => (300_000, false),
        1 => (2_000, false),
        _ => (0, false),
      }
    } else {
      (0, false)
    }
  } else {
    match consecutive {
      5 => (10_000_000, true),
      4 => match open_ends {
        2 => {
          if is_on_turn {
            (500_000, false)
          } else {
            (150_000, false)
          }
        }
        1 => {
          if is_on_turn {
            (80_000, false)
          } else {
            (2_000, false)
          }
        }
        _ => (0, false),
      },
      3 => match open_ends {
        2 => {
          if is_on_turn {
            (50_000, false)
          } else {
            (1_000, false)
          }
        }
        1 => (10, false),
        _ => (0, false),
      },
      2 => match open_ends {
        2 => (10, false),
        _ => (0, false),
      },
      _ => (0, false),
    }
  }
}

fn eval_sequence(sequence: &[&Tile], evaluate_for: bool, is_on_turn: bool) -> (Score, bool) {
  let mut score = 0;
  let mut consecutive = 0;
  let mut open_ends = 0;
  let mut has_hole = false;

  let mut is_win = false;

  for (index, tile) in sequence.iter().enumerate() {
    if let Some(player) = tile {
      if *player == evaluate_for {
        consecutive += 1
      } else {
        let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole, is_on_turn);
        is_win |= is_win_shape;
        score += shape_score;

        consecutive = 0;
        open_ends = 0;
      }
    } else if consecutive == 0 {
      open_ends = 1;
      has_hole = false;
    } else {
      if !has_hole && index + 1 < sequence.len() && *sequence[index + 1] == Some(evaluate_for) {
        has_hole = true;
        consecutive += 1;
        continue;
      }

      open_ends += 1;

      let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole, is_on_turn);
      is_win |= is_win_shape;
      score += shape_score;

      consecutive = 0;
      open_ends = 1;
      has_hole = false;
    }
  }

  let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole, is_on_turn);
  is_win |= is_win_shape;
  score += shape_score;

  (score, is_win)
}

fn evaluate_board(
  board: &mut Board,
  stats_arc: &Arc<Mutex<Stats>>,
  cache_arc: &Arc<Mutex<Cache>>,
  current_player: bool,
) -> (Score, bool) {
  stats_arc.lock().unwrap().boards_evaluated += 1;

  if let Some(&(cached_score, owner, is_game_end)) = cache_arc.lock().unwrap().lookup(board) {
    let score = if current_player == owner {
      cached_score
    } else {
      -cached_score
    };

    return (score, is_game_end);
  }

  let mut is_game_end = false;

  let score = board
    .get_all_tile_sequences()
    .into_iter()
    .fold(0, |total, sequence| {
      let (player_score, is_win) = eval_sequence(&sequence, current_player, false);
      let (opponent_score, is_lose) = eval_sequence(&sequence, next_player(current_player), true);

      if is_win || is_lose {
        is_game_end = true;
      }

      // total + player - opponent
      total + player_score - opponent_score
    });

  let cache_data = (score, current_player, is_game_end);
  cache_arc.lock().unwrap().insert(board, cache_data);

  (score, is_game_end)
}

fn moves_sorted_by_shallow_eval(
  moves: &[TilePointer],
  board: &mut Board,
  stats_arc: &Arc<Mutex<Stats>>,
  cache_arc: &Arc<Mutex<Cache>>,
  current_player: bool,
) -> Vec<MoveWithEnd> {
  let middle = f32::from(board.get_size() - 1) / 2.0;

  let dist = |p1: TilePointer| {
    let x = f32::from(p1.x);
    let y = f32::from(p1.y);
    let raw_dist = (x - middle).powi(2) + (y - middle).powi(2);

    #[allow(clippy::cast_possible_truncation)]
    let dist = raw_dist.round() as Score;

    dist
  };

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

fn eval_to_depth_one(
  available_moves: Vec<TilePointer>,
  board: &mut Board,
  current_player: bool,
  stats: &Arc<Mutex<Stats>>,
  cache: &Arc<Mutex<Cache>>,
  beta: Score,
) -> Move {
  let mut best_tile = TilePointer { x: 0, y: 0 };
  let mut alpha = ALPHA_DEFAULT;

  for tile in available_moves {
    board.set_tile(tile, Some(current_player));
    let (score, ..) = evaluate_board(board, stats, cache, current_player);
    board.set_tile(tile, None);

    if score > beta {
      stats.lock().unwrap().pruned += 1;

      return Move {
        tile,
        score: -score,
      };
    }

    if score > alpha {
      alpha = score;
      best_tile = tile;
    }
  }

  Move {
    tile: best_tile,
    score: -alpha,
  }
}

fn minimax(
  board: &mut Board,
  stats_arc: &Arc<Mutex<Stats>>,
  cache_arc: &Arc<Mutex<Cache>>,
  current_player: bool,
  remaining_depth: u8,
  beta: Score,
) -> Move {
  let available_moves = board.get_empty_tiles();

  if available_moves.is_empty() {
    return Move {
      tile: TilePointer { x: 0, y: 0 },
      score: -300_000, // bad but not as bad as losing
    };
  }

  let mut best_tile = TilePointer { x: 0, y: 0 };
  let mut alpha = ALPHA_DEFAULT;

  if remaining_depth > 0 {
    // eval each move to depth 1,
    // sort them based on (the result and
    // the distance from middle of the board)

    let presorted_moves = moves_sorted_by_shallow_eval(
      &available_moves,
      board,
      stats_arc,
      cache_arc,
      current_player,
    );

    // then use 10 best of them to eval deeper
    for MoveWithEnd {
      tile,
      mut score,
      is_end,
    } in presorted_moves.into_iter().take(10)
    {
      board.set_tile(tile, Some(current_player));
      if is_end {
        stats_arc.lock().unwrap().pruned += 1;
      } else {
        score = minimax(
          board,
          stats_arc,
          cache_arc,
          next_player(current_player),
          remaining_depth - 1,
          -alpha,
        )
        .score;
      }
      board.set_tile(tile, None);

      if score > beta {
        stats_arc.lock().unwrap().pruned += 1;

        return Move {
          tile,
          score: -score,
        };
      }

      if score > alpha {
        alpha = score;
        best_tile = tile;
      }
    }

    Move {
      tile: best_tile,
      score: -alpha,
    }
  } else {
    eval_to_depth_one(
      available_moves,
      board,
      current_player,
      stats_arc,
      cache_arc,
      beta,
    )
  }
}

fn minimax_top_level(
  board: &mut Board,
  stats_ref: &mut Stats,
  cache_ref: &mut Cache,
  current_player: bool,
  remaining_depth: u8,
) -> Move {
  let available_moves = board.get_empty_tiles();

  if available_moves.is_empty() {
    return Move {
      tile: TilePointer { x: 0, y: 0 },
      score: -1_000_000_000,
    };
  }

  let best_tile;
  let mut alpha = ALPHA_DEFAULT;
  let beta = BETA_DEFAULT;

  let cache = cache_ref.clone();
  let stats = stats_ref.clone();
  let cache_arc = Arc::new(Mutex::new(cache));
  let stats_arc = Arc::new(Mutex::new(stats));

  if remaining_depth > 0 {
    // eval each move to depth 1,
    // sort them based on (the result and
    // the distance from middle of the board)

    let presorted_moves = moves_sorted_by_shallow_eval(
      &available_moves,
      board,
      &stats_arc,
      &cache_arc,
      current_player,
    );

    let moves_count = 20;

    let results = Vec::with_capacity(moves_count);
    let results_arc = Arc::new(Mutex::new(results));

    let cores = num_cpus::get() * 2;
    let pool = ThreadPool::new(cores);

    for MoveWithEnd { tile, .. } in presorted_moves.into_iter().take(moves_count) {
      let mut board_clone = board.clone();
      board_clone.set_tile(tile, Some(current_player));

      let cache_arc_clone = cache_arc.clone();
      let stats_arc_clone = stats_arc.clone();
      let results_arc_clone = results_arc.clone();

      pool.execute(move || {
        let move_ = minimax(
          &mut board_clone,
          &stats_arc_clone,
          &cache_arc_clone,
          next_player(current_player),
          remaining_depth - 1,
          -alpha,
        );

        let move_ = Move {
          tile,
          score: move_.score,
        };

        let mut results_lock = results_arc_clone.lock().unwrap();
        results_lock.push(move_);

        println!("calculated {}/{}", results_lock.len(), moves_count);
      });
    }
    pool.join();

    println!();

    let move_results_lock = results_arc.lock().unwrap();

    let Move { tile, score } = move_results_lock.iter().max().unwrap();

    alpha = *score;
    best_tile = *tile;
  } else {
    let Move { tile, score } = eval_to_depth_one(
      available_moves,
      board,
      current_player,
      &stats_arc,
      &cache_arc,
      beta,
    );

    best_tile = tile;
    alpha = -score;
  }

  *stats_ref = stats_arc.lock().unwrap().to_owned();
  *cache_ref = cache_arc.lock().unwrap().to_owned();

  Move {
    tile: best_tile,
    score: alpha,
  }
}

pub fn decide(board: &Board, player: bool, analysis_depth: u8) -> (Board, Move, Stats) {
  let mut cache = Cache::new(board.get_size());

  let result = decide_with_cache(board, player, analysis_depth, &mut cache);

  println!("cache: {:?}", cache.stats);

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

  let move_ = minimax_top_level(&mut board, &mut stats, cache, player, analysis_depth);

  board.set_tile(move_.tile, Some(player));

  (board, move_, stats)
}
