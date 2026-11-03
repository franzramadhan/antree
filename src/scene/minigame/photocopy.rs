use macroquad::prelude::*;

use super::{cannot_pay, pass_time, Outcome};
use crate::canvas;
use crate::data::Data;
use crate::game::photocopy::{Item, Order, BRIEF_SECONDS, ITEMS, RUN_MINUTES};
use crate::game::run::Run;
use crate::scene::{say, Ctx};
use crate::ui;

pub struct PhotoGame {
    want: Order,
    have: Order,
    cursor: usize,
    /// Seconds the order stays on screen.
    brief: f32,
    msg: String,
}

fn row_rect(i: usize) -> Rect {
    Rect::new(80.0, 80.0 + i as f32 * 26.0, canvas::WIDTH - 160.0, 24.0)
}

fn value(d: &Data, o: &Order, item: Item) -> String {
    match item {
        Item::Copies => o.copies.to_string(),
        Item::DutyStamp => d.s(&format!("stamp_{}", o.stamp), &[]),
        Item::Folder => d.s(&format!("folder_{}", o.folder), &[]),
        Item::Certified => d.s(if o.certified { "cert_yes" } else { "cert_no" }, &[]),
    }
}

fn order_text(d: &Data, o: &Order) -> String {
    d.s(
        "photo_order",
        &[
            ("copies", &value(d, o, Item::Copies)),
            ("stamp", &value(d, o, Item::DutyStamp)),
            ("folder", &value(d, o, Item::Folder)),
            ("certified", &value(d, o, Item::Certified)),
        ],
    )
}

impl PhotoGame {
    pub fn new(run: &Run, office: &str, d: &Data) -> Self {
        let want = Order::random(&mut run.visit_rng("photocopy", office));
        Self {
            want,
            have: Order::default(),
            cursor: 0,
            brief: BRIEF_SECONDS,
            msg: say(d, office, "p_brief", run).replace("{order}", &order_text(d, &want)),
        }
    }

    pub fn update(&mut self, ctx: &Ctx, run: &mut Run, office: &str) -> Outcome {
        let d = &ctx.assets.data;
        if self.brief > 0.0 {
            self.brief -= ctx.dt;
            if self.brief <= 0.0 {
                // The order is only heard once.
                self.msg.clear();
            }
            return Outcome::Running;
        }
        let n = ITEMS.len();
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.cursor = (self.cursor + n - 1) % n;
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.cursor = (self.cursor + 1) % n;
        }
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            self.have.bump(ITEMS[self.cursor], true);
        }
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            self.have.bump(ITEMS[self.cursor], false);
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(i) = (0..n).find(|&i| row_rect(i).contains(ctx.mouse)) {
                self.cursor = i;
                self.have.bump(ITEMS[i], true);
            }
        }
        if !is_key_pressed(KeyCode::Enter) {
            return Outcome::Running;
        }
        let cost = self.have.cost();
        if !run.meters.spend(cost) {
            self.msg = cannot_pay(d, office, run, cost);
            return Outcome::Running;
        }
        if let Some(closed) = pass_time(run, RUN_MINUTES, d, office) {
            return closed;
        }
        if !run.clock.numbers_open() {
            return Outcome::Failed {
                line: say(d, office, "p_closed", run),
                end_day: false,
            };
        }
        let Some(item) = self.want.first_mismatch(&self.have) else {
            return Outcome::Passed(None);
        };
        let item_key = match item {
            Item::Copies => "item_copies",
            Item::DutyStamp => "item_stamp",
            Item::Folder => "item_folder",
            Item::Certified => "item_cert",
        };
        self.msg = format!(
            "{} \"{}\"",
            say(d, office, "p_wrong", run).replace("{item}", &d.s(item_key, &[])),
            order_text(d, &self.want)
        );
        self.brief = BRIEF_SECONDS;
        Outcome::Running
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        ui::panel(64.0, 36.0, canvas::WIDTH - 128.0, 150.0);
        if self.brief > 0.0 {
            ui::text(font, &d.s("photo_brief", &[]), 80.0, 60.0, 16, ui::FADED);
            ui::text_wrapped(
                font,
                &self.msg,
                80.0,
                100.0,
                canvas::WIDTH - 160.0,
                16,
                ui::INK,
            );
            let w = (canvas::WIDTH - 160.0) * self.brief / BRIEF_SECONDS;
            draw_rectangle(80.0, 160.0, w, 6.0, ui::STAMP);
            return;
        }
        ui::text(font, &d.s("photo_shelf", &[]), 80.0, 60.0, 16, ui::FADED);
        let labels = ["photo_copies", "photo_stamp", "photo_folder", "photo_cert"];
        for (i, (item, label)) in ITEMS.iter().zip(labels).enumerate() {
            ui::picker_row(
                font,
                &d.s(label, &[]),
                &value(d, &self.have, *item),
                row_rect(i),
                i == self.cursor,
            );
        }
        if !self.msg.is_empty() {
            ui::message(font, &self.msg);
        }
        ui::hint(
            font,
            &d.s("photo_hint", &[("cost", &self.have.cost().to_string())]),
        );
    }
}
