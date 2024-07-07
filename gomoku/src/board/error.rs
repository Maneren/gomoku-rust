use std::{error, fmt};

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
impl error::Error for Error {}
