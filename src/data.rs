//! RON data files under assets/data. Loaded once at boot, validated, then read only.

use std::collections::HashMap;

use macroquad::prelude::load_string;
use serde::Deserialize;

use crate::game::grade::Par;
use crate::game::quest::Grant;

#[derive(Clone, Debug, Deserialize)]
pub struct QuestDef {
    pub id: String,
    pub title: String,
    pub goal_doc: String,
    pub par_days: u32,
    pub deadline: u32,
    pub budget: i64,
    pub official_fees: i64,
    pub board: Vec<String>,
    pub intro: String,
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

#[derive(Clone, Debug, Deserialize)]
pub struct OfficeDef {
    pub id: String,
    pub name: String,
    pub short: String,
    pub pos: (f32, f32),
    pub size: (f32, f32),
    pub grants: Vec<Grant>,
    pub greet: Vec<String>,
    pub missing: Vec<String>,
    pub issued: Vec<String>,
    pub nothing: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DocDef {
    pub id: String,
    pub name: String,
}

pub struct Data {
    pub quests: Vec<QuestDef>,
    pub offices: Vec<OfficeDef>,
    pub docs: Vec<DocDef>,
    pub strings: HashMap<String, String>,
}

impl Data {
    pub async fn load() -> Data {
        let data = Data {
            quests: parse("assets/data/quests.ron").await,
            offices: parse("assets/data/offices.ron").await,
            docs: parse("assets/data/docs.ron").await,
            strings: parse("assets/data/strings.ron").await,
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
        for q in &self.quests {
            doc_ok(&q.goal_doc, &q.id);
            q.board.iter().for_each(|d| doc_ok(d, &q.id));
        }
        for o in &self.offices {
            for g in &o.grants {
                doc_ok(&g.doc, &o.id);
                g.requires.iter().for_each(|d| doc_ok(d, &o.id));
            }
        }
    }

    pub fn office(&self, id: &str) -> &OfficeDef {
        self.offices
            .iter()
            .find(|o| o.id == id)
            .unwrap_or_else(|| panic!("unknown office '{id}'"))
    }

    pub fn doc_name<'a>(&'a self, id: &'a str) -> &'a str {
        self.docs
            .iter()
            .find(|d| d.id == id)
            .map_or(id, |d| d.name.as_str())
    }

    /// UI string by key, with `{name}` placeholders filled from `vars`.
    pub fn s(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let mut out = self
            .strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("??{key}??"));
        for (k, v) in vars {
            out = out.replace(&format!("{{{k}}}"), v);
        }
        out
    }
}

async fn parse<T: for<'de> Deserialize<'de>>(path: &str) -> T {
    let text = load_string(path)
        .await
        .unwrap_or_else(|e| panic!("{path}: {e}"));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"))
}
