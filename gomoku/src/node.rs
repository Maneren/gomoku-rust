use std::{
  cmp::Ordering,
  fmt,
  sync::{atomic::AtomicBool, Arc},
};

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
use crate::functions::eval_structs::Eval;

#[derive(Clone)]
pub struct MoveSequence {
  pub tile: TilePointer,
  pub score: Score,
  pub original_score: Score,
  pub player: Player,
  pub state: State,
  pub next: Option<Box<Self>>,
}
impl MoveSequence {
  fn new(node: &Node) -> Self {
    MoveSequence {
      tile: node.tile,
      score: node.score,
      original_score: node.original_score,
      player: node.player,
      state: node.state,
      next: node
        .child_nodes
        .get(0)
        .map(|node| Box::new(node.best_moves.clone())),
    }
  }
}

impl fmt::Debug for MoveSequence {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(child) = &self.next {
      write!(
        f,
        "({:?}, {}, {}, {}) => {:#?}",
        self.tile, self.score, self.original_score, self.player, child
      )
    } else if self.state.is_end() {
      write!(
        f,
        "({:?}, {}, {}, {}, {})",
        self.tile, self.score, self.original_score, self.player, self.state
      )
    } else {
      write!(
        f,
        "({:?}, {}, {}, {})",
        self.tile, self.score, self.original_score, self.player
      )
    }
  }
}

#[derive(Clone)]
pub struct Node {
  pub tile: TilePointer,
  pub player: Player,
  pub state: State,
  pub valid: bool,
  pub child_nodes: Vec<Node>,

  score: Score,
  original_score: Score,
  pub best_moves: MoveSequence,
  depth: u8,

  threads: usize,
  end: Arc<AtomicBool>,
}
impl Node {
  pub fn compute_next(&mut self, board: &mut Board, stats: &mut Stats) {
    if self.state.is_end() {
      return;
    }

    if !do_run(&self.end) {
      self.valid = false;
      return;
    }

    self.depth += 1;

    if self.depth <= 1 {
      board.set_tile(self.tile, Some(self.player));
      self.init_child_nodes(board, stats);
      board.set_tile(self.tile, None);

      return;
    }

    let limit = match self.depth {
      0 | 1 => unreachable!(),
      2 | 3 => 16,
      4 | 5 => 12,
      6 | 7 => 8,
      8 | 9 => 4,
      10.. => 2,
    };

    self.child_nodes.truncate(limit);

    board.set_tile(self.tile, Some(self.player));

    // evaluate all child nodes
    let new_stats = self
      .child_nodes
      .par_iter_mut()
      .map(|node| {
        let mut stats = Stats::new();
        node.compute_next(&mut board.clone(), &mut stats);
        stats
      })
      .sum();

    *stats += new_stats;

    board.set_tile(self.tile, None);

    if self.valid {
      self.eval();
    }
  }

  fn eval(&mut self) {
    self.child_nodes.sort_unstable_by(|a, b| b.cmp(a));
    self.analyze_child_nodes();
  }

  fn analyze_child_nodes(&mut self) {
    let best = self.child_nodes.get(0).expect("no children in eval");

    self.score = self.original_score / 10 + -best.score;
    self.state = best.state.inversed();

    self.best_moves = MoveSequence::new(self);

    if self.state != State::NotEnd {
      self.child_nodes = Vec::new();
      return;
    }

    self.child_nodes.retain(|child| !child.state.is_lose());
  }

  fn init_child_nodes(&mut self, board: &mut Board, stats: &mut Stats) {
    let available_tiles = if let Ok(tiles) = board.get_empty_tiles() {
      tiles
    } else {
      // no empty tiles
      self.state = State::Draw;
      self.score = 0;
      return;
    };

    let mut nodes: Vec<Node> = available_tiles
      .into_iter()
      .map(|tile| {
        let next_player = !self.player;
        let mut score = self.original_score;

        let Eval {
          score: prev_score, ..
        } = eval_relevant_sequences(board, tile);

        score -= prev_score[self.player];
        score += prev_score[next_player];

        board.set_tile(tile, Some(next_player));

        let Eval {
          score: new_score,
          win: new_win,
        } = eval_relevant_sequences(board, tile);

        score *= -1;

        score += new_score[next_player];
        score -= new_score[self.player];

        board.set_tile(tile, None);

        let state = {
          match (new_win[next_player], new_win[self.player]) {
            (true, _) => State::Win,
            (_, true) => State::Lose,
            _ => State::NotEnd,
          }
        };

        Node::new(
          tile,
          next_player,
          score - board.squared_distance_from_center(tile),
          state,
          self.end.clone(),
          self.threads,
          stats,
        )
      })
      .collect();

    nodes.sort_unstable_by(|a, b| b.cmp(a));
    nodes.truncate(50);
    self.child_nodes = nodes;

    self.analyze_child_nodes();
  }

  pub fn new(
    tile: TilePointer,
    player: Player,
    score: Score,
    state: State,
    end: Arc<AtomicBool>,
    threads: usize,
    stats: &mut Stats,
  ) -> Node {
    stats.create_node();
    Node {
      tile,
      state,
      valid: true,
      score,
      original_score: score,
      player,
      child_nodes: Vec::new(),
      best_moves: MoveSequence {
        tile,
        player,
        score,
        original_score: score,
        state,
        next: None,
      },
      depth: 0,
      end,
      threads,
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
    self.score.cmp(&other.score)
  }
}
impl fmt::Debug for Node {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if f.alternate() {
      write!(f, "{:?}", self.best_moves)
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
