use swingyships::objects::{make_chaser};
use swingyships::game::Game;
use glium_graphics::Texture;
use std::rc::Rc;

#[derive(Clone, Debug, Deserialize)]
pub struct LevelDef {
    pub chasers: Chasers
}

pub struct Textures {
    pub chaser: Rc<Texture>
}

#[derive(Clone, Debug, Deserialize)]
pub struct Chasers {
    pub defs: Vec<ChaserDef>,
    pub props: ChaserProps
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChaserDef {
    pub x: f32,
    pub y: f32
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChaserProps {
    pub linear_damping: f32,
    pub scale: f64,
    pub density: f32,
    pub restitution: f32
}

pub fn load_level(game: &mut Game, tex: Textures, def: LevelDef) {
    let chasers = def.chasers;
    for chaser in chasers.defs {
        make_chaser(game, &tex.chaser, chaser, &chasers.props);
    }
}
