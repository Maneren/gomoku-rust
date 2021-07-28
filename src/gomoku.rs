pub mod board;
use board::{Board, Tile, TilePointer};

type Score = i64;
const ALPHA_DEFAULT: Score = -1_000_000_000_000;
const BETA_DEFAULT: Score = 1_000_000_000_000;

#[derive(Debug, Clone)]
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

use std::{
  collections::HashMap,
  fmt,
  sync::{Arc, Mutex},
  thread,
};

pub type Cache = HashMap<u128, (Score, bool)>;

fn next_player(current: bool) -> bool {
  !current
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
      if !has_hole && index + 1 < sequence.len() && *sequence[index + 1] == Some(evaluate_for) {
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

fn evaluate_board(
  board: &mut Board,
  stats_arc: &Arc<Mutex<Stats>>,
  cache_arc: &Arc<Mutex<Cache>>,
  current_player: bool,
) -> Score {
  let mut stats_lock = stats_arc.lock().unwrap();
  stats_lock.boards_evaluated += 1;
  drop(stats_lock);

  let board_hash = board.hash();

  let cache_lock = cache_arc.lock().unwrap();

  if let Some(&(cached_score, owner)) = cache_lock.get(&board_hash) {
    let mut stats_lock = stats_arc.lock().unwrap();
    stats_lock.cached_boards_used += 1;
    drop(stats_lock);

    let score = if current_player == owner {
      cached_score
    } else {
      -cached_score
    };

    return score;
  }
  drop(cache_lock);

  let score = board
    .get_all_tile_sequences()
    .iter()
    .fold(0, |total, sequence| {
      let my_sequence_score = eval_sequence(&sequence, current_player, false);
      let opponent_sequence_score = eval_sequence(&sequence, next_player(current_player), true);

      // total + player - opponent
      total + my_sequence_score - opponent_sequence_score
    });

  let mut cache_lock = cache_arc.lock().unwrap();
  cache_lock.insert(board_hash, (score, current_player));

  score
}

pub struct Move {
  pub tile: TilePointer,
  pub score: Score,
}
impl fmt::Debug for Move {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({:?}, {})", self.tile, self.score)
  }
}

fn sort_moves_by_shallow_eval(
  mut moves: Vec<TilePointer>,
  board: &mut Board,
  stats_arc: &Arc<Mutex<Stats>>,
  cache_arc: &Arc<Mutex<Cache>>,
  current_player: bool,
) -> Vec<TilePointer> {
  let middle = f32::from(board.get_size() - 1) / 2.0;

  let dist = |p1: TilePointer| {
    let x = f32::from(p1.x);
    let y = f32::from(p1.y);
    let raw_dist = (x - middle).powi(2) + (y - middle).powi(2);

    #[allow(clippy::cast_possible_truncation)]
    let dist = raw_dist.round() as Score;

    dist
  };

  moves.sort_by_cached_key(|&tile| {
    board.set_tile(tile, Some(current_player));
    let analysis = evaluate_board(board, stats_arc, cache_arc, current_player);
    board.set_tile(tile, None);

    -(analysis - dist(tile))
  });

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
    let score = evaluate_board(board, stats, cache, current_player);
    board.set_tile(tile, None);

    if score > beta {
      let mut stats_lock = stats.lock().unwrap();
      stats_lock.pruned += 1;
      drop(stats_lock);

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

    let presorted_moves =
      sort_moves_by_shallow_eval(available_moves, board, stats_arc, cache_arc, current_player);

    // then use 10 best of them to eval deeper
    for tile in presorted_moves.into_iter().take(10) {
      board.set_tile(tile, Some(current_player));
      let score = minimax(
        board,
        stats_arc,
        cache_arc,
        next_player(current_player),
        remaining_depth - 1,
        -alpha,
      )
      .score;
      board.set_tile(tile, None);

      if score > beta {
        let mut stats_lock = stats_arc.lock().unwrap();
        stats_lock.pruned += 1;
        drop(stats_lock);

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
  beta: Score,
) -> Move {
  let available_moves = board.get_empty_tiles();

  if available_moves.is_empty() {
    return Move {
      tile: TilePointer { x: 0, y: 0 },
      score: -1_000_000_000,
    };
  }

  let mut best_tile = TilePointer { x: 0, y: 0 };
  let mut alpha = ALPHA_DEFAULT;

  let cache = cache_ref.clone();
  let stats = stats_ref.clone();
  let cache_arc = Arc::new(Mutex::new(cache));
  let stats_arc = Arc::new(Mutex::new(stats));

  if remaining_depth > 0 {
    // eval each move to depth 1,
    // sort them based on (the result and
    // the distance from middle of the board)

    let presorted_moves = sort_moves_by_shallow_eval(
      available_moves,
      board,
      &stats_arc,
      &cache_arc,
      current_player,
    );

    // then use 20 best of them to eval deeper
    let move_results: Vec<Move> = presorted_moves
      .iter()
      .take(50)
      .map(|&tile| {
        board.set_tile(tile, Some(current_player));

        let mut board_copy = board.clone();

        let cache_arc_clone = cache_arc.clone();
        let stats_arc_clone = stats_arc.clone();

        let move_thread = thread::spawn(move || {
          minimax(
            &mut board_copy,
            &stats_arc_clone,
            &cache_arc_clone,
            next_player(current_player),
            remaining_depth - 1,
            -alpha,
          )
        });

        board.set_tile(tile, None);

        (tile, move_thread)
      })
      .map(|(tile, move_thread)| Move {
        tile,
        score: move_thread.join().unwrap().score,
      })
      .collect();

    for Move { tile, score } in move_results {
      if score > alpha {
        alpha = score;
        best_tile = tile;
      }
    }

    *stats_ref = stats_arc.lock().unwrap().to_owned();
    *cache_ref = cache_arc.lock().unwrap().to_owned();

    Move {
      tile: best_tile,
      score: -alpha,
    }
  } else {
    eval_to_depth_one(
      available_moves,
      board,
      current_player,
      &stats_arc,
      &cache_arc,
      beta,
    )
  }
}

pub fn decide(board: &Board, player: bool, analysis_depth: u8) -> (Board, Move, Stats) {
  let mut cache = Cache::new();

  let result = decide_with_cache(board, player, analysis_depth, &mut cache);

  println!("cache: boards {:?}", cache.len());

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

  let move_ = minimax_top_level(
    &mut board,
    &mut stats,
    cache,
    player,
    analysis_depth,
    BETA_DEFAULT,
  );

  board.set_tile(move_.tile, Some(player));

  (board, move_, stats)
}
