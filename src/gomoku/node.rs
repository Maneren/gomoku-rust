use super::{
  evaluate_board, get_dist_fn, time_remaining, Board, Cache, Move, Player, Score, Stats,
  TilePointer,
};
use std::{
  cmp::Ordering,
  fmt,
  sync::{Arc, Mutex},
  time::Instant,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum State {
  NotEnded,
  Win,
  Lose,
}
impl State {
  pub fn is_end(self) -> bool {
    !matches!(self, Self::NotEnded)
  }

  pub fn is_win(self) -> bool {
    matches!(self, Self::Win)
  }

  pub fn is_lose(self) -> bool {
    matches!(self, Self::Lose)
  }

  pub fn inversed(self) -> Self {
    match self {
      Self::NotEnded => Self::NotEnded,
      Self::Win => Self::Lose,
      Self::Lose => Self::Win,
    }
  }
}

#[derive(Clone)]
pub struct Node {
  pub tile: TilePointer,
  pub score: Score,
  pub state: State,
  pub valid: bool,

  player: Player,
  child_nodes: Vec<Node>,
  depth: u8,

  end_time: Arc<Instant>,
  stats_arc: Arc<Mutex<Stats>>,
  cache_arc: Arc<Mutex<Cache>>,
}
impl Node {
  pub fn new(
    tile: TilePointer,
    player: Player,
    score: Score,
    state: State,
    end_time: Arc<Instant>,
    stats_arc: Arc<Mutex<Stats>>,
    cache_arc: Arc<Mutex<Cache>>,
  ) -> Node {
    stats_arc.lock().unwrap().create_node();
    Node {
      tile,
      score,
      state,
      valid: true,
      player,
      child_nodes: Vec::new(),
      depth: 0,
      end_time,
      stats_arc,
      cache_arc,
    }
  }

  pub fn to_move(&self) -> Move {
    Move {
      tile: self.tile,
      score: self.score,
    }
  }

  pub fn shallow_clone(&self) -> Node {
    Node {
      tile: self.tile,
      score: self.score,
      state: self.state,
      valid: self.valid,
      player: self.player,
      child_nodes: Vec::new(),
      depth: self.depth,
      end_time: self.end_time.clone(),
      stats_arc: self.stats_arc.clone(),
      cache_arc: self.cache_arc.clone(),
    }
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
    if !time_remaining(&self.end_time) || self.child_nodes.iter().any(|node| !node.valid) {
      self.valid = false;
      return;
    }

    self.child_nodes.sort_unstable_by(|a, b| b.cmp(a));
    let Node { score, state, .. } = self.child_nodes.get(0).unwrap();

    self.score += -score;
    self.state = state.inversed();

    self.child_nodes.retain(|child| !child.state.is_lose());
  }

  fn init_child_nodes(&mut self, board: &mut Board) {
    let available_tiles;
    if let Ok(tiles) = board.get_empty_tiles() {
      available_tiles = tiles;
    } else {
      // no empty tiles
      self.state = State::Lose;
      self.score = -100_000;
      return;
    }

    let dist = get_dist_fn(board.get_size());

    let mut nodes: Vec<Node> = available_tiles
      .into_iter()
      .map(|tile| {
        let next_player = self.player.next();

        board.set_tile(tile, Some(next_player));
        let (analysis, is_win) = evaluate_board(board, &self.cache_arc, next_player);
        board.set_tile(tile, None);

        Node::new(
          tile,
          next_player,
          analysis - dist(tile),
          is_win,
          self.end_time.clone(),
          self.stats_arc.clone(),
          self.cache_arc.clone(),
        )
      })
      .collect();

    nodes.sort_unstable_by(|a, b| b.cmp(a));

    self.child_nodes = nodes.into_iter().take(10).collect();

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
