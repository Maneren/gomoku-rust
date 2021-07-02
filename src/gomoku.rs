// use std::cmp;

// for shuffling
use rand::seq::SliceRandom;
use rand::thread_rng;

pub mod board;
use board::{Board, TilePointer};

pub type Move = (TilePointer, i128);
type MoveWithPriority = (TilePointer, i32);
type MovesWithPriority = Vec<MoveWithPriority>;

fn find_empty_tiles(board: &Board) -> Vec<TilePointer> {
  let mut empty_fields: Vec<TilePointer> = vec![];
  let board_size = board.get_size();

  for y in 0..board_size {
    for x in 0..board_size {
      let ptr = (x, y);
      if board.get_tile(ptr).is_none() {
        empty_fields.push(ptr);
      }
    }
  }

  empty_fields
}

use cached::proc_macro::cached;
#[cached]
fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool, is_on_turn: bool) -> i128 {
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

  score
}

fn eval_sequence(sequence: &[&Option<bool>], evaluate_for: bool, is_on_turn: bool) -> i128 {
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
            score += shape_score(consecutive, open_ends, has_hole, is_on_turn);
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
          score += shape_score(consecutive, open_ends, has_hole, is_on_turn);
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
    score += shape_score(consecutive, open_ends, has_hole, is_on_turn);
  }

  score
}

/* fn wins_from_sequence(
  board: &Board,
  sequence: &[TilePointer],
  current_player: bool,
) -> MovesWithPriority {
  let mut moves: MovesWithPriority = vec![];
  let mut consecutive = 0;
  let mut before: Option<TilePointer> = None;

  for ptr in sequence {
    let ptr = *ptr;
    let tile = board.get_tile(ptr);

    match *tile {
      None => {
        match consecutive {
          4 => {
            if let Some(value) = before {
              moves.push((ptr, 5));
              moves.push((value, 5));
            } else {
              moves.push((ptr, 5));
            }
          }
          3 => {
            if let Some(value) = before {
              moves.push((value, 1));
              moves.push((ptr, 1));
            }
          }
          _ => (),
        }
        consecutive = 0;
        before = Some(ptr);
      }
      Some(player) => {
        if player == current_player {
          consecutive += 1
        } else {
          if consecutive == 4 {
            if let Some(value) = before {
              moves.push((value, 5));
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
      moves.push((value, 5))
    }
  }

  moves
}

fn  get_forced_moves(board: &Board, current_player: bool) -> Vec<TilePointer> {
  let mut forced_moves: MovesWithPriority = vec![];

  for sequence in &board.sequences {
    let mut wins = wins_from_sequence(board, sequence, current_player);
    for move_with_priority in &mut wins {
      move_with_priority.1 *= 2;
    }

    let mut loses = wins_from_sequence(board, sequence, Utils::next(current_player));

    forced_moves.append(&mut wins);
    forced_moves.append(&mut loses);
  }

  forced_moves.shuffle(&mut thread_rng());
  forced_moves.sort_unstable_by_key(|value| value.1);
  let forced_moves = forced_moves.iter().map(|move_| move_.0).collect();

  forced_moves
}*/

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

use std::thread;
pub struct AI {
  pub board: Board,
  pub stats: Stats,
}
impl AI {
  pub fn new(board: Board, stats: Stats) -> AI {
    AI { board, stats }
  }

  pub fn decide(&mut self, current_player: bool, analysis_depth: u32) -> Move {
    self.minimax(
      current_player,
      analysis_depth,
      -(10_i128.pow(10)),
      10_i128.pow(10),
    )
  }

  fn minimax(
    &mut self,
    current_player: bool,
    remaining_depth: u32,
    alpha: i128,
    beta: i128,
  ) -> Move {
    /* let forced_moves = get_forced_moves(&self.board, current_player);
    if !forced_moves.is_empty() {
      let ptr = forced_moves[0];

      self.board.set_tile(ptr, Some(current_player));
      let analysis = self.evaluate_board(current_player);
      self.board.set_tile(ptr, None);

      self.stats.pruned += 1;
      return (ptr, analysis);
    } */

    let mut available_moves = find_empty_tiles(&self.board);
    available_moves.shuffle(&mut thread_rng());

    let moves_to_consider: Vec<(usize, usize)>;

    if remaining_depth > 0 {
      let mut move_results: Vec<Move> = vec![];
      for move_ in &available_moves {
        let move_ = *move_;

        self.board.set_tile(move_, Some(current_player));
        let analysis = self.evaluate_board(current_player);
        self.board.set_tile(move_, None);

        move_results.push((move_, analysis));
      }

      move_results.sort_unstable_by_key(|move_result| move_result.1);
      move_results.reverse(); // descending order

      moves_to_consider = move_results[0..5].iter().map(|result| result.0).collect();
    } else {
      moves_to_consider = available_moves;
    }

    let mut best_move = moves_to_consider[0];
    let mut alpha = alpha;

    // let threads: Vec<_> = moves_to_consider
    //   .iter()
    //   .map(|move_| {
    //     thread::spawn(move || {
    //       let move_ = *move_;
    //       board.set_tile(move_, Some(current_player));

    //       let score: i128 = if remaining_depth > 0 {
    //         self
    //           .minimax(
    //             Utils::next(current_player),
    //             remaining_depth - 1,
    //             -beta,
    //             -alpha,
    //           )
    //           .1
    //       } else {
    //         self.evaluate_board(current_player)
    //       };

    //       self.board.set_tile(move_, None);

    //       return (move_, score);
    //     })
    //   })
    //   .collect();

    // let scores: Vec<Move>;
    // for handle in threads {
    //   scores.push(handle.join().unwrap());
    // }

    for move_ in &moves_to_consider {
      let move_ = *move_;
      self.board.set_tile(move_, Some(current_player));

      let score: i128 = if remaining_depth > 0 {
        self
          .minimax(
            Utils::next(current_player),
            remaining_depth - 1,
            -beta,
            -alpha,
          )
          .1
      } else {
        self.evaluate_board(current_player)
      };

      self.board.set_tile(move_, None);

      if score > beta {
        self.stats.pruned += 1;
        return (best_move, beta);
      }
      if score > alpha {
        alpha = score;
        best_move = move_;
      }
    }

    (best_move, -alpha)
  }

  fn evaluate_board(&mut self, current_player: bool) -> i128 {
    self.stats.boards_evaluated += 1;

    let board = &self.board;
    let tile_sequences: Vec<Vec<&Option<bool>>> = board
      .sequences
      .iter()
      .map(|sequence| sequence.iter().map(|ptr| board.get_tile(*ptr)).collect())
      .collect();

    let score_current = tile_sequences
      .iter()
      .map(|sequence| eval_sequence(&sequence, current_player, false))
      .sum::<i128>();

    let score_opponent = tile_sequences
      .iter()
      .map(|sequence| eval_sequence(&sequence, Utils::next(current_player), true))
      .sum::<i128>();

    score_current - score_opponent
  }
}
