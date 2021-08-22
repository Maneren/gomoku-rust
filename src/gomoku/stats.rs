use std::{fmt, ops::Add};

#[derive(Debug, Copy, Clone)]
pub struct Stats {
  pub nodes_evaluated: u32,
}
impl Stats {
  pub fn new() -> Stats {
    Stats { nodes_evaluated: 0 }
  }

  pub fn create_node(&mut self) {
    self.nodes_evaluated += 1;
  }
}
impl fmt::Display for Stats {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Nodes evaluated: {}", self.nodes_evaluated)
  }
}
impl Add for Stats {
  type Output = Stats;

  fn add(self, other: Stats) -> Self::Output {
    Stats {
      nodes_evaluated: self.nodes_evaluated + other.nodes_evaluated,
    }
  }
}
