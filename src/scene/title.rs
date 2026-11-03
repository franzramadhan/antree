use macroquad::prelude::*;

use super::{confirm_pressed, Ctx, QuestSelect, Scene, Tutorial};
use crate::canvas;
use crate::fx::Slam;
use crate::save::Save;
use crate::ui;

pub struct Title {
    slam: Slam,
}

impl Title {
    pub fn new() -> Self {
        Self {
            slam: Slam::approve(),
        }
    }

    pub fn update(&mut self, ctx: &Ctx) -> Option<Scene> {
        self.slam.update(ctx.fx, ctx.dt);
        if !confirm_pressed() {
            return None;
        }
        Some(if Save::load().tutorial_done {
            Scene::QuestSelect(QuestSelect::new())
        } else {
            Scene::Tutorial(Tutorial::new(&ctx.assets.data))
        })
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
        self.slam.draw_texture(
            &ctx.assets.stamp,
            canvas::WIDTH / 2.0 + 120.0,
            80.0,
            48.0,
            -0.3,
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
        ui::text_centered(
            font,
            &d.s("sound_hint", &[]),
            canvas::WIDTH / 2.0,
            252.0,
            16,
            ui::FADED,
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
