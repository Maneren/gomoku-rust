
use super::{board::Board, Score};
use rand::Rng;
use std::{collections::HashMap, fmt};

#[derive(Clone)]
pub struct Stats {
  cache_hit: u32,
  size: u32,
}
impl Stats {
  pub fn new() -> Stats {
    Stats {
      cache_hit: 0,
      size: 0,
    }
  }
}

#[derive(Clone)]
pub struct Cache {
  cache: HashMap<u128, (Score, bool, bool)>, // (score, player, is_end)
  hash_table: Vec<Vec<u128>>,
  pub stats: Stats,
}
impl Cache {
  pub fn new(board_size: u8) -> Cache {
    let mut rng = rand::thread_rng();

    let num_of_tiles = board_size * board_size;
    let num_of_tile_types = 3; // empty, x, o

    // hash_table[x][y]
    // x is current tile, y is tile_type

    let get_row = |_| (0..num_of_tile_types).map(|_| rng.gen::<u128>()).collect();
    let hash_table = (0..num_of_tiles).map(get_row).collect();

    Cache {
      cache: HashMap::new(),
      hash_table,
      stats: Stats::new(),
    }
  }

  pub fn lookup(&mut self, board: &Board) -> Option<&(Score, bool, bool)> {
    let hash = board.hash(&self.hash_table);

    let result = self.cache.get(&hash);

    if result.is_some() {
      self.stats.cache_hit += 1;
    }

    result
  }

  pub fn insert(&mut self, board: &Board, data: (Score, bool, bool)) {
    let hash = board.hash(&self.hash_table);
    self.stats.size += 1;
    self.cache.insert(hash, data);
  }
}
impl fmt::Debug for Stats {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "size: {}, hits: {}", self.size, self.cache_hit)
  }
}
