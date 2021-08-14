use super::{
  evaluate_board, get_dist_fn, time_remaining, Board, Move, Player, Score, Stats, TilePointer,
};
use std::{
  cmp::Ordering,
  fmt,
  sync::{Arc, Mutex},
  time::Instant,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum State {
  NotEnd,
  Win,
  Lose,
  Draw,
}
impl State {
  pub fn is_end(self) -> bool {
    !matches!(self, Self::NotEnd)
  }

  pub fn is_win(self) -> bool {
    matches!(self, Self::Win)
  }

  pub fn is_lose(self) -> bool {
    matches!(self, Self::Lose)
  }

  pub fn inversed(self) -> Self {
    match self {
      Self::NotEnd => Self::NotEnd,
      Self::Draw => Self::Draw,
      Self::Win => Self::Lose,
      Self::Lose => Self::Win,
    }
  }
}

#[derive(Clone)]
pub struct Node {
  pub tile: TilePointer,
  pub state: State,
  pub valid: bool,

  score: Score,
  original_score: Score,
  player: Player,
  child_nodes: Vec<Node>,
  depth: u8,

  end_time: Arc<Instant>,
  stats_arc: Arc<Mutex<Stats>>,
}
impl Node {
  pub fn new(
    tile: TilePointer,
    player: Player,
    score: Score,
    state: State,
    end_time: Arc<Instant>,
    stats_arc: Arc<Mutex<Stats>>,
  ) -> Node {
    stats_arc.lock().unwrap().create_node();
    Node {
      tile,
      state,
      valid: true,
      score,
      original_score: score,
      player,
      child_nodes: Vec::new(),
      depth: 0,
      end_time,
      stats_arc,
    }
  }

  pub fn to_move(&self) -> Move {
    Move {
      tile: self.tile,
      score: self.score,
    }
  }

  fn get_best_child(&self) -> Option<&Node> {
    if self.state.is_end() {
      return None;
    }

    self.child_nodes.get(0)
  }

  pub fn compute_next(&mut self, board: &mut Board) {
    if self.state.is_end() {
      return;
    }

    if !time_remaining(&self.end_time) {
      self.valid = false;
      return;
    }

    self.depth += 1;

    board.set_tile(self.tile, Some(self.player));

    if self.child_nodes.is_empty() {
      self.init_child_nodes(board);
    } else {
      for node in &mut self.child_nodes {
        if !time_remaining(&self.end_time) {
          self.valid = false;
          return;
        }

        node.compute_next(board);
      }

      self.eval();
    }

    board.set_tile(self.tile, None);
  }

  fn eval(&mut self) {
    if self.child_nodes.iter().any(|node| !node.valid) {
      self.valid = false;
      return;
    }

    self.child_nodes.sort_unstable_by(|a, b| b.cmp(a));
    let best = self
      .child_nodes
      .get(0)
      .unwrap_or_else(|| panic!("no children in eval"));

    self.score = self.original_score + -best.score;
    self.state = best.state.inversed();

    self.child_nodes.retain(|child| !child.state.is_lose());
  }

  fn init_child_nodes(&mut self, board: &mut Board) {
    let available_tiles;
    if let Ok(tiles) = board.get_empty_tiles() {
      available_tiles = tiles;
    } else {
      // no empty tiles
      self.state = State::Draw;
      self.score = 0;
      return;
    }

    let dist = get_dist_fn(board.get_size());

    let mut nodes: Vec<Node> = available_tiles
      .into_iter()
      .map(|tile| {
        let next_player = self.player.next();

        board.set_tile(tile, Some(next_player));
        let (analysis, is_win) = evaluate_board(board, next_player);
        board.set_tile(tile, None);

        Node::new(
          tile,
          next_player,
          analysis - dist(tile),
          is_win,
          self.end_time.clone(),
          self.stats_arc.clone(),
        )
      })
      .collect();

    nodes.sort_unstable_by(|a, b| b.cmp(a));
    self.child_nodes = nodes.into_iter().take(15).collect();

    self.eval()
  }
}
impl PartialEq for Node {
  fn eq(&self, other: &Self) -> bool {
    self.score == other.score
  }
}
impl PartialOrd for Node {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(&other))
  }
}
impl Eq for Node {}
impl Ord for Node {
  fn cmp(&self, other: &Self) -> Ordering {
    if self.state == other.state && self.depth != other.depth {
      return self.depth.cmp(&other.depth).reverse();
    }

    self.score.cmp(&other.score)
  }
}
impl fmt::Debug for Node {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if f.alternate() {
      if let Some(child) = self.get_best_child() {
        write!(f, "({:?}, {}) => {:#?}", self.tile, self.score, child)
      } else {
        write!(f, "({:?}, {})", self.tile, self.score)
      }
    } else {
      write!(
        f,
        "({:?}, {}, {}, {}, {:?}, {})",
        self.tile,
        self.score,
        self.depth,
        self.player,
        self.state,
        if self.valid { "valid" } else { "invalid" }
      )
    }
  }
}
