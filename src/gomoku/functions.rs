use super::{
  board::{Board, TilePointer},
  node::Node,
  player::Player,
  r#move::Move,
  state::State,
  stats::Stats,
  Score, Tile,
};
use std::sync::{atomic::AtomicBool, Arc};

fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool) -> (Score, bool) {
  if consecutive <= 1 {
    return (0, false);
  }

  if has_hole {
    return match consecutive {
      5 => (500_000, false),
      4 => match open_ends {
        2 => (80_000, false),
        1 => (100, false),
        _ => (0, false),
      },
      _ => (0, false),
    };
  }

  match consecutive {
    5 => (10_000_000, true),
    4 => match open_ends {
      2 => (1_000_000, false),
      1 => (100_000, false),
      _ => (0, false),
    },
    3 => match open_ends {
      2 => (200_000, false),
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
pub type EvalScore = [Score; 2];
pub type EvalWin = [bool; 2];

fn eval_sequence<'a>(sequence: impl Iterator<Item = &'a Tile>) -> (EvalScore, EvalWin) {
  let mut sequence = sequence.peekable();

  let mut score = [0, 0];
  let mut is_win = [false, false];

  let mut current = Player::X;
  let mut consecutive = 0;
  let mut open_ends = 0;
  let mut has_hole = false;

  while let Some(&tile) = sequence.next() {
    if let Some(player) = tile {
      if player == current {
        consecutive += 1;
        continue;
      }

      // opponent's tile
      if consecutive > 0 {
        let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole);
        score[current.index()] += shape_score;
        is_win[current.index()] |= is_win_shape;

        open_ends = 0;
      } else {
        open_ends = 1;
      }

      consecutive = 1;
      current = player;
    } else {
      // empty tile
      if consecutive == 0 {
        open_ends = 1;
        has_hole = false;
        continue;
      }

      if !has_hole && sequence.peek() == Some(&&Some(current)) {
        has_hole = true;
        consecutive += 1;
        continue;
      }

      open_ends += 1;

      let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole);

      score[current.index()] += shape_score;
      is_win[current.index()] |= is_win_shape;

      consecutive = 0;
      open_ends = 1;
      has_hole = false;
    }
  }

  if consecutive > 0 {
    let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole);
    score[current.index()] += shape_score;
    is_win[current.index()] |= is_win_shape;
  }

  (score, is_win)
}

macro_rules! seq_to_iter {
  ($sequence:expr, $board:expr) => {
    $sequence.iter().map(|index| $board.get_tile_raw(*index))
  };
}

pub fn eval_relevant_sequences(board: &Board, tile: TilePointer) -> (EvalScore, EvalWin) {
  let (score, is_win) = board.get_relevant_sequences(tile).iter().fold(
    ([0, 0], [false, false]),
    |(mut total, mut is_win), sequence| {
      let (score, is_winning) = eval_sequence(seq_to_iter!(sequence, board));

      total[0] += score[0];
      total[1] += score[1];

      is_win[0] |= is_winning[0];
      is_win[1] |= is_winning[1];

      (total, is_win)
    },
  );

  (score, is_win)
}

pub fn evaluate_board(board: &Board, current_player: Player) -> (Score, State) {
  let opponent = current_player.next();

  let (score, is_win) = board
    .sequences()
    .iter()
    .fold((0, false), |(total, is_win), sequence| {
      let (score, is_winning) = eval_sequence(seq_to_iter!(sequence, board));

      (
        total + score[current_player.index()] - score[opponent.index()],
        is_win | is_winning[current_player.index()],
      )
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

pub fn check_winning(presorted_nodes: &[Node], stats: Stats) -> Option<(Move, Stats)> {
  presorted_nodes
    .iter()
    .filter(|node| node.state.is_win())
    .max()
    .map(|node| (node.to_move(), stats))
}

pub fn nodes_sorted_by_shallow_eval(
  board: &mut Board,
  empty_tiles: Vec<TilePointer>,
  stats: &mut Stats,
  target_player: Player,
  end: &Arc<AtomicBool>,
) -> Vec<Node> {
  let dist = get_dist_fn(board.get_size());

  let mut nodes: Vec<_> = empty_tiles
    .into_iter()
    .map(|tile| {
      board.set_tile(tile, Some(target_player));
      let (analysis, state) = evaluate_board(board, target_player);
      board.set_tile(tile, None);

      Node::new(
        tile,
        target_player,
        analysis - dist(tile),
        state,
        end.clone(),
        stats,
      )
    })
    .collect();

  nodes.sort_unstable_by(|a, b| b.cmp(a));

  nodes
}
