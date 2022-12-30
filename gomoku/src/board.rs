use std::{error, fmt, iter};

use once_cell::sync::OnceCell;

use super::{Player, Tile};
use crate::Score;

#[derive(Debug)]
pub struct Error {
  msg: String,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}
impl error::Error for Error {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TilePointer {
  pub x: u8,
  pub y: u8,
}
impl fmt::Debug for TilePointer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}{}", (self.x + 0x61) as char, self.y + 1)
  }
}

type Sequence = Vec<usize>;
type Sequences = Vec<Sequence>;

static SEQUENCES: OnceCell<Sequences> = OnceCell::new();

pub fn sequences() -> &'static Sequences {
  if let Some(sequences) = SEQUENCES.get() {
    sequences
  } else {
    panic!("Must initialize the sequences first!")
  }
}

pub fn initialize_sequences(board_size: u8) {
  SEQUENCES
    .set(Board::generate_sequences(board_size))
    .unwrap();
}

#[derive(Clone)]
pub struct Board {
  size: u8,
  data: Vec<Tile>,
}

impl Board {
  pub fn new(data: Vec<Vec<Tile>>) -> Result<Board, Error> {
    if data.len() <= 8 {
      return Err(Error {
        msg: "Too small board".into(),
      });
    }

    for (index, row) in data.iter().enumerate() {
      if row.len() != data.len() {
        return Err(Error {
          msg: format!("Invalid board width {} on row {}", row.len(), index + 1),
        });
      }
    }

    #[allow(clippy::cast_possible_truncation)]
    let board_size = data.len() as u8;
    let flat_data = data.into_iter().flatten().collect();

    Ok(Board {
      data: flat_data,
      size: board_size,
    })
  }

  #[must_use]
  pub fn get_empty_board(size: u8) -> Board {
    let data = iter::repeat(None).take(size.pow(2) as usize).collect();

    Board { size, data }
  }

  fn make_row(size: usize, y: usize) -> Sequence {
    let x = 0;

    (0..size).map(|i| x + i + y * size).collect()
  }

  fn make_col(size: usize, x: usize) -> Sequence {
    let y = 0;

    (0..size).map(|i| x + (y + i) * size).collect()
  }

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

  fn make_diag2(size: usize, a: usize, b: usize) -> Sequence {
    let min = a.min(b);

    let a = a - min;
    let b = b - min;

    let len = size - a - b;

    let base = a + b * size;
    let offset = size + 1;

    (0..len).map(|i| base + i * offset).collect()
  }

  #[must_use]
  pub fn generate_sequences(size: u8) -> Sequences {
    let size = size as usize;

    let rows = (0..size).map(|y| Self::make_row(size, y));
    let columns = (0..size).map(|x| Self::make_col(size, x));

    let diag11 = (0..size).map(|k| Self::make_diag1(size, k, 0)).rev();
    let diag12 = (0..size).map(|k| Self::make_diag1(size, 0, k)).skip(1);

    let diag21 = (0..size).map(|k| Self::make_diag2(size, k, 0)).rev();
    let diag22 = (0..size).map(|k| Self::make_diag2(size, 0, k)).skip(1);

    rows
      .chain(columns)
      .chain(diag11)
      .chain(diag12)
      .chain(diag21)
      .chain(diag22)
      .collect()
  }

  #[must_use]
  pub fn get_relevant_sequences(&self, ptr: TilePointer) -> [&Sequence; 4] {
    let n = self.size as usize;
    let TilePointer { x, y } = ptr;
    let x = x as usize;
    let y = y as usize;

    [
      &sequences()[y],                     // row
      &sequences()[x + n],                 // column
      &sequences()[x + y + 2 * n],         // diagonal
      &sequences()[y + n - x + 4 * n - 2], // other diagonal
    ]
  }

  pub fn get_empty_tiles(&self) -> Result<Vec<TilePointer>, Error> {
    let tiles: Vec<_> = self
      .data
      .iter()
      .enumerate()
      .filter(|(.., tile)| tile.is_none())
      .map(|(index, ..)| Self::get_ptr_from_index(index, self.size))
      .collect();

    if tiles.is_empty() {
      Err(Error {
        msg: "No empty tiles found".into(),
      })
    } else {
      Ok(tiles)
    }
  }

  #[must_use]
  pub fn squared_distance_from_center(&self, p: TilePointer) -> Score {
    let center = f32::from(self.size - 1) / 2.0; // -1 to adjust for 0-indexing

    let x = f32::from(p.x);
    let y = f32::from(p.y);
    let dist = (x - center).powi(2) + (y - center).powi(2);

    dist.round() as Score
  }

  pub fn from_string(input_string: &str) -> Result<Board, Error> {
    // split string into Vec<Vec<chars>>
    let rows = input_string
      .trim()
      .split('\n')
      .map(|row| row.chars().collect())
      .collect::<Vec<Vec<char>>>();

    // parse Vec<Vec<char>> into Vec<Vec<Tile>>
    let parsed_data: Vec<Vec<Tile>> = rows
      .iter()
      .map(|row| {
        row
          .iter()
          .map(|tile| match *tile {
            'x' | 'X' => Some(Player::X),
            'o' | 'O' => Some(Player::O),
            _ => None,
          })
          .collect()
      })
      .collect();

    let board = Board::new(parsed_data)?;

    Ok(board)
  }

  fn get_ptr_from_index(index: usize, size: u8) -> TilePointer {
    let x = index % size as usize;
    let y = index / size as usize;

    TilePointer {
      x: x as u8,
      y: y as u8,
    }
  }

  fn get_index(size: u8, ptr: TilePointer) -> usize {
    let TilePointer { x, y } = ptr;
    Self::get_index_raw(size, x, y)
  }

  fn get_index_raw(size: u8, x: u8, y: u8) -> usize {
    usize::from(size) * usize::from(y) + usize::from(x)
  }

  #[must_use]
  pub fn get_tile(&self, ptr: TilePointer) -> &Tile {
    let index = Self::get_index(self.size, ptr);
    self.get_tile_raw(index)
  }

  #[must_use]
  pub fn get_tile_raw(&self, index: usize) -> &Tile {
    self
      .data
      .get(index)
      .unwrap_or_else(|| panic!("Tile index out of bounds: {index}"))
  }

  pub fn set_tile(&mut self, ptr: TilePointer, value: Tile) {
    let index = Self::get_index(self.size, ptr);

    if (value.is_some() && self.get_tile_raw(index).is_some())
      || (value.is_none() && self.get_tile_raw(index).is_none())
    {
      panic!(
        "attempted to overwrite tile {:?} with value {:?} at board \n{}",
        ptr, value, self
      );
    }

    self.data[index] = value;
  }

  #[must_use]
  pub fn get_size(&self) -> u8 {
    self.size
  }
}
impl fmt::Display for Board {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let board_size = self.size as usize;

    let mut string: String = String::new()
      + if board_size >= 10 { "  " } else { " " }
      + &"abcdefghijklmnopqrstuvwxyz"
        .chars()
        .take(board_size)
        .collect::<String>()
      + "\n";

    for i in 0..board_size {
      let tmp = if i + 1 < 10 && board_size >= 10 {
        format!(" {:?}", i + 1)
      } else {
        format!("{:?}", i + 1)
      };
      string.push_str(&tmp);

      let row_start = i * board_size;
      let row_end = (i + 1) * board_size;

      let row = &self.data[row_start..row_end];
      let row_string: String = row
        .iter()
        .map(|field| field.map_or('-', Player::char))
        .collect();

      string.push_str(&(row_string + "\n"));
    }

    write!(f, "{string}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const BOARD_DATA: &str = "---------
---------
---x-----
---xoo---
----xo---
---xxxo--
------oo-
--------x
---------";
  const BOARD_SIZE: u8 = 9;

  #[test]
  fn test_from_string() {
    let board = Board::from_string(BOARD_DATA).unwrap();

    assert_eq!(board.get_size(), BOARD_SIZE);
  }

  #[test]
  fn test_initialize_sequences() {
    let board_size = BOARD_SIZE;

    initialize_sequences(board_size);

    assert!(!sequences().is_empty());

    let mut visits = vec![0; board_size.pow(2) as usize];

    for sequence in sequences() {
      for index in sequence {
        visits[*index] += 1;
      }
    }

    for visit in &visits {
      assert_eq!(*visit, 4);
    }
  }

  #[test]
  fn test_get_index() {
    let x = 2;
    let y = 3;
    let tile = TilePointer { x, y };
    let target = (x + y * BOARD_SIZE) as usize;

    assert_eq!(Board::get_index_raw(BOARD_SIZE, x, y), target);
    assert_eq!(Board::get_index(BOARD_SIZE, tile), target);
  }

  #[test]
  fn test_get_relevant_sequences() {
    let board = Board::from_string(BOARD_DATA).unwrap();

    initialize_sequences(board.get_size());

    let x = 2;
    let y = 3;
    let tile = TilePointer { x, y };
    let target = Board::get_index(BOARD_SIZE, tile);

    let sequences = board.get_relevant_sequences(tile);

    sequences
      .iter()
      .for_each(|sequence| assert!(sequence.iter().any(|index| *index == target)));
  }
}
