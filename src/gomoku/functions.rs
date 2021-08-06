use std::{
  sync::{Arc, Mutex},
  time::Instant,
};

use super::{Board, Cache, Player, Score, Stats, Tile, TilePointer};

fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool, is_on_turn: bool) -> (Score, bool) {
  if consecutive == 0 {
    return (0, false);
  }

  if has_hole {
    if !is_on_turn {
      return (0, false);
    }

    if consecutive == 5 {
      (500_000, false)
    } else if consecutive == 4 {
      match open_ends {
        2 => (300_000, false),
        1 => (2_000, false),
        _ => (0, false),
      }
    } else {
      (0, false)
    }
  } else {
    match consecutive {
      5 => (10_000_000, true),
      4 => match open_ends {
        2 => {
          if is_on_turn {
            (500_000, false)
          } else {
            (150_000, false)
          }
        }
        1 => {
          if is_on_turn {
            (80_000, false)
          } else {
            (5_000, false)
          }
        }
        _ => (0, false),
      },
      3 => match open_ends {
        2 => {
          if is_on_turn {
            (50_000, false)
          } else {
            (1_000, false)
          }
        }
        1 => (10, false),
        _ => (0, false),
      },
      2 => match open_ends {
        2 => (10, false),
        _ => (0, false),
      },
      _ => (0, false),
    }
  }
}

fn eval_sequence(sequence: &[&Tile], evaluate_for: Player, is_on_turn: bool) -> (Score, bool) {
  let mut score = 0;
  let mut consecutive = 0;
  let mut open_ends = 0;
  let mut has_hole = false;

  let mut is_win = false;

  for (index, tile) in sequence.iter().enumerate() {
    if let Some(player) = tile {
      if *player == evaluate_for {
        consecutive += 1
      } else {
        if consecutive > 0 {
          let (shape_score, is_win_shape) =
            shape_score(consecutive, open_ends, has_hole, is_on_turn);
          is_win |= is_win_shape;
          score += shape_score;
        }

        consecutive = 0;
        open_ends = 0;
      }
    } else if consecutive == 0 {
      open_ends = 1;
      has_hole = false;
    } else {
      if !has_hole && index + 1 < sequence.len() && *sequence[index + 1] == Some(evaluate_for) {
        has_hole = true;
        consecutive += 1;
        continue;
      }

      open_ends += 1;

      let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole, is_on_turn);
      is_win |= is_win_shape;
      score += shape_score;

      consecutive = 0;
      open_ends = 1;
      has_hole = false;
    }
  }

  if consecutive > 0 {
    let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole, is_on_turn);
    is_win |= is_win_shape;
    score += shape_score;
  }

  (score, is_win)
}

pub fn evaluate_board(
  board: &mut Board,
  stats_arc: &Arc<Mutex<Stats>>,
  cache_arc: &Arc<Mutex<Cache>>,
  current_player: Player,
) -> (Score, bool) {
  stats_arc.lock().unwrap().eval();

  if let Some(&(cached_score, owner, is_game_end)) = cache_arc.lock().unwrap().lookup(board) {
    let score = if current_player == owner {
      cached_score
    } else {
      -cached_score
    };

    return (score, is_game_end);
  }

  let mut is_game_end = false;

  let score = board
    .get_all_tile_sequences()
    .into_iter()
    .fold(0, |total, sequence| {
      let (player_score, is_win) = eval_sequence(&sequence, current_player, false);
      let (opponent_score, is_lose) = eval_sequence(&sequence, current_player.next(), true);

      if is_win || is_lose {
        is_game_end = true;
      }

      // total + player - opponent
      total + player_score - opponent_score
    });

  let cache_data = (score, current_player, is_game_end);
  cache_arc.lock().unwrap().insert(board, cache_data);

  (score, is_game_end)
}

pub fn get_dist_fn(board_size: u8) -> Box<dyn Fn(TilePointer) -> Score> {
  let middle = f32::from(board_size - 1) / 2.0;

  let function = move |p1: TilePointer| {
    let x = f32::from(p1.x);
    let y = f32::from(p1.y);
    let raw_dist = (x - middle).powi(2) + (y - middle).powi(2);

    #[allow(clippy::cast_possible_truncation)]
    let dist = raw_dist.round() as Score;

    dist
  };

  Box::new(function)
}

pub fn time_remaining(end_time: Instant) -> bool {
  Instant::now().checked_duration_since(end_time).is_none()
}
