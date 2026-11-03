//! System Down, GDD 5.5. The system comes back at a hidden time. Only Mas
//! Honorer knows when, and he takes coffee.

use super::rng::Rng;

pub const STEP_MINUTES: u32 = 10;
pub const ANNOYED_MINUTES: u32 = 20;
pub const COFFEE: i64 = 15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Wait,
    Refresh,
    AskHonorer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tick {
    pub minutes: u32,
    pub money: i64,
    /// Dialog key for what happened.
    pub key: &'static str,
}

#[derive(Clone, Debug)]
pub struct SystemDown {
    pub elapsed: u32,
    pub revealed: bool,
    back_at: u32,
    streak: u32,
}

impl SystemDown {
    pub fn new(rng: &mut Rng) -> Self {
        Self {
            elapsed: 0,
            revealed: false,
            back_at: rng.range(3, 13) * STEP_MINUTES,
            streak: 0,
        }
    }

    pub fn act(&mut self, act: Act) -> Tick {
        let tick = match act {
            Act::Wait => {
                self.streak = 0;
                Tick {
                    minutes: STEP_MINUTES,
                    money: 0,
                    key: "o_wait",
                }
            }
            Act::Refresh => {
                self.streak += 1;
                if self.streak >= 3 {
                    self.streak = 0;
                    Tick {
                        minutes: STEP_MINUTES + ANNOYED_MINUTES,
                        money: 0,
                        key: "o_annoyed",
                    }
                } else {
                    Tick {
                        minutes: STEP_MINUTES,
                        money: 0,
                        key: "o_refresh",
                    }
                }
            }
            Act::AskHonorer => {
                self.revealed = true;
                Tick {
                    minutes: 0,
                    money: COFFEE,
                    key: "o_honorer",
                }
            }
        };
        self.elapsed += tick.minutes;
        tick
    }

    pub fn done(&self) -> bool {
        self.elapsed >= self.back_at
    }

    pub fn remaining(&self) -> u32 {
        self.back_at.saturating_sub(self.elapsed)
    }

    /// What the progress bar claims. It sticks at familiar numbers.
    pub fn shown_percent(&self) -> u32 {
        if self.done() {
            return 100;
        }
        let real = self.elapsed * 100 / self.back_at;
        [0, 3, 12, 47, 48, 99]
            .into_iter()
            .rev()
            .find(|&p| p <= real)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waiting_brings_it_back_within_two_hours() {
        for seed in 0..100 {
            let mut s = SystemDown::new(&mut Rng::new(seed));
            let mut waits = 0;
            while !s.done() {
                s.act(Act::Wait);
                waits += 1;
            }
            assert!((3..=12).contains(&waits), "seed {seed}: {waits}");
            assert_eq!(s.shown_percent(), 100);
        }
    }

    #[test]
    fn third_refresh_in_a_row_annoys_the_clerk() {
        let mut s = SystemDown::new(&mut Rng::new(1));
        let keys: Vec<_> = (0..3).map(|_| s.act(Act::Refresh).key).collect();
        assert_eq!(keys, ["o_refresh", "o_refresh", "o_annoyed"]);
        assert_eq!(s.elapsed, 50);
    }

    #[test]
    fn honorer_costs_coffee_not_time() {
        let mut s = SystemDown::new(&mut Rng::new(2));
        let before = s.remaining();
        let t = s.act(Act::AskHonorer);
        assert_eq!((t.minutes, t.money), (0, COFFEE));
        assert!(s.revealed);
        assert_eq!(s.remaining(), before);
    }

    #[test]
    fn progress_bar_lies_but_never_says_done_early() {
        let mut s = SystemDown::new(&mut Rng::new(3));
        let mut last = 0;
        while !s.done() {
            let p = s.shown_percent();
            assert!(p < 100 && p >= last);
            last = p;
            s.act(Act::Wait);
        }
    }
}
