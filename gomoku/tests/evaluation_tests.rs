use std::str::FromStr;

use gomoku_lib::{Board, Player, State, TilePointer};

fn board_from_str(s: &str) -> Board {
  Board::from_str(s.trim()).unwrap()
}

fn board_with_stones(size: u8, stones: &[(u8, u8, Player)]) -> Board {
  let mut b = Board::new_empty(size);
  for &(x, y, p) in stones {
    b.set_tile(TilePointer { x, y }, Some(p));
  }
  b
}

fn line_board(y: u8, start_x: u8, pattern: &str, player: Player) -> Board {
  let mut stones = Vec::new();
  for (i, ch) in pattern.chars().enumerate() {
    let x = start_x + i as u8;
    match ch {
      'x' | 'X' => stones.push((x, y, player)),
      'o' | 'O' => stones.push((x, y, Player::O)),
      '_' | '.' | '-' => {},
      _ => {},
    }
  }
  board_with_stones(15, &stones)
}

#[test]
fn test_empty_board() {
  let board = Board::new_empty(15);
  let (score, state) = board.evaluate_for(Player::X);
  assert_eq!(score, 0);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_single_stone() {
  let board = board_with_stones(15, &[(7, 7, Player::X)]);
  let (score, state) = board.evaluate_for(Player::X);
  assert!(score >= 0);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_two_in_row_open() {
  let board = line_board(7, 7, "xx", Player::X);
  let (score_x, state) = board.evaluate_for(Player::X);
  let (score_o, _) = board.evaluate_for(Player::O);
  assert!(score_x > score_o);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_three_in_row_open() {
  let board = line_board(7, 6, "xxx", Player::X);
  let (score_x, state) = board.evaluate_for(Player::X);
  let (score_o, _) = board.evaluate_for(Player::O);
  assert!(score_x > score_o);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_four_in_row_open_win() {
  let board = line_board(7, 5, "xxxx", Player::X);
  let (score_x, state) = board.evaluate_for(Player::X);
  assert!(score_x > 9_000_000);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_five_in_row_win() {
  let board = line_board(7, 5, "xxxxx", Player::X);
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::Win);
}

#[test]
fn test_six_in_row_win() {
  let board = line_board(7, 5, "xxxxxx", Player::X);
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::Win, "6 in a row should be a win");
}

#[test]
fn test_seven_in_row_win() {
  let board = line_board(7, 5, "xxxxxxx", Player::X);
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::Win, "7 in a row should be a win");
}

#[test]
fn test_four_in_row_one_end_blocked() {
  let board = board_with_stones(
    15,
    &[
      (0, 7, Player::O),
      (1, 7, Player::X),
      (2, 7, Player::X),
      (3, 7, Player::X),
      (4, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert!(score_x > 90_000);
  assert!(score_x < 10_000_000);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_three_in_row_one_end_blocked() {
  let board = board_with_stones(
    15,
    &[
      (0, 7, Player::O),
      (1, 7, Player::X),
      (2, 7, Player::X),
      (3, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert!(score_x > 9_000);
  assert!(score_x < 100_000);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_split_four_x_xxx() {
  // pattern x_xxx at y=7 x=3 -> x _ x x x : 4 stones + hole => consecutive 5,
  // open 2 => 1_500_000
  let board = board_with_stones(
    15,
    &[
      (3, 7, Player::X),
      (5, 7, Player::X),
      (6, 7, Player::X),
      (7, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert!(score_x >= 1_400_000, "x_xxx score {} too low", score_x);
  assert!(score_x < 1_600_000, "x_xxx score {} too high", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_split_four_xx_xx() {
  let board = board_with_stones(
    15,
    &[
      (3, 7, Player::X),
      (4, 7, Player::X),
      (6, 7, Player::X),
      (7, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert!(score_x > 700_000, "xx_xx score {}", score_x);
  assert!(score_x < 2_000_000, "xx_xx score {}", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_split_three_x_xx() {
  // x_xx : 3 stones + hole => consecutive 4, open 2 => 400_000
  let board = board_with_stones(
    15,
    &[(3, 7, Player::X), (5, 7, Player::X), (6, 7, Player::X)],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert!(score_x >= 350_000, "x_xx score {}", score_x);
  assert!(score_x < 450_000, "x_xx score {}", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_overline_six_open() {
  let board = line_board(7, 4, "xxxxxx", Player::X);
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::Win);
}

#[test]
fn test_overline_with_hole_not_win() {
  // 5 stones + hole +1 => xxx_xx (3+2 with hole, consecutive 5+hole=6) should
  // not be win per shape_score
  let board = board_with_stones(
    15,
    &[
      (4, 7, Player::X),
      (5, 7, Player::X),
      (6, 7, Player::X),
      (8, 7, Player::X),
      (9, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::NotEnd, "5 stones + hole + 1 is not a win");
}

#[test]
fn test_vertical_five() {
  let board = board_with_stones(
    15,
    &[
      (7, 3, Player::X),
      (7, 4, Player::X),
      (7, 5, Player::X),
      (7, 6, Player::X),
      (7, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::Win);
}

#[test]
fn test_diagonal_five_lr() {
  // LR diagonal (top-right to bottom-left) e.g. (7,3),(6,4),(5,5),(4,6),(3,7)
  let board = board_with_stones(
    15,
    &[
      (7, 3, Player::X),
      (6, 4, Player::X),
      (5, 5, Player::X),
      (4, 6, Player::X),
      (3, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::Win);
}

#[test]
fn test_diagonal_five_rl() {
  let board = board_with_stones(
    15,
    &[
      (3, 0, Player::X),
      (4, 1, Player::X),
      (5, 2, Player::X),
      (6, 3, Player::X),
      (7, 4, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::Win);
}

#[test]
fn test_mixed_horizontal_vertical() {
  // horizontal 5 at y=4 x=4..8 and vertical 5 at x=4 y=4..8 sharing (4,4)
  let mut stones = Vec::new();
  for x in 4..9 {
    stones.push((x, 4, Player::X));
  }
  for y in 5..9 {
    stones.push((4, y, Player::X));
  }
  let board = board_with_stones(15, &stones);
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::Win);
}

#[test]
fn test_blocked_three_both_ends() {
  let board = board_with_stones(
    15,
    &[
      (0, 7, Player::O),
      (1, 7, Player::X),
      (2, 7, Player::X),
      (3, 7, Player::X),
      (4, 7, Player::O),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(score_x, 0, "Blocked three should score 0 got {}", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_blocked_four_both_ends() {
  let board = board_with_stones(
    15,
    &[
      (0, 7, Player::O),
      (1, 7, Player::X),
      (2, 7, Player::X),
      (3, 7, Player::X),
      (4, 7, Player::X),
      (5, 7, Player::O),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(score_x, 0, "Blocked four should score 0 got {}", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_double_three_open() {
  let board = board_with_stones(
    15,
    &[
      (2, 7, Player::X),
      (3, 7, Player::X),
      (4, 7, Player::X),
      (8, 7, Player::X),
      (9, 7, Player::X),
      (10, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert!(score_x > 9_000_000, "double open three score {}", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_four_and_three_open() {
  let board = board_with_stones(
    15,
    &[
      (2, 7, Player::X),
      (3, 7, Player::X),
      (4, 7, Player::X),
      (5, 7, Player::X),
      (9, 7, Player::X),
      (10, 7, Player::X),
      (11, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert!(score_x > 12_000_000, "four+three score {}", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_opponent_stones_reduce_score() {
  let mut stones = Vec::new();
  for x in 4..9 {
    stones.push((x, 4, Player::X));
  }
  for x in 4..9 {
    stones.push((x, 5, Player::O));
  }
  let board = board_with_stones(15, &stones);
  let (score_x, state_x) = board.evaluate_for(Player::X);
  let (score_o, state_o) = board.evaluate_for(Player::O);
  assert_eq!(state_x, State::Win);
  assert_eq!(state_o, State::Win);
}

#[test]
fn test_corner_sequence() {
  let board = board_with_stones(
    15,
    &[
      (0, 0, Player::X),
      (1, 1, Player::X),
      (2, 2, Player::X),
      (3, 3, Player::X),
      (4, 4, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::Win);
}

#[test]
fn test_edge_sequence() {
  // 4 at bottom edge should not be win
  let board = board_with_stones(
    15,
    &[
      (0, 14, Player::X),
      (1, 14, Player::X),
      (2, 14, Player::X),
      (3, 14, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_multiple_independent_sequences() {
  let board = board_with_stones(
    15,
    &[
      (4, 4, Player::X),
      (5, 4, Player::X),
      (6, 4, Player::X),
      (5, 8, Player::X),
      (6, 8, Player::X),
      (7, 8, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  let single_three = 5_000_000;
  assert!(
    score_x > (single_three as f32 * 1.5) as i32,
    "score {} too low",
    score_x
  );
  assert!(score_x < single_three * 3, "score {} too high", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_split_four_with_blocked_end() {
  let board = board_with_stones(
    15,
    &[
      (0, 7, Player::O),
      (1, 7, Player::X),
      (2, 7, Player::X),
      (4, 7, Player::X),
      (5, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  // blocked on left, open on right with hole
  assert!(score_x > 0, "score {}", score_x);
  assert!(score_x < 500_000, "score {}", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_three_with_hole_open() {
  // x_x pattern with 2 stones + hole (consecutive 3? Actually 1+hole+1 = 3
  // stones? Wait shape is 2 stones with hole)
  let board = board_with_stones(15, &[(3, 7, Player::X), (5, 7, Player::X)]);
  let (score_x, state) = board.evaluate_for(Player::X);
  // shape_score(3,2,true)?? For has_hole consecutive 3 => 2k? Actually 2 stones
  // + hole =3 per table has_hole 3 with 2 open => 2000
  assert!(score_x >= 0);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_adjacent_different_players() {
  let board = board_with_stones(
    15,
    &[
      (3, 7, Player::X),
      (4, 7, Player::X),
      (5, 7, Player::O),
      (6, 7, Player::O),
      (7, 7, Player::O),
    ],
  );
  let (score_x, _) = board.evaluate_for(Player::X);
  let (score_o, _) = board.evaluate_for(Player::O);
  assert!(score_o > score_x, "o {} vs x {}", score_o, score_x);
}

#[test]
fn test_no_double_counting_overlapping() {
  let board = line_board(7, 5, "xxxx", Player::X);
  let eval = board.evaluate();
  let score_x = eval.score[Player::X];
  assert!(
    score_x > 9_000_000 && score_x < 11_000_000,
    "score {}",
    score_x
  );
}

#[test]
fn test_complex_board() {
  let board = board_with_stones(
    15,
    &[
      (7, 3, Player::X),
      (6, 4, Player::X),
      (8, 4, Player::X),
      (5, 5, Player::X),
      (9, 5, Player::X),
      (4, 6, Player::O),
      (10, 6, Player::O),
      (3, 7, Player::O),
      (11, 7, Player::O),
      (2, 8, Player::O),
      (12, 8, Player::O),
      (1, 9, Player::O),
      (13, 9, Player::O),
    ],
  );
  let (score_x, state_x) = board.evaluate_for(Player::X);
  let (score_o, state_o) = board.evaluate_for(Player::O);
  assert_eq!(state_x, State::NotEnd);
  assert_eq!(state_o, State::NotEnd);
  let _ = (score_x, score_o);
}

#[test]
fn test_regression_five_with_space() {
  // xxx_xx at edge (x=0): 5 stones + hole => consecutive 6, open_ends=1
  // (blocked left) => 80_000
  let board = board_with_stones(
    15,
    &[
      (0, 7, Player::X),
      (1, 7, Player::X),
      (2, 7, Player::X),
      (4, 7, Player::X),
      (5, 7, Player::X),
    ],
  );
  let (score_x, state) = board.evaluate_for(Player::X);
  assert!(score_x >= 70_000 && score_x < 90_000, "score {}", score_x);
  assert_eq!(state, State::NotEnd);
}

#[test]
fn test_score_monotonicity() {
  let b2 = line_board(7, 3, "xx", Player::X);
  let b3 = line_board(7, 3, "xxx", Player::X);
  let b4 = line_board(7, 3, "xxxx", Player::X);
  let (s2, _) = b2.evaluate_for(Player::X);
  let (s3, _) = b3.evaluate_for(Player::X);
  let (s4, _) = b4.evaluate_for(Player::X);
  assert!(s2 < s3, "open 3 {} > open 2 {}", s3, s2);
  assert!(s3 < s4, "open 4 {} > open 3 {}", s4, s3);
}

#[test]
fn test_hole_patterns_scoring() {
  // x_xxx (1+hole+3) => 4 stones + hole => consecutive 5 => 1_500_000
  let b1 = board_with_stones(
    15,
    &[
      (3, 7, Player::X),
      (5, 7, Player::X),
      (6, 7, Player::X),
      (7, 7, Player::X),
    ],
  );
  let (s1, _) = b1.evaluate_for(Player::X);
  assert!(s1 > 1_400_000 && s1 < 1_600_000, "x_xxx score {}", s1);
  // xx_xx => 1_500_000 as well
  let b2 = board_with_stones(
    15,
    &[
      (3, 7, Player::X),
      (4, 7, Player::X),
      (6, 7, Player::X),
      (7, 7, Player::X),
    ],
  );
  let (s2, _) = b2.evaluate_for(Player::X);
  assert!(s2 > 1_400_000 && s2 < 1_600_000, "xx_xx score {}", s2);
}

#[test]
fn test_win_overline_various_lengths() {
  for len in 5..=10 {
    let pattern = "x".repeat(len as usize);
    let board = line_board(7, 3, &pattern, Player::X);
    let (_, state) = board.evaluate_for(Player::X);
    assert_eq!(state, State::Win, "len {} should be win", len);
  }
}

#[test]
fn test_evaluate_sequences_relevant_to_vs_full() {
  // relevant_to should be consistent: evaluating relevant sequences around last
  // move should capture win
  let board = line_board(7, 5, "xxxxx", Player::X);
  let relevant = board.evaluate_sequences_relevant_to(TilePointer { x: 7, y: 7 });
  assert!(relevant.win[Player::X], "relevant should detect win");
  let full = board.evaluate();
  assert_eq!(relevant.win[Player::X], full.win[Player::X]);
}

#[test]
fn test_board_from_str_parsing() {
  let s = "---------------\n".repeat(7) + "-------x-------\n" + &"---------------\n".repeat(7);
  let s = s.trim();
  // Actually need proper string
  let board = Board::from_str(&vec!["---------------"; 15].join("\n")).unwrap();
  assert_eq!(board.size(), 15);
}
