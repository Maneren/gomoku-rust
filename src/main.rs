#![warn(clippy::pedantic)]
#![allow(clippy::similar_names)]

use std::{
  fs::File,
  io::{self, prelude::Read},
  str::FromStr,
  time::Instant,
};

use gomoku_lib::{self, utils, Board, Move, Player, TilePointer};

type Error = Box<dyn std::error::Error>;

use clap::{Arg, Command};

fn main() {
  let matches = parse_args();

  if let Some(matches) = matches.subcommand_matches("fen") {
    let mut string = matches.value_of_t("string").unwrap();

    // if argument is "--" read from stdin instead
    if string == "--" {
      let mut buffer = String::new();
      let mut stdin = io::stdin();

      if let Err(err) = stdin.read_to_string(&mut buffer) {
        println!("{err}");
        return;
      }

      string = buffer;
    }

    match utils::parse_fen_string(&string) {
      Ok(s) => println!("{s}"),
      Err(err) => println!("{err}"),
    };

    return;
  }

  let threads = matches
    .value_of_t("threads")
    .unwrap_or_else(|_| num_cpus::get());

  gomoku_lib::set_thread_count(threads).unwrap();

  let player = matches.value_of_t("player").unwrap_or(Player::O);

  let time_limit = matches.value_of_t("time").unwrap_or(1000);
  let board_size = matches.value_of_t("board").unwrap_or(15);

  if let Some(path) = matches.value_of("debug") {
    match run_debug(path, player, time_limit) {
      Ok(()) => println!("Done!"),
      Err(msg) => println!("Error: {msg}"),
    }
  } else {
    run(player, time_limit, board_size);
  }
}

fn parse_args() -> clap::ArgMatches {
  Command::new("Gomoku")
    .version("6.2.1")
    .subcommand(
      Command::new("fen").arg(
        Arg::new("string")
          .index(1)
          .required(true)
          .help("Incomplete fen string"),
      ),
    )
    .arg(
      Arg::new("player")
        .help("X or O")
        .index(1)
        .possible_values(["X", "O", "x", "o"]),
    )
    .arg(
      Arg::new("time")
        .help("Time limit in milliseconds (default is 5000)")
        .index(2),
    )
    .arg(
      Arg::new("debug")
        .short('d')
        .long("debug")
        .help("Run in debug mode")
        .takes_value(true)
        .value_name("FILE"),
    )
    .arg(
      Arg::new("threads")
        .short('t')
        .long("threads")
        .help("How many threads to use (default is thread count of your CPU)")
        .takes_value(true),
    )
    .arg(
      Arg::new("board")
        .short('b')
        .long("board")
        .value_name("SIZE")
        .conflicts_with("debug")
        .help("Size of game board")
        .takes_value(true),
    )
    .get_matches()
}

fn run_debug(path_to_input: &str, player: Player, time_limit: u64) -> Result<(), Error> {
  let input_string = load_input(path_to_input)?;
  let mut board = Board::from_string(&input_string)?;

  println!("{board}");

  println!("Searching with max time {time_limit} ms\n");

  let start = Instant::now();

  let result = gomoku_lib::decide(&mut board, player, time_limit);
  let run_time = start.elapsed().as_micros();

  let (best_move, stats) = match result {
    Ok(result) => result,
    Err(err) => {
      println!("Error occured: {err:?}");
      return Ok(());
    }
  };

  println!();
  println!("{stats}");
  println!();
  println!("{board}");
  let Move { tile, score } = best_move;
  println!("{tile:?}, {score:?}");

  print_runtime(run_time);

  Ok(())
}

fn load_input(path: &str) -> Result<String, Error> {
  let mut file = File::open(path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(contents)
}

fn run(mut player: Player, time_limit: u64, board_size: u8) {
  use text_io::read;
  let mut board = Board::new_empty(board_size);

  let prefix = '!';
  if player == Player::X {
    let middle = board_size / 2;
    let tile = TilePointer {
      x: middle,
      y: middle,
    };
    board.set_tile(tile, Some(player));
    println!("{prefix}{tile:?}");
    player = !player;
  }

  println!("board:\n{board}");

  loop {
    let line: String = read!("{}\n");
    let line = line.trim();
    println!("input: {line}");

    if line.starts_with('$') {
      return;
    }

    let line = if line.starts_with(prefix) {
      &line[1..]
    } else {
      line
    };

    let Ok(tile_ptr) = TilePointer::try_from(line) else {
      println!("Invalid input: {line:?}");
      continue;
    };

    if board.get_tile(tile_ptr).is_some() {
      println!("Tile already used");
      continue;
    }

    board.set_tile(tile_ptr, Some(player));

    if utils::is_game_end(&board, player) {
      println!("Engine loses!\n$");
      println!("{board}");
      break;
    }

    player = !player;

    let start = Instant::now();
    let result = gomoku_lib::decide(&mut board, player, time_limit);
    let run_time = start.elapsed().as_micros();

    let unwrapped = match result {
      Ok(result) => result,
      Err(err) => {
        println!("Error occured: {err:?}");
        continue;
      }
    };

    let (Move { tile, score }, stats) = unwrapped;

    print_runtime(run_time);
    println!();
    println!("{stats}");
    println!("score: {score:?}");
    println!();
    println!("board:\n{board}");

    if utils::is_game_end(&board, player) {
      println!("Engine wins!\n$");
      break;
    }

    println!("{prefix}{tile:?}");
    player = !player;
  }
}

fn print_runtime(run_time: u128) {
  if run_time < 10_000 {
    println!("Time: {run_time} \u{03bc}s");
  } else if run_time < 10_000_000 {
    println!("Time: {} ms", run_time / 1000);
  } else {
    println!("Time: {} s", run_time / 1_000_000);
  }
}
