use macroquad::prelude::*;

use super::{cannot_pay, pass_time, Outcome};
use crate::canvas;
use crate::data::Data;
use crate::fx::Sfx;
use crate::game::offline::{Act, SystemDown, COFFEE};
use crate::game::run::Run;
use crate::scene::{say, Ctx};
use crate::ui;

pub struct OfflineGame {
    sys: SystemDown,
    msg: String,
}

impl OfflineGame {
    pub fn new(run: &Run, office: &str, d: &Data) -> Self {
        Self {
            sys: SystemDown::new(&mut run.visit_rng("offline", office)),
            msg: say(d, office, "ob_system", run),
        }
    }

    pub fn update(&mut self, ctx: &Ctx, run: &mut Run, office: &str) -> Outcome {
        let d = &ctx.assets.data;
        let Some(i) = ui::choice_input(4, ctx.mouse, |i| i != 2 || !self.sys.revealed) else {
            return Outcome::Running;
        };
        let act = match i {
            0 => Act::Wait,
            1 => Act::Refresh,
            2 if !run.meters.can_afford(COFFEE) => {
                self.msg = cannot_pay(d, office, run, COFFEE);
                return Outcome::Running;
            }
            2 => Act::AskHonorer,
            _ => {
                return Outcome::Failed {
                    line: say(d, office, "o_home", run),
                    end_day: true,
                }
            }
        };
        if act == Act::Refresh {
            ctx.fx.play(Sfx::Dialup);
        }
        let tick = self.sys.act(act);
        run.meters.spend(tick.money);
        if let Some(closed) = pass_time(run, tick.minutes, d, office) {
            return closed;
        }
        if self.sys.done() {
            return Outcome::Passed(Some(say(d, office, "o_back", run)));
        }
        self.msg =
            say(d, office, tick.key, run).replace("{min}", &self.sys.remaining().to_string());
        Outcome::Running
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        ui::panel(120.0, 80.0, canvas::WIDTH - 240.0, 90.0);
        ui::text(font, &d.s("off_title", &[]), 136.0, 102.0, 16, ui::INK);
        let pct = self.sys.shown_percent();
        let w = canvas::WIDTH - 272.0;
        draw_rectangle(136.0, 116.0, w, 14.0, ui::INK);
        draw_rectangle(136.0, 116.0, w * pct as f32 / 100.0, 14.0, ui::WALL);
        ui::text(
            font,
            &format!("{pct}%"),
            canvas::WIDTH - 180.0,
            102.0,
            16,
            ui::INK,
        );
        if self.sys.revealed {
            let known = d.s("off_known", &[("min", &self.sys.remaining().to_string())]);
            ui::text(font, &known, 136.0, 154.0, 16, ui::FADED);
        }
        ui::message(font, &self.msg);
        let labels = ["off_wait", "off_refresh", "off_honorer", "off_home"].map(|k| d.s(k, &[]));
        ui::choice_buttons(font, &labels, ctx.mouse, |i| i != 2 || !self.sys.revealed);
        ui::hint(font, &d.s("hint_walkout", &[]));
    }
}
