extern crate wrapped2d;
extern crate ai_behavior;
extern crate sprite;
extern crate glium_graphics;

use glium_graphics::Texture;

use swingyships::game::{Game, GameObject, GameObjectType};
use swingyships::level_loader::{ChaserDef, ChaserProps};

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

pub fn make_chaser(
        game: &mut Game,
        tex: &Rc<Texture>,
        def: ChaserDef,
        props: &ChaserProps) -> GameObject
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

        let handle = body.create_fast_fixture(&shape, 2.);
        let mut fixture = body.fixture_mut(handle);
        fixture.set_restitution(props.restitution);
        fixture.set_density(props.density);
    }

    let ball_id;
    let mut sprite = Sprite::from_texture(tex.clone());
    ball_id = game.scene.add_child(sprite);
    game.scene.run(ball_id, &Action(ScaleBy(0., -(1. - props.scale), -(1. - props.scale))));

    game.objects.push(GameObject::new(ball_handle, ball_id, GameObjectType::Chaser));
    GameObject::new(ball_handle, ball_id, GameObjectType::Chaser)
}

pub fn make_ball(
        game: &mut Game,
        tex: Rc<Texture>,
        x: f32,
        y: f32) -> GameObject
{
    let mut def = b2::BodyDef {
        body_type: b2::BodyType::Dynamic,
        position: b2::Vec2 { x, y },
        .. b2::BodyDef::new()
    };

    let mut sprite = Sprite::from_texture(tex.clone());
    let whip_id = game.scene.add_child(sprite);
    game.scene.run(whip_id, &Action(ScaleBy(0., -0.75, -0.75)));


    def.position = b2::Vec2{x, y};
    let whip_handle = game.world.create_body(&def);
    {
        let mut body = game.world.body_mut(whip_handle);
        body.set_linear_damping(0.5);
        body.set_angular_damping(0.9);

        let mut shape = b2::CircleShape::new();
        shape.set_radius(1.8);

        let handle = body.create_fast_fixture(&shape, 4.);
        let mut fixture = body.fixture_mut(handle);
        fixture.set_restitution(0.8);
    }

    game.objects.push(GameObject::new(whip_handle, whip_id, GameObjectType::Default));
    GameObject::new(whip_handle, whip_id, GameObjectType::Default)
}

pub fn make_rope_joint(
        game: &mut Game,
        handle1: TypedHandle<b2::Body>,
        handle2: TypedHandle<b2::Body>,
        length: f32) {
    let mut rope_joint_def = b2::RopeJointDef::new(handle1, handle2);
    rope_joint_def.collide_connected = false;
    rope_joint_def.max_length = length;
    let rope_handle = game.world.create_joint(&rope_joint_def);
}

pub fn make_chain(
        game: &mut Game,
        handle1: TypedHandle<b2::Body>,
        handle2: TypedHandle<b2::Body>,
        tex: Rc<Texture>,
        x: f32,
        y: f32,
        length: i32) {

    let center1 = game.world.body(handle1).local_center().clone();
    let mut link_prev: GameObject = make_chain_link(game, handle1, tex.clone(), x, y, center1);

    for i in 1 .. length {
        link_prev = make_chain_link(game, link_prev.physics_handle, tex.clone(), x, y, b2::Vec2{x: 0.18, y: 0.18});
    }

    let mut rev_def = b2::RopeJointDef::new(link_prev.physics_handle, handle2);
    rev_def.collide_connected = false;
    rev_def.local_anchor_a = b2::Vec2{x: 0.18, y: 0.18};
    rev_def.local_anchor_b = game.world.body(handle2).local_center().clone();
    rev_def.max_length = 0.3;
    game.world.create_joint(&rev_def);
}

fn make_chain_link(
        game: &mut Game,
        handle_prev: TypedHandle<b2::Body>,
        tex: Rc<Texture>,
        x: f32,
        y: f32,
        local_anchor_prev: b2::Vec2) -> GameObject {
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
        //fixture.set_filter_data(&b2::Filter{category_bits: 0, mask_bits: 0, group_index: 0});
    }

    let mut rev_def = b2::RopeJointDef::new(handle_prev, link_handle);
    rev_def.collide_connected = false;
    rev_def.local_anchor_a = local_anchor_prev;
    rev_def.local_anchor_b = b2::Vec2{x: 0.18, y: 0.18};
    rev_def.max_length = 1.0;
    game.world.create_joint(&rev_def);

    game.objects.push(GameObject::new(link_handle, link_id, GameObjectType::Default));
    GameObject::new(link_handle, link_id, GameObjectType::Default)
}
