//! Stamp Hunt, GDD 5.2. One clerk holds the stamp. The others point at a desk,
//! and 30 percent of them point at the wrong one.

use super::rng::Rng;

pub const VISIT_MINUTES: u32 = 10;
/// The hunt fails once the clock reaches this time.
pub const GIVE_UP_AT: u32 = 14 * 60 + 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holder {
    Present,
    OnBreak,
    InMeeting,
    FieldDuty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visit {
    /// "Try desk N." Desk numbers are 0-based here, 1-based on screen.
    Clue(usize),
    Stamped,
    /// Holder found but away, back in this many minutes.
    Away(Holder, u32),
    /// Holder is out on field duty. The visit is over.
    Gone,
}

#[derive(Clone, Debug)]
pub struct StampHunt {
    pub desks: usize,
    pub visited: Vec<bool>,
    holder: usize,
    status: Holder,
    back_in: u32,
    clues: Vec<usize>,
}

impl StampHunt {
    pub fn new(rng: &mut Rng) -> Self {
        let desks = rng.range(6, 11) as usize;
        let holder = rng.range(0, desks as u32) as usize;
        let status = match rng.range(0, 100) {
            0..=34 => Holder::Present,
            35..=59 => Holder::OnBreak,
            60..=79 => Holder::InMeeting,
            _ => Holder::FieldDuty,
        };
        let back_in = rng.range(3, 7) * 10;
        let clues = (0..desks)
            .map(|d| {
                if rng.chance(30) {
                    let liars: Vec<usize> = (0..desks).filter(|&x| x != d && x != holder).collect();
                    *rng.pick(&liars)
                } else {
                    holder
                }
            })
            .collect();
        Self {
            desks,
            visited: vec![false; desks],
            holder,
            status,
            back_in,
            clues,
        }
    }

    pub fn visit(&mut self, desk: usize) -> Visit {
        self.visited[desk] = true;
        if desk != self.holder {
            return Visit::Clue(self.clues[desk]);
        }
        match self.status {
            Holder::Present => Visit::Stamped,
            Holder::FieldDuty => Visit::Gone,
            away => Visit::Away(away, self.back_in),
        }
    }

    /// Wait at the holder's desk until they return. Returns the minutes waited.
    pub fn wait(&mut self) -> u32 {
        self.status = Holder::Present;
        std::mem::take(&mut self.back_in)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_holder(h: &mut StampHunt) -> (usize, Visit) {
        (0..h.desks)
            .map(|d| (d, h.visit(d)))
            .find(|(_, v)| !matches!(v, Visit::Clue(_)))
            .unwrap()
    }

    #[test]
    fn clues_are_right_about_seventy_percent() {
        let (mut right, mut total) = (0, 0);
        for seed in 0..500 {
            let mut h = StampHunt::new(&mut Rng::new(seed));
            let (holder, _) = find_holder(&mut h.clone());
            for d in (0..h.desks).filter(|&d| d != holder) {
                total += 1;
                if h.visit(d) == Visit::Clue(holder) {
                    right += 1;
                }
            }
        }
        let rate = right as f64 / total as f64;
        assert!((0.66..0.74).contains(&rate), "rate {rate}");
    }

    #[test]
    fn waiting_for_an_away_holder_gets_the_stamp() {
        let mut saw_away = false;
        for seed in 0..50 {
            let mut h = StampHunt::new(&mut Rng::new(seed));
            let (holder, v) = find_holder(&mut h);
            if let Visit::Away(_, mins) = v {
                saw_away = true;
                assert!((30..=60).contains(&mins));
                assert_eq!(h.wait(), mins);
                assert_eq!(h.visit(holder), Visit::Stamped);
            }
        }
        assert!(saw_away);
    }
}
