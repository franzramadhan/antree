use macroquad::prelude::*;

use super::{cannot_pay, pass_time, Outcome};
use crate::canvas;
use crate::data::Data;
use crate::game::run::Run;
use crate::game::theory::{Test, FEE, LENGTH};
use crate::scene::{say, Ctx};
use crate::ui;

/// Minutes the whole test takes.
const TEST_MINUTES: u32 = 30;

pub struct TheoryGame {
    test: Test,
    paid: bool,
    msg: String,
}

fn answer_rect(i: usize) -> Rect {
    Rect::new(40.0, 120.0 + i as f32 * 30.0, canvas::WIDTH - 80.0, 26.0)
}

impl TheoryGame {
    pub fn new(run: &mut Run, office: &str, d: &Data) -> Self {
        Self {
            test: Test::new(&mut run.visit_rng("theory", office), d.theory.len()),
            paid: run.meters.spend(FEE),
            msg: say(d, office, "t_intro", run),
        }
    }

    pub fn update(&mut self, ctx: &Ctx, run: &mut Run, office: &str) -> Outcome {
        let d = &ctx.assets.data;
        if !self.paid {
            return Outcome::Failed {
                line: cannot_pay(d, office, run, FEE),
                end_day: false,
            };
        }
        let n = self.test.current(&d.theory).options.len();
        let clicked = is_mouse_button_pressed(MouseButton::Left)
            .then(|| (0..n).find(|&i| answer_rect(i).contains(ctx.mouse)))
            .flatten();
        let Some(choice) = ui::number_pressed().or(clicked).filter(|&i| i < n) else {
            return Outcome::Running;
        };
        self.test.answer(&d.theory, choice);
        if !self.test.done() {
            return Outcome::Running;
        }
        if let Some(closed) = pass_time(run, TEST_MINUTES, d, office) {
            return closed;
        }
        if self.test.passed() {
            Outcome::Passed(Some(say(d, office, "t_pass", run)))
        } else {
            Outcome::Failed {
                line: say(d, office, "t_fail", run)
                    .replace("{score}", &self.test.correct.to_string()),
                end_day: true,
            }
        }
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        ui::panel(24.0, 30.0, canvas::WIDTH - 48.0, 300.0);
        if self.test.done() {
            return;
        }
        let n = (self.test.index + 1).min(LENGTH).to_string();
        ui::text(
            font,
            &d.s("theory_title", &[("n", &n)]),
            40.0,
            50.0,
            16,
            ui::FADED,
        );
        let q = self.test.current(&d.theory);
        ui::text_wrapped(font, &q.q, 40.0, 74.0, canvas::WIDTH - 80.0, 16, ui::INK);
        for (i, opt) in q.options.iter().enumerate() {
            let r = answer_rect(i);
            let hover = r.contains(ctx.mouse);
            if hover {
                draw_rectangle(r.x, r.y, r.w, r.h, Color::new(1.0, 0.98, 0.9, 1.0));
            }
            ui::text(
                font,
                &format!("{}. {opt}", i + 1),
                r.x + 6.0,
                r.y + 18.0,
                16,
                ui::INK,
            );
        }
        if self.test.index == 0 {
            ui::text_wrapped(
                font,
                &self.msg,
                40.0,
                262.0,
                canvas::WIDTH - 80.0,
                16,
                ui::FADED,
            );
        }
        ui::hint(font, &d.s("theory_hint", &[]));
    }
}
