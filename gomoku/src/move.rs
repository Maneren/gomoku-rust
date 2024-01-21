use std::fmt;

use super::{Score, TilePointer};

/// A move in the game
///
/// Consists of a target tile and a score, independent of the player
pub struct Move {
  /// Target tile
  pub tile: TilePointer,
  /// Score of the move
  pub score: Score,
}
impl fmt::Debug for Move {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({:?}, {})", self.tile, self.score)
  }
}
