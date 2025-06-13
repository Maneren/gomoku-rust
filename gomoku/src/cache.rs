use std::sync::atomic::{self, AtomicU64};

use crate::{board::ZobristHash, Board, Score};

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct CacheEntry {
  pub score: Score,
  pub node_type: NodeType,
  pub depth: u8,
}

const SCORE_OFFSET: u32 = 0;
const DEPTH_OFFSET: u32 = Score::BITS;
const NODE_TYPE_OFFSET: u32 = DEPTH_OFFSET + u8::BITS;
const CHECKSUM_OFFSET: u32 = NODE_TYPE_OFFSET + 2;
const CHECKSUM_MASK: u64 = u64::MAX >> CHECKSUM_OFFSET;

impl CacheEntry {
  pub fn encode(self, key: ZobristHash) -> ZobristHash {
    let score = (self.score.cast_unsigned() as ZobristHash) << SCORE_OFFSET;
    let node_type = (self.node_type as ZobristHash) << NODE_TYPE_OFFSET;
    let depth = (self.depth as ZobristHash) << DEPTH_OFFSET;
    let checksum = CHECKSUM_MASK << CHECKSUM_OFFSET;

    score | node_type | depth | (key & checksum)
  }

  pub fn decode(encoded: ZobristHash) -> Option<(Self, u32)> {
    let checksum = encoded >> CHECKSUM_OFFSET;
    let score = (encoded >> SCORE_OFFSET) as Score;
    let depth = (encoded >> DEPTH_OFFSET) as u8;
    let node_type = (encoded >> NODE_TYPE_OFFSET) & 0b11;

    let node_type = match node_type {
      0 => NodeType::Exact,
      1 => NodeType::LowerBound,
      2 => NodeType::UpperBound,
      _ => {
        return None;
      },
    };

    Some((
      Self {
        score,
        node_type,
        depth,
      },
      checksum as u32,
    ))
  }
}

fn should_replace(old: CacheEntry, new: CacheEntry) -> bool {
  fn extra_depth(analysis: CacheEntry) -> u8 {
    // +1 depth for Exact scores and lower bounds
    matches!(analysis.node_type, NodeType::Exact | NodeType::LowerBound) as u8
  }

  let new_depth = new.depth + extra_depth(new);
  let prev_depth = old.depth + extra_depth(old);

  new_depth * 2 + 1 >= prev_depth
}

const HASH_TABLE_SIZE: usize = 1 << 28;
#[derive(Debug)]
pub struct HashTable {
  data: Box<[AtomicU64]>,
}

impl HashTable {
  pub fn new(size: usize) -> HashTable {
    HashTable {
      data: (0..size).map(|_| AtomicU64::default()).collect::<Box<_>>(),
    }
  }

  fn index(&self, hash: ZobristHash) -> usize {
    let key = u128::from(hash);
    let len = self.data.len() as u128;
    ((key * len) >> 64) as usize
  }

  pub fn get(&self, hash: ZobristHash) -> Option<CacheEntry> {
    let index = self.index(hash);
    let entry_value = self.data[index].load(atomic::Ordering::Relaxed);
    let (entry, checksum) = CacheEntry::decode(entry_value)?;

    if checksum == (hash >> CHECKSUM_OFFSET) as u32 {
      Some(entry)
    } else {
      None
    }
  }

  pub fn insert(&self, hash: ZobristHash, entry: CacheEntry) {
    let index = self.index(hash);

    let old_value = self.data[index].load(atomic::Ordering::Relaxed);

    if old_value == 0 {
      self.data[index].store(entry.encode(hash), atomic::Ordering::Relaxed);
      return;
    }

    let Some((old_entry, _)) = CacheEntry::decode(old_value) else {
      unreachable!("Corrupted cache entry found");
    };

    if should_replace(old_entry, entry) {
      self.data[index].store(entry.encode(hash), atomic::Ordering::Relaxed);
    }
  }

  pub fn size(&self) -> usize {
    self.data.len()
  }
}

impl Default for HashTable {
  fn default() -> Self {
    HashTable::new(HASH_TABLE_SIZE)
  }
}

#[derive(Default)]
pub struct Cache {
  pub evaluated: HashTable,
}

impl Cache {
  pub fn get(&self, board: &Board) -> Option<CacheEntry> {
    self.evaluated.get(board.zobrist_hash())
  }

  pub fn insert(&self, board: &Board, entry: CacheEntry) {
    self.evaluated.insert(board.zobrist_hash(), entry);
  }

  pub fn with_capacity(capacity: usize) -> Cache {
    Cache {
      evaluated: HashTable::new(capacity),
    }
  }

  pub fn size(&self) -> usize {
    self.evaluated.size()
  }
}

unsafe impl Send for HashTable {}
unsafe impl Sync for HashTable {}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub enum NodeType {
  #[default]
  Exact,
  LowerBound,
  UpperBound,
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

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
  fn test_hash_table() {
    let table = HashTable::default();
    assert!(table.size() > 0);

    let board = Board::from_str(BOARD_DATA).unwrap();

    let hash = board.zobrist_hash();

    println!("Hash of board \n{board} is {hash:x}");

    let entry = CacheEntry {
      score: 1,
      node_type: NodeType::Exact,
      depth: 2,
    };

    table.insert(hash, entry);

    let retrieved = table.get(hash);

    assert_eq!(retrieved, Some(entry));
  }
}
