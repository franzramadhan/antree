//! Days, Money, Conscience.

/// `days` is the current day number, starting at 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meters {
    pub days: u32,
    pub money: i64,
    pub conscience: i32,
}

impl Meters {
    pub fn new(budget: i64) -> Self {
        Self {
            days: 1,
            money: budget,
            conscience: 100,
        }
    }

    /// Spend `kv`. Returns false and leaves money untouched when short.
    pub fn spend(&mut self, kv: i64) -> bool {
        if kv > self.money {
            return false;
        }
        self.money -= kv;
        true
    }

    pub fn can_afford(&self, kv: i64) -> bool {
        kv <= self.money
    }

    /// Conscience only goes down. Clamped at 0.
    pub fn shortcut(&mut self, cost: i32) {
        self.conscience = (self.conscience - cost).max(0);
    }

    pub fn next_day(&mut self) {
        self.days += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spend_refuses_overdraft() {
        let mut m = Meters::new(100);
        assert!(m.spend(60));
        assert!(!m.spend(60));
        assert_eq!(m.money, 40);
    }

    #[test]
    fn conscience_floors_at_zero() {
        let mut m = Meters::new(0);
        m.shortcut(70);
        m.shortcut(70);
        assert_eq!(m.conscience, 0);
    }
}
