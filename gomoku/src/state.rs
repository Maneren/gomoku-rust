use std::fmt;

/// Terminal state of a search node.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum State {
  /// Game not ended.
  NotEnd,
  /// Current player wins.
  Win,
  /// Current player loses.
  Lose,
  /// Draw.
  Draw,
}
impl State {
  /// Whether the position is terminal.
  pub fn is_end(self) -> bool {
    !matches!(self, Self::NotEnd)
  }

  /// Whether the position is a win.
  pub fn is_win(self) -> bool {
    matches!(self, Self::Win)
  }

  /// Whether the position is a loss.
  pub fn is_lose(self) -> bool {
    matches!(self, Self::Lose)
  }

  /// Invert win/loss perspective.
  #[must_use]
  pub fn inversed(self) -> Self {
    match self {
      Self::NotEnd => Self::NotEnd,
      Self::Draw => Self::Draw,
      Self::Win => Self::Lose,
      Self::Lose => Self::Win,
    }
  }
}
impl From<bool> for State {
  fn from(b: bool) -> Self {
    if b { Self::Win } else { Self::NotEnd }
  }
}

impl fmt::Display for State {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::NotEnd => "Not an end",
        Self::Draw => "Draw",
        Self::Win => "Win",
        Self::Lose => "Lose",
      }
    )
  }
}
