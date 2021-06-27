#![warn(clippy::pedantic)]

use std::cmp;
use std::env;
use std::fs::File;
use std::io::prelude::*;

// for shuffling
use rand::seq::SliceRandom;
use rand::thread_rng;

mod board;
use board::{Board, TilePointer};

type Error = Box<dyn std::error::Error>;
type Move = (TilePointer, i128);
type MovesWithPriority = Vec<(Move, i32)>;

fn main() {
  match &env::args().collect::<Vec<String>>()[..] {
    [_, path, player] => match run(path, player) {
      Ok(_) => println!("Done!"),
      Err(msg) => println!("Error: {}", msg),
    },
    _ => println!("Usage: gomoku input_file player"),
  }
}

fn run(path_to_input: &str, player: &str) -> Result<(), Error> {
  let input_string = load_input(&path_to_input)?;
  let board = Board::from_string(&input_string)?;

  let player = if player == "x" {
    true
  } else if player == "o" {
    false
  } else {
    panic!("Invalid player")
  };

  println!("{}", board);

  println!("Solving!\n");

  let start = std::time::Instant::now();

  let (solved, best_move, stats) = solve(&board, player);

  let run_time = start.elapsed().as_micros();

  // println!("{}", render_solution(&board, &solved));
  println!(
    "evaluated {} boards, a-b pruned {} times\n",
    stats.boards_evaluated, stats.pruned
  );

  println!("{}", solved);
  println!("{:?}", best_move);
  if run_time < 5000 {
    println!("Time taken: {} \u{03bc}s", run_time);
  } else {
    println!("Time taken: {} ms", run_time / 1000);
  }

  Ok(())
}

fn load_input(path: &str) -> Result<String, Error> {
  let mut file = File::open(path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(contents)
}

fn solve(board: &Board, current_player: bool) -> (Board, Move, Stats) {
  let mut board = board.clone();
  let mut stats = Stats {
    boards_evaluated: 0,
    pruned: 0,
  };

  let best_move = AI::decide(&mut board, current_player, 6, &mut stats);

  board.set_tile(best_move.0, Some(current_player));

  (board, best_move, stats)
}

pub struct Stats {
  boards_evaluated: u32,
  pruned: u32,
}

struct Utils {}

impl Utils {
  fn next(current: bool) -> bool {
    !current
  }
}

struct AI {}
impl AI {
  pub fn decide(
    board: &mut Board,
    current_player: bool,
    analysis_depth: u32,
    stats: &mut Stats,
  ) -> Move {
    let alpha = i128::MIN;
    let beta = i128::MAX;
    AI::minimax(
      board,
      current_player,
      current_player,
      analysis_depth,
      alpha,
      beta,
      stats,
    )
  }

  fn minimax(
    board: &mut Board,
    decide_as: bool,
    current_player: bool,
    remaining_depth: u32,
    alpha: i128,
    beta: i128,
    stats: &mut Stats,
  ) -> Move {
    let maxing = decide_as == current_player;

    let forced_moves = AI::get_forced_moves(board, current_player);
    if !forced_moves.is_empty() {
      return *forced_moves.get(0).unwrap();
    }

    let mut available_moves = AI::find_empty_tiles(board);
    available_moves.shuffle(&mut thread_rng());

    let moves: Vec<(usize, usize)>;
    let mut best_move;

    if remaining_depth > 0 {
      let mut move_results: Vec<Move> = vec![];
      for move_ in &available_moves {
        stats.boards_evaluated += 1;

        board.set_tile(*move_, Some(current_player));
        let analysis = AI::evaluate_board(board, decide_as);
        board.set_tile(*move_, None);

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
      board.set_tile(*move_, Some(current_player));

      if remaining_depth > 0 {
        let result = AI::minimax(
          board,
          decide_as,
          Utils::next(current_player),
          remaining_depth - 1,
          alpha,
          beta,
          stats,
        );
        score = result.1;
      } else {
        score = AI::evaluate_board(board, decide_as);
      }

      board.set_tile(*move_, None);

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
        stats.pruned += 1;
        break;
      }
    }

    (best_move, best_score)
  }

  fn wins_loses_from_sequence(
    board: &Board,
    sequence: &[TilePointer],
    current_player: bool,
    wins_or_loses: bool
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

  fn get_forced_moves(board: &Board, current_player: bool) -> Vec<Move> {
    let mut forced_moves = vec![];
    for sequence in &board.sequences {
      forced_moves.append(&mut Self::wins_loses_from_sequence(
        board,
        sequence,
        current_player,
        true
      ));
      forced_moves.append(&mut Self::wins_loses_from_sequence(
        board,
        sequence,
        !current_player, // opponent
        false
      ));
    }

    forced_moves.shuffle(&mut thread_rng());
    forced_moves.sort_unstable_by_key(|value| value.1);
    // println!("forced: {:?}", forced_moves);
    forced_moves.iter().map(|move_| move_.0).collect()
  }

  fn evaluate_board(board: &mut Board, current_player: bool) -> i128 {
    let eval_sequence = |sequence: &Vec<TilePointer>| {
      let sequence_tiles = sequence.iter().map(|ptr| board.get_tile(*ptr));

      let get_score = |count_consecutive, open_ends, owner| -> i128 {
        let is_on_turn = owner != current_player;
        AI::gomoku_shape_score(count_consecutive, open_ends, is_on_turn) as i128
      };

      let mut count_consecutive = 0;
      let mut open_ends = 0;
      let mut owner = current_player;
      let mut score: i128 = 0;

      for tile in sequence_tiles {
        match *tile {
          Some(player) => {
            if player == owner {
              count_consecutive += 1
            } else if count_consecutive == 0 {
              count_consecutive = 1
            } else {
              score += get_score(count_consecutive, open_ends, owner);
              count_consecutive = 0;
              open_ends = 0;
            }

            owner = player;
          }

          None => {
            if count_consecutive == 0 {
              open_ends = 1
            } else {
              score += get_score(count_consecutive, open_ends + 1, owner);
              count_consecutive = 0;
              open_ends = 1;
            }
          }
        };
      }

      if count_consecutive > 0 {
        score += get_score(count_consecutive, open_ends, owner);
      }

      score
    };

    board
      .sequences
      .iter()
      .map(|sequence| eval_sequence(&sequence))
      .sum()
  }

  fn gomoku_shape_score(consecutive: u8, open_ends: u8, is_on_turn: bool) -> i128 {
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
    }
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
}
