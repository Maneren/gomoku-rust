#![warn(clippy::pedantic)]

use std::{fs::File, io::prelude::*, time::Instant};

// mod board;

mod gomoku;
use gomoku::{board::Board, board::TilePointer, Move};

type Error = Box<dyn std::error::Error>;

use clap::{value_t, App, Arg, SubCommand};

fn main() {
  let matches = App::new("Gomoku")
    .version("1.0")
    .arg(
      Arg::with_name("player")
        .help("X or O")
        .required(true)
        .index(1)
        .possible_values(&["X", "O", "x", "o"]),
    )
    .arg(
      Arg::with_name("depth")
        .help("depth of the minimax; default = 4")
        .index(2),
    )
    .arg(
      Arg::with_name("start")
        .help("is this player starting")
        .index(3)
        .possible_values(&["true", "false"]),
    )
    .subcommand(SubCommand::with_name("debug").arg(Arg::with_name("path").index(1).required(true)))
    .get_matches();

  let player = match matches.value_of("player").unwrap_or("o") {
    "x" | "X" => true,
    "o" | "O" => false,
    _ => panic!("Invalid player"),
  };

  let start = value_t!(matches, "start", bool).unwrap_or(false);

  let depth = value_t!(matches, "depth", u8).unwrap_or(4);

  if let Some(matches) = matches.subcommand_matches("debug") {
    let path_to_input = matches.value_of("path").unwrap();
    match run_debug(path_to_input, player, depth) {
      Ok(_) => println!("Done!"),
      Err(msg) => println!("Error: {}", msg),
    }
  } else {
    run(player, depth, start);
  }
}

fn run_debug(path_to_input: &str, player: bool, depth: u8) -> Result<(), Error> {
  let input_string = load_input(&path_to_input)?;
  let board = Board::from_string(&input_string)?;

  println!("{}", board);

  println!("Searching to depth {}\n", depth);

  let start = Instant::now();

  let (solved, best_move, stats) = gomoku::decide(&board, player, depth);

  let run_time = start.elapsed().as_micros();

  println!("stats: {:?}", stats);

  println!("{}", solved);
  let Move { tile, score } = best_move;
  println!("{:?}, {:?}", tile, score);
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

fn run(player: bool, depth: u8, start: bool) {
  use text_io::read;

  let board_size = 15;
  let mut board = Board::empty(board_size);
  let mut cache = gomoku::Cache::new();

  let prefix = '!';
  if start {
    let middle = board_size / 2;
    let tile = TilePointer {
      x: middle,
      y: middle,
    };
    board.set_tile(tile, Some(player));
    println!("board:\n{}", board);
    println!("{}{},{}", prefix, tile.x, tile.y);
  }

  loop {
    let line: String = read!("{}\n");
    let line = line.trim().to_string();
    println!("input: {}", line);

    if line.starts_with('$') {
      return;
    }

    let splitted: Vec<_> = line.split(',').collect();
    if splitted.len() != 2 {
      println!("Invalid input: {:?}", splitted);
      continue;
    }

    let x = splitted[0].parse();
    let y = splitted[1].parse();

    if x.is_err() || y.is_err() {
      println!("Invalid input: {:?}", splitted);
      continue;
    }

    let x = x.unwrap();
    let y = y.unwrap();

    let tile_ptr = TilePointer { x, y };

    if board.get_tile(&tile_ptr).is_some() {
      println!("Tile already used");
      continue;
    }

    board.set_tile(tile_ptr, Some(!player));

    if is_game_end(&board, !player) {
      println!("Engine loses!\n$");
      break;
    }

    let start = Instant::now();
    let (_, move_, stats) = gomoku::decide_with_cache(&board, player, depth, &mut cache);
    let run_time = start.elapsed().as_micros();

    if run_time < 5000 {
      println!("Time taken: {} \u{03bc}s", run_time)
    } else if run_time < 5_000_000 {
      println!("Time taken: {} ms", run_time / 1000);
    } else {
      println!("Time taken: {} s", run_time / 1_000_000);
    }

    let Move { tile, score } = move_;
    board.set_tile(tile, Some(player));

    println!("stats: {:?}", stats);
    println!("cache: boards {:?}", cache.boards.len(),);
    println!("score: {:?}", score);
    println!("board:\n{}", board);

    if is_game_end(&board, player) {
      println!("Engine wins!\n$");
      break;
    }

    println!("{}{},{}", prefix, tile.x, tile.y);
  }
}

fn is_game_end(board: &Board, current_player: bool) -> bool {
  board
    .get_all_tile_sequences()
    .iter()
    .any(|sequence| is_game_end_sequence(sequence, current_player))
}

fn is_game_end_sequence(sequence: &[&Option<bool>], current_player: bool) -> bool {
  let mut consecutive = 0;
  for tile in sequence {
    if let Some(player) = tile {
      if *player == current_player {
        consecutive += 1;
        if consecutive >= 5 {
          return true;
        }
      } else {
        consecutive = 0;
      }
    } else {
      consecutive = 0
    };
  }

  false
}
