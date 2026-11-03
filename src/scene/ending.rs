use macroquad::prelude::*;

use super::{confirm_pressed, Ctx, Scene, Title};
use crate::canvas;
use crate::game::grade::{score, Scored};
use crate::game::rng::seed_to_code;
use crate::game::run::{Run, Status};
use crate::ui;

pub struct Ending {
    run: Run,
    scored: Scored,
}

impl Ending {
    pub fn new(run: Run) -> Self {
        let scored = score(&run.meters, &run.par, run.status() == Status::Complete);
        Self { run, scored }
    }

    pub fn update(&mut self, _ctx: &Ctx) -> Option<Scene> {
        confirm_pressed().then_some(Scene::Title(Title))
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let letter = self.scored.grade.letter();
        clear_background(ui::PAPER);
        ui::panel(60.0, 30.0, canvas::WIDTH - 120.0, 300.0);
        ui::text_centered(font, letter, canvas::WIDTH / 2.0, 110.0, 64, ui::STAMP);
        ui::text_centered(
            font,
            &d.s(&format!("grade_{letter}_title"), &[]),
            canvas::WIDTH / 2.0,
            140.0,
            16,
            ui::INK,
        );
        ui::text_wrapped(
            font,
            &d.s(&format!("grade_{letter}_line"), &[]),
            90.0,
            170.0,
            canvas::WIDTH - 180.0,
            16,
            ui::INK,
        );
        let m = &self.run.meters;
        let rows = [
            format!("Days: {} (par {})", m.days, self.run.par.par_days),
            format!("Money left: Kv {} of {}", m.money, self.run.par.budget),
            format!("Conscience: {}", m.conscience),
            format!("Score: {}", self.scored.total),
        ];
        for (i, row) in rows.iter().enumerate() {
            ui::text(font, row, 90.0, 230.0 + 18.0 * i as f32, 16, ui::INK);
        }
        let seed = d.s("ending_seed", &[("seed", &seed_to_code(self.run.seed))]);
        ui::text(font, &seed, 380.0, 230.0, 16, ui::FADED);
        ui::hint(font, &d.s("ending_restart", &[]));
    }
}
