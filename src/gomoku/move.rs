use super::{Score, TilePointer};
use std::{cmp::Ordering, fmt};

#[derive(Eq)]
pub struct Move {
  pub tile: TilePointer,
  pub score: Score,
}
impl PartialEq for Move {
  fn eq(&self, other: &Self) -> bool {
    self.score == other.score
  }
}
impl PartialOrd for Move {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.score.partial_cmp(&other.score)
  }
}
impl Ord for Move {
  fn cmp(&self, other: &Self) -> Ordering {
    self.score.cmp(&other.score)
  }
}
impl fmt::Debug for Move {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({:?}, {})", self.tile, self.score)
  }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Eq)]
pub struct MoveWithEnd {
  pub tile: TilePointer,
  pub score: Score,
  pub is_end: bool,
}
impl PartialEq for MoveWithEnd {
  fn eq(&self, other: &Self) -> bool {
    self.score == other.score
  }
}
impl PartialOrd for MoveWithEnd {
  fn partial_cmp(&self, other: &MoveWithEnd) -> Option<Ordering> {
    self.score.partial_cmp(&other.score)
  }
}
impl Ord for MoveWithEnd {
  fn cmp(&self, other: &Self) -> Ordering {
    self.score.cmp(&other.score)
  }
}
impl fmt::Debug for MoveWithEnd {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({:?}, {}, {})", self.tile, self.score, self.is_end)
  }
}
