use macroquad::prelude::*;

use super::{cannot_pay, pass_time, Outcome};
use crate::fx::Sfx;
use crate::game::queue::{Action, Event, Outcome as Step, Queue, TICK_MINUTES};
use crate::game::run::Run;
use crate::scene::{say, Ctx};
use crate::ui;

const ACTIONS: [Action; 4] = [
    Action::Wait,
    Action::Confront,
    Action::Cut,
    Action::PayFixer,
];
const SKIN: Color = Color::new(0.85, 0.65, 0.5, 1.0);

pub struct QueueGame {
    queue: Queue,
    msg: String,
}

impl QueueGame {
    pub fn new(run: &Run, office: &str) -> Self {
        Self {
            queue: Queue::new(run.visit_rng("queue", office)),
            msg: String::new(),
        }
    }

    pub fn update(&mut self, ctx: &Ctx, run: &mut Run, office: &str) -> Outcome {
        let d = &ctx.assets.data;
        let Some(i) = ui::choice_input(4, ctx.mouse, |i| self.queue.can(ACTIONS[i])) else {
            return Outcome::Running;
        };
        let action = ACTIONS[i];
        if let Some(price) = self.queue.fixer_price() {
            if action == Action::PayFixer && !run.meters.can_afford(price) {
                self.msg = cannot_pay(d, office, run, price);
                return Outcome::Running;
            }
        }
        let ahead = self.queue.ahead;
        let step = self.queue.step(action);
        if self.queue.ahead < ahead || step.outcome == Step::Served {
            ctx.fx.play(Sfx::Ting);
        }
        run.meters.spend(step.money);
        run.meters.shortcut(step.conscience);
        self.msg = say(d, office, step.message, run);
        match step.outcome {
            Step::Served => return Outcome::Passed(None),
            Step::Fight => {
                return Outcome::Failed {
                    line: self.msg.clone(),
                    end_day: true,
                }
            }
            Step::Waiting => {}
        }
        if pass_time(run, TICK_MINUTES, d, office).is_some() {
            return Outcome::Failed {
                line: say(d, office, "q_closed", run),
                end_day: true,
            };
        }
        let event = match self.queue.pending {
            Some(Event::Auntie) => say(d, office, "q_event_auntie", run),
            Some(Event::InsideMan) => say(d, office, "q_event_inside", run),
            Some(Event::FixerOffer { price }) => {
                say(d, office, "q_event_fixer", run).replace("{price}", &price.to_string())
            }
            None => String::new(),
        };
        if !event.is_empty() {
            self.msg = format!("{} {event}", self.msg);
        }
        Outcome::Running
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let q = &self.queue;
        let person = |i: u32, body: Color| {
            let x = 40.0 + (i % 15) as f32 * 38.0;
            let y = 140.0 - (i / 15) as f32 * 36.0;
            draw_rectangle(x, y, 10.0, 16.0, body);
            draw_rectangle(x + 2.0, y - 5.0, 6.0, 6.0, SKIN);
            (x, y)
        };
        for i in 0..q.ahead.min(29) {
            person(i, ui::INK);
        }
        let (me_x, me_y) = person(q.ahead.min(29), ui::STAMP);
        ui::text(font, "you", me_x - 6.0, me_y + 30.0, 16, ui::PAPER);
        let ahead = d.s("queue_ahead", &[("n", &q.ahead.to_string())]);
        ui::text(font, &ahead, 16.0, 186.0, 16, ui::PAPER);

        ui::message(font, &self.msg);
        let labels = [
            d.s("act_wait", &[]),
            d.s("act_confront", &[]),
            d.s("act_cut", &[]),
            match q.fixer_price() {
                Some(p) => d.s("act_fixer", &[("price", &p.to_string())]),
                None => d.s("act_fixer_none", &[]),
            },
        ];
        ui::choice_buttons(font, &labels, ctx.mouse, |i| q.can(ACTIONS[i]));
        ui::hint(font, &d.s("hint_walkout", &[]));
    }
}
