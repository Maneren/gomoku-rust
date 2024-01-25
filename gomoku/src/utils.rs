use std::{
  sync::atomic::Ordering,
  time::{Duration, Instant},
};

/// Check if the engine should continue running.
///
/// This is done by reading the `END` variable.
#[inline]
pub(crate) fn do_run() -> bool {
  !END.load(Ordering::Acquire)
}

/// Print an engine status message to the console.
///
/// The format is: `<msg> (<time> remaining)`
pub(crate) fn print_status(msg: &str, end_time: &Instant) {
  println!(
    "{} ({:?} remaining)",
    msg,
    (*end_time)
      .checked_duration_since(Instant::now())
      .unwrap_or(Duration::ZERO)
  );
}

/// Format a number into a readable string with SI suffix.
///
/// # Examples
/// #[doctest]
/// ```rust
/// # use gomoku_lib::utils::format_number;
///
/// assert_eq!(format_number(0.0), "0");
/// assert_eq!(format_number(1.1), "1.1");
/// assert_eq!(format_number(1000.0), "1k");
/// assert_eq!(format_number(50000.0), "50k");
/// assert_eq!(format_number(1250000.0), "1.25M");
/// ```
pub fn format_number(input: f32) -> String {
  let (number, i) = if input < 1.0 {
    (input, 0.0)
  } else {
    let base = 1000.0;
    let i = input.log(base).floor();
    (input / base.powi(i as i32), i)
  };

  let string = format!("{number:.2}")
    .trim_end_matches('0')
    .trim_end_matches('.')
    .to_owned();

  if i >= 1.0 {
    let sizes = ['-', 'k', 'M', 'G', 'T'];
    format!("{string}{}", sizes[i as usize])
  } else {
    string
  }
}

#[cfg(feature = "fen")]
pub use fen::{parse_fen_string, to_fen_string};

#[cfg(feature = "fen")]
mod fen {
  use std::error::Error;

  use regex::{Captures, Regex};

  use crate::Board;

  /// Helper function for replacing all matches in a string using a replacement function
  fn replace_all<E>(
    re: &Regex,
    haystack: &str,
    replacement: impl Fn(&Captures) -> Result<String, E>,
  ) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
      let m = caps.get(0).expect("capture group 0 is guaranteed to exist");
      new.push_str(&haystack[last_match..m.start()]);
      new.push_str(&replacement(&caps)?);
      last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
  }

  /// Parses an shortened FEN string to full one
  ///
  /// Expects the input to be in the format `size|data`, where data is a string of rows
  /// separated by `/` and each row contains `x`, `o`, `-` or a number specifying the count of `-`.
  ///
  /// # Errors
  /// Returns an error if the format is incorrect, size doesn't match the line count or line length,
  /// or the data contains invalid characters.
  #[allow(clippy::missing_panics_doc)] // https://github.com/rust-lang/rust-clippy/issues/11436
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

    let re = Regex::new(r"\d+").expect("the regex is valid");

    let replace_function = |captures: &Captures| -> Result<String, Box<dyn Error>> {
      let number = captures[0].parse()?;
      Ok("-".repeat(number))
    };

    let parse_row = |part| -> Result<String, Box<dyn Error>> {
      let parsed = replace_all(&re, part, replace_function)?.to_string();

      if parsed.len() > size {
        return Err("Row too long".into());
      }

      let padding = "-".repeat(size - parsed.len());

      Ok(parsed + &padding)
    };

    parts
      .into_iter()
      .map(parse_row)
      .collect::<Result<Vec<_>, _>>()
      .map(|rows| rows.join("/"))
  }

  /// Converts a board to a shortened FEN string
  #[must_use]
  #[allow(clippy::missing_panics_doc)] // https://github.com/rust-lang/rust-clippy/issues/11436
  pub fn to_fen_string(board: &Board) -> String {
    let re = Regex::new(r#"-+"#).expect("the regex is valid");

    let replace_function = |captures: &Captures| captures[0].len().to_string();

    let compress_row = |row: String| -> String {
      re.replace_all(row.trim_end_matches('-'), replace_function)
        .to_string()
    };

    let data = board
      .tiles()
      .chunks(board.size() as usize)
      .map(|row| {
        row
          .iter()
          .map(|tile| match tile {
            Some(player) => player.char(),
            None => '-',
          })
          .collect()
      })
      .map(compress_row)
      .collect::<Vec<_>>()
      .join("/");

    format!("{}|{}", board.size(), data)
  }
}

use crate::{Board, Player, END};

/// Check if the game has ended.
///
/// Iterate over all sequences and check if any of them is a win or loss for the current player.
pub fn is_game_end(board: &Board, current_player: Player) -> bool {
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

  board
    .sequences()
    .iter()
    .any(|sequence| is_game_end_sequence(sequence, current_player, board))
}
