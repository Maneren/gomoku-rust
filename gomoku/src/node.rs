use std::fmt;

use crate::{
  alpha_beta::AlphaBeta,
  cache::{Cache, CacheEntry, NodeType},
  state::State,
};

use super::{
  board::{Board, TilePointer},
  player::Player,
  stats::Stats,
  Score,
};

#[derive(Clone)]
pub struct MoveSequence {
  pub tile: TilePointer,
  pub score: Score,
  pub player: Player,
  pub state: State,
  pub next: Option<Box<Self>>,
}

impl fmt::Debug for MoveSequence {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(child) = &self.next {
      write!(
        f,
        "({:?}, {}, {}) => {child:#?}",
        self.tile, self.score, self.player
      )
    } else if self.state.is_end() {
      write!(
        f,
        "({:?}, {}, {}, {})",
        self.tile, self.score, self.player, self.state
      )
    } else {
      write!(f, "({:?}, {}, {})", self.tile, self.score, self.player)
    }
  }
}

pub fn alpha_beta_negamax(
  board: &mut Board,
  player: Player,
  tile: TilePointer,
  depth: u8,
  mut alphabeta: AlphaBeta,
  stats: &mut Stats,
  cache: &Cache,
) -> (Score, MoveSequence) {
  if depth == 0 {
    return static_eval(board, player, tile, depth, alphabeta, stats, cache);
  }

  if let Some(
    entry @ CacheEntry {
      score,
      depth: cached_depth,
      ..
    },
  ) = cache.get(board)
  {
    if cached_depth >= depth && entry.is_useful(alphabeta) {
      stats.cache_hit();
      stats.prune_nodes((32u64).pow(depth as u32));
      return (
        score,
        MoveSequence {
          tile,
          score,
          player,
          state: State::NotEnd,
          next: None,
        },
      );
    }
  }

  let shallow_eval = board.pointers_to_empty_tiles().collect::<Vec<_>>();

  if shallow_eval.is_empty() {
    return (
      alphabeta.alpha,
      MoveSequence {
        tile,
        score: alphabeta.alpha,
        player,
        state: State::Draw,
        next: None,
      },
    );
  }

  let mut with_score = Vec::with_capacity(shallow_eval.len());

  for &tile in &shallow_eval {
    let score = heuristic_diff(board, player, tile);
    with_score.push((score - board.squared_distance_from_center(tile), tile));
  }

  with_score.sort_unstable_by_key(|&(score, _)| -score);

  let mut best_score = Score::MIN;
  let mut best_move_seq = None;

  for &(_, tile) in &with_score[..48] {
    board.set_tile(tile, Some(player));
    let (score, move_seq) =
      alpha_beta_negamax(board, !player, tile, depth - 1, -alphabeta, stats, cache);
    board.set_tile(tile, None);
    let score = -score;

    if score > best_score {
      best_score = score;
      best_move_seq = Some(MoveSequence {
        tile,
        score,
        player,
        state: State::NotEnd,
        next: Some(Box::new(move_seq)),
      });
    }

    alphabeta.update(score);

    if alphabeta.should_prune(score) {
      stats.prune_nodes((32u64).pow(depth as u32));
      cache.insert(
        board,
        CacheEntry {
          score,
          node_type: NodeType::UpperBound,
          depth,
        },
      );
      break;
    }
  }

  stats.cache_miss();
  cache.insert(
    board,
    CacheEntry {
      score: best_score,
      node_type: NodeType::Exact,
      depth,
    },
  );

  (best_score, best_move_seq.unwrap())
}

fn static_eval(
  board: &mut Board,
  player: Player,
  tile: TilePointer,
  depth: u8,
  alphabeta: AlphaBeta,
  stats: &mut Stats,
  cache: &Cache,
) -> (i32, MoveSequence) {
  if let Some(entry @ CacheEntry { score, .. }) = cache.get(board) {
    if entry.is_useful(alphabeta) {
      stats.cache_hit();
      stats.prune_nodes((32u64).pow(depth as u32));
      return (
        score,
        MoveSequence {
          tile,
          score,
          player,
          state: State::NotEnd,
          next: None,
        },
      );
    }
  }

  stats.evaluate_node();
  let score = board.evaluate_for(player);
  stats.cache_miss();
  cache.insert(
    board,
    CacheEntry {
      score,
      node_type: NodeType::Exact,
      depth,
    },
  );

  (
    score,
    MoveSequence {
      tile,
      score,
      player,
      state: State::NotEnd,
      next: None,
    },
  )
}

pub fn heuristic_diff(board: &mut Board, player: Player, tile: TilePointer) -> Score {
  let before = board.evaluate_sequences_relevant_to(tile);
  board.set_tile(tile, Some(player));
  let after = board.evaluate_sequences_relevant_to(tile);
  board.set_tile(tile, None);
  after.score[player] - before.score[player]
}
