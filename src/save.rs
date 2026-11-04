//! Unlocks and best results, and one run in progress. localStorage on web, a
//! `local.data` file on native.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::game::grade::Grade;
use crate::game::quest::Grant;
use crate::game::run::Run;

const KEY: &str = "antree.save.v1";
const VERSION: u32 = 1;
const RUN_KEY: &str = "antree.run.v1";
const RUN_VERSION: u32 = 1;

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
    /// The orientation form was approved once.
    #[serde(default)]
    pub tutorial_done: bool,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            version: VERSION,
            unlocked: Vec::new(),
            best: BTreeMap::new(),
            inside_man: false,
            tutorial_done: false,
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

/// A run saved from town: the run itself and where the player stood.
#[derive(Serialize, Deserialize)]
struct SavedRun {
    version: u32,
    run: Run,
    player: (f32, f32),
}

pub fn store_run(run: &Run, player: (f32, f32)) {
    let saved = SavedRun {
        version: RUN_VERSION,
        run: run.clone(),
        player,
    };
    let text = ron::to_string(&saved).expect("run serializes");
    quad_storage::STORAGE.lock().unwrap().set(RUN_KEY, &text);
}

/// The saved run, with its grants restored from the data files.
pub fn load_run(grants: Vec<Grant>) -> Option<(Run, (f32, f32))> {
    parse_run(quad_storage::STORAGE.lock().unwrap().get(RUN_KEY))
        .map(|(run, player)| (run.with_grants(grants), player))
}

/// Forget the saved run once that same run has ended.
pub fn clear_run_if_ended(run: &Run) {
    let saved = parse_run(quad_storage::STORAGE.lock().unwrap().get(RUN_KEY));
    if saved.is_some_and(|(r, _)| r.seed == run.seed && r.quest_id == run.quest_id) {
        quad_storage::STORAGE.lock().unwrap().remove(RUN_KEY);
    }
}

fn parse_run(text: Option<String>) -> Option<(Run, (f32, f32))> {
    text.and_then(|t| ron::from_str::<SavedRun>(&t).ok())
        .filter(|s| s.version == RUN_VERSION)
        .map(|s| (s.run, s.player))
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
    fn a_saved_run_comes_back_exactly() {
        let quest: crate::game::quest::QuestDef = ron::from_str(
            r#"(id: "q", title: "Q", intro: "", goal_doc: "g", start_docs: ["a"], board: ["g"],
                hidden_pool: [("h", "o")], wrong_pool: ["w"], par_days: 3, deadline: 6,
                budget: 400, official_fees: 0)"#,
        )
        .unwrap();
        let mut run = Run::new(77, &quest, vec![], false);
        run.meters.money -= 40;
        run.inv.add("loss_report");
        let saved = SavedRun {
            version: RUN_VERSION,
            run: run.clone(),
            player: (120.0, 182.0),
        };
        let (back, player) = parse_run(Some(ron::to_string(&saved).unwrap())).unwrap();
        assert_eq!(player, (120.0, 182.0));
        assert_eq!((back.seed, back.meters.money), (77, run.meters.money));
        assert!(back.inv.has("loss_report") && back.inv.has("a"));
        assert!(parse_run(Some("(".into())).is_none());
    }

    #[test]
    fn round_trips_and_ignores_garbage_or_other_versions() {
        let mut s = Save::default();
        s.record("a", Some("b"), best(Grade::X, 66));
        let text = ron::to_string(&s).unwrap();
        assert_eq!(Save::parse(Some(text.clone())), s);
        assert_eq!(Save::parse(Some("{".into())), Save::default());
        assert_eq!(Save::parse(None), Save::default());
        let before_tutorial = "(version:1,unlocked:[],best:{},inside_man:true)";
        let s = Save::parse(Some(before_tutorial.into()));
        assert!(s.inside_man && !s.tutorial_done);
        let old = text.replace("version:1", "version:0");
        assert_eq!(Save::parse(Some(old)), Save::default());
    }
}
