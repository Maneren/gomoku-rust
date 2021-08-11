use std::fmt;

#[derive(Debug, Clone)]
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
