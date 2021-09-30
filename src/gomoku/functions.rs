use super::{
  node::{Node, State},
  Board, Player, Score, Stats, TilePointer,
};
use std::{
  sync::Arc,
  time::{Duration, Instant},
};

fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool, is_on_turn: bool) -> (Score, bool) {
  if consecutive <= 1 {
    return (0, false);
  }

  if has_hole {
    return if is_on_turn {
      match consecutive {
        5 => (500_000, false),
        4 => match open_ends {
          2 => (100_000, false),
          1 => (1_000, false),
          _ => (0, false),
        },
        _ => (0, false),
      }
    } else if consecutive >= 5 {
      (1_000, false)
    } else {
      (0, false)
    };
  }

  match consecutive {
    5 => (10_000_000, true),
    4 => match open_ends {
      2 => {
        if is_on_turn {
          (1_000_000, false)
        } else {
          (200_000, false)
        }
      }
      1 => {
        if is_on_turn {
          (500_000, false)
        } else {
          (5_000, false)
        }
      }
      _ => (0, false),
    },
    3 => match open_ends {
      2 => {
        if is_on_turn {
          (80_000, false)
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

fn eval_sequence(
  sequence: &[usize],
  evaluate_for: Player,
  is_on_turn: bool,
  board: &Board,
) -> (Score, bool) {
  let mut score = 0;
  let mut consecutive = 0;
  let mut open_ends = 0;
  let mut has_hole = false;

  let mut is_win = false;

  for (index, &tile) in sequence.iter().enumerate() {
    if let Some(player) = board.get_tile_raw(tile) {
      if player == &evaluate_for {
        consecutive += 1;
        continue;
      }

      // opponent's tile
      if consecutive > 0 {
        let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole, is_on_turn);
        score += shape_score;
        if is_win_shape {
          is_win = true;
        }
      }

      consecutive = 0;
      open_ends = 0;
    } else {
      // empty tile
      if consecutive == 0 {
        open_ends = 1;
        has_hole = false;
        continue;
      }

      if !has_hole
        && index + 1 < sequence.len()
        && board.get_tile_raw(sequence[index + 1]) == &Some(evaluate_for)
      {
        has_hole = true;
        consecutive += 1;
        continue;
      }

      open_ends += 1;

      let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole, is_on_turn);
      score += shape_score;
      if is_win_shape {
        is_win = true;
      }

      consecutive = 0;
      open_ends = 1;
      has_hole = false;
    }
  }

  if consecutive > 0 {
    let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole, is_on_turn);
    score += shape_score;
    if is_win_shape {
      is_win = true;
    }
  }

  (score, is_win)
}

pub fn evaluate_board(board: &Board, current_player: Player) -> (Score, State) {
  let mut is_win = false;

  let score = board.sequences().iter().fold(0, |total, sequence| {
    let (player_score, is_winning) = eval_sequence(sequence, current_player, false, board);
    let (opponent_score, _) = eval_sequence(sequence, current_player.next(), true, board);

    if is_winning {
      is_win |= is_winning;
    }

    total + player_score - opponent_score
  });

  let state = if is_win { State::Win } else { State::NotEnd };

  (score, state)
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

pub fn time_remaining(end_time: &Arc<Instant>) -> bool {
  end_time.checked_duration_since(Instant::now()).is_some()
}

pub fn nodes_sorted_by_shallow_eval(
  board: &mut Board,
  empty_tiles: Vec<TilePointer>,
  stats: &mut Stats,
  current_player: Player,
  end_time: &Arc<Instant>,
) -> Vec<Node> {
  let dist = get_dist_fn(board.get_size());

  let mut nodes: Vec<_> = empty_tiles
    .into_iter()
    .map(|tile| {
      board.set_tile(tile, Some(current_player));
      let (analysis, state) = evaluate_board(board, current_player);
      board.set_tile(tile, None);

      Node::new(
        tile,
        current_player,
        analysis - dist(tile),
        state,
        end_time.clone(),
        stats,
      )
    })
    .collect();

  nodes.sort_unstable_by(|a, b| b.cmp(a));

  nodes
}

pub fn print_status(msg: &str, end_time: Instant) {
  println!(
    "{} ({:?} remaining)",
    msg,
    end_time
      .checked_duration_since(Instant::now())
      .unwrap_or(Duration::ZERO)
  );
}
