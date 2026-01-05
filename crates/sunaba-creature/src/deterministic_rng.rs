//! Deterministic RNG for creature brain initialization
//!
//! Simple xorshift64 implementation that doesn't depend on external crates.
//! Used for deterministic weight initialization from genome seeds.

/// Simple deterministic xorshift64 RNG
/// https://en.wikipedia.org/wiki/Xorshift
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    /// Create from seed
    pub fn from_seed(seed: u64) -> Self {
        // Ensure seed is never zero (xorshift requirement)
        let state = if seed == 0 { 1 } else { seed };
        Self { state }
    }

    /// Generate next u64
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Generate f32 in [0, 1)
    pub fn gen_f32(&mut self) -> f32 {
        // Use upper 24 bits for precision
        let val = (self.next_u64() >> 40) as u32;
        (val as f32) / (1u32 << 24) as f32
    }

    /// Generate f32 in range [min, max)
    pub fn gen_range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.gen_f32() * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut rng1 = DeterministicRng::from_seed(42);
        let mut rng2 = DeterministicRng::from_seed(42);

        for _ in 0..100 {
            assert_eq!(rng1.gen_f32(), rng2.gen_f32());
        }
    }

    #[test]
    fn test_zero_seed() {
        let mut rng = DeterministicRng::from_seed(0);
        let val = rng.gen_f32();
        assert!((0.0..1.0).contains(&val));
    }

    #[test]
    fn test_range() {
        let mut rng = DeterministicRng::from_seed(123);
        for _ in 0..100 {
            let val = rng.gen_range_f32(-1.0, 1.0);
            assert!((-1.0..1.0).contains(&val));
        }
    }

    #[test]
    fn test_different_seeds() {
        let mut rng1 = DeterministicRng::from_seed(12345);
        let mut rng2 = DeterministicRng::from_seed(67890);

        // Check multiple values to ensure different sequences
        let mut found_difference = false;
        for _ in 0..10 {
            let val1 = rng1.gen_f32();
            let val2 = rng2.gen_f32();
            if (val1 - val2).abs() > 1e-6 {
                found_difference = true;
                break;
            }
        }
        assert!(
            found_difference,
            "Different seeds should produce different random sequences"
        );
    }
}
