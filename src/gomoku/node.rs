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

#[derive(Clone)]
pub struct Node {
  pub tile: TilePointer,
  pub score: Score,
  pub is_end: bool,
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
    is_win: bool,
    end_time: Arc<Instant>,
    stats_arc: Arc<Mutex<Stats>>,
    cache_arc: Arc<Mutex<Cache>>,
  ) -> Node {
    stats_arc.lock().unwrap().create_node();
    Node {
      tile,
      score,
      is_end: is_win,
      valid: true,
      player,
      child_nodes: Vec::new(),
      depth: 0,
      end_time,
      stats_arc,
      cache_arc,
    }
  }

  pub fn compute_next(&mut self, board: &mut Board) {
    if !time_remaining(&self.end_time) {
      self.valid = false;
      return;
    }

    if self.is_end {
      return;
    }

    self.depth += 1;

    board.set_tile(self.tile, Some(self.player));

    if self.child_nodes.is_empty() {
      self.init_child_nodes(board);
    } else {
      self.child_nodes.iter_mut().for_each(|node| {
        node.compute_next(board);
      });
      self.eval();
    }

    if self.is_end {
      board.set_tile(self.tile, None);
      return;
    }

    board.set_tile(self.tile, None);
  }

  pub fn eval(&mut self) {
    if self.is_end || self.depth == 0 {
      return;
    }

    if !time_remaining(&self.end_time) {
      self.valid = false;
      return;
    }

    if self.child_nodes.iter().any(|node| !node.valid) {
      self.valid = false;
      return;
    }

    self.child_nodes.sort_unstable_by(|a, b| a.cmp(b).reverse());
    let Node { score, is_end, .. } = self.child_nodes.get(0).unwrap();

    self.score += -score;
    self.is_end = *is_end;
  }

  fn init_child_nodes(&mut self, board: &mut Board) {
    let available_tiles;
    if let Ok(tiles) = board.get_empty_tiles() {
      available_tiles = tiles;
    } else {
      // no empty tiles
      self.is_end = true;
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

    nodes.sort_unstable_by(|a, b| a.cmp(b).reverse());

    self.child_nodes = nodes.into_iter().take(10).collect();

    let Node { score, is_end, .. } = self.child_nodes.get(0).unwrap();
    self.score += -score;
    self.is_end = *is_end;
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
      is_end: self.is_end,
      valid: self.valid,
      player: self.player,
      child_nodes: Vec::new(),
      depth: self.depth,
      end_time: self.end_time.clone(),
      stats_arc: self.stats_arc.clone(),
      cache_arc: self.cache_arc.clone(),
    }
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
    if self.is_end && other.is_end && self.depth != other.depth {
      self.depth.cmp(&other.depth).reverse()
    } else {
      self.score.cmp(&other.score)
    }
  }
}
impl fmt::Debug for Node {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "({:?}, {}, {}, {}, {}, {})",
      self.tile,
      self.score,
      self.depth,
      self.player,
      self.is_end,
      if self.valid { "valid" } else { "invalid" }
    )
  }
}
