use macroquad::prelude::*;

use super::{confirm_pressed, Ctx, Ending, Office, Scene};
use crate::canvas;
use crate::data::OfficeDef;
use crate::game::run::{Run, Status};
use crate::ui;

const SPEED: f32 = 110.0;
const TRAVEL_MINUTES: u32 = 15;

pub struct Overworld {
    run: Run,
    player: Vec2,
    toast: Option<String>,
}

impl Overworld {
    pub fn at(run: Run, player: Vec2, toast: Option<String>) -> Self {
        Self { run, player, toast }
    }

    fn nearby<'a>(&self, offices: &'a [OfficeDef]) -> Option<&'a OfficeDef> {
        offices.iter().find(|o| {
            Rect::new(
                o.pos.0 - 10.0,
                o.pos.1 - 10.0,
                o.size.0 + 20.0,
                o.size.1 + 20.0,
            )
            .contains(self.player)
        })
    }

    pub fn update(&mut self, ctx: &Ctx) -> Option<Scene> {
        let d = &ctx.assets.data;
        if self.run.status() != Status::Playing {
            return Some(Scene::Ending(Ending::new(self.run.clone(), d)));
        }

        let mut dir = Vec2::ZERO;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            dir.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            dir.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            dir.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            dir.x += 1.0;
        }
        if dir != Vec2::ZERO {
            self.toast = None;
            self.player += dir.normalize() * SPEED * ctx.dt;
            self.player.x = self.player.x.clamp(8.0, canvas::WIDTH - 8.0);
            self.player.y = self.player.y.clamp(ui::HUD_H + 8.0, canvas::HEIGHT - 8.0);
        }

        if is_key_pressed(KeyCode::H) {
            self.run.end_day();
            self.toast = Some(d.s("day_ended", &[]));
            return None;
        }

        if confirm_pressed() {
            if let Some(office) = self.nearby(&d.offices) {
                let id = office.id.clone();
                if self.run.advance(TRAVEL_MINUTES) {
                    self.run.end_day();
                    self.toast = Some(d.s("travel_closed", &[]));
                    return None;
                }
                let run = self.run.clone();
                return Some(Scene::Office(Office::new(run, id, self.player, d)));
            }
        }
        None
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        clear_background(ui::GRASS);
        draw_rectangle(0.0, 168.0, canvas::WIDTH, 28.0, ui::ROAD);
        draw_rectangle(300.0, ui::HUD_H, 40.0, canvas::HEIGHT, ui::ROAD);

        let near = self.nearby(&d.offices).map(|o| o.id.as_str());
        for o in &d.offices {
            let (x, y) = o.pos;
            let (w, h) = o.size;
            draw_rectangle(x, y, w, h, ui::PAPER);
            draw_rectangle(x, y, w, 14.0, ui::WALL);
            draw_rectangle(x + w / 2.0 - 8.0, y + h - 18.0, 16.0, 18.0, ui::WOOD);
            let outline = if near == Some(o.id.as_str()) {
                ui::STAMP
            } else {
                ui::INK
            };
            draw_rectangle_lines(x, y, w, h, 2.0, outline);
            ui::text_centered(font, &o.short, x + w / 2.0, y + h / 2.0 + 4.0, 16, ui::INK);
        }

        draw_rectangle(
            self.player.x - 5.0,
            self.player.y - 7.0,
            10.0,
            14.0,
            ui::INK,
        );
        draw_rectangle(
            self.player.x - 3.0,
            self.player.y - 11.0,
            6.0,
            6.0,
            Color::new(0.85, 0.65, 0.5, 1.0),
        );

        self.draw_board(ctx);
        ui::hud(font, d, &self.run);
        match (&self.toast, near) {
            (Some(t), _) => {
                let lines = ui::wrap(font, t, 388.0, 16).len() as f32;
                ui::panel(216.0, 60.0, 408.0, 16.0 + lines * 20.0);
                ui::text_wrapped(font, t, 226.0, 80.0, 388.0, 16, ui::INK);
                ui::hint(font, &d.s("controls_overworld", &[]));
            }
            (None, Some(_)) => ui::hint(font, &d.s("hint_enter", &[])),
            (None, None) => ui::hint(font, &d.s("controls_overworld", &[])),
        }
    }

    fn draw_board(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let quest = d.quest(&self.run.quest_id);
        let board = self.run.board(&quest.board);
        let (x, y, w) = (8.0, 204.0, 284.0);
        let h = 40.0 + 15.0 * board.len() as f32;
        ui::panel(x, y, w, h);
        ui::text(font, &quest.title, x + 8.0, y + 16.0, 16, ui::INK);
        ui::text(
            font,
            &d.s("board_title", &[]),
            x + 8.0,
            y + 31.0,
            16,
            ui::FADED,
        );
        for (i, doc) in board.iter().enumerate() {
            let mark = if self.run.inv.has(doc) { "[x]" } else { "[ ]" };
            let line = format!("{mark} {}", d.doc_name(doc));
            ui::text(
                font,
                &line,
                x + 8.0,
                y + 47.0 + 15.0 * i as f32,
                16,
                ui::INK,
            );
        }
    }
}
