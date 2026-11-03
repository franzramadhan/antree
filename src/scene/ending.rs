use macroquad::prelude::*;

use super::{confirm_pressed, Ctx, QuestSelect, Scene};
use crate::canvas;
use crate::data::Data;
use crate::game::grade::{score, Grade, Scored};
use crate::game::rng::seed_to_code;
use crate::game::run::{Run, Status};
use crate::save::{Best, Save};
use crate::ui;

pub struct Ending {
    run: Run,
    scored: Scored,
    note: String,
    /// Title of the quest this run unlocked for the first time.
    unlocked: Option<String>,
}

impl Ending {
    /// Grades the run and writes the save.
    pub fn new(run: Run, d: &Data) -> Self {
        let scored = score(&run.meters, &run.par, run.status() == Status::Complete);
        let index = d.quests.iter().position(|q| q.id == run.quest_id).unwrap();
        let next = d.quests.get(index + 1);
        let mut save = Save::load();
        let was_locked = next.is_some_and(|q| !save.is_unlocked(index + 1, &q.id));
        save.record(
            &run.quest_id,
            next.map(|q| q.id.as_str()),
            Best {
                grade: scored.grade,
                total: scored.total,
                days: run.meters.days,
                money: run.meters.money,
                conscience: run.meters.conscience,
                seed: seed_to_code(run.seed),
            },
        );
        save.store();
        let unlocked = next
            .filter(|q| was_locked && save.is_unlocked(index + 1, &q.id))
            .map(|q| q.title.clone());
        let note = d.line("common", "clerk_note", run.seed as usize);
        Self {
            run,
            scored,
            note,
            unlocked,
        }
    }

    pub fn update(&mut self, _ctx: &Ctx) -> Option<Scene> {
        confirm_pressed().then(|| Scene::QuestSelect(QuestSelect::new()))
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let letter = self.scored.grade.letter();
        clear_background(ui::PAPER);
        ui::panel(60.0, 30.0, canvas::WIDTH - 120.0, 300.0);
        let color = if self.scored.grade == Grade::X {
            ui::INK
        } else {
            ui::STAMP
        };
        ui::text_centered(font, letter, canvas::WIDTH / 2.0, 100.0, 64, color);
        ui::text_centered(
            font,
            &d.s(&format!("grade_{letter}_title"), &[]),
            canvas::WIDTH / 2.0,
            128.0,
            16,
            ui::INK,
        );
        let y = ui::text_wrapped(
            font,
            &d.s(&format!("grade_{letter}_line"), &[]),
            90.0,
            156.0,
            canvas::WIDTH - 180.0,
            16,
            ui::INK,
        );
        ui::text_wrapped(
            font,
            &self.note,
            90.0,
            y + 4.0,
            canvas::WIDTH - 180.0,
            16,
            ui::FADED,
        );
        let m = &self.run.meters;
        let rows = [
            format!("Days: {} (par {})", m.days, self.run.par.par_days),
            format!("Money left: Kv {} of {}", m.money, self.run.par.budget),
            format!("Conscience: {}", m.conscience),
            format!("Score: {}", self.scored.total),
        ];
        for (i, row) in rows.iter().enumerate() {
            ui::text(font, row, 90.0, 236.0 + 18.0 * i as f32, 16, ui::INK);
        }
        let seed = d.s("ending_seed", &[("seed", &seed_to_code(self.run.seed))]);
        ui::text(font, &seed, 380.0, 236.0, 16, ui::FADED);
        if let Some(q) = &self.unlocked {
            ui::text(
                font,
                &d.s("ending_unlocked", &[("quest", q)]),
                380.0,
                254.0,
                16,
                ui::WALL,
            );
        }
        ui::hint(font, &d.s("ending_restart", &[]));
    }
}
