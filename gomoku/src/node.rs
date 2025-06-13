use crate::cache::{self, Cache, CacheEntry, NodeType};

use super::{
  board::{Board, TilePointer},
  player::Player,
  state::State,
  stats::Stats,
  Score,
};

pub fn alpha_beta_negamax(
  board: &mut Board,
  player: Player,
  depth: u8,
  mut alpha: Score,
  beta: Score,
  stats: &mut Stats,
  cache: &Cache,
) -> Score {
  let cache_entry = cache.get(board);

  if depth == 0 {
    if let Some(CacheEntry {
      score, node_type, ..
    }) = cache_entry
    {
      if (matches!(node_type, NodeType::Exact)
        || matches!(node_type, NodeType::LowerBound if score >= beta)
        || matches!(node_type, NodeType::UpperBound if score <= alpha))
      {
        stats.cache_hit();
        stats.prune_nodes((32u64).pow(depth as u32));
        return score;
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

    return score;
  }

  if let Some(CacheEntry {
    score,
    node_type,
    depth: cached_depth,
  }) = cache_entry
  {
    if cached_depth >= depth
      && (matches!(node_type, NodeType::Exact)
        || matches!(node_type, NodeType::LowerBound if score >= beta)
        || matches!(node_type, NodeType::UpperBound if score <= alpha))
    {
      stats.cache_hit();
      stats.prune_nodes((32u64).pow(depth as u32));
      return score;
    }
  }

  let shallow_eval = board.pointers_to_empty_tiles().collect::<Vec<_>>();

  if shallow_eval.is_empty() {
    return alpha;
  }

  let mut with_score = Vec::with_capacity(shallow_eval.len());

  for &tile in &shallow_eval {
    let score = heuristic_diff(board, player, tile);
    with_score.push((score - board.squared_distance_from_center(tile), tile));
  }

  with_score.sort_unstable_by_key(|&(score, _)| -score);

  let mut best_score = Score::MIN;

  for &(_, tile) in &with_score[..48] {
    board.set_tile(tile, Some(player));
    let score = -alpha_beta_negamax(board, !player, depth - 1, -beta, -alpha, stats, cache);
    board.set_tile(tile, None);

    if score > best_score {
      best_score = score;
    }

    alpha = alpha.max(best_score);

    if score >= beta {
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

  best_score
}

fn heuristic_diff(board: &mut Board, player: Player, tile: TilePointer) -> Score {
  let before = board.evaluate_sequences_relevant_to(tile);
  board.set_tile(tile, Some(player));
  let after = board.evaluate_sequences_relevant_to(tile);
  board.set_tile(tile, None);
  after.score[player] - before.score[player]
}
