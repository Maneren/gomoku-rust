use std::sync::OnceLock;

use dashmap::DashMap;

use crate::{Score, board::TilePointer, player::Player, state::State};

/// Zobrist keys — 19×19 board, 2 players + side to move.
/// Deterministic splitmix64 from a fixed seed.
static ZOBRIST: OnceLock<Vec<u64>> = OnceLock::new();

fn init_zobrist() -> Vec<u64> {
  let mut keys = Vec::with_capacity(19 * 19 * 2 + 2);
  let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
  for _ in 0..(19 * 19 * 2 + 2) {
    seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    keys.push(z);
  }
  keys
}

/// Zobrist key for a stone at `(x, y)` for `player`.
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

/// Zobrist key for side to move.
#[inline]
pub fn zobrist_side(player: Player) -> u64 {
  let keys = ZOBRIST.get_or_init(init_zobrist);
  // last two keys for side to move
  match player {
    Player::X => keys[19 * 19 * 2],
    Player::O => keys[19 * 19 * 2 + 1],
  }
}

/// Bound type for a transposition-table entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
  /// Exact score (PV node).
  Exact,
  /// Lower bound (fail-high).
  Lower,
  /// Upper bound (fail-low).
  Upper,
}

/// Entry stored in the transposition table.
#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
  /// Stored score.
  pub score: Score,
  /// Depth at which the score was computed.
  pub depth: u8,
  /// Bound type.
  pub bound: Bound,
  /// Best move from this position.
  pub best_move: Option<TilePointer>,
  /// Terminal state.
  pub state: State,
}

/// Concurrent transposition table.
pub struct TranspositionTable {
  table: DashMap<u64, TTEntry>,
}

impl TranspositionTable {
  /// Create a new empty table.
  pub fn new() -> Self {
    Self {
      table: DashMap::with_capacity(1 << 16),
    }
  }

  /// Probe for an entry usable within `[alpha, beta]` at `depth`.
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

  /// Store an entry, keeping the deeper one on collision.
  pub fn store(&self, hash: u64, entry: TTEntry) {
    // Keep deeper entries.
    if let Some(old) = self.table.get(&hash)
      && old.depth > entry.depth
    {
      return;
    }
    self.table.insert(hash, entry);
  }

  /// Number of entries.
  pub fn len(&self) -> usize {
    self.table.len()
  }

  /// Whether the table is empty.
  pub fn is_empty(&self) -> bool {
    self.table.is_empty()
  }

  /// Clear all entries.
  pub fn clear(&self) {
    self.table.clear();
  }

  /// Keep only terminal entries.
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
