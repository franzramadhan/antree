use macroquad::prelude::*;

use super::{draw_town, Ctx, Overworld, QuestSelect, Scene, Tutorial};
use crate::art::{self, Facing};
use crate::canvas;
use crate::fx::Slam;
use crate::save::{self, Save};
use crate::ui;

/// Menu panel: x, y, width.
const MENU: (f32, f32, f32) = (220.0, 124.0, 200.0);
const SIGN: Rect = Rect::new(170.0, 30.0, 300.0, 78.0);

#[derive(Clone, Copy)]
enum Item {
    Continue,
    New,
    Orientation,
    Sound,
    Language,
}

pub struct Title {
    slam: Slam,
    cursor: usize,
    items: Vec<Item>,
}

impl Title {
    pub fn new() -> Self {
        let mut items = vec![Item::New, Item::Orientation, Item::Sound, Item::Language];
        if save::load_run(vec![]).is_some() {
            items.insert(0, Item::Continue);
        }
        Self {
            slam: Slam::approve(),
            cursor: 0,
            items,
        }
    }

    pub fn update(&mut self, ctx: &Ctx) -> Option<Scene> {
        self.slam.update(ctx.fx, ctx.dt);
        let (x, y, w) = MENU;
        let picked = ui::menu_input(&mut self.cursor, self.items.len(), ctx.mouse, x, y, w)?;
        let d = &ctx.assets.data;
        match self.items[picked] {
            Item::Continue => {
                let (run, p) = save::load_run(d.all_grants())?;
                let toast = Some(d.s("progress_loaded", &[]));
                Some(Scene::Overworld(Overworld::at(run, vec2(p.0, p.1), toast)))
            }
            Item::New if Save::load().tutorial_done => Some(Scene::QuestSelect(QuestSelect::new())),
            Item::New | Item::Orientation => Some(Scene::Tutorial(Tutorial::new(d))),
            Item::Sound => {
                ctx.fx.toggle_mute();
                None
            }
            Item::Language => {
                let mut save = Save::load();
                save.lang = save.lang.next();
                save.store();
                let mut title = Title::new();
                title.cursor = picked;
                Some(Scene::Title(title))
            }
        }
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let t = get_time() as f32;
        // Someone always heading off to queue somewhere.
        let walker = vec2((t * 45.0) % 720.0 - 40.0, 188.0);
        draw_town(
            ctx,
            None,
            Some((Facing::Right, art::walk_frame(Some(t)), walker)),
        );

        let s = SIGN;
        draw_rectangle(s.x + 3.0, s.y + 3.0, s.w, s.h, Color { a: 0.35, ..BLACK });
        draw_rectangle(s.x, s.y, s.w, s.h, ui::WOOD);
        draw_rectangle(
            s.x + 4.0,
            s.y + 4.0,
            s.w - 8.0,
            s.h - 8.0,
            Color::from_hex(0xb07440),
        );
        draw_rectangle_lines(s.x, s.y, s.w, s.h, 2.0, ui::INK);
        let cx = s.center().x;
        let title = d.s("title", &[]);
        ui::text_centered(font, &title, cx + 2.0, s.y + 50.0, 48, ui::INK);
        ui::text_centered(font, &title, cx, s.y + 48.0, 48, ui::PAPER);
        ui::text_centered(font, &d.s("subtitle", &[]), cx, s.y + 68.0, 16, ui::PAPER);
        self.slam
            .draw_texture(&ctx.assets.stamp, s.right() - 18.0, s.y + 14.0, 44.0, -0.3);

        let labels: Vec<String> = self
            .items
            .iter()
            .map(|i| match i {
                Item::Continue => d.s("menu_continue", &[]),
                Item::New => d.s("menu_new", &[]),
                Item::Orientation => d.s("menu_orientation", &[]),
                Item::Sound if ctx.fx.muted() => d.s("menu_sound_off", &[]),
                Item::Sound => d.s("menu_sound_on", &[]),
                Item::Language => d.s("menu_lang", &[]),
            })
            .collect();
        let (x, y, w) = MENU;
        ui::menu(font, &labels, self.cursor, x, y, w);

        let disclaimer = d.s("disclaimer", &[]);
        let lines = ui::wrap(font, &disclaimer, canvas::WIDTH - 100.0, 16).len() as f32;
        let top = 312.0 - lines * 20.0;
        ui::panel(40.0, top, canvas::WIDTH - 80.0, 6.0 + lines * 20.0);
        ui::text_wrapped(
            font,
            &disclaimer,
            50.0,
            top + 16.0,
            canvas::WIDTH - 100.0,
            16,
            ui::FADED,
        );
        ui::hint(font, &d.s("menu_hint", &[]));
    }
}
