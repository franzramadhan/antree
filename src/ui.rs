//! Code-drawn widgets. Everything here draws into the 640x360 canvas.

use macroquad::prelude::*;

use crate::canvas;
use crate::data::Data;
use crate::game::run::Run;

pub const PAPER: Color = Color::new(0.894, 0.863, 0.769, 1.0);
pub const INK: Color = Color::new(0.157, 0.125, 0.094, 1.0);
pub const WALL: Color = Color::new(0.376, 0.549, 0.471, 1.0);
pub const WOOD: Color = Color::new(0.478, 0.333, 0.212, 1.0);
pub const STAMP: Color = Color::new(0.784, 0.118, 0.118, 1.0);
pub const GRASS: Color = Color::new(0.561, 0.655, 0.420, 1.0);
pub const ROAD: Color = Color::new(0.541, 0.518, 0.471, 1.0);
pub const FADED: Color = Color::new(0.4, 0.37, 0.33, 1.0);

pub const HUD_H: f32 = 24.0;

pub fn text(font: &Font, s: &str, x: f32, y: f32, size: u16, color: Color) {
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font: Some(font),
            font_size: size,
            color,
            ..Default::default()
        },
    );
}

pub fn text_width(font: &Font, s: &str, size: u16) -> f32 {
    measure_text(s, Some(font), size, 1.0).width
}

pub fn text_centered(font: &Font, s: &str, cx: f32, y: f32, size: u16, color: Color) {
    text(
        font,
        s,
        (cx - text_width(font, s, size) / 2.0).round(),
        y,
        size,
        color,
    );
}

/// Word-wrapped text. Returns the y after the last line.
pub fn text_wrapped(
    font: &Font,
    s: &str,
    x: f32,
    y: f32,
    max_w: f32,
    size: u16,
    color: Color,
) -> f32 {
    let line_h = size as f32 * 1.25;
    let mut line = String::new();
    let mut cy = y;
    for word in s.split(' ') {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if text_width(font, &candidate, size) > max_w && !line.is_empty() {
            text(font, &line, x, cy, size, color);
            cy += line_h;
            line = word.to_owned();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        text(font, &line, x, cy, size, color);
        cy += line_h;
    }
    cy
}

pub fn panel(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x + 3.0, y + 3.0, w, h, Color::new(0.0, 0.0, 0.0, 0.25));
    draw_rectangle(x, y, w, h, PAPER);
    draw_rectangle_lines(x, y, w, h, 2.0, INK);
}

/// A button. Returns true when clicked this frame. Hotkey handled by caller.
pub fn button(font: &Font, label: &str, r: Rect, mouse: Vec2, enabled: bool) -> bool {
    let (x, y, w, h) = (r.x, r.y, r.w, r.h);
    let hover = enabled && r.contains(mouse);
    let fill = if !enabled {
        Color::new(0.75, 0.72, 0.65, 1.0)
    } else if hover {
        Color::new(1.0, 0.98, 0.9, 1.0)
    } else {
        PAPER
    };
    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(x, y, w, h, 2.0, if enabled { INK } else { FADED });
    text(
        font,
        label,
        x + 8.0,
        y + h / 2.0 + 5.0,
        16,
        if enabled { INK } else { FADED },
    );
    hover && is_mouse_button_pressed(MouseButton::Left)
}

/// Top strip: day, clock, money, conscience bar.
pub fn hud(font: &Font, data: &Data, run: &Run) {
    draw_rectangle(0.0, 0.0, canvas::WIDTH, HUD_H, WALL);
    let day = data.s(
        "hud_day",
        &[
            ("days", &run.meters.days.to_string()),
            ("deadline", &run.par.deadline.to_string()),
        ],
    );
    text(font, &day, 8.0, 17.0, 16, PAPER);
    text(font, &run.clock.hhmm(), 110.0, 17.0, 16, PAPER);
    let money = data.s("hud_money", &[("money", &run.meters.money.to_string())]);
    text(font, &money, 180.0, 17.0, 16, PAPER);
    let label = data.s("hud_conscience", &[]);
    text(font, &label, 400.0, 17.0, 16, PAPER);
    let bar_x = 400.0 + text_width(font, &label, 16) + 8.0;
    let bar_w = canvas::WIDTH - bar_x - 8.0;
    draw_rectangle(bar_x, 7.0, bar_w, 10.0, INK);
    let fill = bar_w * run.meters.conscience as f32 / 100.0;
    let color = if run.meters.conscience <= 40 {
        STAMP
    } else {
        PAPER
    };
    draw_rectangle(bar_x, 7.0, fill, 10.0, color);
}

/// Bottom dialog box with a speaker name.
pub fn dialog(font: &Font, speaker: &str, body: &str) {
    let (x, y, w, h) = (16.0, 236.0, canvas::WIDTH - 32.0, 94.0);
    panel(x, y, w, h);
    draw_rectangle(x, y, w, 20.0, WOOD);
    text(font, speaker, x + 8.0, y + 15.0, 16, PAPER);
    text_wrapped(font, body, x + 10.0, y + 40.0, w - 20.0, 16, INK);
}

pub fn hint(font: &Font, s: &str) {
    text_centered(
        font,
        s,
        canvas::WIDTH / 2.0,
        canvas::HEIGHT - 6.0,
        16,
        FADED,
    );
}
