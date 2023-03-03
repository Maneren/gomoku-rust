use std::sync::{atomic::AtomicBool, Arc};

use self::eval_structs::{Eval, EvalWinPotential};
use super::{
  board::{Board, TilePointer},
  node::Node,
  player::Player,
  r#move::Move,
  state::State,
  stats::Stats,
  Score, Tile,
};

pub mod eval_structs;

/// Return score, win and win potential modifier for the given shape
fn shape_score(consecutive: u8, open_ends: u8, has_hole: bool) -> (Score, bool, Score) {
  if has_hole {
    return match consecutive {
      5.. => (40_000, false, 2),
      4 => match open_ends {
        2 => (20_000, false, 2),
        1 => (500, false, 0),
        _ => (0, false, 0),
      },
      _ => (0, false, 0),
    };
  }

  match consecutive {
    5.. => (100_000_000, true, 4),
    4 => match open_ends {
      2 => (10_000_000, false, 4),
      1 => (100_000, false, 2),
      _ => (0, false, 0),
    },
    3 => match open_ends {
      2 => (5_000_000, false, 2),
      1 => (10_000, false, 0),
      _ => (0, false, 0),
    },
    2 => match open_ends {
      2 => (2_000, false, 0),
      _ => (0, false, 0),
    },
    _ => (0, false, 0),
  }
}

fn eval_sequence<'a>(sequence: impl Iterator<Item = &'a Tile>) -> Eval {
  let mut sequence = sequence.peekable();

  let mut eval = Eval::default();
  let mut win_potentials = EvalWinPotential::default();

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
        let (shape_score, is_win_shape, win_potential) =
          shape_score(consecutive, open_ends, has_hole);
        eval.score[current] += shape_score;
        eval.win[current] |= is_win_shape;
        win_potentials[current] += win_potential;

        open_ends = 0;
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

      if !has_hole && sequence.peek() == Some(&&Some(current)) && consecutive < 5 {
        has_hole = true;
        consecutive += 1;
        continue;
      }

      open_ends += 1;

      let (shape_score, is_win_shape, win_potential) =
        shape_score(consecutive, open_ends, has_hole);

      eval.score[current] += shape_score;
      eval.win[current] |= is_win_shape;
      win_potentials[current] += win_potential;

      consecutive = 0;
      open_ends = 1;
      has_hole = false;
    }
  }

  if consecutive > 0 {
    let (shape_score, is_win_shape, win_potential) = shape_score(consecutive, open_ends, has_hole);
    eval.score[current] += shape_score;
    eval.win[current] |= is_win_shape;
    win_potentials[current] += win_potential;
  }

  eval.score *= win_potentials;

  eval
}

fn seq_to_iter<'a>(sequence: &'a [usize], board: &'a Board) -> impl Iterator<Item = &'a Tile> {
  sequence.iter().map(|index| board.get_tile_raw(*index))
}

pub fn eval_relevant_sequences(board: &Board, tile: TilePointer) -> Eval {
  board
    .get_relevant_sequences(tile)
    .into_iter()
    .map(|sequence| seq_to_iter(sequence, board))
    .map(eval_sequence)
    .sum()
}

pub fn evaluate_board(board: &Board, current_player: Player) -> (Score, State) {
  let opponent = !current_player;

  let (score, is_win) = board
    .sequences()
    .iter()
    .fold((0, false), |(total, is_win), sequence| {
      let Eval { score, win } = eval_sequence(seq_to_iter(sequence, board));

      (
        total + score[current_player] - score[opponent],
        is_win | win[current_player],
      )
    });

  let state = if is_win { State::Win } else { State::NotEnd };

  (score, state)
}

pub fn check_winning(presorted_nodes: &[Node]) -> Option<Move> {
  presorted_nodes
    .iter()
    .find(|node| node.state.is_win())
    .map(Node::to_move)
}

pub fn nodes_sorted_by_shallow_eval(
  board: &mut Board,
  empty_tiles: Vec<TilePointer>,
  stats: &mut Stats,
  target_player: Player,
  end: &Arc<AtomicBool>,
) -> Vec<Node> {
  let mut nodes: Vec<_> = empty_tiles
    .into_iter()
    .map(|tile| {
      board.set_tile(tile, Some(target_player));
      let (analysis, state) = evaluate_board(board, target_player);
      board.set_tile(tile, None);

      Node::new(
        tile,
        target_player,
        analysis - board.squared_distance_from_center(tile),
        state,
        end.clone(),
        stats,
      )
    })
    .collect();

  nodes.sort_unstable_by(|a, b| b.cmp(a));

  nodes
}

pub fn score_sqrt(n: Score) -> Score {
  let n = n as f32;
  (n.signum() * n.abs().sqrt()) as Score
}
pub fn score_square(n: Score) -> Score {
  n.signum() * n.pow(2)
}

#[cfg(test)]
mod tests {

  use eval_structs::{Eval, EvalScore, EvalWin};

  use super::*;

  #[test]
  fn test_shape_score() {
    let shapes = [
      shape_score(0, 0, false),
      shape_score(1, 0, false),
      shape_score(2, 0, false),
      shape_score(3, 0, false),
      shape_score(3, 0, true),
      shape_score(0, 2, false),
      shape_score(1, 2, false),
      shape_score(4, 1, true),
      shape_score(2, 2, false),
      shape_score(3, 1, false),
      shape_score(4, 2, true),
      shape_score(5, 1, true),
      shape_score(5, 2, true),
      shape_score(4, 1, false),
      shape_score(3, 2, false),
      shape_score(4, 2, false),
      shape_score(5, 0, false),
      shape_score(5, 1, false),
      shape_score(5, 2, false),
      shape_score(6, 2, false),
      shape_score(10, 2, false),
    ];

    shapes
      .iter()
      .zip(shapes[1..].iter())
      .enumerate()
      .for_each(|(i, (a, b))| assert!(a.0 <= b.0, "{i}: {a:?} {b:?}"));
  }

  #[test]
  fn test_eval_sequence() {
    let x = Some(Player::X);
    let o = Some(Player::O);
    let n = None;

    let test_sequences = vec![
      (vec![n, n, n, n, n, n, n, n, n, n, n, n], vec![], vec![]),
      (
        vec![n, x, x, x, x, x, n],
        vec![shape_score(5, 2, false)],
        vec![],
      ),
      (
        vec![n, x, x, x, x, x],
        vec![shape_score(5, 1, false)],
        vec![],
      ),
      (
        vec![n, o, o, o, o, o, n],
        vec![],
        vec![shape_score(5, 2, false)],
      ),
      (
        vec![n, x, o, o, o, o, o, x, n],
        vec![],
        vec![shape_score(5, 0, false)],
      ),
      (
        vec![n, o, n, o, o, o],
        vec![],
        vec![shape_score(5, 1, true)],
      ),
      (
        vec![n, o, x, o, o, o, n],
        vec![],
        vec![shape_score(3, 1, false)],
      ),
      (
        vec![n, o, o, o, o, n],
        vec![],
        vec![shape_score(4, 2, false)],
      ),
      (vec![n, o, o, o, o], vec![], vec![shape_score(4, 1, false)]),
      (vec![o, o, o, o], vec![], vec![shape_score(4, 0, false)]),
      (
        vec![n, o, n, o, o, n],
        vec![],
        vec![shape_score(4, 2, true)],
      ),
      (vec![o, o, o, o], vec![], vec![shape_score(4, 0, true)]),
      (vec![n, o, o, o, n], vec![], vec![shape_score(3, 2, false)]),
      (
        vec![n, o, o, o, n, x, x, x, n],
        vec![shape_score(3, 2, false)],
        vec![shape_score(3, 2, false)],
      ),
      (
        vec![n, o, o, o, x, x, x, n],
        vec![shape_score(3, 1, false)],
        vec![shape_score(3, 1, false)],
      ),
      (
        vec![o, o, o, n, n, o, o, o],
        vec![],
        vec![shape_score(3, 1, false), shape_score(3, 1, false)],
      ),
    ];

    macro_rules! sum {
      ($vec:expr) => {
        $vec.iter().fold(
          (0, false),
          |(total, is_win), (score, is_winning, modifier)| {
            (total + score * modifier, is_win | is_winning)
          },
        )
      };
    }

    // this is kinda wonky, but it works
    // basically it compares the output of eval_sequence with sum of shapes from expected_outputs
    for (i, (sequence, x_vec, y_vec)) in test_sequences.iter().enumerate() {
      // unpack eval_sequence output
      let Eval {
        score: EvalScore(x_score, y_score),
        win: EvalWin(x_win, y_win),
      } = eval_sequence(sequence.iter().peekable());

      let x = (x_score, x_win);
      let y = (y_score, y_win);

      // sum the shapes and convert to format similar to x, y above
      let expected_x = sum!(x_vec);
      let expected_y = sum!(y_vec);

      println!("{i}");
      assert_eq!(x, expected_x);
      assert_eq!(y, expected_y);
    }
  }

  #[test]
  fn test_score_square() {
    let data = vec![(10, 100), (-5, -25), (0, 0)];

    for (src, target) in data {
      assert_eq!(score_square(src), target);
    }
  }

  #[test]
  fn test_score_sqrt() {
    let data = vec![(100, 10), (-25, -5), (0, 0), (30, 5)];

    for (src, target) in data {
      assert_eq!(score_sqrt(src), target);
    }
  }
}
