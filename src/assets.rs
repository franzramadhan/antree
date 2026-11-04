use macroquad::prelude::*;

use crate::art::Sprites;
use crate::data::Data;

pub struct Assets {
    pub font: Font,
    pub stamp: Texture2D,
    pub art: Sprites,
    pub data: Data,
}

impl Assets {
    pub async fn load() -> Assets {
        let font = load_ttf_font("assets/fonts/PixelOperator.ttf")
            .await
            .expect("font");
        let stamp = load_texture("assets/sprites/stamp.png")
            .await
            .expect("stamp");
        stamp.set_filter(FilterMode::Nearest);
        Assets {
            font,
            stamp,
            art: Sprites::load().await,
            data: Data::load().await,
        }
    }
}
