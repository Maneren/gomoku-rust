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
  draft: usize,
  mut alpha: Score,
  beta: Score,
  stats: &mut Stats,
) -> (Score, State) {
  if draft == 0 {
    // TODO: caching
    stats.evaluate_node();
    return board.evaluate_for(player);
  }

  let shallow_eval = board.pointers_to_empty_tiles().collect::<Vec<_>>();

  if shallow_eval.is_empty() {
    return (0, State::Draw);
  }

  let mut with_score = Vec::with_capacity(shallow_eval.len());

  for &tile in &shallow_eval {
    // let score = heuristic_diff(board, player, tile);
    with_score.push((-board.squared_distance_from_center(tile), tile));
  }

  with_score.sort_unstable_by_key(|&(score, _)| -score);

  let mut best_score = Score::MIN;

  for &(_, tile) in &with_score {
    board.set_tile(tile, Some(player));
    let (score, _) = alpha_beta_negamax(board, !player, draft - 1, -beta, -alpha, stats);
    board.set_tile(tile, None);

    let score = -score;

    if score > best_score {
      best_score = score;
    }

    alpha = alpha.max(best_score);

    if score >= beta {
      stats.prune_nodes((15u64.pow(2)).pow(draft as u32));
      break;
    }
  }

  (best_score, State::NotEnd)
}

fn heuristic_diff(board: &mut Board, player: Player, tile: TilePointer) -> Score {
  let before = board.evaluate_sequences_relevant_to(tile);
  board.set_tile(tile, Some(player));
  let after = board.evaluate_sequences_relevant_to(tile);
  board.set_tile(tile, None);
  after.score[player] - before.score[player]
}
