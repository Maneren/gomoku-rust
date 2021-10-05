use super::{
  evaluate_board, get_dist_fn, do_run, Board, Move, Player, Score, Stats, TilePointer,
};
use std::{
  cmp::Ordering,
  fmt,
  sync::{atomic::AtomicBool, Arc},
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
impl fmt::Display for State {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::NotEnd => "Not an end",
        Self::Draw => "Draw",
        Self::Win => "Win",
        Self::Lose => "Lose",
      }
    )
  }
}

#[derive(Clone)]
struct MoveSequence {
  tile: TilePointer,
  score: Score,
  state: State,
  next: Option<Box<Self>>,
}
impl MoveSequence {
  fn new(node: &Node) -> Self {
    MoveSequence {
      tile: node.tile,
      score: node.score,
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
      write!(f, "({:?}, {}) => {:#?}", self.tile, self.score, child)
    } else if self.state.is_end() {
      write!(f, "({:?}, {}, {})", self.tile, self.score, self.state)
    } else {
      write!(f, "({:?}, {})", self.tile, self.score)
    }
  }
}

#[derive(Clone)]
pub struct Node {
  pub tile: TilePointer,
  pub state: State,
  pub valid: bool,
  pub child_nodes: Vec<Node>,

  score: Score,
  original_score: Score,
  player: Player,
  best_moves: MoveSequence,
  depth: u8,

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
      0 => 10,
      1 | 2 => 5,
      3 | 4 | 5 => 3,
      6 | 7 | 8 => 2,
      _ => 1,
    };
    while self.child_nodes.len() > limit && self.child_nodes.last().unwrap().score < 0 {
      self.child_nodes.pop();
    }

    if !self.child_nodes.is_empty() {
      board.set_tile(self.tile, Some(self.player));

      for node in &mut self.child_nodes {
        node.compute_next(board, stats);

        if !node.valid {
          self.valid = false;
          break;
        }
      }

      board.set_tile(self.tile, None);

      if self.valid {
        self.eval();
      }
    }
  }

  fn eval(&mut self) {
    self.child_nodes.sort_unstable_by(|a, b| b.cmp(a));
    self.analyze_child_nodes();
  }

  fn analyze_child_nodes(&mut self) {
    let best = self
      .child_nodes
      .get(0)
      .unwrap_or_else(|| panic!("no children in eval"));

    self.score = self.original_score / 10 + -best.score;
    self.state = best.state.inversed();

    self.best_moves = MoveSequence::new(&*self);

    self.child_nodes.retain(|child| !child.state.is_lose());
  }

  fn init_child_nodes(&mut self, board: &mut Board, stats: &mut Stats) {
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
        let (analysis, state) = evaluate_board(board, next_player);
        board.set_tile(tile, None);

        Node::new(
          tile,
          next_player,
          analysis - dist(tile),
          state,
          self.end.clone(),
          stats,
        )
      })
      .collect();

    nodes.sort_unstable_by(|a, b| b.cmp(a));
    self.child_nodes = nodes.into_iter().take(10).collect();

    self.analyze_child_nodes();
  }

  pub fn new(
    tile: TilePointer,
    player: Player,
    score: Score,
    state: State,
    end: Arc<AtomicBool>,
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
        score,
        state,
        next: None,
      },
      depth: 0,
      end,
    }
  }

  pub fn node_count(&self) -> usize {
    1 + self
      .child_nodes
      .iter()
      .fold(0, |total, n| total + n.node_count())
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
    if self.state == other.state && self.depth != other.depth {
      return self.depth.cmp(&other.depth).reverse();
    }

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
        "({:?}, {}, {}, {}, {:?}, {}, {})",
        self.tile,
        self.score,
        self.depth,
        self.player,
        self.state,
        if self.valid { "valid" } else { "invalid" },
        self.node_count()
      )
    }
  }
}
