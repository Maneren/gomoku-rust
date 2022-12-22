use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};

#[inline]
pub fn do_run(end: &Arc<AtomicBool>) -> bool {
  !end.load(Ordering::Relaxed)
}

pub fn print_status(msg: &str, end_time: &Instant) {
  println!(
    "{} ({:?} remaining)",
    msg,
    (*end_time)
      .checked_duration_since(Instant::now())
      .unwrap_or(Duration::ZERO)
  );
}

pub fn format_number(input: f32) -> String {
  let sizes = ['-', 'k', 'M', 'G', 'T'];
  let base = 1000.0;

  let i = input.log(base).floor();
  let number = input / base.powi(i as i32);

  let string = format!("{number:.2}")
    .trim_end_matches('0')
    .trim_end_matches('.')
    .to_owned();

  if i >= 1.0 {
    format!("{string}{}", sizes[i as usize])
  } else {
    string
  }
}

use std::error::Error;

use regex::{Captures, Regex};

use crate::{board, Board, Player};

pub fn parse_fen_string(input: &str) -> Result<String, Box<dyn Error>> {
  let input = input.trim();

  let (prefix, data) = {
    let splitted: Vec<_> = input.split('|').collect();

    match splitted[..] {
      [prefix, data] => Ok((prefix, data)),
      _ => Err("Incorrect format"),
    }
  }?;

  let size = prefix.parse()?;

  let parts: Vec<_> = data.split('/').collect();

  if parts.len() != size {
    return Err("Incorrect row count".into());
  }

  let re = Regex::new(r#"\d+"#).unwrap();

  let replace_function = |captures: &Captures| {
    let number = captures[0].parse().unwrap();
    "-".repeat(number)
  };

  let parse_row = |part| -> Result<String, Box<dyn Error>> {
    // calls replace_function for each match
    let parsed = re.replace_all(part, replace_function).to_string();

    if parsed.len() > size {
      return Err("Row too long".into());
    }

    let padding = "-".repeat(size - parsed.len());

    Ok(parsed + &padding)
  };

  parts
    .into_iter()
    .map(|row| parse_row(row))
    .collect::<Result<Vec<_>, _>>()
    .map(|rows| rows.join("/"))
}

pub fn is_game_end(board: &Board, current_player: Player) -> bool {
  board::sequences()
    .iter()
    .any(|sequence| is_game_end_sequence(sequence, current_player, board))
}

fn is_game_end_sequence(sequence: &[usize], current_player: Player, board: &Board) -> bool {
  let mut consecutive = 0;

  for &tile in sequence {
    if board.get_tile_raw(tile) == &Some(current_player) {
      consecutive += 1;
      if consecutive >= 5 {
        return true;
      }
    } else {
      consecutive = 0;
    };
  }

  false
}
