mod assets;
mod canvas;
mod data;
mod fx;
mod game;
mod save;
mod scene;
mod ui;

use canvas::Canvas;
use macroquad::prelude::*;
use scene::{Ctx, Scene, Title};

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
    let assets = assets::Assets::load().await;
    let fx = fx::Fx::load().await;
    let canvas = Canvas::new();
    let mut scene = Scene::Title(Title::new());

    loop {
        let ctx = Ctx {
            assets: &assets,
            fx: &fx,
            mouse: canvas.mouse(),
            dt: get_frame_time().min(0.1),
        };
        if is_key_pressed(KeyCode::M) {
            fx.toggle_mute();
        }
        if let Some(next) = scene.update(&ctx) {
            scene = next;
        }
        fx.blip_if_silent(scene::input_pressed());
        canvas.begin();
        scene.draw(&ctx);
        canvas.end(fx.tick(ctx.dt));
        next_frame().await;
    }
}
