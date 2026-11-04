use macroquad::prelude::*;

use super::{confirm_pressed, escape_pressed, Ctx, Ending, Office, Scene, Title};
use crate::art::{self, Facing, Sprites};
use crate::canvas;
use crate::data::OfficeDef;
use crate::fx::Sfx;
use crate::game::run::{Run, Status};
use crate::save;
use crate::ui;

const SPEED: f32 = 110.0;
const TRAVEL_MINUTES: u32 = 15;
/// Pause menu panel: x, y, width.
const MENU: (f32, f32, f32) = (220.0, 120.0, 200.0);
const ROAD_H: Rect = Rect::new(0.0, 168.0, canvas::WIDTH, 28.0);
const ROAD_V: Rect = Rect::new(300.0, ui::HUD_H, 40.0, canvas::HEIGHT);
const ROOF_H: f32 = 22.0;
/// Trunk bases. Trees are solid, bushes are not.
const TREES: [(f32, f32); 9] = [
    (372.0, 238.0),
    (424.0, 266.0),
    (480.0, 230.0),
    (604.0, 238.0),
    (562.0, 284.0),
    (392.0, 318.0),
    (454.0, 336.0),
    (516.0, 306.0),
    (618.0, 330.0),
];
const BUSHES: [(f32, f32); 4] = [
    (356.0, 212.0),
    (628.0, 210.0),
    (536.0, 340.0),
    (446.0, 296.0),
];
/// Feet position and sheet row of the townsfolk who stand around.
const VILLAGERS: [(f32, f32); 3] = [(246.0, 188.0), (470.0, 192.0), (586.0, 258.0)];

pub struct Overworld {
    run: Run,
    player: Vec2,
    toast: Option<String>,
    facing: Facing,
    walking: Option<f32>,
    /// Cursor of the pause menu while it is open.
    menu: Option<usize>,
}

/// One thing on the map, drawn back to front by the y of its base.
enum Prop {
    Office(usize),
    Tree(Vec2),
    Bush(Vec2),
    Villager(usize),
    Player,
}

fn trunk(t: (f32, f32)) -> Rect {
    Rect::new(t.0 - 5.0, t.1 - 4.0, 10.0, 5.0)
}

impl Overworld {
    pub fn at(run: Run, player: Vec2, toast: Option<String>) -> Self {
        Self {
            run,
            player,
            toast,
            facing: Facing::Down,
            walking: None,
            menu: None,
        }
    }

    /// Whether the player's feet at `p` overlap a building or a tree trunk.
    fn blocked(p: Vec2, offices: &[OfficeDef]) -> bool {
        let feet = Rect::new(p.x - 5.0, p.y - 3.0, 10.0, 4.0);
        offices
            .iter()
            .map(|o| Rect::new(o.pos.0, o.pos.1, o.size.0, o.size.1))
            .chain(TREES.iter().copied().map(trunk))
            .any(|r| r.overlaps(&feet))
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
        if let Some(cursor) = &mut self.menu {
            if escape_pressed() {
                self.menu = None;
                return None;
            }
            let (x, y, w) = MENU;
            match ui::menu_input(cursor, 3, ctx.mouse, x, y, w) {
                Some(0) => self.menu = None,
                Some(1) => {
                    save::store_run(&self.run, (self.player.x, self.player.y));
                    ctx.fx.play(Sfx::Stamp);
                    self.toast = Some(d.s("progress_saved", &[]));
                    self.menu = None;
                }
                Some(_) => return Some(Scene::Title(Title::new())),
                None => {}
            }
            return None;
        }
        if escape_pressed() {
            self.menu = Some(0);
            self.walking = None;
            return None;
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
        if dir == Vec2::ZERO {
            self.walking = None;
        } else {
            self.toast = None;
            self.facing = if dir.x < 0.0 {
                Facing::Left
            } else if dir.x > 0.0 {
                Facing::Right
            } else if dir.y < 0.0 {
                Facing::Up
            } else {
                Facing::Down
            };
            self.walking = Some(self.walking.unwrap_or(0.0) + ctx.dt);
            // One axis at a time, so the player slides along walls.
            let step = dir.normalize() * SPEED * ctx.dt;
            for delta in [vec2(step.x, 0.0), vec2(0.0, step.y)] {
                let mut next = self.player + delta;
                next.x = next.x.clamp(8.0, canvas::WIDTH - 8.0);
                next.y = next.y.clamp(ui::HUD_H + 24.0, canvas::HEIGHT - 4.0);
                if !Self::blocked(next, &d.offices) {
                    self.player = next;
                }
            }
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
                ctx.fx.play(Sfx::Ting);
                return Some(Scene::Office(Office::new(run, id, self.player, d)));
            }
        }
        None
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let near = self.nearby(&d.offices).map(|o| o.id.as_str());
        let frame = art::walk_frame(self.walking);
        draw_town(ctx, near, Some((self.facing, frame, self.player)));

        self.draw_board(ctx);
        ui::hud(font, d, &self.run);
        match (&self.toast, near) {
            _ if self.menu.is_some() => {}
            (Some(t), _) => {
                let lines = ui::wrap(font, t, 388.0, 16).len() as f32;
                ui::panel(216.0, 60.0, 408.0, 16.0 + lines * 20.0);
                ui::text_wrapped(font, t, 226.0, 80.0, 388.0, 16, ui::INK);
                ui::hint(font, &d.s("controls_overworld", &[]));
            }
            (None, Some(_)) => ui::hint(font, &d.s("hint_enter", &[])),
            (None, None) => ui::hint(font, &d.s("controls_overworld", &[])),
        }
        if let Some(cursor) = self.menu {
            draw_rectangle(
                0.0,
                0.0,
                canvas::WIDTH,
                canvas::HEIGHT,
                Color::new(0.0, 0.0, 0.0, 0.35),
            );
            let (x, y, w) = MENU;
            let title = d.s("pause_title", &[]);
            ui::panel(x, y - 30.0, w, 26.0);
            ui::text_centered(font, &title, x + w / 2.0, y - 12.0, 16, ui::INK);
            let items = ["pause_resume", "pause_save", "pause_quit"].map(|k| d.s(k, &[]));
            ui::menu(font, &items, cursor, x, y, w);
            ui::hint(font, &d.s("pause_hint", &[]));
        }
    }

    fn draw_board(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let quest = d.quest(&self.run.quest_id);
        let board = self.run.board(&quest.board);
        let (x, y, w) = (8.0, 204.0, 284.0);
        let h = 40.0 + 15.0 * board.len() as f32;
        // See-through, and fainter still when the player walks behind it.
        let behind = Rect::new(x, y, w, h + 24.0).contains(self.player);
        let (fill, ink) = if behind { (0.2, 0.45) } else { (0.55, 1.0) };
        draw_rectangle(
            x,
            y,
            w,
            h,
            Color {
                a: fill,
                ..ui::PAPER
            },
        );
        draw_rectangle_lines(
            x,
            y,
            w,
            h,
            2.0,
            Color {
                a: ink * 0.8,
                ..ui::INK
            },
        );
        let ink_color = Color { a: ink, ..ui::INK };
        let faded = Color {
            a: ink,
            ..ui::FADED
        };
        ui::text(font, &quest.title, x + 8.0, y + 16.0, 16, ink_color);
        ui::text(font, &d.s("board_title", &[]), x + 8.0, y + 31.0, 16, faded);
        for (i, doc) in board.iter().enumerate() {
            let mark = if self.run.inv.has(doc) { "[x]" } else { "[ ]" };
            let line = format!("{mark} {}", d.doc_name(doc));
            ui::text(
                font,
                &line,
                x + 8.0,
                y + 47.0 + 15.0 * i as f32,
                16,
                ink_color,
            );
        }
    }
}

fn draw_ground(art: &Sprites) {
    for ty in 0..(canvas::HEIGHT as i32 / 16 + 1) {
        for tx in 0..(canvas::WIDTH as i32 / 16) {
            let v = match art::hash(tx, ty) % 12 {
                0 => 2,
                1 | 2 => 1,
                _ => 0,
            };
            let src = Rect::new(v as f32 * 16.0, 0.0, 16.0, 16.0);
            art::blit(
                &art.grass,
                src,
                tx as f32 * 16.0,
                ty as f32 * 16.0,
                1.0,
                false,
            );
        }
    }
    let tile = Rect::new(0.0, 0.0, 16.0, 16.0);
    let edge = Color::from_hex(0x4f9a38);
    for road in [ROAD_H, ROAD_V] {
        art::fill(&art.path, tile, road, 1.0, 0.0);
    }
    for road in [ROAD_H, ROAD_V] {
        draw_rectangle_lines(road.x, road.y, road.w, road.h, 2.0, edge);
    }
    // Clear the edge lines where the two roads cross.
    art::fill(
        &art.path,
        tile,
        Rect::new(ROAD_V.x + 2.0, ROAD_H.y, ROAD_V.w - 4.0, ROAD_H.h),
        1.0,
        0.0,
    );
    art::fill(
        &art.path,
        tile,
        Rect::new(ROAD_V.x, ROAD_H.y + 2.0, ROAD_V.w, ROAD_H.h - 4.0),
        1.0,
        0.0,
    );
}

/// A building put together from roof shingles, plaster, windows and a door,
/// so any size in offices.ron works.
fn draw_office(art: &Sprites, font: &Font, o: &OfficeDef, index: usize, near: bool, t: f32) {
    let (x, y) = o.pos;
    let (w, h) = o.size;
    let cx = x + w / 2.0;
    let wall_top = y + ROOF_H;

    art::fill(
        &art.wall,
        Rect::new(0.0, 0.0, 8.0, 8.0),
        Rect::new(x, wall_top, w, h - ROOF_H),
        1.0,
        0.0,
    );
    draw_rectangle(x, y + h - 3.0, w, 3.0, Color::from_hex(0xc8b48a));
    draw_rectangle_lines(x, wall_top - 1.0, w, h - ROOF_H + 1.0, 1.0, ui::INK);
    let mut off = 26.0;
    while cx - off - 8.0 >= x + 4.0 {
        for wx in [cx - off - 8.0, cx + off - 8.0] {
            art::image(&art.window, wx, wall_top + 5.0, 1.0);
        }
        off += 28.0;
    }
    art::image(&art.door, cx - 8.0, y + h - 24.0, 1.0);

    let shingle = Rect::new((index % 4) as f32 * 8.0, 0.0, 8.0, 8.0);
    let mut row = 0.0;
    while row < ROOF_H {
        let band = Rect::new(x - 3.0, y + row, w + 6.0, (ROOF_H - row).min(8.0));
        let phase = if (row / 8.0) as i32 % 2 == 1 {
            4.0
        } else {
            0.0
        };
        art::fill(&art.roofs, shingle, band, 1.0, phase);
        row += 8.0;
    }
    draw_rectangle_lines(x - 3.0, y, w + 6.0, ROOF_H, 1.0, ui::INK);
    draw_rectangle(x - 3.0, wall_top - 2.0, w + 6.0, 2.0, ui::INK);

    let sign_w = ui::text_width(font, &o.short, 16) + 10.0;
    let sign = Rect::new((cx - sign_w / 2.0).round(), y + 3.0, sign_w.round(), 15.0);
    draw_rectangle(sign.x, sign.y, sign.w, sign.h, Color::from_hex(0xb07440));
    draw_rectangle_lines(sign.x, sign.y, sign.w, sign.h, 1.0, ui::INK);
    ui::text_centered(font, &o.short, cx, y + 15.0, 16, ui::PAPER);

    if near {
        // A bouncing arrow over the door: this is where you go in.
        let top = (y + h - 34.0 - (t * 6.0).sin().abs() * 3.0).round();
        let (a, b, c) = (
            vec2(cx - 5.0, top),
            vec2(cx + 5.0, top),
            vec2(cx, top + 6.0),
        );
        draw_triangle(
            a - vec2(1.0, 1.0),
            b + vec2(1.0, -1.0),
            c + vec2(0.0, 2.0),
            ui::INK,
        );
        draw_triangle(a, b, c, Color::from_hex(0xf6d84a));
    }
}

/// The town: ground, then buildings, trees, bushes, townsfolk, and an optional
/// walker, back to front. The title screen draws it too.
pub fn draw_town(ctx: &Ctx, near: Option<&str>, walker: Option<(Facing, usize, Vec2)>) {
    let font = &ctx.assets.font;
    let d = &ctx.assets.data;
    let art = &ctx.assets.art;
    draw_ground(art);

    let mut props: Vec<(f32, Prop)> = d
        .offices
        .iter()
        .enumerate()
        .map(|(i, o)| (o.pos.1 + o.size.1, Prop::Office(i)))
        .collect();
    props.extend(TREES.iter().map(|&(x, y)| (y, Prop::Tree(vec2(x, y)))));
    props.extend(BUSHES.iter().map(|&(x, y)| (y, Prop::Bush(vec2(x, y)))));
    props.extend((0..VILLAGERS.len()).map(|i| (VILLAGERS[i].1, Prop::Villager(i))));
    if let Some((_, _, feet)) = walker {
        props.push((feet.y, Prop::Player));
    }
    props.sort_by(|a, b| a.0.total_cmp(&b.0));

    let t = get_time() as f32;
    for (_, prop) in &props {
        match prop {
            Prop::Office(i) => {
                let o = &d.offices[*i];
                draw_office(art, font, o, *i, near == Some(o.id.as_str()), t);
            }
            Prop::Tree(p) => art::image(&art.tree, p.x - 16.0, p.y - 38.0, 1.0),
            Prop::Bush(p) => art::image(&art.bush, p.x - 8.0, p.y - 13.0, 1.0),
            Prop::Villager(i) => {
                let turn = [Facing::Down, Facing::Right, Facing::Down, Facing::Left];
                let facing = turn[(t / 2.5 + *i as f32 * 1.3) as usize % 4];
                let (x, y) = VILLAGERS[*i];
                art::character(art, &art.villagers, *i, facing, 0, vec2(x, y));
            }
            Prop::Player => {
                if let Some((facing, frame, feet)) = walker {
                    art::character(art, &art.player, 0, facing, frame, feet);
                }
            }
        }
    }
}
