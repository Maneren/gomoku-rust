use std::cmp;
use std::error;
use std::fmt;

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

pub type Tile = Option<bool>;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TilePointer {
  pub x: u8,
  pub y: u8,
}
impl fmt::Debug for TilePointer {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({},{})", self.x, self.y)
  }
}

#[derive(Clone)]
pub struct Board {
  data: Vec<Tile>,
  size: u8,

  tile_ptrs: Vec<TilePointer>,
  sequences: Vec<Vec<TilePointer>>,
}

impl Board {
  pub fn new(data: Vec<Vec<Option<bool>>>) -> Result<Board, Error> {
    if data.len() <= 8 {
      return Err(Error {
        msg: "Too small board height".into(),
      });
    }

    let height = data.len();

    for (index, row) in data.iter().enumerate() {
      if row.len() != height {
        return Err(Error {
          msg: format!("Invalid board width {} on row {}", row.len(), index + 1),
        });
      }
    }

    #[allow(clippy::cast_possible_truncation)]
    let board_size = data.len() as u8;
    let sequences = Board::get_all_sequences(board_size);
    let tile_ptrs = Self::get_tile_ptrs(board_size);

    let flat_data = data.into_iter().flatten().collect();

    Ok(Board {
      data: flat_data,
      size: board_size,
      tile_ptrs,
      sequences,
    })
  }

  pub fn get_empty_board(size: u8) -> Board {
    let data = (0..size)
      .map(|_| (0..size).map(|_| None).collect())
      .collect();

    Board::new(data).unwrap()
  }

  fn get_all_sequences(board_size: u8) -> Vec<Vec<TilePointer>> {
    let mut sequences = Vec::new();

    // horizontal
    for x in 0..board_size {
      let temp = (0..board_size).map(|y| TilePointer { x, y }).collect();
      sequences.push(temp)
    }

    // vertical
    for y in 0..board_size {
      let temp = (0..board_size).map(|x| TilePointer { x, y }).collect();
      sequences.push(temp)
    }

    // diag1
    for i in 0..(2 * board_size - 1) {
      let row = cmp::min(i, board_size - 1);
      let col = i - row;
      let len = cmp::min(row, board_size - 1 - col) + 1;

      let temp: Vec<TilePointer> = (0..len)
        .map(|j| {
          let x = row - j;
          let y = col + j;
          TilePointer { x, y }
        })
        .collect();

      if !temp.is_empty() {
        sequences.push(temp)
      };
    }

    // diag2
    for i in 0..(2 * board_size - 1) {
      let row = cmp::min(i, board_size - 1);
      let col = i - row;
      let len = cmp::min(row, board_size - 1 - col) + 1;

      let temp: Vec<TilePointer> = (0..len)
        .map(|j| {
          let x = board_size - (row - j) - 1;
          let y = col + j;
          TilePointer { x, y }
        })
        .collect();

      if !temp.is_empty() {
        sequences.push(temp)
      };
    }

    sequences
  }

  fn get_tile_ptrs(size: u8) -> Vec<TilePointer> {
    (0..size)
      .flat_map(|x| (0..size).map(move |y| TilePointer { x, y }))
      .collect()
  }

  pub fn get_all_tile_sequences(&self) -> Vec<Vec<&Option<bool>>> {
    self
      .sequences
      .iter()
      .map(|sequence| sequence.iter().map(|ptr| self.get_tile(ptr)).collect())
      .collect()
  }

  pub fn get_empty_tiles(&self) -> Vec<TilePointer> {
    self
      .tile_ptrs
      .iter()
      .filter(|ptr| self.get_tile(ptr).is_none())
      .map(TilePointer::to_owned)
      .collect()
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
          .map(|tile| {
            if *tile == 'x' {
              Some(true)
            } else if *tile == 'o' {
              Some(false)
            } else {
              None
            }
          })
          .collect()
      })
      .collect();

    let board = Board::new(parsed_data)?;

    Ok(board)
  }

  fn get_index(&self, x: u8, y: u8) -> usize {
    let index = self.size * y + x;
    index as usize
  }

  pub fn get_tile(&self, ptr: &TilePointer) -> &Tile {
    let TilePointer { x, y } = *ptr;
    let index = self.get_index(x, y);
    &self.data[index]
  }

  pub fn set_tile(&mut self, ptr: TilePointer, value: Tile) {
    let TilePointer { x, y } = ptr;
    let index = self.get_index(x, y);
    self.data[index as usize] = value;
  }

  pub fn get_size(&self) -> u8 {
    self.size
  }

  // for caching
  pub fn hash(&self, hash_table: &[Vec<u128>]) -> u128 {
    self.data.iter().enumerate().fold(0, |hash, (index, tile)| {
      let tile_index = tile.map_or(0, |player| if player { 1 } else { 2 });
      hash ^ hash_table[index][tile_index]
    })
  }
}

impl fmt::Display for Board {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut string = String::from("  0123456789\n");
    let board_size = self.get_size();
    for i in 0..board_size {
      let tmp = if i < 10 && board_size >= 10 {
        format!(" {:?}", i)
      } else {
        format!("{:?}", i)
      };
      string.push_str(&tmp);

      let row_rng = (i * board_size) as usize..((i + 1) * board_size) as usize;
      let row = &self.data[row_rng];

      string.push_str(
        &(row
          .iter()
          .map(|field| field.map_or('-', |value| if value { 'x' } else { 'o' }))
          .collect::<String>()
          + "\n"),
      );
    }

    write!(f, "{}", string)
  }
}
