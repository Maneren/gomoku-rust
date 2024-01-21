use std::{fmt, ops::Not, str::FromStr};

/// A gomoku player
///
/// Can be X or O
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum Player {
  #[allow(missing_docs)] // self-explanatory
  X,
  #[allow(missing_docs)] // self-explanatory
  O,
}

#[derive(Debug)]
pub struct Error(&'static str);
impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
impl std::error::Error for Error {}

impl Player {
  /// Returns the char representation of the player.
  pub fn char(self) -> char {
    match self {
      Player::X => 'x',
      Player::O => 'o',
    }
  }

  /// Returns the index of the player, used for evaluation structs in minimax.
  #[deprecated]
  pub fn index(self) -> usize {
    match self {
      Player::X => 0,
      Player::O => 1,
    }
  }

  /// Tries to convert a char to a player.
  ///
  /// # Errors
  /// Returns an error if the character is not 'x' or 'o'.
  pub fn from_char(c: char) -> Result<Self, Error> {
    match c {
      'x' => Ok(Player::X),
      'o' => Ok(Player::O),
      _ => Err(Error("Unexpected character!")),
    }
  }

  /// Tries to convert a string to a player.
  ///
  /// # Errors
  /// Returns an error if the string is not "x" or "o".
  pub fn from_string(c: &str) -> Result<Self, Error> {
    match c {
      "x" => Ok(Player::X),
      "o" => Ok(Player::O),
      _ => Err(Error("Unexpected character!")),
    }
  }
}
impl Not for Player {
  type Output = Self;

  fn not(self) -> Self::Output {
    match self {
      Player::X => Player::O,
      Player::O => Player::X,
    }
  }
}
impl fmt::Debug for Player {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{:?}",
      match self {
        Player::X => "Player::X",
        Player::O => "Player::O",
      }
    )
  }
}
impl fmt::Display for Player {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.char())
  }
}
impl FromStr for Player {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Player::from_string(s)
  }
}
