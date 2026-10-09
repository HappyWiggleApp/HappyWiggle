//! Small deterministic random generator (SplitMix64).
//!
//! We do not need cryptographic randomness, only "not robotic" timing, and a
//! seedable generator keeps the tests reproducible.

#[derive(Clone, Debug)]
pub struct Rng(u64);

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix(self.0)
    }

    /// Uniform in `[0, 1)`.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `[lo, hi]` (order of the arguments does not matter).
    pub fn range_f64(&mut self, a: f64, b: f64) -> f64 {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        lo + (hi - lo) * self.unit()
    }

    /// Uniform integer in `[lo, hi]` inclusive.
    pub fn range_i64(&mut self, a: i64, b: i64) -> i64 {
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let span = (hi - lo) as u64 + 1;
        lo + (self.next_u64() % span) as i64
    }
}

/// Stable hash of a few numbers, used to derive per-day schedule jitter so the
/// plan for a given day does not change when the app restarts.
pub fn hash64(parts: &[u64]) -> u64 {
    let mut h = 0x243F_6A88_85A3_08D3u64;
    for p in parts {
        h = mix(h ^ p.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    }
    h
}
