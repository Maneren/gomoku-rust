use std::str::FromStr;

use divan::{Bencher, black_box};
use gomoku_lib::{Board, Player, TilePointer};

fn main() {
  divan::main();
}

fn board_from_str(s: &str) -> Board {
  Board::from_str(s.trim()).unwrap()
}

const COMPLEX_BOARD: &str = "---------------
---------------
---------------
-------x-------
------x-x------
-----x---x-----
----o-----o----
---o-------o---
--o---------o--
-o-----------o-
---------------
---------------
---------------
---------------
---------------";

const MIDGAME_BOARD: &str = "---------------
---------------
---------------
-------x-------
------xox------
-----x-o-x-----
----o---o-o----
---o-------o---
--o---------o--
-o-----------o-
---------------
---------------
---------------
---------------
---------------";

const OPEN_BOARD: &str = "---------------
---------------
---------------
---------------
---------------
---------------
---------------
-------x-------
---------------
---------------
---------------
---------------
---------------
---------------
---------------";

const FOUR_OPEN: &str = "---------------
---------------
---------------
---------------
---------------
---------------
---------------
----xxxx-------
---------------
---------------
---------------
---------------
---------------
---------------
---------------";

const FIVE_WIN: &str = "---------------
---------------
---------------
---------------
---------------
---------------
---------------
----xxxxx------
---------------
---------------
---------------
---------------
---------------
---------------
---------------";

const SIX_WIN: &str = "---------------
---------------
---------------
---------------
---------------
---------------
---------------
----xxxxxx-----
---------------
---------------
---------------
---------------
---------------
---------------
---------------";

// ---- Fuller boards from test/ (all 15×15) ----
const TEST13_DENSE: &str = include_str!("../../test/test13.txt");
const TEST15: &str = include_str!("../../test/test15.txt");
const TEST10: &str = include_str!("../../test/test10.txt");
const TEST8: &str = include_str!("../../test/test8.txt");
const TEST9: &str = include_str!("../../test/test9.txt");
const TEST3: &str = include_str!("../../test/test3.txt");
const TEST5: &str = include_str!("../../test/test5.txt");
const TEST6: &str = include_str!("../../test/test6.txt");

#[divan::bench]
fn eval_empty_board(bencher: Bencher) {
  let board = Board::new_empty(15);
  bencher.bench(|| {
    black_box(board.evaluate());
  });
}

#[divan::bench]
fn eval_open_board(bencher: Bencher) {
  let board = board_from_str(OPEN_BOARD);
  bencher.bench(|| {
    black_box(board.evaluate());
  });
}

#[divan::bench]
fn eval_complex_board(bencher: Bencher) {
  let board = board_from_str(COMPLEX_BOARD);
  bencher.bench(|| {
    black_box(board.evaluate());
  });
}

#[divan::bench]
fn eval_midgame_board(bencher: Bencher) {
  let board = board_from_str(MIDGAME_BOARD);
  bencher.bench(|| {
    black_box(board.evaluate());
  });
}

#[divan::bench]
fn eval_four_open(bencher: Bencher) {
  let board = board_from_str(FOUR_OPEN);
  bencher.bench(|| {
    black_box(board.evaluate());
  });
}

#[divan::bench]
fn eval_five_win(bencher: Bencher) {
  let board = board_from_str(FIVE_WIN);
  bencher.bench(|| {
    black_box(board.evaluate());
  });
}

#[divan::bench]
fn eval_six_win(bencher: Bencher) {
  let board = board_from_str(SIX_WIN);
  bencher.bench(|| {
    black_box(board.evaluate());
  });
}

#[divan::bench]
fn eval_relevant_to_center(bencher: Bencher) {
  let board = board_from_str(MIDGAME_BOARD);
  let center = TilePointer { x: 7, y: 7 };
  bencher.bench(|| {
    black_box(board.evaluate_sequences_relevant_to(center));
  });
}

#[divan::bench]
fn eval_relevant_to_corner(bencher: Bencher) {
  let board = board_from_str(MIDGAME_BOARD);
  let corner = TilePointer { x: 0, y: 0 };
  bencher.bench(|| {
    black_box(board.evaluate_sequences_relevant_to(corner));
  });
}

#[divan::bench]
fn eval_for_x(bencher: Bencher) {
  let board = board_from_str(MIDGAME_BOARD);
  bencher.bench(|| {
    black_box(board.evaluate_for(Player::X));
  });
}

#[divan::bench]
fn eval_for_o(bencher: Bencher) {
  let board = board_from_str(MIDGAME_BOARD);
  bencher.bench(|| {
    black_box(board.evaluate_for(Player::O));
  });
}

// ── Fuller board benches (15×15) ───────────────────────────────────────────

#[divan::bench]
fn eval_full_dense_test13(bencher: Bencher) {
  let board = board_from_str(TEST13_DENSE);
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_full_test15(bencher: Bencher) {
  let board = board_from_str(TEST15);
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_full_test10(bencher: Bencher) {
  let board = board_from_str(TEST10);
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_full_test8(bencher: Bencher) {
  let board = board_from_str(TEST8);
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_full_test9(bencher: Bencher) {
  let board = board_from_str(TEST9);
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_full_test3(bencher: Bencher) {
  let board = board_from_str(TEST3);
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_full_test5(bencher: Bencher) {
  let board = board_from_str(TEST5);
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_full_test6(bencher: Bencher) {
  let board = board_from_str(TEST6);
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_full_synthetic_half(bencher: Bencher) {
  // Deterministic half-filled 15×15 board (checker + extra stones) — stresses
  // dense scanning
  let mut board = Board::new_empty(15);
  for y in 0..15u8 {
    for x in 0..15u8 {
      if (x as usize + y as usize) % 3 == 0 {
        let player = if (x + y) % 2 == 0 {
          Player::X
        } else {
          Player::O
        };
        board.set_tile(TilePointer { x, y }, Some(player));
      }
    }
  }
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_full_synthetic_dense(bencher: Bencher) {
  // ~70% filled board (every cell except every 3rd)
  let mut board = Board::new_empty(15);
  for y in 0..15u8 {
    for x in 0..15u8 {
      if (x + y) % 7 != 0 {
        let player = if (x * 7 + y * 13) % 2 == 0 {
          Player::X
        } else {
          Player::O
        };
        board.set_tile(TilePointer { x, y }, Some(player));
      }
    }
  }
  bencher.bench(|| black_box(board.evaluate()));
}

#[divan::bench]
fn eval_relevant_dense_test13_center(bencher: Bencher) {
  let board = board_from_str(TEST13_DENSE);
  let center = TilePointer { x: 7, y: 7 };
  bencher.bench(|| black_box(board.evaluate_sequences_relevant_to(center)));
}

#[divan::bench]
fn eval_relevant_dense_test13_edge(bencher: Bencher) {
  let board = board_from_str(TEST13_DENSE);
  let edge = TilePointer { x: 0, y: 7 };
  bencher.bench(|| black_box(board.evaluate_sequences_relevant_to(edge)));
}

#[divan::bench]
fn eval_for_dense_test13(bencher: Bencher) {
  let board = board_from_str(TEST13_DENSE);
  bencher.bench(|| black_box(board.evaluate_for(Player::X)));
}
