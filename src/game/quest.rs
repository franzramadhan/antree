//! Quests, document inventory, and chain resolution.

use serde::{Deserialize, Serialize};

use super::grade::Par;

#[derive(Clone, Debug, Deserialize)]
pub struct QuestDef {
    pub id: String,
    pub title: String,
    pub intro: String,
    pub goal_doc: String,
    #[serde(default)]
    pub start_docs: Vec<String>,
    pub par_days: u32,
    pub deadline: u32,
    pub budget: i64,
    pub official_fees: i64,
    /// What the information board claims you need.
    pub board: Vec<String>,
    /// (document, the document whose grant secretly requires it). One per run.
    pub hidden_pool: Vec<(String, String)>,
    /// Listed on the board, needed by nobody. One per run.
    pub wrong_pool: Vec<String>,
    /// Documents a surprise new regulation can add to the goal mid-run.
    #[serde(default)]
    pub rule_pool: Vec<String>,
}

impl QuestDef {
    pub fn par(&self) -> Par {
        Par {
            par_days: self.par_days,
            deadline: self.deadline,
            budget: self.budget,
            official_fees: self.official_fees,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inventory(Vec<String>);

impl Inventory {
    pub fn has(&self, doc: &str) -> bool {
        self.0.iter().any(|d| d == doc)
    }

    pub fn add(&mut self, doc: &str) {
        if !self.has(doc) {
            self.0.push(doc.to_owned());
        }
    }
}

/// The minigame a counter runs before it hands anything over.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Task {
    Queue,
    Form,
    Stamp,
    Photocopy,
    Theory,
}

/// One document an office can issue and what it asks for in return.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Grant {
    pub doc: String,
    pub requires: Vec<String>,
    #[serde(default)]
    pub fee: i64,
    /// Overrides the office's task list for this document.
    #[serde(default)]
    pub task: Option<Task>,
}

/// `roots` plus every document needed, transitively, to obtain them.
pub fn closure(roots: &[&str], requires: impl Fn(&str) -> Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut stack: Vec<String> = roots.iter().map(|s| s.to_string()).collect();
    while let Some(doc) = stack.pop() {
        if !out.contains(&doc) {
            stack.extend(requires(&doc));
            out.push(doc);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(doc: &str, req: &[&str]) -> Grant {
        Grant {
            doc: doc.into(),
            requires: req.iter().map(|s| s.to_string()).collect(),
            fee: 0,
            task: None,
        }
    }

    #[test]
    fn closure_follows_chains_and_survives_cycles() {
        let grants = [
            g("ktpss", &["district_letter", "loss_report"]),
            g("district_letter", &["kaling_letter"]),
            g("kaling_letter", &["ksk"]),
            g("spusp", &["kaling_letter", "spusp"]),
        ];
        let req = |d: &str| {
            grants
                .iter()
                .find(|x| x.doc == d)
                .map_or(vec![], |x| x.requires.clone())
        };
        let mut c = closure(&["ktpss"], req);
        c.sort();
        assert_eq!(
            c,
            [
                "district_letter",
                "kaling_letter",
                "ksk",
                "ktpss",
                "loss_report"
            ]
        );
        assert_eq!(closure(&["spusp"], req).len(), 3);
    }
}
