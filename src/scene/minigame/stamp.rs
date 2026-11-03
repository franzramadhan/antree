use macroquad::prelude::*;

use super::{pass_time, Outcome};
use crate::canvas;
use crate::data::Data;
use crate::game::run::Run;
use crate::game::stamp::{Holder, StampHunt, Visit, GIVE_UP_AT, VISIT_MINUTES};
use crate::scene::{say, Ctx};
use crate::ui;

pub struct StampGame {
    hunt: StampHunt,
    /// The holder was found away from their desk.
    away: bool,
    msg: String,
}

fn desk_rect(i: usize) -> Rect {
    Rect::new(
        40.0 + (i % 5) as f32 * 116.0,
        70.0 + (i / 5) as f32 * 64.0,
        92.0,
        44.0,
    )
}

impl StampGame {
    pub fn new(run: &Run, office: &str, d: &Data) -> Self {
        Self {
            hunt: StampHunt::new(&mut run.visit_rng("stamp", office)),
            away: false,
            msg: say(d, office, "s_intro", run),
        }
    }

    pub fn update(&mut self, ctx: &Ctx, run: &mut Run, office: &str) -> Outcome {
        let d = &ctx.assets.data;
        if self.away && is_key_pressed(KeyCode::W) {
            let mins = self.hunt.wait();
            if let Some(closed) = pass_time(run, mins, d, office) {
                return closed;
            }
            return Outcome::Passed(Some(say(d, office, "s_waited", run)));
        }
        let clicked = is_mouse_button_pressed(MouseButton::Left)
            .then(|| (0..self.hunt.desks).find(|&i| desk_rect(i).contains(ctx.mouse)))
            .flatten();
        let Some(desk) = ui::number_pressed()
            .or(clicked)
            .filter(|&i| i < self.hunt.desks)
        else {
            return Outcome::Running;
        };
        if let Some(closed) = pass_time(run, VISIT_MINUTES, d, office) {
            return closed;
        }
        if run.clock.minutes >= GIVE_UP_AT {
            return Outcome::Failed {
                line: say(d, office, "s_late", run),
                end_day: false,
            };
        }
        self.away = false;
        match self.hunt.visit(desk) {
            Visit::Clue(n) => {
                self.msg = say(d, office, "s_clue", run).replace("{desk}", &(n + 1).to_string());
            }
            Visit::Stamped => return Outcome::Passed(None),
            Visit::Away(why, mins) => {
                self.away = true;
                let key = if why == Holder::OnBreak {
                    "s_break"
                } else {
                    "s_meeting"
                };
                self.msg = say(d, office, key, run).replace("{min}", &mins.to_string());
            }
            Visit::Gone => {
                return Outcome::Failed {
                    line: say(d, office, "s_field", run),
                    end_day: false,
                }
            }
        }
        Outcome::Running
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        draw_rectangle(16.0, 30.0, canvas::WIDTH - 32.0, 160.0, ui::WALL);
        ui::text(font, &d.s("stamp_title", &[]), 24.0, 50.0, 16, ui::PAPER);
        for i in 0..self.hunt.desks {
            let r = desk_rect(i);
            let fill = if self.hunt.visited[i] {
                ui::FADED
            } else {
                ui::WOOD
            };
            draw_rectangle(r.x, r.y, r.w, r.h, fill);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, ui::INK);
            ui::text_centered(
                font,
                &(i + 1).to_string(),
                r.x + r.w / 2.0,
                r.y + 28.0,
                16,
                ui::PAPER,
            );
        }
        ui::message(font, &self.msg);
        ui::hint(font, &d.s("stamp_hint", &[]));
    }
}
