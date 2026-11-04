//! The orientation form. Teaches the controls by being a form, and is rejected
//! once for the wrong ink before anyone reads it.

use macroquad::prelude::*;

use super::{confirm_pressed, draw_town, escape_pressed, Ctx, QuestSelect, Scene};
use crate::canvas;
use crate::data::Data;
use crate::fx::{Sfx, Slam};
use crate::save::Save;
use crate::ui;

const ROW_H: f32 = 22.0;
const TOP: f32 = 72.0;

#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// 1-based number of the first blank row.
    Blank(usize),
    Ink,
    Approved,
}

/// Blank rows are always refused. A complete form is refused once for the ink.
fn verdict(picks: &[usize], ink_refused: bool) -> Verdict {
    match picks.iter().position(|&p| p == 0) {
        Some(i) => Verdict::Blank(i + 1),
        None if !ink_refused => Verdict::Ink,
        None => Verdict::Approved,
    }
}

pub struct Tutorial {
    picks: Vec<usize>,
    cursor: usize,
    ink_refused: bool,
    approved: bool,
    msg: String,
    slam: Option<Slam>,
}

fn row_rect(i: usize) -> Rect {
    Rect::new(
        44.0,
        TOP + i as f32 * ROW_H,
        canvas::WIDTH - 88.0,
        ROW_H - 2.0,
    )
}

impl Tutorial {
    pub fn new(d: &Data) -> Self {
        Self {
            picks: vec![0; d.tutorial.rows.len()],
            cursor: 0,
            ink_refused: false,
            approved: false,
            msg: String::new(),
            slam: None,
        }
    }

    pub fn update(&mut self, ctx: &Ctx) -> Option<Scene> {
        let form = &ctx.assets.data.tutorial;
        let n = form.rows.len();
        if let Some(s) = &mut self.slam {
            s.update(ctx.fx, ctx.dt);
        }
        if self.approved {
            return confirm_pressed().then(|| Scene::QuestSelect(QuestSelect::new()));
        }
        // A replay from quest select can be abandoned. The first one cannot.
        if escape_pressed() && Save::load().tutorial_done {
            return Some(Scene::QuestSelect(QuestSelect::new()));
        }

        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.cursor = (self.cursor + n - 1) % n;
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.cursor = (self.cursor + 1) % n;
        }
        let k = form.rows[self.cursor].options.len();
        let pick = &mut self.picks[self.cursor];
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            *pick = (*pick + 1) % k;
        }
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            *pick = (*pick + k - 1) % k;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(i) = (0..n).find(|&i| row_rect(i).contains(ctx.mouse)) {
                self.cursor = i;
                self.picks[i] = (self.picks[i] + 1) % form.rows[i].options.len();
            }
        }
        if !is_key_pressed(KeyCode::Enter) {
            return None;
        }
        ctx.fx.play(Sfx::Printer);
        match verdict(&self.picks, self.ink_refused) {
            Verdict::Blank(i) => {
                self.msg = form.blank.replace("{n}", &i.to_string());
                self.slam = Some(Slam::reject());
            }
            Verdict::Ink => {
                self.ink_refused = true;
                self.msg = form.ink.clone();
                self.slam = Some(Slam::reject());
            }
            Verdict::Approved => {
                let mut save = Save::load();
                save.tutorial_done = true;
                save.store();
                self.approved = true;
                self.msg = form.approved.clone();
                self.slam = Some(Slam::approve());
            }
        }
        None
    }

    pub fn draw(&self, ctx: &Ctx) {
        let font = &ctx.assets.font;
        let d = &ctx.assets.data;
        let form = &d.tutorial;
        draw_town(ctx, None, None);
        ui::panel(32.0, 20.0, canvas::WIDTH - 64.0, 316.0);
        ui::text(font, &form.title, 44.0, 42.0, 16, ui::INK);
        draw_line(44.0, 52.0, canvas::WIDTH - 44.0, 52.0, 1.0, ui::FADED);
        for (i, row) in form.rows.iter().enumerate() {
            ui::picker_row(
                font,
                &row.label,
                &row.options[self.picks[i]],
                row_rect(i),
                i == self.cursor,
            );
        }
        let explain_y = row_rect(form.rows.len()).y + 14.0;
        ui::text_wrapped(
            font,
            &form.rows[self.cursor].text,
            44.0,
            explain_y,
            canvas::WIDTH - 88.0,
            16,
            ui::FADED,
        );
        ui::text_wrapped(
            font,
            &self.msg,
            44.0,
            298.0,
            canvas::WIDTH - 88.0,
            16,
            ui::INK,
        );
        if let Some(s) = &self.slam {
            if self.approved {
                s.draw_texture(&ctx.assets.stamp, 480.0, 150.0, 96.0, -0.2);
            } else {
                s.draw_word(font, &d.s("stamp_rejected", &[]), 470.0, 140.0);
            }
        }
        let hint = if self.approved {
            &form.hint_done
        } else {
            &form.hint
        };
        ui::hint(font, hint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blanks_first_then_the_ink_then_approval() {
        assert_eq!(verdict(&[1, 0, 2, 0], false), Verdict::Blank(2));
        assert_eq!(verdict(&[1, 0, 2, 0], true), Verdict::Blank(2));
        assert_eq!(verdict(&[1, 1, 2, 1], false), Verdict::Ink);
        assert_eq!(verdict(&[1, 1, 2, 1], true), Verdict::Approved);
    }
}
