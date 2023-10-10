use std::{fmt, ops::Not, str::FromStr};

#[derive(Clone, PartialEq, Eq, Copy)]
pub enum Player {
  X,
  O,
}

#[derive(Debug)]
pub struct PlayerError(&'static str);
impl fmt::Display for PlayerError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
impl std::error::Error for PlayerError {}

impl Player {
  pub fn char(self) -> char {
    match self {
      Player::X => 'x',
      Player::O => 'o',
    }
  }

  pub fn index(self) -> usize {
    match self {
      Player::X => 0,
      Player::O => 1,
    }
  }

  pub fn from_char(c: char) -> Result<Self, PlayerError> {
    match c {
      'x' => Ok(Player::X),
      'o' => Ok(Player::O),
      _ => Err(PlayerError("Unexpected character!")),
    }
  }

  pub fn from_string(c: &str) -> Result<Self, PlayerError> {
    match c {
      "x" => Ok(Player::X),
      "o" => Ok(Player::O),
      _ => Err(PlayerError("Unexpected character!")),
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
  type Err = PlayerError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Player::from_string(s)
  }
}

