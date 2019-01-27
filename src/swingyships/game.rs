extern crate wrapped2d;
extern crate ai_behavior;
extern crate sprite;
extern crate glium_graphics;
extern crate uuid;
extern crate piston_window;
extern crate slotmap;
use std::cell::Ref;

use glium_graphics::Texture;
use piston_window::{Event, PressEvent, MouseCursorEvent, MouseRelativeEvent, RenderEvent, AdvancedWindow};

use wrapped2d::b2;
use wrapped2d::user_data::NoUserData;
use wrapped2d::handle::TypedHandle;

use sprite::*;
use ai_behavior::{
    Action,
    Sequence,
    Wait,
    WaitForever,
    While,
};

use slotmap::SlotMap;

use std::collections::HashMap;
use std::rc::Rc;
use uuid::Uuid;

slotmap::new_key_type!(
    pub struct GameObjectKey;
);

pub struct Game {
    pub scene: Scene<Texture>,
    pub world: b2::World<NoUserData>,
    pub objects: SlotMap<GameObjectKey, GameObject>,
    pub player: GameObjectKey,
    pub cursor_captured: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum GameObjectType {
    Default,
    Chaser,
    Player,
}

#[derive(Debug, Clone, Copy)]
pub struct GameObject {
    pub physics_handle: TypedHandle<b2::Body>,
    pub draw_id: Uuid,
    pub obj_type: GameObjectType
}

impl Game {
    pub fn body(&self, key: GameObjectKey) -> Option<Ref<b2::MetaBody<NoUserData>>> {
        Some(self.world.body(self.objects.get(key)?.physics_handle))
    }

    pub fn handle(&self, key: GameObjectKey) -> Option<TypedHandle<b2::Body>> {
        Some(self.objects.get(key)?.physics_handle)
    }

    pub fn draw_id(&self, key: GameObjectKey) -> Option<Uuid> {
        Some(self.objects.get(key)?.draw_id)
    }

    pub fn obj_type(&self, key: GameObjectKey) -> Option<GameObjectType> {
        Some(self.objects.get(key)?.obj_type)
    }
}

impl GameObject {
    pub fn new(physics_handle: TypedHandle<b2::Body>,
            draw_id: Uuid,
            obj_type: GameObjectType)
            -> GameObject {
        GameObject{physics_handle, draw_id, obj_type}
    }
}

impl GameObjectType {
    pub fn update(&self, e: &Event, game: &Game, handle: TypedHandle<b2::Body>) {
        match self {
            &GameObjectType::Default => {},
            &GameObjectType::Player => {
                if game.cursor_captured {
                    if let Some(args) = e.mouse_relative_args() {
                        let mut force = b2::Vec2{x:args[0] as f32 * 10000., y:-args[1] as f32 * 10000.};

                        let magnitude = (force.x * force.x + force.y * force.y).sqrt();
                        if (magnitude > 2000.) {
                            force.x = force.x * (2000. / magnitude);
                            force.y = force.y * (2000. / magnitude);
                        }
                        let mut body = game.world.body_mut(handle);
                        body.apply_force_to_center(&force, true);
                    }
                }
            },
            &GameObjectType::Chaser => {
                if let Some(_) = e.render_args() {
                    let mut chaser_body = game.world.body_mut(handle);
                    let ship_body = game.body(game.player).unwrap();

                    let vec = ship_body.position() - chaser_body.position();
                    let vec = vec / vec.norm() * 2000.;
                    chaser_body.apply_force_to_center(&vec, true);
                }
            }
        }
    }
}
