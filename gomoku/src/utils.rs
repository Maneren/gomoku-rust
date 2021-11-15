use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};

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

#[allow(
  clippy::cast_precision_loss,
  clippy::cast_possible_truncation,
  clippy::cast_sign_loss
)]
pub fn format_number(number: f32) -> String {
  let sizes = [' ', 'k', 'M', 'G', 'T'];

  let base = 1000.0;
  let i = number.log(base).floor();
  let number = format!("{:.2}", number / base.powi(i as i32));
  if i > 1.0 {
    format!("{}{}", number, sizes[i as usize])
  } else {
    number
  }
}

use regex::{Captures, Regex};
use std::error::Error;

use crate::{Board, Player};

#[allow(dead_code)]
pub fn parse_fen_string(input: &str) -> Result<String, Box<dyn Error>> {
  let input = input.trim().to_owned();

  let (prefix, data) = {
    let splitted: Vec<_> = input.split('|').collect();

    let prefix = splitted.get(0);
    let data = splitted.get(1);

    match (prefix, data) {
      (Some(prefix), Some(data)) => Ok((prefix.to_owned(), data.to_owned())),
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
    let capture = captures.get(0).unwrap().as_str();
    let number = capture.parse().unwrap();
    "-".repeat(number)
  };

  let parse_row = |part| -> Result<String, Box<dyn Error>> {
    // calls replace_function for each match
    let parsed = re.replace_all(part, replace_function).to_string();

    if parsed.len() > size {
      return Err("Row too long".into());
    }

    let length_missing = size - parsed.len();
    let padding = "-".repeat(length_missing);

    Ok(parsed + &padding)
  };

  let mut out = String::new();
  // can't use Iter::fold because of the possible Err
  for x in parts {
    out += &(parse_row(x)? + "\n");
  }

  Ok(out)
}

#[allow(dead_code)]
pub fn is_game_end(board: &Board, current_player: Player) -> bool {
  board
    .sequences()
    .iter()
    .any(|sequence| is_game_end_sequence(sequence, current_player, board))
}

#[allow(dead_code)]
fn is_game_end_sequence(sequence: &[usize], current_player: Player, board: &Board) -> bool {
  let mut consecutive = 0;
  for &tile in sequence {
    if let Some(player) = board.get_tile_raw(tile) {
      if *player == current_player {
        consecutive += 1;
        if consecutive >= 5 {
          return true;
        }
      } else {
        consecutive = 0;
      }
    } else {
      consecutive = 0;
    };
  }

  false
}
