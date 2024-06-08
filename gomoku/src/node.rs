use std::{cmp::Ordering, fmt};

use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

use super::{
  board::{Board, TilePointer},
  functions::eval_relevant_sequences,
  player::Player,
  r#move::Move,
  state::State,
  stats::Stats,
  utils::do_run,
  Score,
};
use crate::functions::{eval_structs::Eval, score_sqrt};

#[derive(Clone)]
pub struct Node {
  tile: TilePointer,
  player: Player,
  pub state: State,
  pub valid: bool,
  child_nodes: Vec<Node>,

  score: Score,
  first_score: Score,
  first_score_sqrt: Score,
  depth: u8,
}
impl Node {
  pub fn compute_next(&mut self, board: &mut Board, parent_score: Score) -> Stats {
    debug_assert!(!self.state.is_end());

    let mut stats = Stats::new();

    if !do_run() {
      self.valid = false;
      return stats;
    }

    self.depth += 1;

    if self.depth == 1 {
      self.initialize(board, parent_score, &mut stats);
      return stats;
    }

    board.set_tile(self.tile, Some(self.player));

    if self.depth == 2 {
      self.child_nodes = board
        .pointers_to_empty_tiles()
        .map(|tile| Node::new(tile, !self.player, State::NotEnd))
        .collect();

      if self.child_nodes.is_empty() {
        self.state = State::Draw;
        self.score = 0;
        return stats;
      }
    }

    stats += self
      .child_nodes
      .par_iter_mut()
      .map(|node| node.compute_next(&mut board.clone(), self.first_score))
      .sum();

    self.evaluate_children();

    stats
  }

  fn evaluate_children(&mut self) {
    debug_assert!(
      !self.child_nodes.is_empty(),
      "Children empty while state is {}",
      self.state
    );

    if self.child_nodes.iter().any(|node| !node.valid) {
      self.valid = false;
      return;
    }

    self.child_nodes.sort_unstable_by(|a, b| b.cmp(a));

    let limit = match self.depth {
      0 | 1 => unreachable!("depth 0 or 1 means the chilren are yet to be initialized"),
      2 => (self.child_nodes.len() / 2).max(24),
      3 => 16,
      4..=7 => 8,
      8 => 4,
      9.. => 2,
    };

    self.child_nodes.truncate(limit);

    let best = self
      .child_nodes
      .first()
      // PERF: for some reason beyond my comprehesion, the length of the following message may have
      // negative impact on performance, so benchmarks have to be checked when changing it
      .expect("we already checked that the list is not empty");

    self.score = self.first_score_sqrt - best.score / 2;
    self.state = best.state.inversed();

    if self.state != State::NotEnd {
      self.child_nodes = Vec::new();
      return;
    }

    self
      .child_nodes
      .retain(|child| child.state == State::NotEnd);
  }

  fn initialize(&mut self, board: &mut Board, parent_score: Score, stats: &mut Stats) {
    stats.evaluate_node();

    let opponent = !self.player;
    let mut score = parent_score;
    let tile = self.tile;

    score += 20 * board.squared_distance_from_center(tile);

    let Eval {
      score: prev_score, ..
    } = eval_relevant_sequences(board, tile);

    score += prev_score[self.player];
    score -= prev_score[opponent];

    board.set_tile(tile, Some(self.player));

    let Eval {
      score: new_score,
      win: new_win,
    } = eval_relevant_sequences(board, tile);

    score *= -1;
    score += new_score[self.player];
    score -= new_score[opponent];

    board.set_tile(tile, None);

    self.score = score;
    self.first_score = score;
    self.first_score_sqrt = score_sqrt(score);

    self.state = {
      match (new_win[self.player], new_win[opponent]) {
        (true, true) => {
          unreachable!("Invalid win state: {new_win:?} for child node {tile} of node {self:?} on board:\n{board}")
        }
        (true, _) => State::Win,
        (_, true) => State::Lose,
        _ => State::NotEnd,
      }
    };
  }

  pub fn node_count(&self) -> usize {
    self.child_nodes.iter().map(Node::node_count).sum::<usize>() + 1
  }

  pub fn new(tile: TilePointer, player: Player, state: State) -> Node {
    Node {
      tile,
      state,
      valid: true,
      score: 0,
      first_score: 0,
      first_score_sqrt: 0,
      player,
      child_nodes: Vec::new(),
      depth: 0,
    }
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
    Some(self.cmp(other))
  }
}
impl Eq for Node {}
impl Ord for Node {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self.state, other.state) {
      (State::Win, State::Win) => self.score.cmp(&other.score),
      (State::Win, _) => Ordering::Greater,
      (_, State::Win) => Ordering::Less,
      (_, _) => self.score.cmp(&other.score),
    }
  }
}
impl fmt::Debug for Node {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if f.alternate() {
      if self.state.is_end() {
        return write!(
          f,
          "({}, {}, {}, {}, {})",
          self.tile, self.score, self.depth, self.player, self.state
        );
      }

      write!(
        f,
        "({}, {}, {}, {})",
        self.tile, self.score, self.depth, self.player,
      )?;

      if let Some(best) = self.child_nodes.first() {
        write!(f, " -> {best:#?}")?;
      }

      Ok(())
    } else {
      write!(
        f,
        "({}, {}, {}, {}, {}, {})",
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
