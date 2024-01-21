use super::{Sequence, Sequences};

/// Create `Sequence` representing given row
fn make_row(size: usize, y: usize) -> Sequence {
  (0..size).map(|x| x + y * size).collect()
}

/// Create `Sequence` representing given column
fn make_col(size: usize, x: usize) -> Sequence {
  (0..size).map(|y| x + y * size).collect()
}

/// Create `Sequence` representing given diagonal going
/// from top left to bottom right
fn make_diag1(size: usize, a: usize, b: usize) -> Sequence {
  let min = a.min(b);

  let a = a - min;
  let b = b - min;

  let len = size - a - b;

  let a = size - a - 1;

  let base = a + b * size;
  let offset = size - 1;

  (0..len).map(|i| base + i * offset).collect()
}

/// Create `Sequence` representing given diagonal going
/// from top right to bottom left
fn make_diag2(size: usize, a: usize, b: usize) -> Sequence {
  let min = a.min(b);

  let a = a - min;
  let b = b - min;

  let len = size - a - b;

  let base = a + b * size;
  let offset = size + 1;

  (0..len).map(|i| base + i * offset).collect()
}

/// Generate all possible sequences for the given board size
pub fn generate(size: u8) -> Sequences {
  let size = size as usize;

  let rows = (0..size).map(|y| make_row(size, y));
  let columns = (0..size).map(|x| make_col(size, x));

  let diag11 = (0..size).map(|k| make_diag1(size, k, 0)).rev();
  let diag12 = (0..size).map(|k| make_diag1(size, 0, k)).skip(1);

  let diag21 = (0..size).map(|k| make_diag2(size, k, 0)).rev();
  let diag22 = (0..size).map(|k| make_diag2(size, 0, k)).skip(1);

  rows
    .chain(columns)
    .chain(diag11)
    .chain(diag12)
    .chain(diag21)
    .chain(diag22)
    .collect()
}
