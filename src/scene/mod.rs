//! Scenes own the game state. `update` returns the next scene when it changes.

mod ending;
mod minigame;
mod office;
mod overworld;
mod quest_select;
mod title;

use macroquad::prelude::Vec2;

use crate::assets::Assets;
use crate::data::Data;
use crate::game::run::Run;

pub use ending::Ending;
pub use office::Office;
pub use overworld::Overworld;
pub use quest_select::QuestSelect;
pub use title::Title;

pub struct Ctx<'a> {
    pub assets: &'a Assets,
    pub mouse: Vec2,
    pub dt: f32,
}

pub enum Scene {
    Title(Title),
    QuestSelect(QuestSelect),
    Overworld(Overworld),
    Office(Office),
    Ending(Ending),
}

impl Scene {
    pub fn update(&mut self, ctx: &Ctx) -> Option<Scene> {
        match self {
            Scene::Title(s) => s.update(ctx),
            Scene::QuestSelect(s) => s.update(ctx),
            Scene::Overworld(s) => s.update(ctx),
            Scene::Office(s) => s.update(ctx),
            Scene::Ending(s) => s.update(ctx),
        }
    }

    pub fn draw(&self, ctx: &Ctx) {
        match self {
            Scene::Title(s) => s.draw(ctx),
            Scene::QuestSelect(s) => s.draw(ctx),
            Scene::Overworld(s) => s.draw(ctx),
            Scene::Office(s) => s.draw(ctx),
            Scene::Ending(s) => s.draw(ctx),
        }
    }
}

fn confirm_pressed() -> bool {
    use macroquad::prelude::*;
    is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::E)
}

fn escape_pressed() -> bool {
    macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::Escape)
}

/// A dialog line for this moment of the run. Varies with the clock, stable within a frame.
fn say(d: &Data, office: &str, key: &str, run: &Run) -> String {
    d.line(
        office,
        key,
        (run.meters.days * 1000 + run.clock.minutes) as usize,
    )
}
