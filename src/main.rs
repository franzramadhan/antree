mod canvas;

use canvas::Canvas;
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Antri".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let font = load_ttf_font("assets/fonts/PixelOperator.ttf")
        .await
        .expect("font");
    let stamp = load_texture("assets/sprites/stamp.png")
        .await
        .expect("stamp");
    stamp.set_filter(FilterMode::Nearest);
    let canvas = Canvas::new();

    loop {
        canvas.begin();
        clear_background(Color::from_rgba(228, 220, 196, 255));
        draw_rectangle(
            0.0,
            0.0,
            canvas::WIDTH,
            24.0,
            Color::from_rgba(96, 140, 120, 255),
        );
        draw_texture_ex(
            &stamp,
            24.0,
            150.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(48.0, 48.0)),
                ..Default::default()
            },
        );
        draw_text_ex(
            "Did you photocopy it, sir?",
            88.0,
            182.0,
            TextParams {
                font: Some(&font),
                font_size: 16,
                color: Color::from_rgba(40, 32, 24, 255),
                ..Default::default()
            },
        );
        canvas.end();
        next_frame().await;
    }
}
