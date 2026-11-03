use macroquad::prelude::*;

use super::{confirm_pressed, Ctx, Overworld, Scene};
use crate::canvas;
use crate::game::run::Run;
use crate::ui;

pub struct Title;

impl Title {
    pub fn update(&mut self, ctx: &Ctx) -> Option<Scene> {
        if !confirm_pressed() {
            return None;
        }
        let quest = &ctx.assets.data.quests[0];
        // ponytail: seed from the wall clock, a seed entry field comes with Phase 4.
        let seed = (get_time() * 1000.0) as u64 & ((1 << 30) - 1);
        let run = Run::new(seed, &quest.goal_doc, quest.par());
        Some(Scene::Overworld(Overworld::at(
            run,
            vec2(300.0, 300.0),
            Some(quest.intro.clone()),
        )))
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        clear_background(ui::PAPER);
        draw_rectangle(0.0, 0.0, canvas::WIDTH, 8.0, ui::WALL);
        draw_rectangle(0.0, canvas::HEIGHT - 8.0, canvas::WIDTH, 8.0, ui::WALL);
        ui::text_centered(
            font,
            &d.s("title", &[]),
            canvas::WIDTH / 2.0,
            120.0,
            64,
            ui::INK,
        );
        draw_texture_ex(
            &ctx.assets.stamp,
            canvas::WIDTH / 2.0 + 96.0,
            56.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(48.0, 48.0)),
                rotation: -0.3,
                ..Default::default()
            },
        );
        ui::text_centered(
            font,
            &d.s("subtitle", &[]),
            canvas::WIDTH / 2.0,
            160.0,
            16,
            ui::FADED,
        );
        ui::text_centered(
            font,
            &d.s("press_start", &[]),
            canvas::WIDTH / 2.0,
            230.0,
            16,
            ui::INK,
        );
        ui::text_wrapped(
            font,
            &d.s("disclaimer", &[]),
            40.0,
            316.0,
            canvas::WIDTH - 80.0,
            16,
            ui::FADED,
        );
    }
}
