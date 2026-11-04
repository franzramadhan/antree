//! In-game day: 08:00 to 15:00, queue numbers stop at 11:30.

use serde::{Deserialize, Serialize};

pub const DAY_START: u32 = 8 * 60;
pub const DAY_END: u32 = 15 * 60;
pub const LAST_NUMBER: u32 = 11 * 60 + 30;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clock {
    pub minutes: u32,
}

impl Default for Clock {
    fn default() -> Self {
        Self { minutes: DAY_START }
    }
}

impl Clock {
    /// Advance by `minutes`. Returns true when the day ended.
    pub fn advance(&mut self, minutes: u32) -> bool {
        self.minutes = (self.minutes + minutes).min(DAY_END);
        self.minutes >= DAY_END
    }

    pub fn numbers_open(&self) -> bool {
        self.minutes < LAST_NUMBER
    }

    pub fn reset(&mut self) {
        self.minutes = DAY_START;
    }

    pub fn hhmm(&self) -> String {
        format!("{:02}:{:02}", self.minutes / 60, self.minutes % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_and_ends_day() {
        let mut c = Clock::default();
        assert_eq!(c.hhmm(), "08:00");
        assert!(!c.advance(200));
        assert_eq!(c.hhmm(), "11:20");
        assert!(c.numbers_open());
        assert!(!c.advance(10));
        assert!(!c.numbers_open());
        assert!(c.advance(1000));
        assert_eq!(c.hhmm(), "15:00");
    }
}
