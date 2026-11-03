//! Unlocks and best results. localStorage on web, a `local.data` file on native.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::game::grade::Grade;

const KEY: &str = "antri.save.v1";
const VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Best {
    pub grade: Grade,
    pub total: i32,
    pub days: u32,
    pub money: i64,
    pub conscience: i32,
    pub seed: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    pub version: u32,
    /// Quest ids unlocked beyond the first.
    pub unlocked: Vec<String>,
    pub best: BTreeMap<String, Best>,
    /// Any quest completed once. The inside man takes your calls from then on.
    pub inside_man: bool,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            version: VERSION,
            unlocked: Vec::new(),
            best: BTreeMap::new(),
            inside_man: false,
        }
    }
}

impl Save {
    pub fn load() -> Save {
        Self::parse(quad_storage::STORAGE.lock().unwrap().get(KEY))
    }

    pub fn store(&self) {
        let text = ron::to_string(self).expect("save serializes");
        quad_storage::STORAGE.lock().unwrap().set(KEY, &text);
    }

    /// Anything unreadable or from another version starts fresh.
    fn parse(text: Option<String>) -> Save {
        text.and_then(|t| ron::from_str::<Save>(&t).ok())
            .filter(|s| s.version == VERSION)
            .unwrap_or_default()
    }

    pub fn is_unlocked(&self, index: usize, quest_id: &str) -> bool {
        index == 0 || self.unlocked.iter().any(|q| q == quest_id)
    }

    /// Record a finished run. `next` is the quest a completion unlocks.
    pub fn record(&mut self, quest_id: &str, next: Option<&str>, result: Best) {
        let complete = result.grade != Grade::F;
        if complete {
            self.inside_man = true;
            if let Some(n) = next.filter(|n| !self.unlocked.iter().any(|q| q == n)) {
                self.unlocked.push(n.to_owned());
            }
        }
        let better = match self.best.get(quest_id) {
            None => true,
            Some(old) if old.grade == Grade::F => complete || result.total > old.total,
            Some(old) => complete && result.total > old.total,
        };
        if better {
            self.best.insert(quest_id.to_owned(), result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn best(grade: Grade, total: i32) -> Best {
        Best {
            grade,
            total,
            days: 5,
            money: 100,
            conscience: 70,
            seed: "K7Q2MA".into(),
        }
    }

    #[test]
    fn completion_unlocks_next_quest_and_the_inside_man() {
        let mut s = Save::default();
        assert!(s.is_unlocked(0, "a") && !s.is_unlocked(1, "b"));
        s.record("a", Some("b"), best(Grade::F, 10));
        assert!(!s.is_unlocked(1, "b") && !s.inside_man);
        s.record("a", Some("b"), best(Grade::C, 35));
        s.record("a", Some("b"), best(Grade::B, 55));
        assert!(s.is_unlocked(1, "b") && s.inside_man);
        assert_eq!(s.unlocked, ["b"]);
    }

    #[test]
    fn best_keeps_the_highest_completed_total() {
        let mut s = Save::default();
        s.record("a", None, best(Grade::F, 40));
        s.record("a", None, best(Grade::D, 20));
        assert_eq!(s.best["a"].grade, Grade::D);
        s.record("a", None, best(Grade::F, 90));
        s.record("a", None, best(Grade::A, 75));
        s.record("a", None, best(Grade::B, 60));
        assert_eq!(s.best["a"].total, 75);
    }

    #[test]
    fn round_trips_and_ignores_garbage_or_other_versions() {
        let mut s = Save::default();
        s.record("a", Some("b"), best(Grade::X, 66));
        let text = ron::to_string(&s).unwrap();
        assert_eq!(Save::parse(Some(text.clone())), s);
        assert_eq!(Save::parse(Some("{".into())), Save::default());
        assert_eq!(Save::parse(None), Save::default());
        let old = text.replace("version:1", "version:0");
        assert_eq!(Save::parse(Some(old)), Save::default());
    }
}
