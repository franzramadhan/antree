//! Counter minigames. Each runs inside an office over the office backdrop and
//! reports an `Outcome` back to it.

mod form;
mod nego;
mod offline;
mod photocopy;
mod queue;
mod stamp;
mod theory;

use super::{say, Ctx};
use crate::data::Data;
use crate::game::nego::Who;
use crate::game::quest::Task;
use crate::game::run::Run;

pub enum Outcome {
    Running,
    /// Move on to the next step, after showing the line if there is one.
    Passed(Option<String>),
    /// Skip every remaining step and issue the document.
    Granted(String),
    Failed {
        line: String,
        end_day: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Task(Task),
    Offline,
    Nego(Who),
}

pub enum Minigame {
    Queue(queue::QueueGame),
    Form(form::FormGame),
    Stamp(stamp::StampGame),
    Photocopy(photocopy::PhotoGame),
    Theory(theory::TheoryGame),
    Offline(offline::OfflineGame),
    Nego(nego::NegoGame),
}

impl Minigame {
    pub fn new(kind: Kind, run: &mut Run, office: &str, d: &Data) -> Self {
        match kind {
            Kind::Task(Task::Queue) => Self::Queue(queue::QueueGame::new(run, office)),
            Kind::Task(Task::Form) => Self::Form(form::FormGame::new(run, office, d)),
            Kind::Task(Task::Stamp) => Self::Stamp(stamp::StampGame::new(run, office, d)),
            Kind::Task(Task::Photocopy) => {
                Self::Photocopy(photocopy::PhotoGame::new(run, office, d))
            }
            Kind::Task(Task::Theory) => Self::Theory(theory::TheoryGame::new(run, office, d)),
            Kind::Offline => Self::Offline(offline::OfflineGame::new(run, office, d)),
            Kind::Nego(who) => Self::Nego(nego::NegoGame::new(run, office, d, who)),
        }
    }

    pub fn update(&mut self, ctx: &Ctx, run: &mut Run, office: &str) -> Outcome {
        match self {
            Self::Queue(g) => g.update(ctx, run, office),
            Self::Form(g) => g.update(ctx, run, office),
            Self::Stamp(g) => g.update(ctx, run, office),
            Self::Photocopy(g) => g.update(ctx, run, office),
            Self::Theory(g) => g.update(ctx, run, office),
            Self::Offline(g) => g.update(ctx, run, office),
            Self::Nego(g) => g.update(ctx, run, office),
        }
    }

    pub fn draw(&self, ctx: &Ctx) {
        match self {
            Self::Queue(g) => g.draw(ctx),
            Self::Form(g) => g.draw(ctx),
            Self::Stamp(g) => g.draw(ctx),
            Self::Photocopy(g) => g.draw(ctx),
            Self::Theory(g) => g.draw(ctx),
            Self::Offline(g) => g.draw(ctx),
            Self::Nego(g) => g.draw(ctx),
        }
    }
}

/// Spend clerk time. Returns the failure when the office closes meanwhile.
fn pass_time(run: &mut Run, minutes: u32, d: &Data, office: &str) -> Option<Outcome> {
    run.advance(minutes).then(|| Outcome::Failed {
        line: say(d, office, "closed_now", run),
        end_day: true,
    })
}

fn cannot_pay(d: &Data, office: &str, run: &Run, fee: i64) -> String {
    say(d, office, "cannot_pay", run).replace("{fee}", &fee.to_string())
}
