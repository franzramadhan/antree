use macroquad::prelude::*;

use super::minigame::{Kind, Minigame, Outcome};
use super::{confirm_pressed, escape_pressed, say, Ctx, Overworld, Scene};
use crate::canvas;
use crate::data::Data;
use crate::fx::{Sfx, Slam};
use crate::game::nego::Who;
use crate::game::obstacle::{roll, Obstacle};
use crate::game::quest::Task;
use crate::game::run::Run;
use crate::ui;

const INSIDE_MAN_CONSCIENCE: i32 = 25;
const MOVED_MINUTES: u32 = 20;
/// "Counter moved" rerolls at most this many times before the counter gives up moving.
const MAX_MOVES: u32 = 3;

enum Phase {
    /// A line from the clerk, then leave. `end_day` says whether leaving ends the day.
    Say {
        line: String,
        end_day: bool,
    },
    Greet(String),
    /// A line, then the next step.
    Note(String),
    Play(Minigame),
    Issue {
        doc: String,
        line: String,
    },
}

pub struct Office {
    run: Run,
    office_id: String,
    return_pos: Vec2,
    phase: Phase,
    /// Minigames still to clear before the clerk issues anything, next first.
    steps: Vec<Kind>,
    /// The stamp on the current verdict: REJECTED on a refusal, the seal on issue.
    slam: Option<Slam>,
}

impl Office {
    pub fn new(mut run: Run, office_id: String, return_pos: Vec2, d: &Data) -> Self {
        let office = d.office(&office_id);
        let line = |key, run: &Run| say(d, &office_id, key, run);
        let phase = if !run.clock.numbers_open() {
            Phase::Say {
                line: line("numbers_closed", &run),
                end_day: true,
            }
        } else if run.meters.days < office.opens_day && !run.connected {
            Phase::Say {
                line: line("locked", &run),
                end_day: false,
            }
        } else if run.frozen(&office_id) {
            Phase::Say {
                line: line("under_review", &run),
                end_day: false,
            }
        } else {
            Phase::Greet(line("greet", &run))
        };
        run.enter(&office_id);
        Self {
            run,
            office_id,
            return_pos,
            phase,
            steps: Vec::new(),
            slam: None,
        }
    }

    fn leave(&self, toast: Option<String>) -> Option<Scene> {
        Some(Scene::Overworld(Overworld::at(
            self.run.clone(),
            self.return_pos,
            toast,
        )))
    }

    fn say(&self, d: &Data, key: &str) -> String {
        say(d, &self.office_id, key, &self.run)
    }

    fn after_greet(&mut self, d: &Data) {
        let office = d.office(&self.office_id);
        let Some(grant) = self.run.offer(&office.grants).cloned() else {
            self.phase = Phase::Say {
                line: self.say(d, "nothing"),
                end_day: false,
            };
            return;
        };
        if let Some(doc) = self.run.missing(&grant.doc).first().cloned() {
            self.run.learn(&doc);
            self.slam = Some(Slam::reject());
            self.phase = Phase::Say {
                line: self.say(d, "missing").replace("{doc}", d.doc_phrase(&doc)),
                end_day: false,
            };
            return;
        }

        let main = grant.task.unwrap_or_else(|| {
            *self
                .run
                .visit_rng("task", &self.office_id)
                .pick(&office.tasks)
        });
        let weights = d.weights(&self.office_id);
        let mut rng = self.run.rng(&format!("obstacle:{}", self.office_id));
        let mut notes: Vec<String> = Vec::new();
        let mut pre = None;
        for moves in 0.. {
            match roll(&mut rng, &weights, self.run.mods(&self.office_id)) {
                Obstacle::CounterMoved if moves < MAX_MOVES => {
                    notes.push(self.say(d, "ob_moved"));
                    if self.run.advance(MOVED_MINUTES) {
                        return self.closed(d);
                    }
                    continue;
                }
                Obstacle::Smooth | Obstacle::CounterMoved => {}
                Obstacle::SystemDown => pre = Some(Kind::Offline),
                Obstacle::ClerkBreak => {
                    let mins = rng.range(3, 7) * 10;
                    notes.push(self.say(d, "ob_break").replace("{min}", &mins.to_string()));
                    if self.run.advance(mins) {
                        return self.closed(d);
                    }
                }
                Obstacle::MissingPaperwork => {
                    notes.push(self.say(d, "ob_paperwork"));
                    pre = Some(Kind::Task(Task::Photocopy));
                }
                Obstacle::FormChanged => {
                    notes.push(self.say(d, "ob_form"));
                    pre = Some(Kind::Task(Task::Form));
                }
                Obstacle::FixerOffer => {
                    let who = if rng.chance(50) {
                        Who::Clerk
                    } else {
                        Who::Fixer
                    };
                    pre = Some(Kind::Nego(who));
                }
                Obstacle::CollectiveLeave => {
                    self.phase = Phase::Say {
                        line: self.say(d, "leave_closed"),
                        end_day: true,
                    };
                    return;
                }
            }
            break;
        }

        // Next step is at the end so `pop` takes it.
        self.steps = vec![Kind::Task(main)];
        if let Some(p) = pre.filter(|&p| p != Kind::Task(main)) {
            self.steps.push(p);
        }
        if notes.is_empty() {
            self.next_step(d);
        } else {
            self.phase = Phase::Note(notes.join(" "));
        }
    }

    fn closed(&mut self, d: &Data) {
        self.phase = Phase::Say {
            line: self.say(d, "closed_now"),
            end_day: true,
        };
    }

    fn next_step(&mut self, d: &Data) {
        match self.steps.pop() {
            Some(kind) => {
                self.phase = Phase::Play(Minigame::new(kind, &mut self.run, &self.office_id, d));
            }
            None => self.issue(d, None),
        }
    }

    /// Hand over the document, after the official fee. `lead` is said first.
    fn issue(&mut self, d: &Data, lead: Option<String>) {
        let office = d.office(&self.office_id);
        let grant = self
            .run
            .offer(&office.grants)
            .expect("an office only plays minigames for a grant it can give")
            .clone();
        if !self.run.meters.spend(grant.fee) {
            self.phase = Phase::Say {
                line: self
                    .say(d, "cannot_pay")
                    .replace("{fee}", &grant.fee.to_string()),
                end_day: false,
            };
            return;
        }
        let mut line = lead.map(|l| l + " ").unwrap_or_default();
        line.push_str(&self.say(d, "issued"));
        if grant.fee > 0 {
            line.push(' ');
            line.push_str(&d.s("issued_fee", &[("fee", &grant.fee.to_string())]));
        }
        self.slam = Some(Slam::approve());
        self.phase = Phase::Issue {
            doc: grant.doc,
            line,
        };
    }

    fn call_inside_man(&mut self, d: &Data) {
        self.run.inside_man = false;
        self.run.connected = true;
        self.run.meters.shortcut(INSIDE_MAN_CONSCIENCE);
        let lead = self.say(d, "inside_man");
        self.issue(d, Some(lead));
    }

    fn can_call_inside_man(&self, d: &Data) -> bool {
        self.run.inside_man && self.run.offer(&d.office(&self.office_id).grants).is_some()
    }

    pub fn update(&mut self, ctx: &Ctx) -> Option<Scene> {
        let money = self.run.meters.money;
        let next = self.step(ctx);
        if self.run.meters.money < money {
            ctx.fx.play(Sfx::Coin);
        }
        if let Some(s) = &mut self.slam {
            s.update(ctx.fx, ctx.dt);
        }
        next
    }

    fn step(&mut self, ctx: &Ctx) -> Option<Scene> {
        let d = &ctx.assets.data;
        match &mut self.phase {
            Phase::Say { end_day, .. } => {
                if confirm_pressed() {
                    let toast = end_day.then(|| {
                        self.run.end_day();
                        d.s("day_ended", &[])
                    });
                    return self.leave(toast);
                }
            }
            Phase::Greet(_) => {
                if escape_pressed() {
                    return self.leave(None);
                }
                if is_key_pressed(KeyCode::I) && self.can_call_inside_man(d) {
                    self.call_inside_man(d);
                } else if confirm_pressed() {
                    self.after_greet(d);
                }
            }
            Phase::Note(_) => {
                if confirm_pressed() {
                    self.next_step(d);
                }
            }
            Phase::Play(game) => {
                if escape_pressed() {
                    return self.leave(None);
                }
                match game.update(ctx, &mut self.run, &self.office_id) {
                    Outcome::Running => {}
                    Outcome::Passed(Some(line)) => self.phase = Phase::Note(line),
                    Outcome::Passed(None) => self.next_step(d),
                    Outcome::Granted(line) => self.issue(d, Some(line)),
                    Outcome::Failed { line, end_day } => {
                        self.slam = Some(Slam::reject());
                        self.phase = Phase::Say { line, end_day };
                    }
                }
            }
            Phase::Issue { doc, .. } => {
                if confirm_pressed() {
                    let doc = doc.clone();
                    self.run.inv.add(&doc);
                    return self.leave(None);
                }
            }
        }
        None
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let office = d.office(&self.office_id);
        clear_background(ui::WALL);
        draw_rectangle(0.0, 200.0, canvas::WIDTH, 50.0, ui::WOOD);
        draw_rectangle(0.0, 196.0, canvas::WIDTH, 4.0, ui::INK);
        let name_w = ui::text_width(font, &office.name, 16) + 24.0;
        ui::panel((canvas::WIDTH - name_w) / 2.0, 36.0, name_w, 26.0);
        ui::text_centered(font, &office.name, canvas::WIDTH / 2.0, 54.0, 16, ui::INK);

        match &self.phase {
            Phase::Say { line, .. } => {
                if let Some(s) = self.slam.as_ref().filter(|s| s.rejects()) {
                    s.draw_word(
                        font,
                        &d.s("stamp_rejected", &[]),
                        canvas::WIDTH / 2.0,
                        140.0,
                    );
                }
                ui::dialog(font, &office.short, line);
                ui::hint(font, &d.s("hint_leave", &[]));
            }
            Phase::Greet(line) => {
                ui::dialog(font, &office.short, line);
                let key = if self.can_call_inside_man(d) {
                    "hint_counter_im"
                } else {
                    "hint_counter"
                };
                ui::hint(font, &d.s(key, &[]));
            }
            Phase::Note(line) => {
                ui::dialog(font, &office.short, line);
                ui::hint(font, &d.s("hint_continue", &[]));
            }
            Phase::Issue { line, .. } => {
                if let Some(s) = &self.slam {
                    s.draw_texture(&ctx.assets.stamp, canvas::WIDTH / 2.0, 132.0, 64.0, 0.0);
                }
                ui::dialog(font, &office.short, line);
                ui::hint(font, &d.s("hint_take", &[]));
            }
            Phase::Play(game) => game.draw(ctx),
        }
        ui::hud(font, d, &self.run);
    }
}
