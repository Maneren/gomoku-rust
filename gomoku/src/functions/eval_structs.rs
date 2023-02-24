use std::{
  iter::Sum,
  ops::{Add, AddAssign, BitOr, BitOrAssign, Index, IndexMut, MulAssign},
};

use super::super::{player::Player, Score};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EvalScore(pub Score, pub Score);

impl Index<Player> for EvalScore {
  type Output = Score;
  fn index(&self, player: Player) -> &Self::Output {
    match player {
      Player::X => &self.0,
      Player::O => &self.1,
    }
  }
}

impl IndexMut<Player> for EvalScore {
  fn index_mut(&mut self, player: Player) -> &mut Self::Output {
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

impl MulAssign<EvalWinPotential> for EvalScore {
  fn mul_assign(&mut self, other: EvalWinPotential) {
    self.0 *= other.0;
    self.1 *= other.1;
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EvalWin(pub bool, pub bool);

impl Index<Player> for EvalWin {
  type Output = bool;
  fn index(&self, player: Player) -> &Self::Output {
    match player {
      Player::X => &self.0,
      Player::O => &self.1,
    }
  }
}

impl IndexMut<Player> for EvalWin {
  fn index_mut(&mut self, player: Player) -> &mut Self::Output {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Eval {
  pub score: EvalScore,
  pub win: EvalWin,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EvalWinPotential(pub i32, pub i32);

impl Index<Player> for EvalWinPotential {
  type Output = i32;
  fn index(&self, player: Player) -> &Self::Output {
    match player {
      Player::X => &self.0,
      Player::O => &self.1,
    }
  }
}

impl IndexMut<Player> for EvalWinPotential {
  fn index_mut(&mut self, player: Player) -> &mut Self::Output {
    match player {
      Player::X => &mut self.0,
      Player::O => &mut self.1,
    }
  }
}
