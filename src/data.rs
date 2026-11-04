//! RON data files under assets/data. Loaded once at boot, validated, then read only.

use std::collections::HashMap;

use macroquad::prelude::load_string;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::assets::join_all;
use crate::game::form::FieldDef;
use crate::game::obstacle::Obstacle;
use crate::game::quest::{closure, Grant, QuestDef, Task};
use crate::game::run::{Run, DATA_CORRECTION};
use crate::game::theory::Question;

#[derive(Clone, Debug, Deserialize)]
pub struct OfficeDef {
    pub id: String,
    pub name: String,
    pub short: String,
    pub pos: (f32, f32),
    pub size: (f32, f32),
    pub grants: Vec<Grant>,
    /// What the counter makes you do when a grant names no task.
    #[serde(default)]
    pub tasks: Vec<Task>,
    /// Overrides for the default obstacle weights.
    #[serde(default)]
    pub weights: HashMap<Obstacle, u32>,
    /// The front door stays locked before this day unless you know someone.
    #[serde(default)]
    pub opens_day: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DocDef {
    pub id: String,
    /// Board label.
    pub name: String,
    /// How a clerk says it mid-sentence, when the label reads badly there.
    #[serde(default)]
    pub phrase: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TutorialRow {
    pub label: String,
    pub text: String,
    /// The first option is the blank.
    pub options: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TutorialForm {
    pub title: String,
    pub rows: Vec<TutorialRow>,
    pub blank: String,
    pub ink: String,
    pub approved: String,
    pub hint: String,
    pub hint_done: String,
}

type Dialog = HashMap<String, Vec<String>>;

pub struct Data {
    pub quests: Vec<QuestDef>,
    pub offices: Vec<OfficeDef>,
    pub docs: Vec<DocDef>,
    pub strings: HashMap<String, String>,
    pub obstacles: Vec<(Obstacle, u32)>,
    pub form_fields: Vec<FieldDef>,
    pub theory: Vec<Question>,
    pub tutorial: TutorialForm,
    /// "common" plus one entry per office id.
    dialog: HashMap<String, Dialog>,
}

const DIR: &str = "assets/data";
const FILES: [&str; 8] = [
    "quests.ron",
    "offices.ron",
    "docs.ron",
    "strings.ron",
    "obstacles.ron",
    "forms.ron",
    "theory.ron",
    "tutorial.ron",
];

impl Data {
    pub async fn load() -> Data {
        let texts = join_all(FILES.map(fetch)).await;
        let mut files: HashMap<String, String> =
            FILES.iter().map(|n| n.to_string()).zip(texts).collect();
        let offices: Vec<OfficeDef> = parse("offices.ron", &files["offices.ron"]);
        let dialogs: Vec<String> = offices
            .iter()
            .map(|o| o.id.as_str())
            .chain(["common"])
            .map(|id| format!("dialog/{id}.ron"))
            .collect();
        let texts = join_all(dialogs.iter().map(|n| fetch(n))).await;
        files.extend(dialogs.into_iter().zip(texts));
        Data::build(|name| files[name].clone())
    }

    /// Parse every file through `read` (path relative to assets/data) and validate.
    pub fn build(read: impl Fn(&str) -> String) -> Data {
        let get = |name: &str| read(name);
        let offices: Vec<OfficeDef> = parse("offices.ron", &get("offices.ron"));
        let dialog = offices
            .iter()
            .map(|o| o.id.as_str())
            .chain(["common"])
            .map(|id| {
                let name = format!("dialog/{id}.ron");
                (id.to_owned(), parse(&name, &get(&name)))
            })
            .collect();
        let data = Data {
            quests: parse("quests.ron", &get("quests.ron")),
            offices,
            docs: parse("docs.ron", &get("docs.ron")),
            strings: parse("strings.ron", &get("strings.ron")),
            obstacles: parse("obstacles.ron", &get("obstacles.ron")),
            form_fields: parse("forms.ron", &get("forms.ron")),
            theory: parse("theory.ron", &get("theory.ron")),
            tutorial: parse("tutorial.ron", &get("tutorial.ron")),
            dialog,
        };
        data.validate();
        data
    }

    fn validate(&self) {
        let doc_ok = |id: &str, from: &str| {
            assert!(
                self.docs.iter().any(|d| d.id == id),
                "{from}: unknown document id '{id}'"
            );
        };
        for o in &self.offices {
            for g in &o.grants {
                doc_ok(&g.doc, &o.id);
                g.requires.iter().for_each(|d| doc_ok(d, &o.id));
                assert!(
                    g.task.is_some() || !o.tasks.is_empty(),
                    "{}: grant '{}' has no task and the office has no tasks",
                    o.id,
                    g.doc
                );
            }
        }
        let grantable = |d: &str| {
            self.offices
                .iter()
                .any(|o| o.grants.iter().any(|g| g.doc == d))
        };
        assert!(
            grantable(DATA_CORRECTION),
            "no office grants '{DATA_CORRECTION}'"
        );
        for q in &self.quests {
            doc_ok(&q.goal_doc, &q.id);
            q.start_docs.iter().for_each(|d| doc_ok(d, &q.id));
            q.board.iter().for_each(|d| doc_ok(d, &q.id));
            q.wrong_pool.iter().for_each(|d| doc_ok(d, &q.id));
            assert!(
                !q.hidden_pool.is_empty() && !q.wrong_pool.is_empty(),
                "{}: empty pool",
                q.id
            );
            for (doc, by) in &q.hidden_pool {
                doc_ok(doc, &q.id);
                doc_ok(by, &q.id);
                for wrong in &q.wrong_pool {
                    let mut run = Run::new(0, q, self.all_grants(), false);
                    run.hidden = (doc.clone(), by.clone());
                    run.wrong = wrong.clone();
                    run.correction_due = true;
                    for need in closure(&[&q.goal_doc, wrong], |d| run.requires(d)) {
                        assert!(
                            grantable(&need) || q.start_docs.contains(&need),
                            "{}: nobody issues '{need}' (hidden {doc}, wrong {wrong})",
                            q.id
                        );
                    }
                }
            }
        }
        assert!(!self.tutorial.rows.is_empty());
        for r in &self.tutorial.rows {
            assert!(r.options.len() >= 2, "tutorial: {}", r.label);
        }
        assert!(self.theory.len() >= crate::game::theory::LENGTH);
        for q in &self.theory {
            assert!(
                q.answer < q.options.len() && q.options.len() <= 4,
                "theory: {}",
                q.q
            );
        }
    }

    pub fn all_grants(&self) -> Vec<Grant> {
        self.offices.iter().flat_map(|o| o.grants.clone()).collect()
    }

    pub fn office(&self, id: &str) -> &OfficeDef {
        self.offices
            .iter()
            .find(|o| o.id == id)
            .unwrap_or_else(|| panic!("unknown office '{id}'"))
    }

    pub fn quest(&self, id: &str) -> &QuestDef {
        self.quests
            .iter()
            .find(|q| q.id == id)
            .unwrap_or_else(|| panic!("unknown quest '{id}'"))
    }

    pub fn doc_name<'a>(&'a self, id: &'a str) -> &'a str {
        self.docs
            .iter()
            .find(|d| d.id == id)
            .map_or(id, |d| d.name.as_str())
    }

    pub fn doc_phrase<'a>(&'a self, id: &'a str) -> &'a str {
        self.docs
            .iter()
            .find(|d| d.id == id)
            .and_then(|d| d.phrase.as_deref())
            .unwrap_or_else(|| self.doc_name(id))
    }

    /// Default obstacle weights with the office's overrides applied.
    pub fn weights(&self, office: &str) -> Vec<(Obstacle, u32)> {
        let o = self.office(office);
        self.obstacles
            .iter()
            .map(|&(ob, w)| (ob, o.weights.get(&ob).copied().unwrap_or(w)))
            .collect()
    }

    /// A dialog line: the office's own version if it has one, else the common one.
    /// `salt` picks among variants.
    pub fn line(&self, office: &str, key: &str, salt: usize) -> String {
        let lines = self.dialog.get(office).and_then(|d| d.get(key));
        match lines.or_else(|| self.dialog["common"].get(key)) {
            Some(lines) if !lines.is_empty() => lines[salt % lines.len()].clone(),
            _ => format!("??{key}??"),
        }
    }

    /// UI string by key, with `{name}` placeholders filled from `vars`.
    pub fn s(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let out = self
            .strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("??{key}??"));
        fill(out, vars)
    }
}

pub fn fill(mut s: String, vars: &[(&str, &str)]) -> String {
    for (k, v) in vars {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

async fn fetch(name: &str) -> String {
    let path = format!("{DIR}/{name}");
    load_string(&path)
        .await
        .unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn parse<T: DeserializeOwned>(name: &str, text: &str) -> T {
    ron::from_str(text).unwrap_or_else(|e| panic!("{DIR}/{name}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn real() -> Data {
        Data::build(|name| std::fs::read_to_string(format!("{DIR}/{name}")).unwrap())
    }

    #[test]
    fn shipped_data_validates_and_every_quest_is_winnable() {
        let d = real();
        assert!(d.quests.len() >= 3);
    }

    #[test]
    fn office_lines_fall_back_to_common() {
        let d = real();
        let id = &d.offices[0].id;
        assert!(!d.line(id, "greet", 0).starts_with("??"));
        assert!(!d.line(id, "q_wait", 0).starts_with("??"));
        assert_eq!(d.line(id, "no_such_key", 0), "??no_such_key??");
    }
}
