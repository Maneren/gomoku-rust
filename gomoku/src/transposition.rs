use std::sync::OnceLock;

use dashmap::DashMap;

use crate::{Score, board::TilePointer, player::Player, state::State};

/// Zobrist keys — 19×19 board, 2 players + side to move.
/// Deterministic splitmix64 from a fixed seed.
static ZOBRIST: OnceLock<Vec<u64>> = OnceLock::new();

fn init_zobrist() -> Vec<u64> {
  let mut keys = Vec::with_capacity(19 * 19 * 2 + 2);
  let mut seed: u64 = 0x9E3779B97F4A7C15;
  for _ in 0..(19 * 19 * 2 + 2) {
    seed = seed.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^= z >> 31;
    keys.push(z);
  }
  keys
}

#[inline]
pub fn zobrist_key(x: u8, y: u8, player: Player) -> u64 {
  let keys = ZOBRIST.get_or_init(init_zobrist);
  // 19 stride guarantees unique index for any board ≤19.
  let idx = (y as usize * 19 + x as usize) * 2
    + match player {
      Player::X => 0,
      Player::O => 1,
    };
  keys[idx]
}

#[inline]
pub fn zobrist_side(player: Player) -> u64 {
  let keys = ZOBRIST.get_or_init(init_zobrist);
  // last two keys for side to move
  match player {
    Player::X => keys[19 * 19 * 2],
    Player::O => keys[19 * 19 * 2 + 1],
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
  Exact,
  Lower,
  Upper,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
  pub score: Score,
  pub depth: u8,
  pub bound: Bound,
  pub best_move: Option<TilePointer>,
  pub state: State,
}

pub struct TranspositionTable {
  table: DashMap<u64, TTEntry>,
}

impl TranspositionTable {
  pub fn new() -> Self {
    Self {
      table: DashMap::with_capacity(1 << 16),
    }
  }

  pub fn probe(&self, hash: u64, depth: u8, alpha: Score, beta: Score) -> Option<TTEntry> {
    let entry = self.table.get(&hash).map(|r| *r)?;
    if entry.depth < depth {
      return None;
    }
    match entry.bound {
      Bound::Exact => Some(entry),
      Bound::Lower if entry.score >= beta => Some(entry),
      Bound::Upper if entry.score <= alpha => Some(entry),
      _ => None,
    }
  }

  /// Peek without depth/bound check — used for move ordering.
  pub fn peek(&self, hash: u64) -> Option<TTEntry> {
    self.table.get(&hash).map(|r| *r)
  }

  pub fn store(&self, hash: u64, entry: TTEntry) {
    // Keep deeper entries.
    if let Some(old) = self.table.get(&hash) {
      if old.depth > entry.depth {
        return;
      }
    }
    self.table.insert(hash, entry);
  }

  pub fn len(&self) -> usize {
    self.table.len()
  }

  pub fn clear(&self) {
    self.table.clear();
  }

  pub fn retain_terminal(&self) {
    // Keep only terminal entries (Win/Lose/Draw) which are exact and
    // path-independent. Remove heuristic NotEnd entries that were stored
    // with narrow windows.
    self.table.retain(|_, v| v.state != State::NotEnd);
  }
}

impl Default for TranspositionTable {
  fn default() -> Self {
    Self::new()
  }
}
