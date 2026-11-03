//! One attempt at one quest. Owns the meters, the clock, the inventory, and
//! everything the offices remember about you.

use super::clock::Clock;
use super::grade::Par;
use super::meters::Meters;
use super::obstacle::Mods;
use super::quest::{closure, Grant, Inventory, QuestDef};
use super::rng::Rng;

/// Issued when a nonsense form comes back to haunt you.
pub const DATA_CORRECTION: &str = "data_correction";
const CORRECTION_CHANCE: u32 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Complete,
    Failed,
}

#[derive(Clone, Debug)]
pub struct Run {
    pub seed: u64,
    pub quest_id: String,
    pub goal_doc: String,
    pub par: Par,
    pub meters: Meters,
    pub clock: Clock,
    pub inv: Inventory,
    /// (document, required by). Nobody mentions it until you reach that counter.
    pub hidden: (String, String),
    pub wrong: String,
    /// Documents a clerk has told you about. Shown on the board.
    pub known: Vec<String>,
    /// The inside man is unlocked in the save and not used yet this run.
    pub inside_man: bool,
    /// You used the inside man. Things go smoother from now on.
    pub connected: bool,
    pub correction_due: bool,
    nonsense_forms: u32,
    declined: Vec<String>,
    greased: Vec<(String, u32)>,
    under_review: Vec<(String, u32)>,
    visits: Vec<String>,
    /// Every grant of every office, for chain resolution.
    grants: Vec<Grant>,
}

impl Run {
    pub fn new(seed: u64, quest: &QuestDef, grants: Vec<Grant>, inside_man: bool) -> Self {
        let mut inv = Inventory::default();
        quest.start_docs.iter().for_each(|d| inv.add(d));
        Self {
            seed,
            quest_id: quest.id.clone(),
            goal_doc: quest.goal_doc.clone(),
            par: quest.par(),
            meters: Meters::new(quest.budget),
            clock: Clock::default(),
            inv,
            hidden: Rng::derive(seed, "hidden").pick(&quest.hidden_pool).clone(),
            wrong: Rng::derive(seed, "wrong").pick(&quest.wrong_pool).clone(),
            known: Vec::new(),
            inside_man,
            connected: false,
            correction_due: false,
            nonsense_forms: 0,
            declined: Vec::new(),
            greased: Vec::new(),
            under_review: Vec::new(),
            visits: Vec::new(),
            grants,
        }
    }

    /// A stream for one purpose, stable for the whole day.
    pub fn rng(&self, tag: &str) -> Rng {
        Rng::derive(self.seed, &format!("{tag}:{}", self.meters.days))
    }

    /// A stream for one purpose at one office, fresh on every visit.
    pub fn visit_rng(&self, tag: &str, office: &str) -> Rng {
        let n = self.visits.iter().filter(|v| *v == office).count();
        self.rng(&format!("{tag}:{office}:{n}"))
    }

    pub fn enter(&mut self, office: &str) {
        self.visits.push(office.to_owned());
    }

    /// Advance the clock. Returns true when the day ended.
    pub fn advance(&mut self, minutes: u32) -> bool {
        self.clock.advance(minutes)
    }

    pub fn end_day(&mut self) {
        self.meters.next_day();
        self.clock.reset();
    }

    /// What the clerk will actually ask for before issuing `doc`.
    pub fn requires(&self, doc: &str) -> Vec<String> {
        let mut req = self
            .grants
            .iter()
            .find(|g| g.doc == doc)
            .map_or(vec![], |g| g.requires.clone());
        if self.hidden.1 == doc {
            req.push(self.hidden.0.clone());
        }
        if self.correction_due && doc == self.goal_doc {
            req.push(DATA_CORRECTION.to_owned());
        }
        req
    }

    pub fn missing(&self, doc: &str) -> Vec<String> {
        self.requires(doc)
            .into_iter()
            .filter(|d| !self.inv.has(d))
            .collect()
    }

    /// The first grant at this office worth having: needed for the goal or
    /// listed on the board, and not held yet.
    pub fn offer<'a>(&self, office_grants: &'a [Grant]) -> Option<&'a Grant> {
        let wanted = closure(&[&self.goal_doc, &self.wrong], |d| self.requires(d));
        office_grants
            .iter()
            .find(|g| !self.inv.has(&g.doc) && wanted.contains(&g.doc))
    }

    /// The board as the player sees it: the official list, the wrong item, and
    /// anything a clerk has mentioned since.
    pub fn board(&self, official: &[String]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for d in official.iter().chain([&self.wrong]).chain(&self.known) {
            if !out.contains(d) {
                out.push(d.clone());
            }
        }
        out
    }

    pub fn learn(&mut self, doc: &str) {
        if !self.known.iter().any(|d| d == doc) {
            self.known.push(doc.to_owned());
        }
    }

    /// Official fees still to pay on the way to the goal.
    pub fn fees_left(&self) -> i64 {
        closure(&[&self.goal_doc], |d| self.requires(d))
            .iter()
            .filter(|d| !self.inv.has(d))
            .filter_map(|d| self.grants.iter().find(|g| g.doc == *d))
            .map(|g| g.fee)
            .sum()
    }

    pub fn status(&self) -> Status {
        if self.inv.has(&self.goal_doc) {
            Status::Complete
        } else if self.meters.days > self.par.deadline || self.meters.money < self.fees_left() {
            Status::Failed
        } else {
            Status::Playing
        }
    }

    pub fn mods(&self, office: &str) -> Mods {
        Mods {
            declined: self.declined.iter().any(|o| o == office),
            connected: self.connected,
            greased: self.greased_today(office),
        }
    }

    pub fn decline(&mut self, office: &str) {
        self.declined.push(office.to_owned());
    }

    pub fn grease(&mut self, office: &str) {
        self.greased.push((office.to_owned(), self.meters.days));
    }

    pub fn greased_today(&self, office: &str) -> bool {
        self.greased
            .iter()
            .any(|(o, d)| o == office && *d == self.meters.days)
    }

    pub fn freeze(&mut self, office: &str, days: u32) {
        self.under_review
            .push((office.to_owned(), self.meters.days + days));
    }

    pub fn frozen(&self, office: &str) -> bool {
        self.under_review
            .iter()
            .any(|(o, until)| o == office && self.meters.days < *until)
    }

    /// Filled a form with nonsense. Sometimes the database notices.
    pub fn nonsense_form(&mut self) {
        self.nonsense_forms += 1;
        let tag = format!("nonsense:{}", self.nonsense_forms);
        if Rng::derive(self.seed, &tag).chance(CORRECTION_CHANCE) {
            self.correction_due = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(doc: &str, req: &[&str], fee: i64) -> Grant {
        Grant {
            doc: doc.into(),
            requires: req.iter().map(|s| s.to_string()).collect(),
            fee,
            task: None,
        }
    }

    fn grants() -> Vec<Grant> {
        vec![
            g("loss_report", &[], 0),
            g("kaling_letter", &["ksk"], 0),
            g("ktpss", &["loss_report", "kaling_letter"], 20),
            g("ksk_copy_3", &[], 0),
            g("photo_red", &[], 0),
            g("sktpa", &["photo_red"], 0),
            g("unrelated", &[], 0),
            g(DATA_CORRECTION, &["ksk"], 0),
        ]
    }

    fn quest() -> QuestDef {
        QuestDef {
            id: "q".into(),
            title: "Q".into(),
            intro: String::new(),
            goal_doc: "ktpss".into(),
            start_docs: vec!["ksk".into()],
            par_days: 2,
            deadline: 3,
            budget: 100,
            official_fees: 20,
            board: vec!["loss_report".into(), "ktpss".into()],
            hidden_pool: vec![
                ("ksk_copy_3".into(), "ktpss".into()),
                ("photo_red".into(), "kaling_letter".into()),
            ],
            wrong_pool: vec!["sktpa".into()],
        }
    }

    fn run(seed: u64) -> Run {
        Run::new(seed, &quest(), grants(), false)
    }

    #[test]
    fn completes_on_goal_doc() {
        let mut r = run(1);
        assert!(r.inv.has("ksk"));
        assert_eq!(r.status(), Status::Playing);
        r.inv.add("ktpss");
        assert_eq!(r.status(), Status::Complete);
    }

    #[test]
    fn fails_past_deadline() {
        let mut r = run(1);
        for _ in 0..2 {
            r.end_day();
        }
        assert_eq!(r.status(), Status::Playing);
        r.end_day();
        assert_eq!(r.status(), Status::Failed);
    }

    #[test]
    fn fails_when_the_fees_left_cannot_be_paid() {
        let mut r = run(1);
        r.meters.money = 20;
        assert_eq!(r.status(), Status::Playing);
        r.meters.money = 19;
        assert_eq!(r.status(), Status::Failed);
    }

    #[test]
    fn rng_tag_includes_day_and_visit_streams_change_per_visit() {
        let mut r = run(1);
        let a = r.rng("queue:dukcatan").next_u64();
        assert_eq!(a, r.rng("queue:dukcatan").next_u64());
        let v0 = r.visit_rng("form", "dukcatan").next_u64();
        r.enter("dukcatan");
        assert_ne!(v0, r.visit_rng("form", "dukcatan").next_u64());
        r.end_day();
        assert_ne!(a, r.rng("queue:dukcatan").next_u64());
    }

    #[test]
    fn hidden_requirement_is_seeded_and_lands_on_its_grant() {
        let picks: Vec<_> = (0..20).map(|s| run(s).hidden).collect();
        assert!(picks.iter().any(|h| h.0 == "ksk_copy_3"));
        assert!(picks.iter().any(|h| h.0 == "photo_red"));
        for s in 0..20 {
            let r = run(s);
            assert_eq!(r.hidden, run(s).hidden);
            assert!(r.requires(&r.hidden.1).contains(&r.hidden.0));
        }
    }

    #[test]
    fn offices_only_offer_what_the_quest_or_board_needs() {
        let mut r = run(1);
        assert_eq!(r.offer(&grants()[6..7]), None);
        assert_eq!(r.offer(&grants()[5..6]).unwrap().doc, "sktpa");
        r.inv.add("sktpa");
        assert_eq!(r.offer(&grants()[5..6]), None);
        assert_eq!(r.offer(&grants()[..1]).unwrap().doc, "loss_report");
    }

    #[test]
    fn missing_lists_hidden_last() {
        let r = (0..20).map(run).find(|r| r.hidden.1 == "ktpss").unwrap();
        assert_eq!(
            r.missing("ktpss"),
            ["loss_report", "kaling_letter", "ksk_copy_3"]
        );
    }

    #[test]
    fn data_correction_joins_the_goal_requirements() {
        let mut r = run(1);
        while !r.correction_due {
            r.nonsense_form();
        }
        assert!(r.missing("ktpss").contains(&DATA_CORRECTION.to_owned()));
        assert_eq!(r.offer(&grants()[7..]).unwrap().doc, DATA_CORRECTION);
    }

    #[test]
    fn board_adds_wrong_and_known_without_duplicates() {
        let mut r = run(1);
        r.learn("ksk_copy_3");
        r.learn("ksk_copy_3");
        r.learn("ktpss");
        assert_eq!(
            r.board(&quest().board),
            ["loss_report", "ktpss", "sktpa", "ksk_copy_3"]
        );
    }

    #[test]
    fn office_memory() {
        let mut r = run(1);
        assert!(!r.mods("x").declined);
        r.decline("x");
        r.grease("y");
        r.freeze("z", 2);
        assert!(r.mods("x").declined && r.mods("y").greased && r.frozen("z"));
        r.end_day();
        assert!(!r.mods("y").greased && r.frozen("z"));
        r.end_day();
        assert!(!r.frozen("z"));
    }
}
