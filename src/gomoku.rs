use std::cmp;

// for shuffling
use rand::seq::SliceRandom;
use rand::thread_rng;

pub mod board;
use board::{Board, TilePointer};

pub type Move = (TilePointer, i128);
type MovesWithPriority = Vec<(Move, i32)>;

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

use cached::proc_macro::cached;
#[cached]
pub fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool, is_on_turn: bool) -> i128 {
  println!(
    "consecutive: {:?}, open_ends: {:?}, has_hole: {:?}, is_on_turn: {:?}",
    consecutive, open_ends, has_hole, is_on_turn
  );

  let score: i128 = if has_hole {
    match consecutive {
      5 => {
        if is_on_turn {
          50_000
        } else {
          0
        }
      }
      4 => {
        if open_ends == 2 && is_on_turn {
          20_000
        } else {
          0
        }
      }
      _ => 0,
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
            300
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

  println!("shape_score: {:?}", score);

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
          println!(
            "sequence: {:?}, index: {:?}, has_hole: {:?}",
            sequence, index, has_hole
          );

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

fn wins_loses_from_sequence(
  board: &Board,
  sequence: &[TilePointer],
  current_player: bool,
  wins_or_loses: bool,
) -> MovesWithPriority {
  let mut moves = vec![];
  let mut consecutive = 0;
  let mut before = None;
  let modifier_pritority = if wins_or_loses { 2 } else { 1 };

  for ptr in sequence {
    let tile = board.get_tile(*ptr);
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
              moves.push(((value, 10_000), 2 * modifier_pritority));
              moves.push(((*ptr, 10_000), 2 * modifier_pritority))
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

fn get_forced_moves(board: &Board, current_player: bool) -> Vec<Move> {
  let mut forced_moves = vec![];
  for sequence in &board.sequences {
    forced_moves.append(&mut wins_loses_from_sequence(
      board,
      sequence,
      current_player,
      true,
    ));
    forced_moves.append(&mut wins_loses_from_sequence(
      board,
      sequence,
      !current_player, // opponent
      false,
    ));
  }

  forced_moves.shuffle(&mut thread_rng());
  forced_moves.sort_unstable_by_key(|value| value.1);
  let forced_moves = forced_moves.iter().map(|move_| move_.0).collect();

  forced_moves
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
    self.minimax(
      current_player,
      current_player,
      analysis_depth,
      i128::MIN,
      i128::MAX,
    )
  }

  fn minimax(
    &mut self,
    decide_as: bool,
    current_player: bool,
    remaining_depth: u32,
    alpha: i128,
    beta: i128,
  ) -> Move {
    // let base_score = evaluate_board(&self.board, decide_as);
    // println!("base_score: {:?}", base_score);

    let maxing = decide_as == current_player;

    let forced_moves = get_forced_moves(&self.board, current_player);
    if !forced_moves.is_empty() {
      return *forced_moves.get(0).unwrap();
    }

    // let mut available_moves = find_empty_tiles(&self.board);
    let available_moves = find_empty_tiles(&self.board);
    // available_moves.shuffle(&mut thread_rng());

    let moves_to_consider: Vec<(usize, usize)>;
    let mut best_move;

    if remaining_depth > 0 {
      let mut move_results: Vec<Move> = vec![];
      for move_ in &available_moves {
        let move_ = *move_;
        self.stats.boards_evaluated += 1;

        self.board.set_tile(move_, Some(current_player));
        let analysis = self.evaluate_board(decide_as);
        self.board.set_tile(move_, None);

        move_results.push((move_, analysis));
      }

      move_results.sort_unstable_by_key(|move_result| move_result.1);
      if maxing {
        // descending order
        move_results.reverse();
      }

      moves_to_consider = move_results[0..5].iter().map(|result| result.0).collect();
      best_move = *moves_to_consider.get(0).unwrap();
    } else {
      best_move = *available_moves.get(0).unwrap();
      moves_to_consider = available_moves[1..].to_vec();
    }

    let mut best_score = if maxing { i128::MIN } else { i128::MAX };
    let mut alpha = alpha;
    let mut beta = beta;

    for move_ in &moves_to_consider {
      println!("\n\nmove_: {:?}", move_);

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

  fn evaluate_board(&mut self, current_player: bool) -> i128 {
    let board = &self.board;
    self.stats.boards_evaluated += 1;
    let tile_sequences: Vec<Vec<&Option<bool>>> = board
      .sequences
      .iter()
      .map(|sequence| sequence.iter().map(|ptr| board.get_tile(*ptr)).collect())
      .collect();

    println!("tile_sequences: {:?}", tile_sequences);

    let scores_current: Vec<i128> = tile_sequences
      .iter()
      .map(|sequence| eval_sequence(&sequence, current_player, false))
      .collect();

    println!("current: {:?}", scores_current);
    let score_current: i128 = scores_current.iter().sum();

    let scores_opponent: Vec<i128> = tile_sequences
      .iter()
      .map(|sequence| eval_sequence(&sequence, Utils::next(current_player), true))
      .collect();

    println!("opponent: {:?}", scores_opponent);
    let score_opponent: i128 = scores_opponent.iter().sum();
    let score = score_current - score_opponent;

    println!("current {:?}, opponent {:?}", score_current, score_opponent);
    println!("score {:?}", score);

    score
  }
}
