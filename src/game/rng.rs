//! SplitMix64 with tag-derived child streams, so a route change never shifts
//! the roll another office gets on the same day.

#[derive(Clone, Debug)]
pub struct Rng(u64);

const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ023456789";

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn fnv1a(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// A child stream for one purpose, for example `"queue:dukcatan:3"`.
    pub fn derive(seed: u64, tag: &str) -> Self {
        Self::new(mix(seed ^ fnv1a(tag)))
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        mix(self.0)
    }

    /// Uniform in `lo..hi`. `hi` must be greater than `lo`.
    pub fn range(&mut self, lo: u32, hi: u32) -> u32 {
        lo + (self.next_u64() % u64::from(hi - lo)) as u32
    }

    /// True with the given percent probability.
    pub fn chance(&mut self, percent: u32) -> bool {
        self.range(0, 100) < percent
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.range(0, items.len() as u32) as usize]
    }

    /// Fisher-Yates.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.range(0, i as u32 + 1) as usize);
        }
    }
}

/// Six character share code from the low 30 bits of a seed.
pub fn seed_to_code(seed: u64) -> String {
    (0..6)
        .map(|i| ALPHABET[((seed >> (5 * i)) & 31) as usize] as char)
        .collect()
}

/// Inverse of `seed_to_code`. Case insensitive, None on any other character.
pub fn code_to_seed(code: &str) -> Option<u64> {
    if code.len() != 6 {
        return None;
    }
    code.bytes().enumerate().try_fold(0u64, |acc, (i, b)| {
        let v = ALPHABET.iter().position(|&a| a == b.to_ascii_uppercase())?;
        Some(acc | (v as u64) << (5 * i))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn derived_streams_differ_by_tag_only() {
        let a = Rng::derive(7, "queue:dukcatan:1").next_u64();
        let b = Rng::derive(7, "queue:dukcatan:1").next_u64();
        let c = Rng::derive(7, "queue:dukcatan:2").next_u64();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn range_stays_in_bounds_and_covers() {
        let mut r = Rng::new(1);
        let mut seen = [false; 6];
        for _ in 0..1000 {
            let v = r.range(3, 9);
            assert!((3..9).contains(&v));
            seen[(v - 3) as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn chance_is_roughly_calibrated() {
        let mut r = Rng::new(99);
        let hits = (0..10_000).filter(|_| r.chance(25)).count();
        assert!((2300..2700).contains(&hits), "hits = {hits}");
    }

    #[test]
    fn seed_code_is_six_chars_and_distinct() {
        assert_eq!(seed_to_code(0).len(), 6);
        assert_eq!(ALPHABET.len(), 32);
        assert_eq!(seed_to_code(u64::MAX), "999999");
        assert_ne!(seed_to_code(1), seed_to_code(2));
        assert_eq!(seed_to_code(5), seed_to_code(5 | (1 << 40)));
    }

    #[test]
    fn seed_code_round_trips() {
        for seed in [0, 1, 31, 12345, (1 << 30) - 1] {
            assert_eq!(code_to_seed(&seed_to_code(seed)), Some(seed));
        }
        assert_eq!(code_to_seed("k7q2ma"), code_to_seed("K7Q2MA"));
        assert_eq!(code_to_seed("K7Q2M"), None);
        assert_eq!(code_to_seed("K7Q2MI"), None);
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut v: Vec<u32> = (0..20).collect();
        Rng::new(3).shuffle(&mut v);
        assert_ne!(v, (0..20).collect::<Vec<_>>());
        v.sort();
        assert_eq!(v, (0..20).collect::<Vec<_>>());
    }
}
