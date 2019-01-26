use swingyships::objects::{make_chaser, make_ball, make_chain};
use swingyships::game::Game;
use glium_graphics::Texture;
use slotmap::DefaultKey;

use std::rc::Rc;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct LevelDef {
    pub chasers: Chasers,
    pub weapons: Vec<WeaponInstance>
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
pub struct WeaponInstance {
    pub class: String,
    pub root: String
}

#[derive(Clone, Debug, Deserialize)]
pub struct WeaponDef {
    pub colliders: Vec<ColliderDef>,
    pub chains: Vec<ChainDef>,
    pub name: String
}

// TODO: this is a hack, should I use something other than TOML?
#[derive(Clone, Debug, Deserialize)]
pub struct ColliderDef {
    pub x: f32,
    pub y: f32,
    pub props: String,
    pub name: String
}


#[derive(Clone, Debug, Deserialize)]
pub struct ColliderProps {
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

pub fn load_level(game: &mut Game, tex: Textures, def: LevelDef,
        weapons: &HashMap<String, WeaponDef>, collider_props: &HashMap<String, ColliderProps>) {
    let mut roots = HashMap::<String, DefaultKey>::new();

    roots.insert(String::from("player"), game.player);

    let chasers = def.chasers;
    for chaser_def in chasers.defs {
        let chaser = make_chaser(game, &tex.chaser, chaser_def, &chasers.props);
    }

    for NamedChaserDef{def, name} in chasers.named_defs {
        let chaser = make_chaser(game, &tex.chaser, def, &chasers.props);
        roots.insert(name, chaser);
    }

    for weapon in def.weapons {
        let weapon_def = match weapons.get(&weapon.class) {
            Some(p) => p,
            None => {
                println!("Could not find weapon class {}, skipping weapon", weapon.class);
                continue;
            }
        };
        let root = match roots.get(&weapon.root) {
            Some(p) => p,
            None => {
                println!("Could not find root object {}, skipping weapon", weapon.root);
                continue;
            }
        };
        load_weapon(game, &tex, weapon_def, collider_props, *root);
    }
}

pub fn load_weapon(
        game: &mut Game,
        tex: &Textures,
        def: &WeaponDef,
        collider_props: &HashMap<String, ColliderProps>,
        root: DefaultKey
    ) {
    let mut objects = HashMap::new();
    objects.insert(String::from("root"), root);

    let root_pos = game.body(root).unwrap().position().clone();

    for ref collider in &def.colliders {
        let props = match collider_props.get(&collider.props) {
            Some(p) => p,
            None => {
                println!("Could not find collider property set {}, skipping collider", collider.props);
                continue;
            }
        };
        let name = collider.name.clone();
        let key = make_ball(game, &tex.default, &collider, props, root_pos);
        objects.insert(name, key);
    }

    for ref chain_def in &def.chains {
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
        make_chain(game, *object1, *object2, &tex.default, chain_def, root_pos);
    }
}
