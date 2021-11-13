use std::fmt;

#[derive(Clone, PartialEq, Eq, Copy)]
pub enum Player {
  X,
  O,
}

impl Player {
  pub fn next(self) -> Player {
    match self {
      Player::X => Player::O,
      Player::O => Player::X,
    }
  }

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
