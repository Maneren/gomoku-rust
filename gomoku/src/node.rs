use std::{
  cmp::Ordering,
  fmt,
  sync::{atomic::AtomicBool, Arc, Mutex},
};

use threadpool::ThreadPool;

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
      4 | 5 => 8,
      6 | 7 => 4,
      8 | 9 => 2,
      10.. => 1,
    };

    while self.child_nodes.len() > limit {
      self.child_nodes.pop();
    }

    board.set_tile(self.tile, Some(self.player));

    // evaluate all child nodes
    if self.depth <= 4 {
      // single threaded
      for node in &mut self.child_nodes {
        node.compute_next(board, stats);

        if !node.valid {
          self.valid = false;
          break;
        }
      }
    } else {
      // multi threaded
      let pool = ThreadPool::with_name(
        String::from("node"),
        self.threads.min(self.child_nodes.len()),
      );

      let nodes: Vec<Node> = self.child_nodes.drain(..).collect();
      let nodes_arc = Arc::new(Mutex::new(Vec::new()));
      let stats_arc = Arc::new(Mutex::new(Stats::new()));

      for mut node in nodes {
        let mut board_clone = board.clone();
        let nodes_arc_clone = nodes_arc.clone();
        let stats_arc_clone = stats_arc.clone();

        pool.execute(move || {
          let mut stats = Stats::new();

          node.compute_next(&mut board_clone, &mut stats);

          nodes_arc_clone.lock().unwrap().push(node);
          *stats_arc_clone.lock().unwrap() += stats;
        });
      }

      pool.join();

      assert!(pool.panic_count() == 0, "node threads panicked");

      self.child_nodes = nodes_arc.lock().unwrap().drain(..).collect();

      *stats += *stats_arc.lock().unwrap();
    }

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

    if self.state != State::NotEnd {
      self.child_nodes = Vec::new();
      return;
    }

    self.best_moves = MoveSequence::new(self);

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
        let next_player = self.player.next();
        let mut score = self.original_score;

        let (prev_score, ..) = eval_relevant_sequences(board, tile);

        score -= prev_score[self.player.index()];
        score += prev_score[next_player.index()];

        board.set_tile(tile, Some(next_player));

        let (new_score, new_state) = eval_relevant_sequences(board, tile);

        score *= -1;

        score += new_score[next_player.index()];
        score -= new_score[self.player.index()];

        board.set_tile(tile, None);

        let state = {
          let self_state = new_state[next_player.index()];
          let opponent_state = new_state[self.player.index()];

          match (self_state, opponent_state) {
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
    self.child_nodes = nodes.into_iter().take(50).collect();

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
