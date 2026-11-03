//! One attempt at one quest. Owns the meters, the clock, and the inventory.

use super::clock::Clock;
use super::grade::Par;
use super::meters::Meters;
use super::quest::Inventory;
use super::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Complete,
    Failed,
}

#[derive(Clone, Debug)]
pub struct Run {
    pub seed: u64,
    pub goal_doc: String,
    pub par: Par,
    pub meters: Meters,
    pub clock: Clock,
    pub inv: Inventory,
}

impl Run {
    pub fn new(seed: u64, goal_doc: &str, par: Par) -> Self {
        Self {
            seed,
            goal_doc: goal_doc.to_owned(),
            par,
            meters: Meters::new(par.budget),
            clock: Clock::default(),
            inv: Inventory::default(),
        }
    }

    pub fn rng(&self, tag: &str) -> Rng {
        Rng::derive(self.seed, &format!("{tag}:{}", self.meters.days))
    }

    /// Advance the clock. Returns true when the day ended.
    pub fn advance(&mut self, minutes: u32) -> bool {
        self.clock.advance(minutes)
    }

    pub fn end_day(&mut self) {
        self.meters.next_day();
        self.clock.reset();
    }

    pub fn status(&self) -> Status {
        if self.inv.has(&self.goal_doc) {
            Status::Complete
        } else if self.meters.days > self.par.deadline {
            Status::Failed
        } else {
            Status::Playing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> Run {
        Run::new(
            1,
            "ktpss",
            Par {
                par_days: 2,
                deadline: 3,
                budget: 100,
                official_fees: 0,
            },
        )
    }

    #[test]
    fn completes_on_goal_doc() {
        let mut r = run();
        assert_eq!(r.status(), Status::Playing);
        r.inv.add("ktpss");
        assert_eq!(r.status(), Status::Complete);
    }

    #[test]
    fn fails_past_deadline() {
        let mut r = run();
        for _ in 0..2 {
            r.end_day();
        }
        assert_eq!(r.status(), Status::Playing);
        r.end_day();
        assert_eq!(r.status(), Status::Failed);
    }

    #[test]
    fn rng_tag_includes_day() {
        let mut r = run();
        let a = r.rng("queue:dukcatan").next_u64();
        let b = r.rng("queue:dukcatan").next_u64();
        r.end_day();
        let c = r.rng("queue:dukcatan").next_u64();
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
