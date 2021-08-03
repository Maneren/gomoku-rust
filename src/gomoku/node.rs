use super::{
  check_time, evaluate_board, get_dist_fn, next_player, Board, Cache, Score, Stats, TilePointer,
  ALPHA_DEFAULT,
};
use std::{
  cmp::Ordering,
  fmt,
  sync::{Arc, Mutex},
  time::Instant,
};

#[derive(Eq)]
pub struct Node {
  pub tile: TilePointer,
  player: bool,
  pub score: Score,
  is_end: bool,
  end_time: Instant,
  child_nodes: Vec<Node>,
  remaining_depth: u8,
}
impl Node {
  pub fn new(
    tile: TilePointer,
    player: bool,
    score: Score,
    is_end: bool,
    end_time: Instant,
    start_depth: u8,
  ) -> Node {
    Node {
      tile,
      player,
      score,
      is_end,
      end_time,
      child_nodes: Vec::new(),
      remaining_depth: start_depth,
    }
  }

  pub fn compute_next(
    &mut self,
    board: &mut Board,
    stats_arc: &Arc<Mutex<Stats>>,
    cache_arc: &Arc<Mutex<Cache>>,
    beta: Score,
  ) -> Score {
    if self.is_end || !check_time(self.end_time) {
      return self.score;
    }

    self.remaining_depth += 1;

    self
      .child_nodes
      .iter_mut()
      .map(|child| child.compute_next(board, stats_arc, cache_arc, beta))
      .max()
      .map_or_else(
        || self.eval(board, stats_arc, cache_arc, beta),
        |score| score,
      )
  }

  pub fn eval(
    &mut self,
    board: &mut Board,
    stats_arc: &Arc<Mutex<Stats>>,
    cache_arc: &Arc<Mutex<Cache>>,
    beta: Score,
  ) -> Score {
    if self.remaining_depth == 0 || self.is_end || !check_time(self.end_time) {
      return self.score;
    }

    if self.child_nodes.is_empty() {
      if let Ok(available_moves) = board.get_empty_tiles() {
        self.child_nodes = self.get_child_nodes(&available_moves, board, stats_arc, cache_arc)
      } else {
        self.is_end = true;
        self.score = -100_000;
        return self.score;
      };
    }

    self.score = ALPHA_DEFAULT;

    for node in &mut self.child_nodes {
      let Node {
        tile,
        score,
        is_end,
        ..
      } = *node;

      if is_end {
        stats_arc.lock().unwrap().prune();
        self.score = score;
        break;
      }

      board.set_tile(tile, Some(self.player));
      let score = node.eval(board, stats_arc, cache_arc, beta);
      board.set_tile(tile, None);

      if score > beta {
        stats_arc.lock().unwrap().prune();
        self.score = score;
        break;
      }

      if score > self.score {
        self.score = score;
      }
    }

    -self.score
  }

  fn get_child_nodes(
    &self,
    available_moves: &[TilePointer],
    board: &mut Board,
    stats_arc: &Arc<Mutex<Stats>>,
    cache_arc: &Arc<Mutex<Cache>>,
  ) -> Vec<Node> {
    let dist = get_dist_fn(board.get_size());

    let mut moves: Vec<_> = available_moves
      .iter()
      .map(|&tile| {
        board.set_tile(tile, Some(self.player));
        let (analysis, is_game_end) = evaluate_board(board, stats_arc, cache_arc, self.player);
        board.set_tile(tile, None);

        Node::new(
          tile,
          next_player(self.player),
          -(analysis - dist(tile)),
          is_game_end,
          self.end_time,
          self.remaining_depth - 1,
        )
      })
      .collect();
    moves.sort_unstable();

    moves.into_iter().take(10).collect()
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
      "({:?}, {}, {})",
      self.tile, self.score, self.remaining_depth
    )
  }
}
