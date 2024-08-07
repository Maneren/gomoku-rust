pub type Sequence = Box<[usize]>;
pub type Sequences = Box<[Sequence]>;

/// Create `Sequence` representing given row
fn make_row(size: usize, y: usize) -> Sequence {
  (0..size).map(|x| x + y * size).collect()
}

/// Create `Sequence` representing given column
fn make_col(size: usize, x: usize) -> Sequence {
  (0..size).map(|y| x + y * size).collect()
}

/// Direction of a diagonal, always goes from top to bottom
#[derive(Copy, Clone, Debug)]
enum DiagonalDir {
  LeftToRight = 1,
  RightToLeft = -1,
}

/// Create `Sequence` representing given diagonal.
/// `k` is the index of the diagonal and should be in the range `1..2 * size`.
/// Note that the indexing starts at the respective top corner.
fn make_diagonal(size: usize, k: usize, direction: DiagonalDir) -> Sequence {
  let count = if k <= size { k } else { 2 * size - k };

  let starting_idx = match (direction, k <= size) {
    (DiagonalDir::LeftToRight, true) => size - k,
    (DiagonalDir::LeftToRight, false) => (k - size) * size,
    (DiagonalDir::RightToLeft, true) => k - 1,
    (DiagonalDir::RightToLeft, false) => (k - size + 1) * size - 1,
  };

  let step = size
    .checked_add_signed(direction as isize)
    .expect("direction should be ±1 and size << usize::MAX");

  (0..count).map(|i| starting_idx + i * step).collect()
}

/// Generate all possible sequences for the given board size
pub fn generate(size: u8) -> Sequences {
  let size = size as usize;

  let rows = (0..size).map(|y| make_row(size, y));
  let columns = (0..size).map(|x| make_col(size, x));

  let diagonals = [DiagonalDir::RightToLeft, DiagonalDir::LeftToRight]
    .into_iter()
    .flat_map(|direction| (1..2 * size).map(move |k| make_diagonal(size, k, direction)));

  rows.chain(columns).chain(diagonals).collect()
}

#[cfg(test)]
mod tests {
  use std::sync::LazyLock;

  use super::*;

  const BOARD_SIZE: u8 = 10;

  static EXPECTED_ROWS: LazyLock<[Vec<usize>; BOARD_SIZE as usize]> = LazyLock::new(|| {
    [
      vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
      vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
      vec![20, 21, 22, 23, 24, 25, 26, 27, 28, 29],
      vec![30, 31, 32, 33, 34, 35, 36, 37, 38, 39],
      vec![40, 41, 42, 43, 44, 45, 46, 47, 48, 49],
      vec![50, 51, 52, 53, 54, 55, 56, 57, 58, 59],
      vec![60, 61, 62, 63, 64, 65, 66, 67, 68, 69],
      vec![70, 71, 72, 73, 74, 75, 76, 77, 78, 79],
      vec![80, 81, 82, 83, 84, 85, 86, 87, 88, 89],
      vec![90, 91, 92, 93, 94, 95, 96, 97, 98, 99],
    ]
  });

  static EXPECTED_COLUMNS: LazyLock<[Vec<usize>; BOARD_SIZE as usize]> = LazyLock::new(|| {
    [
      vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90],
      vec![1, 11, 21, 31, 41, 51, 61, 71, 81, 91],
      vec![2, 12, 22, 32, 42, 52, 62, 72, 82, 92],
      vec![3, 13, 23, 33, 43, 53, 63, 73, 83, 93],
      vec![4, 14, 24, 34, 44, 54, 64, 74, 84, 94],
      vec![5, 15, 25, 35, 45, 55, 65, 75, 85, 95],
      vec![6, 16, 26, 36, 46, 56, 66, 76, 86, 96],
      vec![7, 17, 27, 37, 47, 57, 67, 77, 87, 97],
      vec![8, 18, 28, 38, 48, 58, 68, 78, 88, 98],
      vec![9, 19, 29, 39, 49, 59, 69, 79, 89, 99],
    ]
  });

  static EXPECTED_RL_DIAGONALS: LazyLock<[Vec<usize>; (2 * BOARD_SIZE - 1) as usize]> =
    LazyLock::new(|| {
      [
        vec![0],
        vec![1, 10],
        vec![2, 11, 20],
        vec![3, 12, 21, 30],
        vec![4, 13, 22, 31, 40],
        vec![5, 14, 23, 32, 41, 50],
        vec![6, 15, 24, 33, 42, 51, 60],
        vec![7, 16, 25, 34, 43, 52, 61, 70],
        vec![8, 17, 26, 35, 44, 53, 62, 71, 80],
        vec![9, 18, 27, 36, 45, 54, 63, 72, 81, 90],
        vec![19, 28, 37, 46, 55, 64, 73, 82, 91],
        vec![29, 38, 47, 56, 65, 74, 83, 92],
        vec![39, 48, 57, 66, 75, 84, 93],
        vec![49, 58, 67, 76, 85, 94],
        vec![59, 68, 77, 86, 95],
        vec![69, 78, 87, 96],
        vec![79, 88, 97],
        vec![89, 98],
        vec![99],
      ]
    });

  static EXPECTED_LR_DIAGONALS: LazyLock<[Vec<usize>; (2 * BOARD_SIZE - 1) as usize]> =
    LazyLock::new(|| {
      [
        vec![9],
        vec![8, 19],
        vec![7, 18, 29],
        vec![6, 17, 28, 39],
        vec![5, 16, 27, 38, 49],
        vec![4, 15, 26, 37, 48, 59],
        vec![3, 14, 25, 36, 47, 58, 69],
        vec![2, 13, 24, 35, 46, 57, 68, 79],
        vec![1, 12, 23, 34, 45, 56, 67, 78, 89],
        vec![0, 11, 22, 33, 44, 55, 66, 77, 88, 99],
        vec![10, 21, 32, 43, 54, 65, 76, 87, 98],
        vec![20, 31, 42, 53, 64, 75, 86, 97],
        vec![30, 41, 52, 63, 74, 85, 96],
        vec![40, 51, 62, 73, 84, 95],
        vec![50, 61, 72, 83, 94],
        vec![60, 71, 82, 93],
        vec![70, 81, 92],
        vec![80, 91],
        vec![90],
      ]
    });

  #[test]
  fn test_generate() {
    let sequences = generate(10);

    let expected = EXPECTED_ROWS
      .iter()
      .chain(EXPECTED_COLUMNS.iter())
      .chain(EXPECTED_RL_DIAGONALS.iter())
      .chain(EXPECTED_LR_DIAGONALS.iter())
      .cloned()
      .map(Vec::into_boxed_slice)
      .collect::<Box<_>>();

    assert_eq!(expected, sequences);
  }
}
