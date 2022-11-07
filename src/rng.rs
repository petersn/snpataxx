use std::cell::Cell;

pub const RNG_MULT: u64 = 0x243f6a8885a308d3;

pub struct Rng {
  state: Cell<u64>,
}

impl Rng {
  pub fn new(seed: u64) -> Rng {
    Rng {
      state: Cell::new(seed),
    }
  }

  /// Generate a uniformly random u64.
  #[inline]
  pub fn next_random(&self) -> u64 {
    let state = self.state.get().wrapping_add(1);
    self.state.set(state);
    let mut x = state.wrapping_mul(RNG_MULT);
    for _ in 0..3 {
      x ^= x >> 37;
      x = x.wrapping_mul(RNG_MULT);
    }
    x
  }

  /// Generate an approximately uniformly random u32 in the range [0, max).
  #[inline]
  pub fn generate_range(&self, max: u32) -> u32 {
    // I don't care about the at most part per billion bias here.
    (self.next_random() % max as u64) as u32
  }
}
