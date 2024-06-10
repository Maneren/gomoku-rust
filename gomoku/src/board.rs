pub(crate) mod evaluation;
mod sequences;

use std::{error, fmt, str::FromStr};

use evaluation::{shape_score, Eval};
use once_cell::sync::OnceCell;
use sequences::{generate, Sequence, Sequences};

use super::{Player, Score};
use crate::state::State;

#[derive(Debug, Clone)]
pub enum Error {
  TooSmall {
    size: usize,
  },
  NotSquare {
    height: usize,
    line: usize,
    width: usize,
  },
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::TooSmall { size } => write!(f, "board too small: {size}, but minimum is 9"),
      Error::NotSquare {
        height,
        line,
        width,
      } => {
        write!(
          f,
          "board is not a square: line {line} is {width} tiles wide, but {height} was expected"
        )
      }
    }
  }
}
impl error::Error for Error {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}

/// Represents a tile on the board.
///
/// Some(Player) indicates that the tile is occupied.
/// None indicates that the tile is empty.
pub type Tile = Option<Player>;

/// Represents a pointer to a tile on the board.
///
/// Doesn't provide any bounds checking or other guarantees.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TilePointer {
  /// x coordinate
  pub x: u8,
  /// y coordinate
  pub y: u8,
}
impl TryFrom<&str> for TilePointer {
  type Error = Box<dyn std::error::Error>;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let mut chars = value.chars();

    let x = chars.next().ok_or::<Self::Error>("No input".into())?;
    let y = chars.collect::<String>().parse::<u8>()?;

    let x = x as u8 - b'a';
    let y = y - 1;

    Ok(TilePointer { x, y })
  }
}
impl fmt::Debug for TilePointer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}{}", (self.x + b'a') as char, self.y + 1)
  }
}
impl fmt::Display for TilePointer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{self:?}")
  }
}

/// Cached sequences for very fast board access
// HACK: Relies on the fact that the board size is the same thoroughout the whole runtime.
// This is good enough for now, but **should** be refactored in the future.
static SEQUENCES: OnceCell<Sequences> = OnceCell::new();

fn initialize_sequences(board_size: u8) {
  let sequences = SEQUENCES.get_or_init(|| generate(board_size));

  assert_eq!(
    sequences.len(),
    6 * board_size as usize - 2,
    "Incompatible board size and sequences",
  );
}

/// A Gomoku board.
///
/// The board is guaranteed to be a square and at least 9x9.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board {
  size: u8,
  data: Box<[Tile]>,
}

impl Board {
  /// Create a new board from a 2D vector of tiles.
  ///
  /// The board must be a square and at least 9x9.
  ///
  /// # Errors
  /// Returns an error if the board is not a square or is too small.
  pub fn new(data: Vec<Vec<Tile>>) -> Result<Board, Error> {
    if data.len() <= 8 {
      return Err(Error::TooSmall { size: data.len() });
    }

    for (index, row) in data.iter().enumerate() {
      if row.len() != data.len() {
        return Err(Error::NotSquare {
          height: data.len(),
          line: index + 1,
          width: row.len(),
        });
      }
    }

    let board_size = data.len() as u8;
    let flat_data = data.into_iter().flatten().collect();

    initialize_sequences(board_size);

    Ok(Board {
      data: flat_data,
      size: board_size,
    })
  }

  /// Create an empty board of the given size.
  pub fn new_empty(size: u8) -> Board {
    let data = vec![None; size.pow(2) as usize].into_boxed_slice();

    initialize_sequences(size);

    Board { size, data }
  }

  /// Get a reference to the sequences table.
  ///
  /// # Panics
  /// Panics if the sequences table has not been initialized.
  pub fn sequences(&self) -> &'static Sequences {
    SEQUENCES.get().expect("Sequences are initialized")
  }

  /// Get sequences relevant for the given tile.
  ///
  /// Relevant means the column, row and both diagonals that include the tile.
  pub fn relevant_sequences(&self, ptr: TilePointer) -> [&Sequence; 4] {
    let n = self.size;
    let TilePointer { x, y } = ptr;

    let sequences = self.sequences();

    [
      &sequences[usize::from(y)],                       // row
      &sequences[usize::from(n + x)],                   // column
      &sequences[usize::from(2 * n + x + y)],           // diagonal
      &sequences[usize::from((4 * n - 2) + n + y - x)], // other diagonal
    ]
  }

  /// Get iterator over all empty tiles in the board.
  pub fn pointers_to_empty_tiles(&self) -> impl Iterator<Item = TilePointer> + '_ {
    self
      .data
      .iter()
      .enumerate()
      .filter(|(.., tile)| tile.is_none())
      .map(|(index, ..)| self.get_ptr_from_index(index))
  }

  /// Get reference to slice of all tiles in the board.
  pub fn tiles(&self) -> &[Tile] {
    &self.data
  }

  /// Calculate the square of the distance from the center of the board.
  pub fn squared_distance_from_center(&self, p: TilePointer) -> Score {
    let center = f32::from(self.size - 1) / 2.0; // -1 to adjust for 0-indexing

    let x = f32::from(p.x);
    let y = f32::from(p.y);
    let dist = (x - center).powi(2) + (y - center).powi(2);

    dist.round() as Score
  }

  /// Convert a raw index to `TilePointer`.
  pub fn get_ptr_from_index(&self, index: usize) -> TilePointer {
    let x = (index % self.size as usize) as u8;
    let y = (index / self.size as usize) as u8;

    TilePointer { x, y }
  }

  fn get_index(size: u8, ptr: TilePointer) -> usize {
    let TilePointer { x, y } = ptr;
    Self::get_index_raw(size, x, y)
  }

  fn get_index_raw(size: u8, x: u8, y: u8) -> usize {
    usize::from(size) * usize::from(y) + usize::from(x)
  }

  /// Get value of a tile at the given pointer.
  ///
  /// # Panics
  /// Panics if the pointer is out of bounds.
  pub fn get_tile(&self, ptr: TilePointer) -> &Tile {
    let index = Self::get_index(self.size, ptr);
    self.get_tile_raw(index)
  }

  /// Get value of a tile at the given index.
  ///
  /// # Panics
  /// Panics if the index is out of bounds.
  pub fn get_tile_raw(&self, index: usize) -> &Tile {
    self
      .data
      .get(index)
      .unwrap_or_else(|| panic!("Tile index out of bounds: {index}"))
  }

  /// Set a tile at the given pointer.
  ///
  /// # Panics
  /// Panics at attempt to overwrite an already occupied tile.
  pub fn set_tile(&mut self, ptr: TilePointer, value: Tile) {
    let index = Self::get_index(self.size, ptr);

    let tile = self.get_tile_raw(index);

    // either write Some to empty tile (play) or None to occupied tile (undo)
    assert!(
      matches!((value, tile), (Some(_), None) | (None, Some(_))),
      "attempted to overwrite tile {ptr} ({tile:?}) with value {value:?} at board \n{self}"
    );

    self.data[index] = value;
  }

  /// Get the size of the board.
  pub fn size(&self) -> u8 {
    self.size
  }

  fn evaluate_sequence(&self, sequence: &[usize]) -> Eval {
    let mut eval = Eval::default();

    let mut current = Player::X; // current player
    let mut consecutive = 0; // consecutive tiles of the current player
    let mut open_ends = 0; // open ends of consecutive tiles
    let mut has_hole = false; // is there a hole in the consecutive tiles

    for (i, &tile_idx) in sequence.iter().enumerate() {
      if let Some(player) = self.data[tile_idx] {
        if player == current {
          consecutive += 1;
          continue;
        }

        // opponent's tile
        if consecutive > 0 {
          let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole);
          eval.score[current] += shape_score;
          eval.win[current] |= is_win_shape;

          open_ends = 0;
          has_hole = false;
        }

        consecutive = 1;
        current = player;
      } else {
        // empty tile
        if consecutive == 0 {
          open_ends = 1; // If there were no consecutive tiles yet, mark as an open end
          has_hole = false;
          continue;
        }

        // If there is no hole yet, and the next tile is of the current player,
        // and consecutive count is less than 5, mark as a hole
        if !has_hole
          && consecutive < 5
          && sequence.get(i + 1).and_then(|&idx| self.data[idx]) == Some(current)
        {
          has_hole = true;
          consecutive += 1;
          continue;
        }

        open_ends += 1;

        let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole);
        eval.score[current] += shape_score;
        eval.win[current] |= is_win_shape;

        consecutive = 0;
        open_ends = 1;
        has_hole = false;
      }
    }

    // If there are consecutive tiles at the end of the sequence
    if consecutive > 0 {
      let (shape_score, is_win_shape) = shape_score(consecutive, open_ends, has_hole);
      eval.score[current] += shape_score;
      eval.win[current] |= is_win_shape;
    }

    eval
  }

  /// Evaluate sequences relevat to given tile
  ///
  /// Relevant means the column, row and both diagonals that include the tile.
  pub fn evaluate_sequences_relevant_to(&self, tile: TilePointer) -> Eval {
    self
      .relevant_sequences(tile)
      .into_iter()
      .map(|seq| self.evaluate_sequence(seq))
      .sum()
  }

  /// Evaluate the whole board and return summary for both players
  pub fn evaluate(&self) -> Eval {
    self
      .sequences()
      .iter()
      .map(|seq| self.evaluate_sequence(seq))
      .sum()
  }

  /// Evaluate the whole board and return result for target player
  pub fn evaluate_for(&self, target: Player) -> (Score, State) {
    let Eval { score, win } = self.evaluate();

    let score = score[target] - score[!target];

    let state = if win[target] {
      State::Win
    } else {
      State::NotEnd
    };

    (score, state)
  }
}

impl FromStr for Board {
  type Err = Error;

  /// Parse a string into a board.
  ///
  /// Expects the same format, that is produced by [`Board::to_string`].
  ///
  /// # Errors
  /// Returns an error if the board is not a square or is too small.
  fn from_str(input_string: &str) -> Result<Board, Self::Err> {
    // split string into Vec<Vec<chars>>
    let rows = input_string
      .lines()
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
}

impl fmt::Display for Board {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let board_size = self.size as usize;

    let indent = if board_size >= 10 { " " } else { "" };

    writeln!(
      f,
      "{indent} {}",
      &"abcdefghijklmnopqrstuvwxyz"[..board_size]
    )?;

    for (i, row) in self.data.chunks(board_size).enumerate() {
      write!(f, "{}{}", if i + 1 < 10 { indent } else { "" }, i + 1)?;

      row
        .iter()
        .map(|field| field.map_or('-', Player::char))
        .try_for_each(|c| write!(f, "{c}"))?;

      writeln!(f)?;
    }

    Ok(())
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
    let board = Board::from_str(BOARD_DATA).unwrap();

    assert_eq!(board.size(), BOARD_SIZE);
  }

  #[test]
  fn test_initialize_sequences() {
    let board_size = BOARD_SIZE;

    let board = Board::new_empty(board_size);

    assert!(!board.sequences().is_empty());

    let mut visits = vec![0; board_size.pow(2) as usize];

    for sequence in board.sequences().iter() {
      for index in sequence.iter() {
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
    let board = Board::from_str(BOARD_DATA).unwrap();

    for x in 0..BOARD_SIZE {
      for y in 0..BOARD_SIZE {
        let tile = TilePointer { x, y };
        let target = Board::get_index(BOARD_SIZE, tile);

        let sequences = board.relevant_sequences(tile);

        sequences
          .iter()
          .for_each(|sequence| assert!(sequence.iter().any(|index| *index == target)));
      }
    }
  }
}
