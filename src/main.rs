#![warn(clippy::pedantic)]

use std::env;
use std::fs::File;
use std::io::prelude::*;

// mod board;

mod gomoku;
use gomoku::board::{Board};
use gomoku::{Move, Stats, AI};

type Error = Box<dyn std::error::Error>;

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
  let stats = Stats {
    boards_evaluated: 0,
    pruned: 0,
  };

  let mut ai = AI::new(board.clone(), stats);
  let best_move = ai.decide(current_player, 0);

  let mut board = ai.board;

  board.set_tile(best_move.0, Some(current_player));

  (board, best_move, ai.stats)
}
