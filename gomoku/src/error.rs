use std::{error::Error, fmt::Display};

use crate::board;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum GomokuError {
  NoEmptyTiles,
  GameEnd,
  MisshapedBoard(board::Error),
}

impl Error for GomokuError {}

impl Display for GomokuError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      GomokuError::NoEmptyTiles => write!(f, "no empty tiles left"),
      GomokuError::GameEnd => write!(f, "game already ended"),
      GomokuError::MisshapedBoard(error) => write!(f, "{error}"),
    }
  }
}
