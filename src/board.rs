use std::error;
use std::fmt;

#[derive(Debug)]
pub struct BoardError {
  msg: String,
}

impl fmt::Display for BoardError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.msg)
  }
}

impl error::Error for BoardError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}

pub type Tile = Option<bool>;
pub type TilePointer = (usize, usize);


pub struct Board {
  pub data: Vec<Vec<Tile>>,
}

impl Board {
  pub fn new(input_data: Vec<Vec<char>>) -> Result<Board, BoardError> {
    if input_data.len() != 10 {
      return Err(BoardError {
        msg: "Invalid board height".into(),
      });
    }

    for i in 0..input_data.len() {
      if input_data.get(i).unwrap().len() != 10 {
        return Err(BoardError {
          msg: format!("Invalid board width on row {}", i + 1),
        });
      }
    }

    let data = input_data
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

    Ok(Board { data })
  }

  pub fn from_string(input_string: &str) -> Result<Board, BoardError> {
    // split string into Vec<Vec<chars>>
    let rows = input_string
      .trim()
      .split('\n')
      .map(|row| row.chars().collect())
      .collect::<Vec<Vec<char>>>();

    // parse Vec<Vec<char>> into Vec<Vec<Tile>>
    let parsed_data = rows
      .iter()
      .map(|row| row.iter().map(|ch| ch.to_owned()).collect())
      .collect();

    let board = Board::new(parsed_data)?;

    Ok(board)
  }

  pub fn get_tile(&self, ptr: TilePointer) -> &Tile {
    let (x, y) = ptr;
    &self.data[y][x]
  }

  pub fn set_tile(&mut self, ptr: TilePointer, value: Tile) {
    let (x, y) = ptr;
    self.data[y][x] = value;
  }
}

impl std::clone::Clone for Board {
  fn clone(&self) -> Self {
    Board {
      data: self.data.clone(),
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
