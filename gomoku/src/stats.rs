use std::{
  fmt,
  iter::Sum,
  ops::{Add, AddAssign},
};

use super::utils::format_number;

/// Stats for the engine
///
/// Currently only contains the number of nodes evaluated, but more can be added
/// in the future.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub struct Stats {
  /// The number of nodes evaluated by the engine
  pub nodes_evaluated: u32,
  /// The number of nodes pruned by alpha-beta pruning
  pub nodes_pruned: u32,
}
impl Stats {
  /// Create a new stats initialized to 0
  pub fn new() -> Stats {
    Stats {
      nodes_evaluated: 0,
      nodes_pruned: 0,
    }
  }

  /// Increase the number of nodes evaluated by 1
  pub fn evaluate_node(&mut self) {
    self.nodes_evaluated += 1;
  }

  /// Increase the number of pruned nodes by `count`
  pub fn prune_nodes(&mut self, count: u32) {
    self.nodes_pruned += count;
  }
}

impl Default for Stats {
  fn default() -> Self {
    Self::new()
  }
}
impl fmt::Display for Stats {
  #[allow(clippy::cast_precision_loss)]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "Nodes evaluated: {} ({}), Nodes pruned: {}",
      self.nodes_evaluated,
      format_number(self.nodes_evaluated as f32),
      self.nodes_pruned,
    )
  }
}
impl Add for Stats {
  type Output = Stats;

  fn add(self, other: Stats) -> Self::Output {
    Self {
      nodes_evaluated: self.nodes_evaluated + other.nodes_evaluated,
      nodes_pruned: self.nodes_pruned + other.nodes_pruned,
    }
  }
}
impl AddAssign for Stats {
  fn add_assign(&mut self, other: Stats) {
    *self = *self + other;
  }
}
impl Sum for Stats {
  fn sum<I>(iter: I) -> Self
  where
    I: Iterator<Item = Self>,
  {
    iter.fold(Stats::new(), |acc, x| acc + x)
  }
}
