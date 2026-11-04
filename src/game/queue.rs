//! The Queue minigame, GDD 5.4. One `step` is one 10 minute tick.

use super::rng::Rng;

pub const TICK_MINUTES: u32 = 10;
/// Chance the guard sees your first cut. Each cut after that is likelier.
const CATCH_BASE: u32 = 15;
const CATCH_STEP: u32 = 25;
/// Chance you win the argument over a number the machine printed twice.
const DOUBLE_WIN: u32 = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    FixerOffer {
        price: i64,
    },
    InsideMan,
    Auntie,
    /// The machine printed your number twice. Someone else holds it too.
    DoubleNumber,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Wait,
    Confront,
    Cut,
    PayFixer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Waiting,
    Served,
    /// A fight, or the guard caught you cutting. Thrown out for the day.
    ThrownOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub outcome: Outcome,
    pub money: i64,
    pub conscience: i32,
    /// Dialog key for what happened.
    pub message: &'static str,
}

#[derive(Clone, Debug)]
pub struct Queue {
    pub ahead: u32,
    pub serve_every: u32,
    pub pending: Option<Event>,
    cuts: u32,
    ticks: u32,
    rng: Rng,
}

impl Queue {
    pub fn new(mut rng: Rng) -> Self {
        let ahead = rng.range(6, 15);
        let serve_every = rng.range(1, 3);
        Self {
            ahead,
            serve_every,
            pending: None,
            cuts: 0,
            ticks: 0,
            rng,
        }
    }

    pub fn fixer_price(&self) -> Option<i64> {
        match self.pending {
            Some(Event::FixerOffer { price }) => Some(price),
            _ => None,
        }
    }

    pub fn can(&self, action: Action) -> bool {
        match action {
            Action::Wait | Action::Cut => true,
            Action::Confront => matches!(
                self.pending,
                Some(Event::Auntie | Event::InsideMan | Event::DoubleNumber)
            ),
            Action::PayFixer => self.fixer_price().is_some(),
        }
    }

    pub fn step(&mut self, action: Action) -> Step {
        let mut step = Step {
            outcome: Outcome::Waiting,
            money: 0,
            conscience: 0,
            message: "",
        };
        if !self.can(action) {
            step.message = "q_nothing";
            return step;
        }

        match (action, self.pending.take()) {
            (Action::Wait, Some(Event::Auntie)) => {
                self.ahead += 1;
                step.message = "q_auntie_wait";
            }
            (Action::Wait, Some(Event::InsideMan)) => {
                self.ahead += 1;
                step.message = "q_inside_wait";
            }
            (Action::Wait, Some(Event::FixerOffer { .. })) => {
                step.message = "q_fixer_ignored";
            }
            (Action::Wait, Some(Event::DoubleNumber)) => {
                self.ahead += 1;
                step.message = "q_double_wait";
            }
            (Action::Wait, None) => step.message = "q_wait",
            (Action::Confront, Some(Event::DoubleNumber)) => {
                if self.rng.chance(DOUBLE_WIN) {
                    step.message = "q_double_won";
                } else {
                    self.ahead += 2;
                    step.message = "q_double_lost";
                }
            }
            (Action::Confront, _) => {
                let roll = self.rng.range(0, 100);
                if roll < 50 {
                    step.message = "q_confront_ok";
                } else if roll < 70 {
                    step.outcome = Outcome::ThrownOut;
                    step.message = "q_confront_fight";
                    return step;
                } else {
                    self.ahead += 1;
                    step.message = "q_confront_fail";
                }
            }
            (Action::Cut, _) => {
                step.conscience = 10;
                let catch = (CATCH_BASE + CATCH_STEP * self.cuts).min(90);
                self.cuts += 1;
                if self.rng.chance(catch) {
                    step.outcome = Outcome::ThrownOut;
                    step.message = "q_cut_caught";
                    return step;
                } else {
                    let skip = self.rng.range(3, 7);
                    self.ahead = self.ahead.saturating_sub(skip);
                    step.message = "q_cut_ok";
                }
            }
            (Action::PayFixer, Some(Event::FixerOffer { price })) => {
                step.money = price;
                step.conscience = 15;
                self.ahead = 0;
                step.message = "q_fixer_paid";
            }
            (Action::PayFixer, _) => unreachable!("guarded by can()"),
        }

        self.ticks += 1;
        if self.ticks.is_multiple_of(self.serve_every) {
            self.ahead = self.ahead.saturating_sub(1);
        }
        if self.ahead == 0 {
            step.outcome = Outcome::Served;
            return step;
        }

        self.pending = match self.rng.range(0, 100) {
            0..=14 => Some(Event::Auntie),
            15..=22 => Some(Event::InsideMan),
            23..=37 => Some(Event::FixerOffer {
                price: i64::from(self.rng.range(1, 4)) * 50,
            }),
            38..=45 => Some(Event::DoubleNumber),
            _ => None,
        };
        step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play(seed: u64, action: Action) -> (Vec<Step>, u32) {
        let mut q = Queue::new(Rng::derive(seed, "queue:test"));
        let mut steps = Vec::new();
        for _ in 0..200 {
            let a = if q.can(action) { action } else { Action::Wait };
            let s = q.step(a);
            let done = s.outcome != Outcome::Waiting;
            steps.push(s);
            if done {
                break;
            }
        }
        (steps, q.ahead)
    }

    #[test]
    fn a_double_number_costs_a_place_or_an_argument() {
        let mut lost = 0;
        let mut won = 0;
        for seed in 0..200 {
            let mut q = Queue::new(Rng::derive(seed, "queue:double"));
            q.ahead = 20;
            q.pending = Some(Event::DoubleNumber);
            let before = q.ahead;
            let s = q.step(Action::Confront);
            match s.message {
                "q_double_won" => won += 1,
                "q_double_lost" => lost += 1,
                m => panic!("{m}"),
            }
            assert!(q.ahead <= before + 2);
            q.pending = Some(Event::DoubleNumber);
            let before = q.ahead;
            assert_eq!(q.step(Action::Wait).message, "q_double_wait");
            assert!(q.ahead >= before);
        }
        assert!(won > 50 && lost > 50, "{won} {lost}");
    }

    #[test]
    fn same_seed_same_queue() {
        assert_eq!(play(5, Action::Wait), play(5, Action::Wait));
        assert_ne!(play(5, Action::Wait).0, play(6, Action::Wait).0);
    }

    #[test]
    fn waiting_eventually_gets_served() {
        for seed in 0..50 {
            let (steps, ahead) = play(seed, Action::Wait);
            assert_eq!(
                steps.last().unwrap().outcome,
                Outcome::Served,
                "seed {seed}"
            );
            assert_eq!(ahead, 0);
        }
    }

    #[test]
    fn cutting_costs_conscience_every_time() {
        let (steps, _) = play(3, Action::Cut);
        assert!(steps
            .iter()
            .all(|s| s.conscience == 10 || s.outcome == Outcome::Served));
        assert!(steps.len() < play(3, Action::Wait).0.len());
    }

    #[test]
    fn the_guard_catches_repeat_cutters() {
        let caught_on = |seed| {
            let mut q = Queue::new(Rng::derive(seed, "queue:guard"));
            q.ahead = 1000;
            (1..=20).find(|_| q.step(Action::Cut).outcome == Outcome::ThrownOut)
        };
        let first_try = (0..200).filter(|&s| caught_on(s) == Some(1)).count();
        assert!((10..60).contains(&first_try), "{first_try}");
        assert!((0..200).all(|s| caught_on(s).is_some_and(|n| n <= 8)));
    }

    #[test]
    fn fixer_takes_money_and_serves_at_once() {
        let mut q = Queue::new(Rng::new(1));
        q.pending = Some(Event::FixerOffer { price: 100 });
        let s = q.step(Action::PayFixer);
        assert_eq!(s.outcome, Outcome::Served);
        assert_eq!((s.money, s.conscience), (100, 15));
        assert!(!q.can(Action::PayFixer));
    }
}
