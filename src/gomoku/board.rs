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
pub type TilePointer = (usize, usize);

pub struct Board {
  data: Vec<Vec<Tile>>,
  pub sequences: Vec<Vec<TilePointer>>,
}

impl Board {
  pub fn new(input_data: &[Vec<char>]) -> Result<Board, Error> {
    if input_data.len() <= 8 {
      return Err(Error {
        msg: "Invalid board height".into(),
      });
    }

    let height = input_data.len();

    for (index, row) in input_data.iter().enumerate() {
      if row.len() != height {
        return Err(Error {
          msg: format!("Invalid board width on row {}", index + 1),
        });
      }
    }

    let data: Vec<Vec<Tile>> = input_data
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

    let sequences = Board::get_all_sequences(data.len());

    Ok(Board { data, sequences })
  }

  fn get_all_sequences(board_size: usize) -> Vec<Vec<TilePointer>> {
    let mut sequences = Vec::new();

    // horizontal
    for x in 0..board_size {
      let mut temp = vec![];
      for y in 0..board_size {
        temp.push((x, y));
      }
      if !temp.is_empty() {
        sequences.push(temp)
      };
    }

    // vertical
    for y in 0..board_size {
      let mut temp = vec![];
      for x in 0..board_size {
        temp.push((x, y));
      }
      if !temp.is_empty() {
        sequences.push(temp)
      };
    }

    // diag1
    for i in 0..(2 * board_size - 1) {
      let row = cmp::min(i, board_size - 1);
      let col = i - row;
      let len = cmp::min(row, board_size - 1 - col) + 1;

      let mut temp = vec![];
      for j in 0..len {
        let x = row - j;
        let y = col + j;
        temp.push((x, y));
      }

      if !temp.is_empty() {
        sequences.push(temp)
      };
    }

    // diag2
    for i in 0..(2 * board_size - 1) {
      let row = cmp::min(i, board_size - 1);
      let col = i - row;
      let len = cmp::min(row, board_size - 1 - col) + 1;

      let mut temp = vec![];
      for j in 0..len {
        let x = board_size - (row - j) - 1;
        let y = col + j;
        temp.push((x, y));
      }

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
    let mut empty_fields: Vec<TilePointer> = vec![];
    let board_size = self.get_size();

    for y in 0..board_size {
      for x in 0..board_size {
        let ptr = (x, y);
        if self.get_tile(&ptr).is_none() {
          empty_fields.push(ptr);
        }
      }
    }

    empty_fields
  }

  pub fn from_string(input_string: &str) -> Result<Board, Error> {
    // split string into Vec<Vec<chars>>
    let rows = input_string
      .trim()
      .split('\n')
      .map(|row| row.chars().collect())
      .collect::<Vec<Vec<char>>>();

    // parse Vec<Vec<char>> into Vec<Vec<Tile>>
    let parsed_data: Vec<Vec<char>> = rows
      .iter()
      .map(|row| row.iter().map(char::to_owned).collect())
      .collect();

    let board = Board::new(&parsed_data)?;

    Ok(board)
  }

  pub fn get_tile(&self, ptr: &TilePointer) -> &Tile {
    let (x, y) = *ptr;
    &self.data[y][x]
  }

  pub fn set_tile(&mut self, ptr: &TilePointer, value: Tile) {
    let (x, y) = *ptr;
    self.data[y][x] = value;
  }

  pub fn get_size(&self) -> usize {
    self.data.len()
  }

  pub fn hash(&self) -> i128 {
    let mut hash = 0;
    for tile in self.data.iter().flatten() {
      hash += match *tile {
        Some(player) => {
          if player {
            1
          } else {
            2
          }
        }
        None => 0,
      };
      if hash >= i128::MAX / 3 {
        hash /= 10
      }
      hash *= 3;
    }
    hash
  }
}

impl std::clone::Clone for Board {
  fn clone(&self) -> Self {
    Board {
      data: self.data.clone(),
      sequences: self.sequences.clone(),
    }
  }
}

impl fmt::Display for Board {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut string = String::from(" 0123456789\n");
    for i in 0..self.data.len() {
      let row = &self.data[i];
      string.push_str(&format!("{:?}", i));
      string.push_str(
        &(row
          .iter()
          .map(|field| match field {
            Some(val) => {
              if *val {
                'x'
              } else {
                'o'
              }
            }
            None => '-',
          })
          .collect::<String>()
          + "\n"),
      );
    }

    write!(f, "{}", string)
  }
}
