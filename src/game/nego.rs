//! Negotiation, GDD 5.6. Someone hints at a fee. The game never comments.

use super::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    Clerk,
    Fixer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Pay,
    Decline,
    /// Clerk only.
    Report,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Paid,
    Declined,
    /// The report worked. A new clerk processes the file on the spot.
    Replaced,
    /// The report backfired. Your file is frozen for `REVIEW_DAYS`.
    UnderReview,
}

pub const REVIEW_DAYS: u32 = 2;

#[derive(Clone, Debug)]
pub struct Nego {
    pub who: Who,
    pub price: i64,
    rng: Rng,
}

impl Nego {
    pub fn new(mut rng: Rng, who: Who) -> Self {
        let (lo, hi) = match who {
            Who::Clerk => (10, 31),
            Who::Fixer => (5, 16),
        };
        let price = i64::from(rng.range(lo, hi)) * 10;
        Self { who, price, rng }
    }

    /// Conscience cost of paying, from GDD section 3.
    pub fn conscience(&self) -> i32 {
        match self.who {
            Who::Clerk => 20,
            Who::Fixer => 15,
        }
    }

    pub fn can(&self, choice: Choice) -> bool {
        choice != Choice::Report || self.who == Who::Clerk
    }

    pub fn resolve(&mut self, choice: Choice) -> Outcome {
        match choice {
            Choice::Pay => Outcome::Paid,
            Choice::Decline => Outcome::Declined,
            Choice::Report if self.rng.chance(10) => Outcome::Replaced,
            Choice::Report => Outcome::UnderReview,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prices_follow_the_gdd_ranges() {
        for seed in 0..200 {
            let c = Nego::new(Rng::new(seed), Who::Clerk).price;
            let f = Nego::new(Rng::new(seed), Who::Fixer).price;
            assert!((100..=300).contains(&c) && c % 10 == 0);
            assert!((50..=150).contains(&f));
        }
    }

    #[test]
    fn reporting_rarely_works() {
        let wins = (0..2000)
            .filter(|&s| {
                Nego::new(Rng::new(s), Who::Clerk).resolve(Choice::Report) == Outcome::Replaced
            })
            .count();
        assert!((140..260).contains(&wins), "wins {wins}");
    }

    #[test]
    fn nobody_reports_a_fixer() {
        assert!(!Nego::new(Rng::new(1), Who::Fixer).can(Choice::Report));
        assert!(Nego::new(Rng::new(1), Who::Clerk).can(Choice::Report));
    }
}
