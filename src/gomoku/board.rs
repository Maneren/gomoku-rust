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

#[derive(Debug, Clone, Copy)]
pub struct TilePointer {
  pub x: u8,
  pub y: u8,
}

#[derive(Clone)]
pub struct Board {
  data: Vec<Vec<Tile>>,
  pub sequences: Vec<Vec<TilePointer>>,
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
    let sequences = Board::get_all_sequences(data.len() as u8);

    Ok(Board { data, sequences })
  }

  pub fn empty(size: u8) -> Board {
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

  pub fn get_all_tile_sequences(&self) -> Vec<Vec<&Option<bool>>> {
    self
      .sequences
      .iter()
      .map(|sequence| sequence.iter().map(|ptr| self.get_tile(ptr)).collect())
      .collect()
  }

  pub fn get_empty_tiles(&self) -> Vec<TilePointer> {
    let board_size = self.get_size();

    (0..board_size)
      .flat_map(|x| (0..board_size).map(move |y| TilePointer { x, y }))
      .filter(|ptr| self.get_tile(ptr).is_none())
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

  pub fn get_tile(&self, ptr: &TilePointer) -> &Tile {
    let TilePointer { x, y } = *ptr;
    &self.data[y as usize][x as usize]
  }

  pub fn set_tile(&mut self, ptr: TilePointer, value: Tile) {
    let TilePointer { x, y } = ptr;
    self.data[y as usize][x as usize] = value;
  }

  pub fn get_size(&self) -> u8 {
    #[allow(clippy::cast_possible_truncation)]
    let length = self.data.len() as u8;

    length
  }

  // just for caching
  pub fn hash(&self) -> u128 {
    self.data.iter().flatten().fold(0, |total, tile| {
      let hash = total + tile.map_or(0, |player| if player { 1 } else { 2 });
      if hash >= u128::MAX / 3 {
        hash / 164_986_984 * 3 // random large number
      } else {
        hash * 3
      }
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

      let row = &self.data[i as usize];
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
