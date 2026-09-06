use std::{cmp::Ordering, fmt};

use super::{
  Score,
  board::{Board, TilePointer, evaluation::Eval},
  r#move::Move,
  player::Player,
  state::State,
  stats::Stats,
  transposition::{Bound, TTEntry, TranspositionTable},
  utils::{do_run, signed_sqrt},
};

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
  /// Negamax bound. Real scores are shape sums (at most hundreds of millions),
  /// so 1e9 is safely above any reachable score while `-INF` stays negatable
  /// (unlike `Score::MIN`, whose negation overflows).
  pub const INF: Score = 2_000_000_000;

  /// Discounted value of a child reply from this node's perspective.
  ///
  /// This is the decaying-sqrt value function preserved from the original
  /// engine: `first_sqrt` is the static eval of the move that created this
  /// node (with sqrt decay), `child_score` is the opponent's best reply.
  /// The `/ 2` is the depth discount — deeper lines count at half weight —
  /// which keeps deep speculative wins from dominating shallow tactics.
  #[inline]
  fn discounted_value(self_first_sqrt: Score, child_score: Score) -> Score {
    self_first_sqrt - child_score / 2
  }

  #[allow(clippy::too_many_lines)]
  pub fn compute_next(
    &mut self,
    board: &mut Board,
    parent_score: Score,
    mut alpha: Score,
    beta: Score,
    tt: &TranspositionTable,
  ) -> Stats {
    if self.state.is_end() {
      return Stats::new();
    }

    let mut stats = Stats::new();

    if !do_run() {
      self.valid = false;
      return stats;
    }

    self.depth += 1;

    // Transposition table probe: only terminal positions (Win/Lose/Draw)
    // are path-independent and can be reused for cutoff. Non-terminal
    // scores depend on the ancestry first_score chain (decaying sqrt) and
    // on beam truncation, so they are only used for move ordering via peek.
    let tt_hash = board.hash_with_player(self.player);
    let orig_alpha = alpha;
    let orig_beta = beta;
    if let Some(entry) = tt.probe(tt_hash, self.depth, alpha, beta)
      && entry.state != State::NotEnd
    {
      stats.tt_hit();
      self.score = entry.score;
      self.state = entry.state;
      self.child_nodes = Vec::new();
      return stats;
    }

    if self.depth == 1 {
      self.initialize(board, parent_score, &mut stats);
      // Store leaf in TT as exact.
      tt.store(
        tt_hash,
        TTEntry {
          score: self.score,
          depth: self.depth,
          bound: Bound::Exact,
          best_move: Some(self.tile),
          state: self.state,
        },
      );
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
        tt.store(
          tt_hash,
          TTEntry {
            score: 0,
            depth: self.depth,
            bound: Bound::Exact,
            best_move: None,
            state: State::Draw,
          },
        );
        return stats;
      }

      // Static ordering for the first deep expansion: children have no
      // search scores yet, so seed with a cheap heuristic (center distance).
      // Center-first approximates descending child score (most dangerous
      // for us) which maximises beta/refutation cutoffs in the in-loop
      // prune; the true scores arrive after the next depth iteration.
      self
        .child_nodes
        .sort_unstable_by_key(|c| board.squared_distance_from_center(c.tile));
    }

    // TT move ordering: if the transposition table remembers a best move
    // for this position, put it first before the score-based sort. The
    // score sort then keeps the TT move at the front while ordering the
    // rest.
    if let Some(tt_entry) = tt.peek(board.hash_with_player(!self.player))
      && let Some(bm) = tt_entry.best_move
      && let Some(pos) = self.child_nodes.iter().position(|c| c.tile == bm)
    {
      self.child_nodes.swap(0, pos);
    }
    // Best-first ordering for the current player. Child scores are from the
    // opponent's perspective, so ascending order visits our best moves first
    // and maximizes beta cutoffs. Keep TT move at front, sort the tail.
    if self.child_nodes.len() > 1 {
      self.child_nodes[1..].sort_unstable();
      // Ensure the whole array is still best-first: if TT move was not the
      // smallest, bubble the true smallest to front only if TT move is not
      // winning. For simplicity keep TT at front for now — its ordering was
      // from a deeper search and is likely best.
    } else {
      self.child_nodes.sort_unstable();
    }

    let mut best = -Self::INF;

    for i in 0..self.child_nodes.len() {
      if !do_run() {
        self.valid = false;
        return stats;
      }

      // Skip already-terminal children — their win/loss was proven at a
      // shallower depth and does not need deeper search. Handle the
      // refutation directly without calling compute_next on a terminal node.
      if self.child_nodes[i].state.is_end() {
        let child_state = self.child_nodes[i].state;
        let child_score = self.child_nodes[i].score;
        let discounted = Self::discounted_value(self.first_score_sqrt, child_score);
        if child_state.is_win() {
          stats.prune_nodes((self.child_nodes.len() - i - 1) as u32);
          self.child_nodes.truncate(i + 1);
          self.score = discounted;
          self.state = State::Lose;
          let bound = Bound::Exact;
          let best_move = self.child_nodes.first().map(|c| c.tile);
          tt.store(
            tt_hash,
            TTEntry {
              score: self.score,
              depth: self.depth,
              bound,
              best_move,
              state: self.state,
            },
          );
          return stats;
        }
        // For Draw/Lose children, fall through to normal best/alpha update
        // without re-searching.
        let discounted = Self::discounted_value(self.first_score_sqrt, child_score);
        if discounted > best {
          best = discounted;
          if discounted > alpha {
            alpha = discounted;
          }
        }
        if discounted >= beta {
          stats.prune_nodes((self.child_nodes.len() - i - 1) as u32);
          self.child_nodes.truncate(i + 1);
          self.score = best;
          tt.store(
            tt_hash,
            TTEntry {
              score: best,
              depth: self.depth,
              bound: Bound::Lower,
              best_move: self.child_nodes.first().map(|c| c.tile),
              state: State::NotEnd,
            },
          );
          return stats;
        }
        continue;
      }

      // PVS / Negascout: first child is the principal variation and is
      // searched with the full window; remaining children are probed with a
      // null window around alpha to cheaply prove they cannot beat the
      // current best. On a fail-high (null probe beats alpha) we re-search
      // with the full window to get an exact score.
      let is_pv = i == 0;
      let (child_score, child_state, probe_stats) = {
        let child = &mut self.child_nodes[i];
        if is_pv {
          let s = child.compute_next(&mut board.clone(), self.first_score, -beta, -alpha, tt);
          if !child.valid {
            self.valid = false;
            return s;
          }
          (child.score, child.state, s)
        } else {
          // Null-window probe: window is one point wide around alpha.
          // For terminal Win/Lose the narrow window may give a heuristic
          // bound; always re-search with full window to get exact state.
          let snapshot = child.clone();
          let mut probe_board = board.clone();
          let probe_stats =
            child.compute_next(&mut probe_board, self.first_score, -alpha - 1, -alpha, tt);
          if !child.valid {
            self.valid = false;
            return probe_stats;
          }
          let is_terminal = child.state.is_win() || child.state.is_lose();
          let probe_discounted = Self::discounted_value(self.first_score_sqrt, child.score);
          // Fail-high or terminal: re-search with full window for exact
          // score/state.
          if is_terminal || (probe_discounted > alpha && probe_discounted < beta) {
            // Restore and re-search with full window for exact score.
            *child = snapshot;
            let full_stats =
              child.compute_next(&mut board.clone(), self.first_score, -beta, -alpha, tt);
            if !child.valid {
              self.valid = false;
              return probe_stats + full_stats;
            }
            (child.score, child.state, probe_stats + full_stats)
          } else {
            (child.score, child.state, probe_stats)
          }
        }
      };
      stats += probe_stats;

      // Map child score into this node's discounted value space so cutoff
      // accounting matches the final `first_sqrt - best/2` blend. The prune
      // is still heuristic (value function is not pure negamax) but the
      // window now refers to the same discounted scale as `evaluate_children`.
      let discounted = Self::discounted_value(self.first_score_sqrt, child_score);

      // A winning reply for the opponent refutes this node outright; remaining
      // siblings cannot change the losing outcome.
      if child_state.is_win() {
        stats.prune_nodes((self.child_nodes.len() - i - 1) as u32);
        self.child_nodes.truncate(i + 1);
        self.score = discounted;
        self.state = State::Lose;
        let bound = Bound::Exact;
        let best_move = self.child_nodes.first().map(|c| c.tile);
        tt.store(
          tt_hash,
          TTEntry {
            score: self.score,
            depth: self.depth,
            bound,
            best_move,
            state: self.state,
          },
        );
        // Also store for the position after this move for child ordering.
        let after_hash = board.hash_with_player(!self.player);
        tt.store(
          after_hash,
          TTEntry {
            score: -self.score,
            depth: self.depth,
            bound,
            best_move: self.child_nodes.first().map(|c| c.tile),
            state: self.state.inversed(),
          },
        );
        return stats;
      }

      if discounted > best {
        best = discounted;
        if discounted > alpha {
          alpha = discounted;
        }
      }

      if discounted >= beta {
        stats.prune_nodes((self.child_nodes.len() - i - 1) as u32);
        self.child_nodes.truncate(i + 1);
        self.score = best;
        let bound = Bound::Lower;
        let best_move = self.child_nodes.first().map(|c| c.tile);
        // Lower bound: score is at least beta, but we pruned剩下的.
        tt.store(
          tt_hash,
          TTEntry {
            score: best,
            depth: self.depth,
            bound,
            best_move,
            state: State::NotEnd,
          },
        );
        return stats;
      }
    }

    self.evaluate_children(board);

    // Store current position in TT. Bound is based on original window.
    let bound = if best <= orig_alpha {
      Bound::Upper
    } else if best >= orig_beta {
      Bound::Lower
    } else {
      Bound::Exact
    };
    let best_move = self.child_nodes.first().map(|c| c.tile);
    tt.store(
      tt_hash,
      TTEntry {
        score: self.score,
        depth: self.depth,
        bound,
        best_move,
        state: self.state,
      },
    );
    // Store for the position after this move (used for ordering children of
    // this node on next iteration).
    if let Some(bm) = best_move {
      let after_hash = board.hash_with_player(!self.player);
      // The score from the after-position perspective is the best child's
      // score.
      if let Some(best_child) = self.child_nodes.first() {
        tt.store(
          after_hash,
          TTEntry {
            score: best_child.score,
            depth: best_child.depth,
            bound: Bound::Exact,
            best_move: Some(bm),
            state: best_child.state,
          },
        );
      }
    }

    stats
  }

  fn evaluate_children(&mut self, board: &Board) {
    if self.child_nodes.is_empty() {
      // Can happen if all children were pruned as losing/drawn or if TT
      // left the node empty. Treat as draw to avoid panic and keep search
      // valid; the snapshot logic will handle timeout cases.
      self.state = State::Draw;
      self.score = 0;
      return;
    }

    if self.child_nodes.iter().any(|node| !node.valid) {
      self.valid = false;
      return;
    }

    self.child_nodes.sort_unstable_by(|a, b| b.cmp(a));

    let mut limit = match self.depth {
      0 | 1 => unreachable!("depth 0 or 1 means the chilren are yet to be initialized"),
      2 | 3 => (self.child_nodes.len() / 2).max(24),
      4..=7 => 16,
      8 => 6,
      9.. => 4,
    };

    // Quiescence / threat extension: if the position is not quiet (contains
    // an open 4 or open 3 with two open ends for either player), the
    // horizon effect would miss a forced win if we truncated. Extend by
    // keeping all children for a few extra plies. The decaying sqrt still
    // applies, so deeper lines are naturally discounted.
    if self.depth < 12 && Self::is_forcing(board, self.tile) {
      limit = self.child_nodes.len();
    }

    self.child_nodes.truncate(limit);

    let best = self
      .child_nodes
      .first()
      // PERF: for some reason beyond my comprehesion, the length of the following message may have
      // negative impact on performance, so benchmarks have to be checked when changing it
      .expect("we already checked that the list is not empty");

    self.score = Self::discounted_value(self.first_score_sqrt, best.score);
    self.state = best.state.inversed();

    if self.state != State::NotEnd {
      self.child_nodes = Vec::new();
      return;
    }

    self
      .child_nodes
      .retain(|child| child.state == State::NotEnd);
  }

  #[inline]
  fn is_forcing(board: &Board, tile: TilePointer) -> bool {
    // A position is not quiet if the last move created a high-value shape
    // for either player. Threshold 100k catches open-4 with one open end
    // (100k), open-3 with two ends (5M) and open-4 with two ends (10M) —
    // the classic forcing threats in Gomoku that must be answered.
    let eval = board.evaluate_sequences_relevant_to(tile);
    eval.score[Player::X] >= 100_000 || eval.score[Player::O] >= 100_000
  }

  fn initialize(&mut self, board: &mut Board, parent_score: Score, stats: &mut Stats) {
    stats.evaluate_node();

    let opponent = !self.player;
    let mut score = parent_score;
    let tile = self.tile;

    // Penalize distance from center
    score -= 20 * board.squared_distance_from_center(tile);

    let Eval {
      score: prev_score, ..
    } = board.evaluate_sequences_relevant_to(tile);

    score += prev_score[self.player];
    score -= prev_score[opponent];

    board.set_tile(tile, Some(self.player));

    let Eval {
      score: new_score,
      win: new_win,
    } = board.evaluate_sequences_relevant_to(tile);

    score *= -1;
    score += new_score[self.player];
    score -= new_score[opponent];

    board.set_tile(tile, None);

    self.score = score;
    self.first_score = score;
    self.first_score_sqrt = signed_sqrt(score);

    self.state = match (new_win[self.player], new_win[opponent]) {
      (true, true) => {
        unreachable!(
          "Invalid win state: {new_win:?} for child node {tile} of node {self:?} on \
           board:\n{board}"
        )
      },
      (true, _) => State::Win,
      (_, true) => State::Lose,
      _ => State::NotEnd,
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
