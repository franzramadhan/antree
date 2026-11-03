use macroquad::prelude::*;

use super::{pass_time, Outcome};
use crate::canvas;
use crate::data::Data;
use crate::game::form::{DateFmt, Form, MAX_REJECTIONS, REJECT_MINUTES, TIME_LIMIT};
use crate::game::run::Run;
use crate::scene::{say, Ctx};
use crate::ui;

const NONSENSE_CONSCIENCE: i32 = 5;
const ROW_H: f32 = 21.0;
const TOP: f32 = 80.0;

pub struct FormGame {
    form: Form,
    picks: Vec<usize>,
    cursor: usize,
    time_left: f32,
    rejections: u32,
    msg: String,
}

fn row_rect(i: usize) -> Rect {
    Rect::new(
        44.0,
        TOP + i as f32 * ROW_H,
        canvas::WIDTH - 88.0,
        ROW_H - 2.0,
    )
}

impl FormGame {
    pub fn new(run: &Run, office: &str, d: &Data) -> Self {
        let form = Form::new(&mut run.visit_rng("form", office), &d.form_fields);
        Self {
            picks: vec![0; form.fields.len()],
            form,
            cursor: 0,
            time_left: TIME_LIMIT,
            rejections: 0,
            msg: say(d, office, "f_intro", run),
        }
    }

    pub fn update(&mut self, ctx: &Ctx, run: &mut Run, office: &str) -> Outcome {
        let d = &ctx.assets.data;
        let n = self.form.fields.len();
        self.time_left -= ctx.dt;

        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.cursor = (self.cursor + n - 1) % n;
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.cursor = (self.cursor + 1) % n;
        }
        let pick = &mut self.picks[self.cursor];
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            *pick = (*pick + 1) % 3;
        }
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            *pick = (*pick + 2) % 3;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(i) = (0..n).find(|&i| row_rect(i).contains(ctx.mouse)) {
                self.cursor = i;
                self.picks[i] = (self.picks[i] + 1) % 3;
            }
        }
        if is_key_pressed(KeyCode::N) {
            run.meters.shortcut(NONSENSE_CONSCIENCE);
            run.nonsense_form();
            return Outcome::Passed(Some(say(d, office, "f_nonsense", run)));
        }

        let timed_out = self.time_left <= 0.0;
        if !(timed_out || is_key_pressed(KeyCode::Enter)) {
            return Outcome::Running;
        }
        let wrong = self.form.wrong(&self.picks);
        if wrong == 0 && !timed_out {
            return Outcome::Passed(None);
        }
        self.rejections += 1;
        if let Some(closed) = pass_time(run, REJECT_MINUTES, d, office) {
            return closed;
        }
        if self.rejections >= MAX_REJECTIONS {
            return Outcome::Failed {
                line: say(d, office, "f_tomorrow", run),
                end_day: true,
            };
        }
        self.time_left = TIME_LIMIT;
        self.msg = if timed_out {
            say(d, office, "f_timeout", run)
        } else {
            say(d, office, "f_rejected", run).replace("{n}", &wrong.to_string())
        };
        Outcome::Running
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        ui::panel(32.0, 30.0, canvas::WIDTH - 64.0, 312.0);
        ui::text(font, &d.s("form_title", &[]), 44.0, 50.0, 16, ui::INK);
        let secs = d.s(
            "form_timer",
            &[("s", &(self.time_left.max(0.0).ceil() as u32).to_string())],
        );
        let timer_color = if self.time_left < 10.0 {
            ui::STAMP
        } else {
            ui::INK
        };
        ui::text(font, &secs, canvas::WIDTH - 90.0, 50.0, 16, timer_color);
        let r = self.form.rules;
        let date = match r.date {
            DateFmt::DdMmYyyy => "date_0",
            DateFmt::YyyyMmDd => "date_1",
            DateFmt::DdMmYy => "date_2",
        };
        let rules = d.s(
            "form_rules",
            &[
                ("date", &d.s(date, &[])),
                (
                    "case",
                    &d.s(
                        if r.capitals {
                            "case_caps"
                        } else {
                            "case_normal"
                        },
                        &[],
                    ),
                ),
                (
                    "dash",
                    &d.s(if r.dashes { "dash_yes" } else { "dash_no" }, &[]),
                ),
            ],
        );
        ui::text(font, &rules, 44.0, 70.0, 16, ui::STAMP);
        for (i, f) in self.form.fields.iter().enumerate() {
            ui::picker_row(
                font,
                &f.label,
                &f.options[self.picks[i]],
                row_rect(i),
                i == self.cursor,
            );
        }
        ui::text_wrapped(
            font,
            &self.msg,
            44.0,
            316.0,
            canvas::WIDTH - 88.0,
            16,
            ui::INK,
        );
        ui::hint(font, &d.s("form_hint", &[]));
    }
}
