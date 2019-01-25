extern crate glium;
extern crate graphics;
extern crate glium_graphics;
extern crate piston;
extern crate ai_behavior;
extern crate sprite;
extern crate piston_window;
extern crate find_folder;
extern crate glutin_window;
extern crate gfx_device_gl;
extern crate uuid;

extern crate wrapped2d;

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

use std::rc::Rc;
use std::iter::FromIterator;
use uuid::Uuid;

use glium_graphics::{
    Flip, Glium2d, GliumWindow, OpenGL, Texture, TextureSettings
};
use piston_window::{Event, PressEvent, MouseCursorEvent, MouseRelativeEvent, RenderEvent, AdvancedWindow};
use piston::event_loop::EventLoop;
use piston::window::WindowSettings;

use glutin_window::GlutinWindow;

#[derive(Debug, Clone, Copy)]
enum GameObjectType {
    Default,
    Chaser,
    Player,
}

struct Game {
    scene: Scene<Texture>,
    world: b2::World::<NoUserData>,
    objects: Vec<GameObject>,
    player: TypedHandle<b2::Body>,
    cursor_captured: bool,
}

#[derive(Debug, Clone)]
struct GameObject {
    physics_handle: TypedHandle<b2::Body>,
    draw_id: Uuid,
    obj_type: GameObjectType
}

impl GameObject {
    fn new(physics_handle: TypedHandle<b2::Body>,
            draw_id: Uuid,
            obj_type: GameObjectType)
            -> GameObject {
        GameObject{physics_handle, draw_id, obj_type}
    }
}

impl GameObjectType {
    fn update(&self, e: &Event, game: &mut Game, handle: TypedHandle<b2::Body>) {
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
                    let ship_body = game.world.body(game.player);

                    let vec = ship_body.position() - chaser_body.position();
                    let vec = vec / vec.norm() * 1000.;
                    chaser_body.apply_force_to_center(&vec, true);
                }
            }
        }
    }
}


fn make_chaser(
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

    let ball_handle = game.world.create_body(&def);
    {
        let mut body = game.world.body_mut(ball_handle);
        body.set_gravity_scale(0.);
        body.set_linear_damping(1.5);
        body.set_rotation_fixed(true);

        let mut shape = b2::CircleShape::new();
        shape.set_radius(3.6);

        let handle = body.create_fast_fixture(&shape, 2.);
        let mut fixture = body.fixture_mut(handle);
        fixture.set_restitution(0.5);
        fixture.set_density(0.01);
    }

    let ball_id;
    let mut sprite = Sprite::from_texture(tex.clone());
    ball_id = game.scene.add_child(sprite);
    game.scene.run(ball_id, &Action(ScaleBy(0., -0.5, -0.5)));

    game.objects.push(GameObject::new(ball_handle, ball_id, GameObjectType::Chaser));
    GameObject::new(ball_handle, ball_id, GameObjectType::Chaser)
}

fn make_ball(
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
        fixture.set_density(0.4);
    }

    game.objects.push(GameObject::new(whip_handle, whip_id, GameObjectType::Default));
    GameObject::new(whip_handle, whip_id, GameObjectType::Default)
}

fn make_rope_joint(
        game: &mut Game,
        handle1: TypedHandle<b2::Body>,
        handle2: TypedHandle<b2::Body>,
        length: f32) {
    let mut rope_joint_def = b2::RopeJointDef::new(handle1, handle2);
    rope_joint_def.collide_connected = false;
    rope_joint_def.max_length = length;
    let rope_handle = game.world.create_joint(&rope_joint_def);
}

fn make_chain(
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

fn main() {
    let opengl = OpenGL::V3_0;
    let (width, height) = (1000, 1000);
    let ref mut window: GliumWindow =
        WindowSettings::new("glium_graphics: image_test", [width, height])
        .exit_on_esc(true).opengl(opengl).build().unwrap();
    window.set_capture_cursor(true);
    let mut g2d = Glium2d::new(opengl, window);

    let mut game_objects = Vec::new();
    let gravity = b2::Vec2 { x: 0., y: -10. };
    let mut world = b2::World::<NoUserData>::new(&gravity);
    let mut scene: Scene<Texture> = Scene::new();


    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let ship_id;
    let tex = Rc::new(Texture::from_path(
                    window,
                    assets.join("rust.png"),
                    Flip::None,
                    &TextureSettings::new()
    ).unwrap());

    let mut sprite = Sprite::from_texture(tex.clone());
    ship_id = scene.add_child(sprite);
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

        let handle = body.create_fast_fixture(&shape, 2.);
        let mut fixture = body.fixture_mut(handle);
        fixture.set_restitution(0.1);
        fixture.set_density(0.01);
    }

    game_objects.push(GameObject::new(ship_handle, ship_id, GameObjectType::Player));

    let mut game = Game{
        objects: game_objects,
        world,
        scene,
        player: ship_handle,
        cursor_captured: true
    };

    let chaser_tex = Rc::new(Texture::from_path(
                    window,
                    assets.join("rust_red.png"),
                    Flip::None,
                    &TextureSettings::new()
    ).unwrap());




/*
    let bar_id;
    let mut sprite = Sprite::from_texture(tex.clone());
    bar_id = scene.add_child(sprite);
*/

    let tex = Rc::new(Texture::from_path(
                    window,
                    assets.join("rust.png"),
                    Flip::None,
                    &TextureSettings::new()
    ).unwrap());

    let mut walls_def = b2::BodyDef {
        body_type: b2::BodyType::Static,
        .. b2::BodyDef::new()
    };

    let walls_handle = game.world.create_body(&walls_def);
    {
        let mut body = game.world.body_mut(walls_handle);

        let mut shape = b2::EdgeShape::new();
        {
            shape.set_v1(b2::Vec2{x:-5., y:5.});
            shape.set_v2(b2::Vec2{x:-5., y:-105.});
            let handle = body.create_fast_fixture(&shape, 2.);
            let mut fixture = body.fixture_mut(handle);
            fixture.set_restitution(1.0);
        }

        {
            shape.set_v1(b2::Vec2{x:-5., y:-105.});
            shape.set_v2(b2::Vec2{x:105., y:-105.});
            let handle = body.create_fast_fixture(&shape, 2.);
            let mut fixture = body.fixture_mut(handle);
            fixture.set_restitution(1.0);
        }

        {
            shape.set_v1(b2::Vec2{x:105., y:5.});
            shape.set_v2(b2::Vec2{x:105., y:-105.});
            let handle = body.create_fast_fixture(&shape, 2.);
            let mut fixture = body.fixture_mut(handle);
            fixture.set_restitution(1.0);
        }

        {
            shape.set_v1(b2::Vec2{x:-5., y:5.});
            shape.set_v2(b2::Vec2{x:105., y:5.});
            let handle = body.create_fast_fixture(&shape, 2.);
            let mut fixture = body.fixture_mut(handle);
            fixture.set_restitution(1.0);
        }

    }



/*
    let bar_handle = world.create_body(&def);
    {
        let mut body = world.body_mut(bar_handle);
        body.set_linear_damping(1.);
        body.set_rotation_fixed(false);

        let mut shape = b2::PolygonShape::new_box(15., 2.);

        let handle = body.create_fast_fixture(&shape, 2.);
        let mut fixture = body.fixture_mut(handle);
        fixture.set_density(1.);
    }

    let mut rev_joint_def = b2::RevoluteJointDef::new(ship_handle, bar_handle);
    rev_joint_def.local_anchor_b = b2::Vec2{x:15., y:0.};
    rev_joint_def.local_anchor_a = b2::Vec2{x:0., y:0.};
    rev_joint_def.collide_connected = false;
    rev_joint_def.enable_limit = false;
    let rev_handle = world.create_joint(&rev_joint_def);
*/

    make_chaser(&mut game, chaser_tex.clone(), 80., -50.);
    make_chaser(&mut game, chaser_tex.clone(), 80., -20.);
    make_chaser(&mut game, chaser_tex.clone(), 80., -80.);

    let ball1 = make_ball(&mut game, tex.clone(), 50., -65.);
    make_rope_joint(&mut game, ball1.physics_handle, ship_handle, 15.);
    make_chain(&mut game, ball1.physics_handle, ship_handle, tex.clone(), 50., -50., 15);


//    let ball2 = make_ball(&mut game, tex.clone(), 50., -70.);
//    make_chain(&mut game, ball1.physics_handle, ball2.physics_handle,  tex.clone(),50., -50., 5);
//    make_rope_joint(&mut game, ball1.physics_handle, ball2.physics_handle, 5.);

    let ball3 = make_ball(&mut game, tex.clone(), 50., -20.);
    let ball4 = make_ball(&mut game, tex.clone(), 50., -20.);

    make_chain(&mut game, ball3.physics_handle, ball4.physics_handle,  tex.clone(),50., -50., 8);
    make_rope_joint(&mut game, ball3.physics_handle, ball4.physics_handle, 8.);

    while let Some(e) = window.next() {
        game.scene.event(&e);

        let objects = game.objects.clone();
        for object in objects {
            object.obj_type.update(&e, &mut game, object.physics_handle)
        }

        if let Some(args) = e.render_args() {
            game.world.step(1./60., 20, 20);

            for object in game.objects.iter() {

                let body = game.world.body(object.physics_handle);
                game.scene.child_mut(object.draw_id).unwrap().set_position(
                        body.position().x as f64 * 10.,
                        -body.position().y as f64 * 10.);
                game.scene.run(object.draw_id, &Action(RotateBy(0., body.angle() as f64)));
            }

            let mut target = window.draw();
            g2d.draw(&mut target, args.viewport(), |c, g| {
                graphics::clear([1.0, 1.0, 1.0, 1.0], g);
                game.scene.draw(c.transform, g);
            });
            target.finish().unwrap();
        }

        if let Some(_) = e.press_args() {
            game.cursor_captured = !game.cursor_captured;
            window.set_capture_cursor(game.cursor_captured);
        }
    }
}
