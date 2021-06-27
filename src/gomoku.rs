use cached::proc_macro::cached;

use std::cmp;
// use std::collections::HashMap;

// for shuffling
use rand::seq::SliceRandom;
use rand::thread_rng;

pub mod board;
use board::{Board, TilePointer};

pub type Move = (TilePointer, i128);
type MovesWithPriority = Vec<(Move, i32)>;

#[cached]
pub fn shape_score(consecutive: u8, open_ends: u8, is_on_turn: bool) -> i128 {
  let score: i128 = match consecutive {
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
          1000
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
  };

  score
}

fn find_empty_tiles(board: &Board) -> Vec<TilePointer> {
  let mut empty_fields: Vec<TilePointer> = vec![];

  for y in 0..board.get_size() {
    for x in 0..board.get_size() {
      let tile = board.get_tile((x, y));
      if *tile == None {
        empty_fields.push((x, y));
      }
    }
  }
  empty_fields
}

fn eval_sequence(tile_sequence: &[&Option<bool>], current_player: bool) -> i128 {
  let mut count_consecutive = 0;
  let mut open_ends = 0;
  let mut owner = current_player;
  let mut score: i128 = 0;
  let is_on_turn = owner != current_player;

  for tile in tile_sequence {
    match *tile {
      Some(player) => {
        if *player == owner {
          count_consecutive += 1
        } else if count_consecutive == 0 {
          count_consecutive = 1
        } else {
          score += shape_score(count_consecutive, open_ends, is_on_turn);
          count_consecutive = 0;
          open_ends = 0;
        }

        owner = *player;
      }

      None => {
        if count_consecutive == 0 {
          open_ends = 1
        } else {
          open_ends += 1;
          score += shape_score(count_consecutive, open_ends, is_on_turn);
          count_consecutive = 0;
          open_ends = 1;
        }
      }
    };
  }

  if count_consecutive > 0 {
    score += shape_score(count_consecutive, open_ends, is_on_turn);
  }

  score
}

struct Utils {}
impl Utils {
  fn next(current: bool) -> bool {
    !current
  }
}

pub struct Stats {
  pub boards_evaluated: u32,
  pub pruned: u32,
}

pub struct AI {
  pub board: Board,
  pub stats: Stats,
}

impl AI {
  pub fn new(board: Board, stats: Stats) -> AI {
    AI { board, stats }
  }

  pub fn decide(&mut self, current_player: bool, analysis_depth: u32) -> Move {
    let alpha = i128::MIN;
    let beta = i128::MAX;
    self.minimax(current_player, current_player, analysis_depth, alpha, beta)
  }

  fn minimax(
    &mut self,
    decide_as: bool,
    current_player: bool,
    remaining_depth: u32,
    alpha: i128,
    beta: i128,
  ) -> Move {
    let maxing = decide_as == current_player;

    let forced_moves = self.get_forced_moves(current_player);
    if !forced_moves.is_empty() {
      return *forced_moves.get(0).unwrap();
    }

    let mut available_moves = find_empty_tiles(&self.board);
    available_moves.shuffle(&mut thread_rng());

    let moves: Vec<(usize, usize)>;
    let mut best_move;

    if remaining_depth > 0 {
      let mut move_results: Vec<Move> = vec![];
      for move_ in &available_moves {
        self.stats.boards_evaluated += 1;

        self.board.set_tile(*move_, Some(current_player));
        let analysis = self.evaluate_board(decide_as);
        self.board.set_tile(*move_, None);

        move_results.push((*move_, analysis));
      }

      move_results.sort_unstable_by_key(|move_result| move_result.1);
      if maxing {
        // descending order
        move_results.reverse();
      }

      moves = move_results[0..5].iter().map(|result| result.0).collect();
      best_move = *moves.get(0).unwrap();
    } else {
      best_move = *available_moves.get(0).unwrap();
      moves = available_moves[1..5].to_vec();
    }

    let mut best_score = if maxing { i128::MIN } else { i128::MAX };
    let mut alpha = alpha;
    let mut beta = beta;

    for move_ in &moves {
      let score: i128;
      self.board.set_tile(*move_, Some(current_player));

      if remaining_depth > 0 {
        let result = self.minimax(
          decide_as,
          Utils::next(current_player),
          remaining_depth - 1,
          alpha,
          beta,
        );
        score = result.1;
      } else {
        score = self.evaluate_board(decide_as);
      }

      self.board.set_tile(*move_, None);

      if maxing {
        alpha = cmp::max(alpha, score);
      } else {
        beta = cmp::min(beta, score);
      }
      if (score > best_score && maxing) || (score < best_score && !maxing) {
        best_score = score;
        best_move = *move_;
      }
      if alpha > beta {
        self.stats.pruned += 1;
        break;
      }
    }

    (best_move, best_score)
  }

  fn wins_loses_from_sequence(
    &self,
    sequence: &[TilePointer],
    current_player: bool,
    wins_or_loses: bool,
  ) -> MovesWithPriority {
    let mut moves = vec![];
    let mut consecutive = 0;
    let mut before = None;
    let modifier_pritority = if wins_or_loses { 2 } else { 1 };

    for ptr in sequence {
      let tile = self.board.get_tile(*ptr);
      match *tile {
        None => {
          match consecutive {
            4 => {
              moves.push(((*ptr, 100_000), 5 * modifier_pritority));
              if let Some(value) = before {
                moves.push(((value, 100_000), 5 * modifier_pritority))
              }
            }
            3 => {
              if let Some(value) = before {
                moves.push(((value, 10000), 2 * modifier_pritority));
                moves.push(((*ptr, 10000), 2 * modifier_pritority))
              }
            }
            _ => (),
          }
          consecutive = 0;
          before = Some(*ptr);
        }
        Some(player) => {
          if player == current_player {
            consecutive += 1
          } else {
            if consecutive == 4 {
              if let Some(value) = before {
                moves.push(((value, 100_000), 5 * modifier_pritority))
              }
            }
            consecutive = 0;
            before = None;
          }
        }
      }
    }

    if consecutive == 4 {
      if let Some(value) = before {
        moves.push(((value, 100_000), 5 * modifier_pritority))
      }
    }

    moves
  }

  fn get_forced_moves(&self, current_player: bool) -> Vec<Move> {
    let mut forced_moves = vec![];
    for sequence in &self.board.sequences {
      forced_moves.append(&mut self.wins_loses_from_sequence(sequence, current_player, true));
      forced_moves.append(&mut self.wins_loses_from_sequence(
        sequence,
        !current_player, // opponent
        false,
      ));
    }

    forced_moves.shuffle(&mut thread_rng());
    forced_moves.sort_unstable_by_key(|value| value.1);
    forced_moves.iter().map(|move_| move_.0).collect()
  }

  fn evaluate_board(&mut self, current_player: bool) -> i128 {
    self
      .board
      .sequences
      .iter()
      .map(|sequence| {
        let tile_sequence: Vec<&Option<bool>> = sequence
          .iter()
          .map(|ptr| self.board.get_tile(*ptr))
          .collect();
        eval_sequence(&tile_sequence, current_player)
      })
      .sum()
  }
}
