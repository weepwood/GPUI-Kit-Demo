//! XorShift128 generator, API-compatible with `rand_xorshift` 0.5 for
//! `rand_core` 0.10.
//!
//! This crate exists because `proptest 1.12.0` (pulled in by `gpui`'s
//! `test-support` feature) requires `rand_xorshift ^0.5`, which is not
//! published on crates.io yet. The repository patches it in so the dependency
//! graph resolves. Delete this crate and the `[patch.crates-io]` entry once
//! `rand_xorshift 0.5` is available.

use rand_core::{Infallible, Rng, SeedableRng, TryRng};

/// Seed for [`XorShiftRng`], matching `rand_xorshift` 0.5.
pub type XorShiftSeed = [u8; 16];

const BAD_SEED: u32 = 0xBAD5_EED;

/// A `xorshift128` generator: two 128-bit rounds of shifts and xors, the same
/// generator `rand_xorshift` has always shipped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XorShiftRng {
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

impl XorShiftRng {
    fn from_state(x: u32, y: u32, z: u32, w: u32) -> Self {
        // An all-zero state would be a fixed point, so reject it the way the
        // upstream crate does.
        if x == 0 && y == 0 && z == 0 && w == 0 {
            return Self {
                x: BAD_SEED,
                y: BAD_SEED,
                z: BAD_SEED,
                w: BAD_SEED,
            };
        }
        Self { x, y, z, w }
    }

    fn step(&mut self) -> u32 {
        let t = self.x ^ (self.x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        self.w = self.w ^ (self.w >> 19) ^ (t ^ (t >> 8));
        self.w
    }
}

impl SeedableRng for XorShiftRng {
    type Seed = XorShiftSeed;

    fn from_seed(seed: XorShiftSeed) -> Self {
        let words = |at: usize| {
            u32::from_le_bytes([seed[at], seed[at + 1], seed[at + 2], seed[at + 3]])
        };
        let mut rng = Self::from_state(words(0), words(4), words(8), words(12));
        // Discard the first outputs so the state is well mixed.
        for _ in 0..16 {
            rng.step();
        }
        rng
    }
}

impl TryRng for XorShiftRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.step())
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        let low = self.step() as u64;
        let high = self.step() as u64;
        Ok((high << 32) | low)
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        for chunk in dst.chunks_mut(4) {
            let word = self.step().to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
        Ok(())
    }
}

// `Rng` is a blanket implementation for infallible generators.
#[allow(dead_code)]
fn assert_rng_traits() {
    fn check<R: Rng>() {}
    check::<XorShiftRng>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_are_reproducible() {
        let seed = [1u8; 16];
        let mut a = XorShiftRng::from_seed(seed);
        let mut b = XorShiftRng::from_seed(seed);
        for _ in 0..8 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = XorShiftRng::from_seed([1u8; 16]);
        let mut b = XorShiftRng::from_seed([2u8; 16]);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn zero_seed_is_replaced() {
        let mut rng = XorShiftRng::from_seed([0u8; 16]);
        assert_ne!(rng.next_u32(), 0);
    }

    #[test]
    fn fills_arbitrary_lengths() {
        let mut rng = XorShiftRng::from_seed([7u8; 16]);
        let mut buffer = [0u8; 13];
        rng.fill_bytes(&mut buffer);
        assert!(buffer.iter().any(|&byte| byte != 0));
    }
}
