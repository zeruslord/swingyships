extern crate wrapped2d;
extern crate ai_behavior;
extern crate sprite;
extern crate glium_graphics;
extern crate slotmap;

use glium_graphics::Texture;
use slotmap::{SlotMap, DefaultKey};

use swingyships::game::{Game, GameObject, GameObjectType};
use swingyships::level_loader::{ChaserDef, ChaserProps, ColliderDef, ColliderProps, ChainDef};

use wrapped2d::b2;
use wrapped2d::user_data::NoUserData;
use wrapped2d::handle::TypedHandle;
use std::rc::Rc;

use sprite::*;
use ai_behavior::{
    Action,
    Sequence,
    Wait,
    WaitForever,
    While,
};

fn draw_scale(scale: f64) -> f64 {
     -(1. - scale)
}

pub fn make_player(
        world: &mut b2::World::<NoUserData>,
        scene: &mut Scene<Texture>,
        tex: &Rc<Texture>,
        game_objects: &mut SlotMap<DefaultKey, GameObject>
        // TODO: parameterize
    ) -> DefaultKey {
    let mut sprite = Sprite::from_texture(tex.clone());
    let ship_id = scene.add_child(sprite);
    scene.run(ship_id, &Action(ScaleBy(0., -0.5, -0.5)));

    let mut def = b2::BodyDef {
        body_type: b2::BodyType::Dynamic,
        position: b2::Vec2 { x: 50., y: -50. },
        .. b2::BodyDef::new()
    };

    let ship_handle = world.create_body(&def);
    {
        let mut body = world.body_mut(ship_handle);
        body.set_gravity_scale(0.);
        body.set_linear_damping(2.);
        body.set_rotation_fixed(true);


        let mut shape = b2::CircleShape::new();
        shape.set_radius(3.6);

        let mut fixture_def = b2::FixtureDef::new();
        fixture_def.density = 2.;
        fixture_def.restitution = 0.5;
        let handle = body.create_fixture(&shape, &mut fixture_def);
    }

    game_objects.insert(GameObject::new(ship_handle, ship_id, GameObjectType::Player))
}

pub fn make_chaser(
        game: &mut Game,
        tex: &Rc<Texture>,
        def: ChaserDef,
        props: &ChaserProps) -> DefaultKey
{
    let mut def = b2::BodyDef {
        body_type: b2::BodyType::Dynamic,
        position: b2::Vec2 { x: def.x, y: def.y },
        .. b2::BodyDef::new()
    };

    let ball_handle = game.world.create_body(&def);
    {
        let mut body = game.world.body_mut(ball_handle);
        body.set_gravity_scale(0.);
        body.set_linear_damping(props.linear_damping);
        body.set_rotation_fixed(true);

        let mut shape = b2::CircleShape::new();
        shape.set_radius(props.scale as f32 * 7.2);

        let mut fixture_def = b2::FixtureDef::new();
        fixture_def.density = props.density;
        fixture_def.restitution = props.restitution;
        let handle = body.create_fixture(&shape, &mut fixture_def);
    }

    let ball_id;
    let mut sprite = Sprite::from_texture(tex.clone());
    ball_id = game.scene.add_child(sprite);
    game.scene.run(ball_id, &Action(ScaleBy(0., draw_scale(props.scale), draw_scale(props.scale))));

    game.objects.insert(GameObject::new(ball_handle, ball_id, GameObjectType::Chaser))
}

pub fn make_ball(
        game: &mut Game,
        tex: &Rc<Texture>,
        def: &ColliderDef,
        props: &ColliderProps,
        root_pos: b2::Vec2
    ) -> DefaultKey
{
    let mut def = b2::BodyDef {
        body_type: b2::BodyType::Dynamic,
        position: b2::Vec2 { x: def.x + root_pos.x, y: def.y + root_pos.y },
        .. b2::BodyDef::new()
    };

    let mut sprite = Sprite::from_texture(tex.clone());
    let whip_id = game.scene.add_child(sprite);
    game.scene.run(whip_id, &Action(ScaleBy(0., draw_scale(props.scale), draw_scale(props.scale))));

    let whip_handle = game.world.create_body(&def);
    {
        let mut body = game.world.body_mut(whip_handle);
        body.set_linear_damping(props.linear_damping);
        body.set_angular_damping(props.angular_damping);

        let mut shape = b2::CircleShape::new();
        shape.set_radius(1.8);

        let mut fixture_def = b2::FixtureDef::new();
        fixture_def.density = props.density;
        fixture_def.restitution = props.restitution;
        let handle = body.create_fixture(&shape, &mut fixture_def);
    }

    game.objects.insert(GameObject::new(whip_handle, whip_id, GameObjectType::Default))
}

pub fn make_rope_joint(
        game: &mut Game,
        handle1: DefaultKey,
        handle2: DefaultKey,
        length: f32) -> Option<TypedHandle<b2::Joint>>{
    let mut rope_joint_def = b2::RopeJointDef::new(
        game.objects.get(handle1)?.physics_handle,
        game.objects.get(handle2)?.physics_handle);
    rope_joint_def.collide_connected = false;
    rope_joint_def.max_length = length;
    Some(game.world.create_joint(&rope_joint_def))
}

pub fn make_chain(
        game: &mut Game,
        key1: DefaultKey,
        key2: DefaultKey,
        tex: &Rc<Texture>,
        def: &ChainDef,
        root_pos: b2::Vec2) -> Option<TypedHandle<b2::Joint>> {

    let center1 = game.body(key1)?.local_center().clone();
    let handle_prev = game.handle(key1)?;
    let mut link_prev = make_chain_link(game, handle_prev, tex.clone(), def.x + root_pos.x, def.y + root_pos.y, center1);

    for i in 0 .. def.length {
        let handle_prev = game.handle(link_prev)?;
        link_prev = make_chain_link(game, handle_prev, tex.clone(), def.x + root_pos.x, def.y + root_pos.y, b2::Vec2{x: 0.18, y: 0.18});
    }

    let mut rev_def = b2::RopeJointDef::new(game.handle(link_prev)?, game.handle(key2)?);
    rev_def.collide_connected = false;
    rev_def.local_anchor_a = b2::Vec2{x: 0.18, y: 0.18};
    rev_def.local_anchor_b = game.body(key2)?.local_center().clone();
    rev_def.max_length = 0.3;
    game.world.create_joint(&rev_def);
    make_rope_joint(game, key1, key2, def.length as f32)
}

fn make_chain_link(
        game: &mut Game,
        handle_prev: TypedHandle<b2::Body>,
        tex: Rc<Texture>,
        x: f32,
        y: f32,
        local_anchor_prev: b2::Vec2) -> DefaultKey {
    let mut sprite = Sprite::from_texture(tex);
    let link_id = game.scene.add_child(sprite);
    game.scene.run(link_id, &Action(ScaleBy(0., -0.92, -0.92)));
    let mut def = b2::BodyDef {
        body_type: b2::BodyType::Dynamic,
        position: b2::Vec2 { x, y },
        fixed_rotation: false,
        .. b2::BodyDef::new()
    };

    let link_handle = game.world.create_body(&def);
    {
        let mut body = game.world.body_mut(link_handle);
        body.set_rotation_fixed(false);

        let shape = b2::PolygonShape::new_box(0.36,0.36);

        let handle = body.create_fast_fixture(&shape, 0.01);
        let mut fixture = body.fixture_mut(handle);
        fixture.set_filter_data(&b2::Filter{category_bits: 0, mask_bits: 0, group_index: 0});
    }

    let mut rev_def = b2::RopeJointDef::new(handle_prev, link_handle);
    rev_def.collide_connected = false;
    rev_def.local_anchor_a = local_anchor_prev;
    rev_def.local_anchor_b = b2::Vec2{x: 0.18, y: 0.18};
    rev_def.max_length = 1.0;
    game.world.create_joint(&rev_def);

    game.objects.insert(GameObject::new(link_handle, link_id, GameObjectType::Default))
}
