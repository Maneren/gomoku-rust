#[derive(Debug, Clone)]
pub struct Stats {
  pub boards_evaluated: u32,
  pub pruned: u32,
}
impl Stats {
  pub fn new() -> Stats {
    Stats {
      boards_evaluated: 0,
      pruned: 0,
    }
  }

  pub fn eval(&mut self) {
    self.boards_evaluated += 1;
  }

  pub fn prune(&mut self) {
    self.pruned += 1;
  }
}
