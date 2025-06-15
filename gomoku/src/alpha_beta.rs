use std::{
  ops,
  sync::atomic::{self, AtomicI32},
};

use crate::Score;

pub type AlphaBeta = AlphaBetaImpl<Score>;
pub type AtomicAlphaBeta = AlphaBetaImpl<AtomicI32>;

#[derive(Debug, Clone, Copy)]
pub struct AlphaBetaImpl<T> {
  pub alpha: T,
  pub beta: T,
}

impl<T> AlphaBetaImpl<T> {
  pub fn new(alpha: T, beta: T) -> Self {
    Self { alpha, beta }
  }
}

impl AlphaBeta {
  pub fn update(&mut self, score: Score) {
    self.alpha = self.alpha.max(score);
  }

  pub fn should_prune(self, score: Score) -> bool {
    score >= self.beta
  }
}

impl AtomicAlphaBeta {
  pub fn load(&self, ordering: atomic::Ordering) -> AlphaBeta {
    AlphaBeta::new(self.alpha.load(ordering), self.beta.load(ordering))
  }
}

impl<T: From<Score>> Default for AlphaBetaImpl<T> {
  fn default() -> Self {
    Self::new((-Score::MAX).into(), Score::MAX.into())
  }
}

impl<T: ops::Neg<Output = T>> ops::Neg for AlphaBetaImpl<T> {
  type Output = Self;
  fn neg(self) -> Self::Output {
    AlphaBetaImpl::new(-self.beta, -self.alpha)
  }
}
