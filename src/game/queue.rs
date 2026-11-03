//! The Queue minigame, GDD 5.4. One `step` is one 10 minute tick.

use super::rng::Rng;

pub const TICK_MINUTES: u32 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    FixerOffer { price: i64 },
    InsideMan,
    Auntie,
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
    /// A confrontation turned into a fight. The guard throws you out for the day.
    Fight,
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
    pub start_ahead: u32,
    pub serve_every: u32,
    pub pending: Option<Event>,
    ticks: u32,
    rng: Rng,
}

impl Queue {
    pub fn new(mut rng: Rng) -> Self {
        let ahead = rng.range(6, 15);
        let serve_every = rng.range(1, 3);
        Self {
            ahead,
            start_ahead: ahead,
            serve_every,
            pending: None,
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
            Action::Confront => matches!(self.pending, Some(Event::Auntie | Event::InsideMan)),
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
            (Action::Wait, None) => step.message = "q_wait",
            (Action::Confront, _) => {
                let roll = self.rng.range(0, 100);
                if roll < 50 {
                    step.message = "q_confront_ok";
                } else if roll < 70 {
                    step.outcome = Outcome::Fight;
                    step.message = "q_confront_fight";
                    return step;
                } else {
                    self.ahead += 1;
                    step.message = "q_confront_fail";
                }
            }
            (Action::Cut, _) => {
                step.conscience = 10;
                if self.rng.chance(15) {
                    self.ahead = self.start_ahead;
                    step.message = "q_cut_caught";
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
    fn fixer_takes_money_and_serves_at_once() {
        let mut q = Queue::new(Rng::new(1));
        q.pending = Some(Event::FixerOffer { price: 100 });
        let s = q.step(Action::PayFixer);
        assert_eq!(s.outcome, Outcome::Served);
        assert_eq!((s.money, s.conscience), (100, 15));
        assert!(!q.can(Action::PayFixer));
    }
}
