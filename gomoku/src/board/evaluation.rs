use std::{
  iter::Sum,
  ops::{Add, AddAssign, BitOr, BitOrAssign, Index, IndexMut},
};

use super::super::{player::Player, Score};

/// Return score and win state for the given shape
///
/// Shape is defined by number of consecutive symbols, number of open ends and
/// if the shape contains a hole (in that case it is included in consecutive).
pub fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool) -> (Score, bool) {
  if has_hole {
    return match consecutive {
      5.. => (40_000, false),
      4 => match open_ends {
        2 => (20_000, false),
        1 => (500, false),
        _ => (0, false),
      },
      _ => (0, false),
    };
  }

  match consecutive {
    5.. => (100_000_000, true),
    4 => match open_ends {
      2 => (10_000_000, false),
      1 => (100_000, false),
      _ => (0, false),
    },
    3 => match open_ends {
      2 => (5_000_000, false),
      1 => (10_000, false),
      _ => (0, false),
    },
    2 => match open_ends {
      2 => (2_000, false),
      _ => (0, false),
    },
    _ => (0, false),
  }
}

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_shape_score() {
    let shapes = [
      shape_score(0, 0, false),
      shape_score(1, 0, false),
      shape_score(2, 0, false),
      shape_score(3, 0, false),
      shape_score(3, 0, true),
      shape_score(0, 2, false),
      shape_score(1, 2, false),
      shape_score(4, 1, true),
      shape_score(2, 2, false),
      shape_score(3, 1, false),
      shape_score(4, 2, true),
      shape_score(5, 1, true),
      shape_score(5, 2, true),
      shape_score(4, 1, false),
      shape_score(3, 2, false),
      shape_score(4, 2, false),
      shape_score(5, 0, false),
      shape_score(5, 1, false),
      shape_score(5, 2, false),
      shape_score(6, 2, false),
      shape_score(10, 2, false),
    ];

    shapes
      .iter()
      .zip(shapes[1..].iter())
      .enumerate()
      .for_each(|(i, (a, b))| assert!(a.0 <= b.0, "{i}: {a:?} {b:?}"));
  }
}
