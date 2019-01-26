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
#[macro_use]
extern crate serde_derive;
extern crate toml;

extern crate wrapped2d;
mod swingyships;
use swingyships::objects::*;
use swingyships::game::*;
use swingyships::level_loader::{LevelDef, load_level};

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

use std::io::prelude::*;
use std::fs::File;
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
    let mut level_file = File::open(assets.join("chasers.toml")).unwrap();
    let mut level_contents = String::new();
    level_file.read_to_string(&mut level_contents).unwrap();
    let level_def: LevelDef = toml::from_str(&level_contents).unwrap();

    load_level(&mut game, swingyships::level_loader::Textures{chaser: chaser_tex}, level_def);

    let ball1 = make_ball(&mut game, tex.clone(), 50., -65.);
    make_rope_joint(&mut game, ball1.physics_handle, ship_handle, 15.);
    make_chain(&mut game, ball1.physics_handle, ship_handle, tex.clone(), 50., -50., 15);


    let ball2 = make_ball(&mut game, tex.clone(), 50., -70.);
    make_chain(&mut game, ball1.physics_handle, ball2.physics_handle,  tex.clone(),50., -50., 5);
    make_rope_joint(&mut game, ball1.physics_handle, ball2.physics_handle, 5.);

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
