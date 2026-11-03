//! Photocopy Run, GDD 5.3. Remember the order, buy it across the road, bring it back.

use super::rng::Rng;

pub const BRIEF_SECONDS: f32 = 4.0;
/// Across the road and back.
pub const RUN_MINUTES: u32 = 30;
pub const MAX_COPIES: u32 = 5;
pub const DUTY_STAMPS: [i64; 3] = [0, 6, 10];
pub const FOLDERS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    Copies,
    DutyStamp,
    Folder,
    Certified,
}

pub const ITEMS: [Item; 4] = [Item::Copies, Item::DutyStamp, Item::Folder, Item::Certified];

/// `stamp` indexes `DUTY_STAMPS`, `folder` indexes the folder colours in strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Order {
    pub copies: u32,
    pub stamp: usize,
    pub folder: usize,
    pub certified: bool,
}

impl Default for Order {
    fn default() -> Self {
        Self {
            copies: 1,
            stamp: 0,
            folder: 0,
            certified: false,
        }
    }
}

impl Order {
    pub fn random(rng: &mut Rng) -> Self {
        Self {
            copies: rng.range(1, MAX_COPIES + 1),
            stamp: rng.range(0, DUTY_STAMPS.len() as u32) as usize,
            folder: rng.range(0, FOLDERS as u32) as usize,
            certified: rng.chance(50),
        }
    }

    /// Kv paid at the stall.
    pub fn cost(&self) -> i64 {
        2 * i64::from(self.copies)
            + DUTY_STAMPS[self.stamp]
            + 5
            + if self.certified { 10 } else { 0 }
    }

    /// The first item the clerk will complain about.
    pub fn first_mismatch(&self, brought: &Order) -> Option<Item> {
        ITEMS
            .into_iter()
            .find(|&item| self.get(item) != brought.get(item))
    }

    pub fn get(&self, item: Item) -> usize {
        match item {
            Item::Copies => self.copies as usize,
            Item::DutyStamp => self.stamp,
            Item::Folder => self.folder,
            Item::Certified => usize::from(self.certified),
        }
    }

    /// Step one item up or down, wrapping within its range.
    pub fn bump(&mut self, item: Item, up: bool) {
        let step = |v: usize, lo: usize, n: usize| {
            let i = v - lo;
            lo + if up { (i + 1) % n } else { (i + n - 1) % n }
        };
        match item {
            Item::Copies => self.copies = step(self.copies as usize, 1, MAX_COPIES as usize) as u32,
            Item::DutyStamp => self.stamp = step(self.stamp, 0, DUTY_STAMPS.len()),
            Item::Folder => self.folder = step(self.folder, 0, FOLDERS),
            Item::Certified => self.certified = !self.certified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_adds_every_item() {
        let o = Order {
            copies: 3,
            stamp: 2,
            folder: 1,
            certified: true,
        };
        assert_eq!(o.cost(), 6 + 10 + 5 + 10);
        assert_eq!(Order::default().cost(), 7);
    }

    #[test]
    fn mismatch_reports_first_wrong_item() {
        let want = Order {
            copies: 3,
            stamp: 1,
            folder: 2,
            certified: false,
        };
        assert_eq!(want.first_mismatch(&want), None);
        let mut got = want;
        got.folder = 0;
        assert_eq!(want.first_mismatch(&got), Some(Item::Folder));
        got.copies = 1;
        assert_eq!(want.first_mismatch(&got), Some(Item::Copies));
    }

    #[test]
    fn bump_wraps_and_reaches_every_order() {
        let mut o = Order::default();
        o.bump(Item::Copies, false);
        assert_eq!(o.copies, MAX_COPIES);
        o.bump(Item::Copies, true);
        assert_eq!(o.copies, 1);
        let target = Order::random(&mut Rng::new(9));
        let mut o = Order::default();
        for item in ITEMS {
            while o.get(item) != target.get(item) {
                o.bump(item, true);
            }
        }
        assert_eq!(target.first_mismatch(&o), None);
    }
}
