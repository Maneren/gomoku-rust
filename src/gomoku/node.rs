use super::{
  evaluate_board, get_dist_fn, time_remaining, Board, Cache, Move, Player, Score, Stats,
  TilePointer, ALPHA_DEFAULT,
};
use std::{
  cmp::Ordering,
  fmt,
  sync::{Arc, Mutex},
  time::Instant,
};

#[derive(Eq, Clone)]
pub struct Node {
  pub tile: TilePointer,
  pub score: Score,
  pub is_end: bool,
  pub valid: bool,

  player: Player,
  end_time: Instant,
  child_nodes: Vec<Node>,
  remaining_depth: u8,
}
impl Node {
  pub fn new(
    tile: TilePointer,
    player: Player,
    score: Score,
    is_end: bool,
    end_time: Instant,
  ) -> Node {
    Node {
      tile,
      score,
      is_end,
      valid: true,
      player,
      end_time,
      child_nodes: Vec::new(),
      remaining_depth: 0,
    }
  }

  pub fn compute_next(
    &mut self,
    board: &mut Board,
    stats_arc: &Arc<Mutex<Stats>>,
    cache_arc: &Arc<Mutex<Cache>>,
    beta: Score,
  ) -> Score {
    if !time_remaining(self.end_time) {
      self.valid = false;
      return self.score;
    }

    if self.is_end {
      return self.score;
    }

    self.remaining_depth += 1;

    if !self.child_nodes.is_empty() {
      board.set_tile(self.tile, Some(self.player));
      self.child_nodes.iter_mut().for_each(|node| {
        node.compute_next(board, stats_arc, cache_arc, beta);
      });
      board.set_tile(self.tile, None);
    }

    -self.eval(board, stats_arc, cache_arc, beta)
  }

  pub fn eval(
    &mut self,
    board: &mut Board,
    stats_arc: &Arc<Mutex<Stats>>,
    cache_arc: &Arc<Mutex<Cache>>,
    beta: Score,
  ) -> Score {
    if !time_remaining(self.end_time) {
      self.valid = false;
      self.delete_children();
      return -self.score;
    }

    if self.remaining_depth == 0 || self.is_end {
      return -self.score;
    }

    board.set_tile(self.tile, Some(self.player));

    if self.child_nodes.is_empty() {
      if let Ok(available_tiles) = board.get_empty_tiles() {
        self.child_nodes = self.get_child_nodes(available_tiles, board, stats_arc, cache_arc);
      } else {
        self.is_end = true;
        self.score = -100_000;

        board.set_tile(self.tile, None);
        return -self.score;
      };
    }

    let mut best_score = ALPHA_DEFAULT;

    for node in &mut self.child_nodes {
      let Node { score, is_end, .. } = *node;

      if is_end {
        stats_arc.lock().unwrap().prune();
        best_score = score;
        self.is_end = true;
        self.delete_children();
        break;
      }

      let score = -node.eval(board, stats_arc, cache_arc, beta);

      if !node.valid {
        self.valid = false;
        self.delete_children();
        break;
      }

      if score > beta {
        stats_arc.lock().unwrap().prune();
        best_score = score;
        self.is_end = true;
        break;
      }

      if score > best_score {
        best_score = score;
      }
    }

    board.set_tile(self.tile, None);

    self.score = -best_score;

    -self.score
  }

  fn delete_children(&mut self) {
    self.child_nodes.resize(0, self.clone());
  }

  fn get_child_nodes(
    &self,
    available_tiles: Vec<TilePointer>,
    board: &mut Board,
    stats_arc: &Arc<Mutex<Stats>>,
    cache_arc: &Arc<Mutex<Cache>>,
  ) -> Vec<Node> {
    let dist = get_dist_fn(board.get_size());

    let mut nodes: Vec<_> = available_tiles
      .into_iter()
      .map(|tile| {
        let next_player = self.player.next();
        board.set_tile(tile, Some(next_player));
        let (analysis, is_game_end) = evaluate_board(board, stats_arc, cache_arc, next_player);
        board.set_tile(tile, None);

        Node::new(
          tile,
          next_player,
          analysis - dist(tile),
          is_game_end,
          self.end_time,
        )
      })
      .collect();

    nodes.sort_unstable_by_key(|node| -node.score);

    nodes.into_iter().take(10).collect()
  }

  pub fn to_move(&self) -> Move {
    Move {
      tile: self.tile,
      score: self.score,
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
    self.score.partial_cmp(&other.score)
  }
}
impl Ord for Node {
  fn cmp(&self, other: &Self) -> Ordering {
    self.score.cmp(&other.score)
  }
}
impl fmt::Debug for Node {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "({:?}, {}, {}, {}, {}, {})",
      self.tile, self.score, self.remaining_depth, self.player, self.is_end, self.valid
    )
  }
}
