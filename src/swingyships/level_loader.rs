use swingyships::objects::{make_chaser, make_ball, make_chain};
use swingyships::game::Game;
use glium_graphics::Texture;
use slotmap::DefaultKey;

use std::rc::Rc;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct LevelDef {
    pub chasers: Chasers,
    pub weapons: Weapons,
    #[serde(default)]
    pub chains: Vec<ChainDef>
}

pub struct Textures {
    pub chaser: Rc<Texture>,
    pub default: Rc<Texture>
}

#[derive(Clone, Debug, Deserialize)]
pub struct Chasers {
    #[serde(default)]
    pub defs: Vec<ChaserDef>,
    #[serde(default)]
    pub named_defs: Vec<NamedChaserDef>,
    pub props: ChaserProps
}

// TODO: this is a hack, should I use something other than TOML?
#[derive(Clone, Debug, Deserialize)]
pub struct NamedChaserDef {
    pub def: ChaserDef,
    pub name: String
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChaserDef {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChaserProps {
    pub linear_damping: f32,
    pub scale: f64,
    pub density: f32,
    pub restitution: f32
}

#[derive(Clone, Debug, Deserialize)]
pub struct Weapons {
    #[serde(default)]
    pub defs: Vec<WeaponDef>,
    #[serde(default)]
    pub props: HashMap<String, WeaponProps>
}

// TODO: this is a hack, should I use something other than TOML?
#[derive(Clone, Debug, Deserialize)]
pub struct WeaponDef {
    pub x: f32,
    pub y: f32,
    pub props: String,
    pub name: String
}


#[derive(Clone, Debug, Deserialize)]
pub struct WeaponProps {
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub scale: f64,
    pub density: f32,
    pub restitution: f32
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChainDef {
    object1: String,
    object2: String,
    pub x: f32,
    pub y: f32,
    pub length: i32
}

pub fn load_level(game: &mut Game, tex: Textures, def: LevelDef) {
    let mut objects = HashMap::<String, DefaultKey>::new();

    objects.insert(String::from("player"), game.player);

    let chasers = def.chasers;
    for chaser_def in chasers.defs {
        let chaser = make_chaser(game, &tex.chaser, chaser_def, &chasers.props);
    }

    for NamedChaserDef{def, name} in chasers.named_defs {
        let chaser = make_chaser(game, &tex.chaser, def, &chasers.props);
        objects.insert(name, chaser);
    }

    for weapon_def in def.weapons.defs {
        let props = match def.weapons.props.get(&weapon_def.props) {
            Some(p) => p,
            None => {
                println!("Could not find weapon property set {}, skipping weapon", weapon_def.props);
                continue;
            }
        };
        let name = weapon_def.name.clone();
        let weapon = make_ball(game, &tex.default, weapon_def, props);
        objects.insert(name, weapon);
    }

    for chain_def in def.chains {
        let object1 = match objects.get(&chain_def.object1) {
            Some(o) => o,
            None => {
                println!("Could not find object {}, skipping chain", chain_def.object1);
                continue;
            }
        };
        let object2 = match objects.get(&chain_def.object2) {
            Some(o) => o,
            None => {
                println!("Could not find object {}, skipping chain", chain_def.object2);
                continue;
            }
        };
        make_chain(game, *object1, *object2, &tex.default, chain_def);
    }
}
