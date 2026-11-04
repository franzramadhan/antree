mod art;
mod assets;
mod canvas;
mod data;
mod fx;
mod game;
mod music;
mod save;
mod scene;
mod ui;

use canvas::Canvas;
use macroquad::prelude::*;
use scene::{Ctx, Scene, Title};

fn window_conf() -> Conf {
    Conf {
        window_title: "Antree".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut lang = save::Save::load().lang;
    let mut assets = assets::Assets::load(lang).await;
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
        let changed = scene.update(&ctx).map(|next| scene = next).is_some();
        fx.blip_if_silent(scene::input_pressed());
        canvas.begin();
        scene.draw(&ctx);
        canvas.end(fx.tick(ctx.dt));
        next_frame().await;
        // The title menu switches language by saving it and reopening itself.
        if changed && save::Save::load().lang != lang {
            lang = save::Save::load().lang;
            assets.data = data::Data::load(lang).await;
        }
    }
}
