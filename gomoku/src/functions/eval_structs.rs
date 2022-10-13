use std::{
  iter::Sum,
  ops::{Add, AddAssign, BitOr, BitOrAssign, Index, IndexMut},
};

use super::super::{player::Player, Score};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalScore(pub Score, pub Score);

impl Default for EvalScore {
  fn default() -> Self {
    Self(0, 0)
  }
}

impl Index<Player> for EvalScore {
  type Output = Score;
  fn index(&self, player: Player) -> &Score {
    match player {
      Player::X => &self.0,
      Player::O => &self.1,
    }
  }
}

impl IndexMut<Player> for EvalScore {
  fn index_mut(&mut self, player: Player) -> &mut Score {
    match player {
      Player::X => &mut self.0,
      Player::O => &mut self.1,
    }
  }
}

impl Add for EvalScore {
  type Output = Self;
  fn add(self, other: Self) -> Self {
    Self(self.0 + other.0, self.1 + other.1)
  }
}

impl AddAssign for EvalScore {
  fn add_assign(&mut self, other: Self) {
    self.0 += other.0;
    self.1 += other.1;
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalWin(pub bool, pub bool);

impl Default for EvalWin {
  fn default() -> Self {
    Self(false, false)
  }
}

impl Index<Player> for EvalWin {
  type Output = bool;
  fn index(&self, player: Player) -> &bool {
    match player {
      Player::X => &self.0,
      Player::O => &self.1,
    }
  }
}

impl IndexMut<Player> for EvalWin {
  fn index_mut(&mut self, player: Player) -> &mut bool {
    match player {
      Player::X => &mut self.0,
      Player::O => &mut self.1,
    }
  }
}

impl BitOr for EvalWin {
  type Output = Self;
  fn bitor(self, other: Self) -> Self {
    Self(self.0 | other.0, self.1 | other.1)
  }
}

impl BitOrAssign for EvalWin {
  fn bitor_assign(&mut self, other: Self) {
    self.0 |= other.0;
    self.1 |= other.1;
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eval {
  pub score: EvalScore,
  pub win: EvalWin,
}

impl Default for Eval {
  fn default() -> Self {
    Self {
      score: EvalScore::default(),
      win: EvalWin::default(),
    }
  }
}

impl Add for Eval {
  type Output = Self;
  fn add(self, other: Self) -> Self {
    Self {
      score: self.score + other.score,
      win: self.win | other.win,
    }
  }
}

impl Sum for Eval {
  fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
    iter.fold(Eval::default(), |acc, x| acc + x)
  }
}
