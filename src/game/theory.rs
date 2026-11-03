//! Queue Theory Test, GDD 5.7. Ten questions, pass at seven.

use serde::Deserialize;

use super::rng::Rng;

pub const LENGTH: usize = 10;
pub const PASS: u32 = 7;
pub const FEE: i64 = 25;

#[derive(Clone, Debug, Deserialize)]
pub struct Question {
    pub q: String,
    pub options: Vec<String>,
    pub answer: usize,
}

#[derive(Clone, Debug)]
pub struct Test {
    pub order: Vec<usize>,
    pub index: usize,
    pub correct: u32,
}

impl Test {
    pub fn new(rng: &mut Rng, pool: usize) -> Self {
        let mut order: Vec<usize> = (0..pool).collect();
        rng.shuffle(&mut order);
        order.truncate(LENGTH);
        Self {
            order,
            index: 0,
            correct: 0,
        }
    }

    pub fn current<'a>(&self, pool: &'a [Question]) -> &'a Question {
        &pool[self.order[self.index]]
    }

    pub fn answer(&mut self, pool: &[Question], choice: usize) {
        if self.current(pool).answer == choice {
            self.correct += 1;
        }
        self.index += 1;
    }

    pub fn done(&self) -> bool {
        self.index >= self.order.len()
    }

    pub fn passed(&self) -> bool {
        self.correct >= PASS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> Vec<Question> {
        (0..14)
            .map(|i| Question {
                q: format!("q{i}"),
                options: vec!["a".into(), "b".into(), "c".into()],
                answer: i % 3,
            })
            .collect()
    }

    #[test]
    fn seven_right_passes_six_fails() {
        let pool = pool();
        for (right, pass) in [(7, true), (6, false), (10, true)] {
            let mut t = Test::new(&mut Rng::new(5), pool.len());
            for i in 0..LENGTH {
                let a = t.current(&pool).answer;
                t.answer(&pool, if i < right { a } else { (a + 1) % 3 });
            }
            assert!(t.done());
            assert_eq!(t.passed(), pass);
        }
    }

    #[test]
    fn asks_ten_distinct_questions() {
        let mut t = Test::new(&mut Rng::new(1), 14);
        t.order.sort();
        t.order.dedup();
        assert_eq!(t.order.len(), LENGTH);
    }
}
