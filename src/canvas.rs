//! Fixed 640x360 internal canvas, letterboxed to the window. Integer scaled
//! from 2x up; a smaller window (a phone) gets a fractional scale that fits.

use macroquad::prelude::*;

pub const WIDTH: f32 = 640.0;
pub const HEIGHT: f32 = 360.0;

pub struct Canvas {
    target: RenderTarget,
    camera: Camera2D,
}

impl Canvas {
    pub fn new() -> Self {
        let target = render_target(WIDTH as u32, HEIGHT as u32);
        target.texture.set_filter(FilterMode::Nearest);
        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, WIDTH, HEIGHT));
        camera.render_target = Some(target.clone());
        Self { target, camera }
    }

    /// Route all following draw calls into the internal canvas.
    pub fn begin(&self) {
        set_camera(&self.camera);
    }

    /// Blit the internal canvas to the window, scaled by `fit`, with black bars,
    /// moved by `shake` canvas pixels.
    pub fn end(&self, shake: Vec2) {
        set_default_camera();
        clear_background(BLACK);
        let (x, y, w, h) = fit(screen_width(), screen_height());
        let scale = w / WIDTH;
        draw_texture_ex(
            &self.target.texture,
            x + shake.x * scale,
            y + shake.y * scale,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                flip_y: true,
                ..Default::default()
            },
        );
    }

    /// Mouse position in canvas coordinates.
    pub fn mouse(&self) -> Vec2 {
        let (mx, my) = mouse_position();
        let (x, y, w, _) = fit(screen_width(), screen_height());
        let scale = w / WIDTH;
        vec2((mx - x) / scale, (my - y) / scale)
    }
}

/// Returns the destination rectangle (x, y, w, h) for the canvas in a window of the given size.
pub fn fit(screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
    let fits = (screen_w / WIDTH).min(screen_h / HEIGHT);
    let scale = if fits < 2.0 { fits } else { fits.floor() };
    let w = WIDTH * scale;
    let h = HEIGHT * scale;
    (
        ((screen_w - w) / 2.0).floor(),
        ((screen_h - h) / 2.0).floor(),
        w,
        h,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_scale_and_centering() {
        assert_eq!(fit(1280.0, 720.0), (0.0, 0.0, 1280.0, 720.0));
        assert_eq!(fit(1920.0, 1080.0), (0.0, 0.0, 1920.0, 1080.0));
        assert_eq!(fit(1366.0, 768.0), (43.0, 24.0, 1280.0, 720.0));
        assert_eq!(fit(1000.0, 1000.0), (0.0, 218.0, 1000.0, 562.5));
        // a phone in portrait, physical pixels
        assert_eq!(fit(1170.0, 2532.0), (0.0, 936.0, 1170.0, 658.125));
    }

    #[test]
    fn shrinks_to_fit_a_small_window() {
        assert_eq!(fit(320.0, 180.0), (0.0, 0.0, 320.0, 180.0));
    }
}
