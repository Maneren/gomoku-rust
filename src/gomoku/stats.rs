use std::fmt;

#[derive(Debug, Clone)]
pub struct Stats {
  pub nodes_created: u32,
}
impl Stats {
  pub fn new() -> Stats {
    Stats { nodes_created: 0 }
  }

  pub fn create_node(&mut self) {
    self.nodes_created += 1;
  }
}
impl fmt::Display for Stats {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "nodes_created: {}", self.nodes_created)
  }
}
