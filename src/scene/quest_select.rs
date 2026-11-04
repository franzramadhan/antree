use macroquad::prelude::*;

use super::{draw_town, escape_pressed, Ctx, Overworld, Scene, Title, Tutorial};
use crate::canvas;
use crate::game::rng::code_to_seed;
use crate::game::run::Run;
use crate::save::Save;
use crate::ui;

const START: Vec2 = Vec2::new(320.0, 182.0);

pub struct QuestSelect {
    save: Save,
    cursor: usize,
    seed: String,
}

fn row_rect(i: usize) -> Rect {
    Rect::new(60.0, 70.0 + i as f32 * 30.0, canvas::WIDTH - 120.0, 26.0)
}

impl QuestSelect {
    pub fn new() -> Self {
        Self {
            save: Save::load(),
            cursor: 0,
            seed: String::new(),
        }
    }

    pub fn update(&mut self, ctx: &Ctx) -> Option<Scene> {
        let d = &ctx.assets.data;
        let n = d.quests.len();
        if escape_pressed() {
            return Some(Scene::Title(Title::new()));
        }
        if is_key_pressed(KeyCode::Tab) {
            return Some(Scene::Tutorial(Tutorial::new(d)));
        }
        if is_key_pressed(KeyCode::Up) {
            self.cursor = (self.cursor + n - 1) % n;
        }
        if is_key_pressed(KeyCode::Down) {
            self.cursor = (self.cursor + 1) % n;
        }
        while let Some(c) = get_char_pressed() {
            if c.is_ascii_alphanumeric() && self.seed.len() < 6 {
                self.seed.push(c.to_ascii_uppercase());
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.seed.pop();
        }
        let mut start = is_key_pressed(KeyCode::Enter);
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(i) = (0..n).find(|&i| row_rect(i).contains(ctx.mouse)) {
                start = i == self.cursor;
                self.cursor = i;
            }
        }
        let quest = &d.quests[self.cursor];
        if !start || !self.save.is_unlocked(self.cursor, &quest.id) {
            return None;
        }
        let seed = code_to_seed(&self.seed)
            .unwrap_or_else(|| (miniquad::date::now() * 1000.0) as u64)
            & ((1 << 30) - 1);
        let run = Run::new(seed, quest, d.all_grants(), self.save.inside_man);
        Some(Scene::Overworld(Overworld::at(
            run,
            START,
            Some(quest.intro.clone()),
        )))
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        draw_town(ctx, None, None);
        ui::panel(44.0, 22.0, canvas::WIDTH - 88.0, 300.0);
        ui::text_centered(
            font,
            &d.s("qs_title", &[]),
            canvas::WIDTH / 2.0,
            44.0,
            16,
            ui::INK,
        );
        for (i, q) in d.quests.iter().enumerate() {
            let r = row_rect(i);
            let open = self.save.is_unlocked(i, &q.id);
            if i == self.cursor {
                draw_rectangle(r.x, r.y, r.w, r.h, Color::new(1.0, 0.98, 0.9, 1.0));
                draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, ui::INK);
            }
            let color = if open { ui::INK } else { ui::FADED };
            ui::text(
                font,
                &format!("{}. {}", i + 1, q.title),
                r.x + 8.0,
                r.y + 18.0,
                16,
                color,
            );
            let status = match (open, self.save.best.get(&q.id)) {
                (false, _) => d.s("qs_locked", &[]),
                (true, None) => d.s("qs_new", &[]),
                (true, Some(b)) => d.s(
                    "qs_best",
                    &[("grade", b.grade.letter()), ("total", &b.total.to_string())],
                ),
            };
            ui::text(font, &status, r.x + r.w - 160.0, r.y + 18.0, 16, color);
        }
        let y = row_rect(d.quests.len()).y + 20.0;
        let seed = if self.seed.is_empty() {
            d.s("qs_seed_random", &[])
        } else {
            format!("{:_<6}", self.seed)
        };
        ui::text(
            font,
            &d.s("qs_seed", &[("seed", &seed)]),
            60.0,
            y,
            16,
            ui::INK,
        );
        ui::text(
            font,
            &d.s("qs_seed_hint", &[]),
            60.0,
            y + 18.0,
            16,
            ui::FADED,
        );
        if self.save.inside_man {
            ui::text(
                font,
                &d.s("qs_inside_man", &[]),
                60.0,
                y + 40.0,
                16,
                ui::WALL,
            );
        }
        ui::hint(font, &d.s("qs_hint", &[]));
    }
}
