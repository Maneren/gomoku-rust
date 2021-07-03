// use std::cmp;

// for shuffling
use rand::seq::SliceRandom;
use rand::thread_rng;

pub mod board;
use board::{Board, TilePointer};

pub type Move = (TilePointer, i128);

pub struct Stats {
  pub boards_evaluated: u32,
  pub pruned: u32,
  pub cached_boards_used: u32,
  pub cached_sequences_used: u32,
}
impl Stats {
  pub fn new() -> Stats {
    Stats {
      boards_evaluated: 0,
      pruned: 0,
      cached_boards_used: 0,
      cached_sequences_used: 0,
    }
  }
}

use std::collections::HashMap;
type SequencesCache = HashMap<(u32, bool, bool), i128>;
type ShapesCache = HashMap<(u8, u8, bool, bool), i128>;
pub struct Cache {
  pub boards: HashMap<(i128, bool), i128>,
  pub sequences: SequencesCache,
  pub shapes: ShapesCache,
}
impl Cache {
  pub fn new() -> Cache {
    Cache {
      boards: HashMap::new(),
      sequences: HashMap::new(),
      shapes: HashMap::new(),
    }
  }
}

fn next_player(current: bool) -> bool {
  !current
}

fn evaluate_board(
  board: &mut Board,
  stats: &mut Stats,
  cache: &mut Cache,
  current_player: bool,
) -> i128 {
  stats.boards_evaluated += 1;

  let board_hash = (board.hash(), current_player);
  if cache.boards.contains_key(&board_hash) {
    stats.cached_boards_used += 1;
    return cache.boards[&board_hash];
  }

  let tile_sequences = board.get_all_tile_sequences();

  let score_current = tile_sequences
    .iter()
    .map(|sequence| eval_sequence(cache, stats, &sequence, current_player, false))
    .sum::<i128>();

  let score_opponent = tile_sequences
    .iter()
    .map(|sequence| {
      eval_sequence(
        cache,
        stats,
        &sequence,
        next_player(current_player),
        true,
      )
    })
    .sum::<i128>();

  let score = score_current - score_opponent;

  cache.boards.insert(board_hash, score);

  score
}

fn eval_sequence(
  cache: &mut Cache,
  stats: &mut Stats,
  sequence: &[&Option<bool>],
  evaluate_for: bool,
  is_on_turn: bool,
) -> i128 {
  let sequence_hash = (hash_sequence(sequence), evaluate_for, is_on_turn);
  let cached_sequences = &mut cache.sequences;
  if cached_sequences.contains_key(&sequence_hash) {
    stats.cached_sequences_used += 1;
    return cached_sequences[&sequence_hash];
  }

  let mut score: i128 = 0;

  let mut consecutive = 0;
  let mut open_ends = 0;
  let mut has_hole = false;

  for (index, tile) in sequence.iter().enumerate() {
    let tile = *tile;

    match tile {
      Some(player) => {
        let player = *player;

        if player == evaluate_for {
          consecutive += 1
        } else {
          if consecutive > 0 {
            score += shape_score(
              &mut cache.shapes,
              consecutive,
              open_ends,
              has_hole,
              is_on_turn,
            );
          }
          consecutive = 0;
          open_ends = 0;
        }
      }
      None => {
        if consecutive == 0 {
          open_ends = 1
        } else {
          if !has_hole && index + 1 < sequence.len() && *sequence[index + 1] == Some(evaluate_for) {
            has_hole = true;
            consecutive += 1;
            continue;
          }

          open_ends += 1;
          score += shape_score(
            &mut cache.shapes,
            consecutive,
            open_ends,
            has_hole,
            is_on_turn,
          );
          consecutive = 0;
          open_ends = 1;

          if has_hole {
            has_hole = false;
          }
        }
      }
    };
  }

  if consecutive > 0 {
    score += shape_score(
      &mut cache.shapes,
      consecutive,
      open_ends,
      has_hole,
      is_on_turn,
    );
  }

  cached_sequences.insert(sequence_hash, score);

  score
}

fn hash_sequence(sequence: &[&Option<bool>]) -> u32 {
  let mut hash = 0;
  for tile in sequence {
    hash += match *tile {
      Some(player) => {
        if *player {
          1
        } else {
          2
        }
      }
      None => 0,
    };
    hash *= 3;
  }
  hash
}

fn shape_score(
  cached_shapes: &mut ShapesCache,
  consecutive: u8,
  open_ends: u8,
  has_hole: bool,
  is_on_turn: bool,
) -> i128 {
  let sequence_hash = (consecutive, open_ends, has_hole, is_on_turn);
  if cached_shapes.contains_key(&sequence_hash) {
    // stats.cached_sequences_used += 1;
    return cached_shapes[&sequence_hash];
  }

  let score: i128 = if has_hole {
    if is_on_turn {
      match consecutive {
        5 => 50_000,
        4 => {
          if open_ends == 2 {
            20_000
          } else {
            0
          }
        }
        _ => 0,
      }
    } else {
      0
    }
  } else {
    match consecutive {
      5 => 100_000,
      4 => match open_ends {
        2 => 50_000,
        1 => {
          if is_on_turn {
            5000
          } else {
            50
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
        1 => 3,
        _ => 0,
      },
      _ => 0,
    }
  };

  cached_shapes.insert(sequence_hash, score);

  score
}

fn minimax(
  board: &mut Board,
  stats: &mut Stats,
  cache: &mut Cache,
  current_player: bool,
  remaining_depth: u32,
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
      let analysis = evaluate_board(board, stats, cache, current_player);
      board.set_tile(move_, None);

      move_results.push((*move_, analysis));
    }

    move_results.sort_unstable_by_key(|move_result| move_result.1);
    move_results.reverse(); // descending order

    moves_to_consider = move_results[0..10].iter().map(|result| result.0).collect();
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
      evaluate_board(board, stats, cache, current_player)
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

pub fn decide(board: &Board, current_player: bool, analysis_depth: u32) -> (Board, Move, Stats) {
  let mut board = board.clone();
  let mut stats = Stats::new();
  let mut cache = Cache::new();

  let move_ = minimax(
    &mut board,
    &mut stats,
    &mut cache,
    current_player,
    analysis_depth,
    -(10_i128.pow(10)),
    10_i128.pow(10),
  );

  println!("cache: {:?}", cache.sequences.len());

  board.set_tile(&move_.0, Some(current_player));

  (board, move_, stats)
}
