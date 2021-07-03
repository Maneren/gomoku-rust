#![warn(clippy::pedantic)]

use std::{env, fs::File, io::prelude::*, time::Instant};

// mod board;

mod gomoku;
use gomoku::board::Board;

type Error = Box<dyn std::error::Error>;

fn main() {
  match &env::args().collect::<Vec<String>>()[1..] {
    [path, player, depth] => match run(path, player, depth) {
      Ok(_) => println!("Done!"),
      Err(msg) => println!("Error: {}", msg),
    },
    _ => println!("Usage: gomoku input_file player"),
  }
}

fn run(path_to_input: &str, player: &str, depth: &str) -> Result<(), Error> {
  let depth: u32 = depth.parse()?;

  let input_string = load_input(&path_to_input)?;
  let board = Board::from_string(&input_string)?;

  let player = match player {
    "x" => true,
    "o" => false,
    _ => {
      return Err("Invalid player".into());
    }
  };

  println!("{}", board);

  println!("Searching to depth {}\n", depth);

  let start = Instant::now();

  let (solved, best_move, stats) = gomoku::decide(&board, player, depth);

  let run_time = start.elapsed().as_micros();

  println!(
    "evaluated {} boards, a-b pruned {} times, cached boards used: {}, cached sequences used: {}\n",
    stats.boards_evaluated, stats.pruned, stats.cached_boards_used, stats.cached_sequences_used
  );

  println!("{}", solved);
  println!("{:?}", best_move);
  if run_time < 5000 {
    println!("Time taken: {} \u{03bc}s", run_time)
  } else if run_time < 5_000_000 {
    println!("Time taken: {} ms", run_time / 1000);
  } else {
    println!("Time taken: {} s", run_time / 1_000_000);
  }

  Ok(())
}

fn load_input(path: &str) -> Result<String, Error> {
  let mut file = File::open(path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(contents)
}
