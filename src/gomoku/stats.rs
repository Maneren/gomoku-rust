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
