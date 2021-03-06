#![warn(clippy::pedantic)]
#![allow(clippy::similar_names)]

use std::io::{self};
use std::{fs::File, io::prelude::Read, time::Instant};

use gomoku_lib::{self, perf, utils, Board, Move, Player, TilePointer};

type Error = Box<dyn std::error::Error>;

use clap::{value_t, App, Arg};

fn main() {
  let matches = parse_args();

  if let Some(matches) = matches.subcommand_matches("fen") {
    let mut string = value_t!(matches, "string", String).unwrap();

    // if argument is "--" read from stdin instead
    if string == "--" {
      let mut buffer = String::new();
      let mut stdin = io::stdin();

      if let Err(err) = stdin.read_to_string(&mut buffer) {
        println!("{}", err);
        return;
      }

      string = buffer;
    }

    match utils::parse_fen_string(&string) {
      Ok(s) => println!("{}", s),
      Err(err) => println!("{}", err),
    };

    return;
  }

  let threads = value_t!(matches, "threads", usize).unwrap_or_else(|_| num_cpus::get());

  if let Some(matches) = matches.subcommand_matches("perf") {
    let time_limit = value_t!(matches, "time", u64).unwrap_or(10);
    perf(time_limit, threads, 15);
    return;
  }

  let player = match matches.value_of("player").unwrap_or("o") {
    "x" | "X" => Player::X,
    "o" | "O" => Player::O,
    _ => panic!("Invalid player"),
  };

  let time_limit = value_t!(matches, "time", u64).unwrap_or(1000);
  let board_size = value_t!(matches, "board", u8).unwrap_or(15);

  if let Some(path) = matches.value_of("debug") {
    match run_debug(path, player, time_limit, threads) {
      Ok(_) => println!("Done!"),
      Err(msg) => println!("Error: {}", msg),
    }
  } else {
    run(player, time_limit, threads, board_size);
  }
}

fn parse_args<'a>() -> clap::ArgMatches<'a> {
  App::new("Gomoku")
    .version("5.0")
    .subcommand(
      App::new("perf")
        .arg(
          Arg::with_name("threads")
            .short("t")
            .long("threads")
            .help("How many threads to use (default is thread count of your CPU)")
            .takes_value(true),
        )
        .arg(
          Arg::with_name("time")
            .short("m")
            .long("time")
            .help("Time limit in seconds (default is 10)")
            .takes_value(true),
        )
        .arg(
          Arg::with_name("board")
            .short("b")
            .long("board")
            .help("Size of game board")
            .takes_value(true),
        ),
    )
    .subcommand(
      App::new("fen").arg(
        Arg::with_name("string")
          .index(1)
          .required(true)
          .help("Incomplete fen string"),
      ),
    )
    .arg(
      Arg::with_name("player")
        .help("X or O")
        .index(1)
        .possible_values(&["X", "O", "x", "o"]),
    )
    .arg(
      Arg::with_name("time")
        .help("Time limit in milliseconds (default is 1000)")
        .index(2),
    )
    .arg(
      Arg::with_name("debug")
        .short("d")
        .long("debug")
        .help("Run in debug mode")
        .takes_value(true)
        .value_name("FILE"),
    )
    .arg(
      Arg::with_name("threads")
        .short("t")
        .long("threads")
        .help("How many threads to use (default is thread count of your CPU)")
        .takes_value(true),
    )
    .arg(
      Arg::with_name("board")
        .short("b")
        .long("board")
        .value_name("SIZE")
        .conflicts_with("debug")
        .help("Size of game board")
        .takes_value(true),
    )
    .get_matches()
}

fn run_debug(
  path_to_input: &str,
  player: Player,
  time_limit: u64,
  threads: usize,
) -> Result<(), Error> {
  let input_string = load_input(path_to_input)?;
  let mut board = Board::from_string(&input_string)?;

  println!("{}", board);

  println!("Searching with max time {} ms\n", time_limit);

  let start = Instant::now();

  let result = gomoku_lib::decide(&mut board, player, time_limit, threads);
  let run_time = start.elapsed().as_micros();

  let (best_move, stats) = match result {
    Ok(result) => result,
    Err(err) => {
      println!("Error occured: {:?}", err);
      return Ok(());
    }
  };

  println!();
  println!("{}", stats);
  println!();
  println!("{}", board);
  let Move { tile, score } = best_move;
  println!("{:?}, {:?}", tile, score);

  print_runtime(run_time);

  Ok(())
}

fn load_input(path: &str) -> Result<String, Error> {
  let mut file = File::open(path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(contents)
}

fn run(player: Player, time_limit: u64, threads: usize, board_size: u8) {
  use text_io::read;
  let mut board = Board::get_empty_board(board_size);

  let prefix = '!';
  if player == Player::X {
    let middle = board_size / 2;
    let tile = TilePointer {
      x: middle,
      y: middle,
    };
    board.set_tile(tile, Some(player));
    println!("board:\n{}", board);
    println!("{}{:?}", prefix, tile);
  }

  loop {
    let line: String = read!("{}\n");
    let line = line.trim().to_string();
    println!("input: {}", line);

    if line.starts_with('$') {
      return;
    }

    let mut chars = line.chars().peekable();

    if chars.peek() == Some(&prefix) {
      chars.next();
    }

    let x = chars.next();
    let y = chars.collect::<String>().parse::<u8>();

    if x.is_none() || y.is_err() {
      println!("Invalid input: {:?}", line);
      continue;
    }

    let x = x.unwrap() as u8 - 0x61;
    let y = y.unwrap() - 1;

    let tile_ptr = TilePointer { x, y };

    if board.get_tile(&tile_ptr).is_some() {
      println!("Tile already used");
      continue;
    }

    board.set_tile(tile_ptr, Some(player.next()));

    if utils::is_game_end(&board, player.next()) {
      println!("Engine loses!\n$");
      println!("{}", board);
      break;
    }

    let start = Instant::now();
    let result = gomoku_lib::decide(&mut board, player, time_limit, threads);
    let run_time = start.elapsed().as_micros();

    let unwrapped;
    match result {
      Ok(result) => unwrapped = result,
      Err(err) => {
        println!("Error occured: {:?}", err);
        continue;
      }
    }
    let (Move { tile, score }, stats) = unwrapped;

    print_runtime(run_time);
    println!();
    println!("{}", stats);
    println!("score: {:?}", score);
    println!();
    println!("board:\n{}", board);

    if utils::is_game_end(&board, player) {
      println!("Engine wins!\n$");
      break;
    }

    println!("{}{:?}", prefix, tile);
  }
}

fn print_runtime(run_time: u128) {
  if run_time < 10_000 {
    println!("Time: {} \u{03bc}s", run_time);
  } else if run_time < 10_000_000 {
    println!("Time: {} ms", run_time / 1000);
  } else {
    println!("Time: {} s", run_time / 1_000_000);
  }
}
