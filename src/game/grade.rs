//! End-of-run grade. Formula from GDD section 3, table from section 7.

use serde::{Deserialize, Serialize};

use super::meters::Meters;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Grade {
    S,
    A,
    B,
    C,
    D,
    X,
    F,
}

impl Grade {
    pub fn letter(self) -> &'static str {
        match self {
            Grade::S => "S",
            Grade::A => "A",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
            Grade::X => "X",
            Grade::F => "F",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Par {
    pub par_days: u32,
    pub deadline: u32,
    pub budget: i64,
    pub official_fees: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scored {
    pub grade: Grade,
    pub total: i32,
    pub day_score: i32,
    pub money_score: i32,
}

pub fn score(m: &Meters, par: &Par, complete: bool) -> Scored {
    let over = m.days.saturating_sub(par.par_days) as i32;
    let day_score = (100 - 12 * over).clamp(0, 100);
    let spent = par.budget - m.money;
    let extra = (spent - par.official_fees).max(0);
    let slack = (par.budget - par.official_fees).max(1);
    let money_score = (100 - (100 * extra) / slack).clamp(0, 100) as i32;
    let total = (0.4 * day_score as f64 + 0.3 * money_score as f64 + 0.3 * m.conscience as f64)
        .round() as i32;

    let grade = if !complete {
        Grade::F
    } else if m.days <= par.par_days && m.conscience <= 40 {
        Grade::X
    } else if total >= 85 && m.conscience >= 80 {
        Grade::S
    } else if total >= 70 {
        Grade::A
    } else if total >= 50 {
        Grade::B
    } else if total >= 30 {
        Grade::C
    } else {
        Grade::D
    };

    Scored {
        grade,
        total,
        day_score,
        money_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAR: Par = Par {
        par_days: 4,
        deadline: 8,
        budget: 400,
        official_fees: 0,
    };

    fn m(days: u32, money: i64, conscience: i32) -> Meters {
        Meters {
            days,
            money,
            conscience,
        }
    }

    #[test]
    fn perfect_run_is_s() {
        let s = score(&m(3, 400, 100), &PAR, true);
        assert_eq!(s.total, 100);
        assert_eq!(s.grade, Grade::S);
    }

    #[test]
    fn day_penalty_is_twelve_per_day_over_par() {
        assert_eq!(score(&m(6, 400, 100), &PAR, true).day_score, 76);
        assert_eq!(score(&m(20, 400, 100), &PAR, true).day_score, 0);
    }

    #[test]
    fn money_score_only_counts_spend_above_official_fees() {
        let par = Par {
            official_fees: 100,
            ..PAR
        };
        assert_eq!(score(&m(1, 300, 100), &par, true).money_score, 100);
        assert_eq!(score(&m(1, 150, 100), &par, true).money_score, 50);
        assert_eq!(score(&m(1, 0, 100), &par, true).money_score, 0);
    }

    #[test]
    fn fast_and_dirty_is_part_of_the_system() {
        let s = score(&m(2, 100, 30), &PAR, true);
        assert_eq!(s.grade, Grade::X);
    }

    #[test]
    fn letter_thresholds() {
        assert_eq!(score(&m(4, 400, 79), &PAR, true).grade, Grade::A);
        assert_eq!(score(&m(7, 200, 60), &PAR, true).grade, Grade::B);
        assert_eq!(score(&m(9, 100, 50), &PAR, true).grade, Grade::C);
        assert_eq!(score(&m(12, 0, 50), &PAR, true).grade, Grade::D);
    }

    #[test]
    fn incomplete_is_f() {
        assert_eq!(score(&m(1, 400, 100), &PAR, false).grade, Grade::F);
    }
}
