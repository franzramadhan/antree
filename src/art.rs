//! Pixel art sheets made by tools/sprites.py, and the helpers that draw them.

use macroquad::prelude::*;

use crate::assets::join_all;

pub struct Sprites {
    pub player: Texture2D,
    pub villagers: Texture2D,
    pub clerks: Texture2D,
    pub tree: Texture2D,
    pub bush: Texture2D,
    pub grass: Texture2D,
    pub path: Texture2D,
    pub roofs: Texture2D,
    pub wall: Texture2D,
    pub door: Texture2D,
    pub window: Texture2D,
    pub shadow: Texture2D,
    pub wallpaper: Texture2D,
    pub floor: Texture2D,
    pub clock: Texture2D,
    pub board: Texture2D,
    pub plant: Texture2D,
}

async fn sheet(name: &str) -> Texture2D {
    let tex = load_texture(&format!("assets/sprites/{name}.png"))
        .await
        .expect(name);
    tex.set_filter(FilterMode::Nearest);
    tex
}

impl Sprites {
    pub async fn load() -> Sprites {
        let names = [
            "player",
            "villagers",
            "clerks",
            "tree",
            "bush",
            "grass",
            "path",
            "roofs",
            "wall",
            "door",
            "window",
            "shadow",
            "wallpaper",
            "floor",
            "clock",
            "board",
            "plant",
        ];
        let Ok(
            [player, villagers, clerks, tree, bush, grass, path, roofs, wall, door, window, shadow, wallpaper, floor, clock, board, plant],
        ) = <[Texture2D; 17]>::try_from(join_all(names.map(sheet)).await)
        else {
            unreachable!()
        };
        Sprites {
            player,
            villagers,
            clerks,
            tree,
            bush,
            grass,
            path,
            roofs,
            wall,
            door,
            window,
            shadow,
            wallpaper,
            floor,
            clock,
            board,
            plant,
        }
    }
}

/// Draw part of a sheet with its top left at (x, y), `scale` canvas pixels per art pixel.
pub fn blit(tex: &Texture2D, src: Rect, x: f32, y: f32, scale: f32, flip_x: bool) {
    draw_texture_ex(
        tex,
        x.round(),
        y.round(),
        WHITE,
        DrawTextureParams {
            source: Some(src),
            dest_size: Some(vec2(src.w * scale, src.h * scale)),
            flip_x,
            ..Default::default()
        },
    );
}

/// The whole texture at (x, y).
pub fn image(tex: &Texture2D, x: f32, y: f32, scale: f32) {
    blit(
        tex,
        Rect::new(0.0, 0.0, tex.width(), tex.height()),
        x,
        y,
        scale,
        false,
    );
}

/// Fill `area` with a repeated tile, clipping at the edges. `phase` shifts the
/// pattern left in canvas pixels, for staggered rows.
pub fn fill(tex: &Texture2D, src: Rect, area: Rect, scale: f32, phase: f32) {
    let (tw, th) = (src.w * scale, src.h * scale);
    let mut y = area.y;
    while y < area.bottom() {
        let h = th.min(area.bottom() - y);
        let mut x = area.x - phase.rem_euclid(tw);
        while x < area.right() {
            let left = (area.x - x).max(0.0);
            let w = (tw - left).min(area.right() - x - left);
            let part = Rect::new(src.x + left / scale, src.y, w / scale, h / scale);
            blit(tex, part, x + left, y, scale, false);
            x += tw;
        }
        y += th;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Facing {
    Down,
    Up,
    Left,
    Right,
}

/// Walk cycle frame for the time spent walking. Standing is frame 0.
pub fn walk_frame(walking: Option<f32>) -> usize {
    walking.map_or(0, |t| [1, 0, 2, 0][(t * 8.0) as usize % 4])
}

/// A 16x24 character standing with its feet at `feet`. `sheet_row` picks a
/// palette variant on sheets that stack several characters.
pub fn character(
    s: &Sprites,
    tex: &Texture2D,
    sheet_row: usize,
    facing: Facing,
    frame: usize,
    feet: Vec2,
) {
    let row = match facing {
        Facing::Down => 0,
        Facing::Up => 1,
        Facing::Left | Facing::Right => 2,
    } + sheet_row * 3;
    image(&s.shadow, feet.x - 6.0, feet.y - 2.0, 1.0);
    let src = Rect::new(frame as f32 * 16.0, row as f32 * 24.0, 16.0, 24.0);
    blit(
        tex,
        src,
        feet.x - 8.0,
        feet.y - 23.0,
        1.0,
        facing == Facing::Left,
    );
}

/// Cheap stable hash for picking tile variants.
pub fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standing_is_frame_zero_and_walking_alternates_feet() {
        assert_eq!(walk_frame(None), 0);
        let frames: Vec<usize> = (0..4).map(|i| walk_frame(Some(i as f32 / 8.0))).collect();
        assert_eq!(frames, [1, 0, 2, 0]);
    }
}
