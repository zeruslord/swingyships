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
extern crate slotmap;
extern crate wrapped2d;
mod swingyships;
use swingyships::objects::*;
use swingyships::game::*;
use swingyships::physics::FixRestitutionListener;
use swingyships::level_loader::{LevelDef, ColliderProps, WeaponDef, load_level};

use wrapped2d::b2;
use wrapped2d::user_data::NoUserData;
use wrapped2d::handle::TypedHandle;
use std::env;

use slotmap::SlotMap;

use sprite::*;
use ai_behavior::{
    Action,
    Sequence,
    Wait,
    WaitForever,
    While,
};

use std::collections::HashMap;
use std::io::prelude::*;
use std::fs::{File, ReadDir, DirEntry};
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

    let mut game_objects: SlotMap<GameObjectKey, GameObject> = SlotMap::with_key();
    let gravity = b2::Vec2 { x: 0., y: -10. };
    let mut world = b2::World::<NoUserData>::new(&gravity);
    world.set_contact_listener(Box::new(FixRestitutionListener{}));
    let mut scene: Scene<Texture> = Scene::new();


    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();

    let tex = Rc::new(Texture::from_path(
                    window,
                    assets.join("rust.png"),
                    Flip::None,
                    &TextureSettings::new()
    ).unwrap());

    let chaser_tex = Rc::new(Texture::from_path(
                    window,
                    assets.join("rust_red.png"),
                    Flip::None,
                    &TextureSettings::new()
    ).unwrap());

    let player = make_player(&mut world, &mut scene, &tex, &mut game_objects);

    let mut game = Game{
        objects: game_objects,
        world,
        scene,
        player: player,
        cursor_captured: true
    };




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
            shape.set_v1(b2::Vec2{x:-2., y:2.});
            shape.set_v2(b2::Vec2{x:-2., y:-102.});
            let mut fixture_def = b2::FixtureDef::new();
            fixture_def.restitution = 0.6;
            let handle = body.create_fixture(&shape, &mut fixture_def);
        }

        {
            shape.set_v1(b2::Vec2{x:-2., y:-102.});
            shape.set_v2(b2::Vec2{x:102., y:-102.});
            let mut fixture_def = b2::FixtureDef::new();
            fixture_def.restitution = 0.6;
            let handle = body.create_fixture(&shape, &mut fixture_def);
        }

        {
            shape.set_v1(b2::Vec2{x:102., y:2.});
            shape.set_v2(b2::Vec2{x:102., y:-102.});
            let mut fixture_def = b2::FixtureDef::new();
            fixture_def.restitution = 0.6;
            let handle = body.create_fixture(&shape, &mut fixture_def);
        }

        {
            shape.set_v1(b2::Vec2{x:-2., y:2.});
            shape.set_v2(b2::Vec2{x:102., y:2.});
            let mut fixture_def = b2::FixtureDef::new();
            fixture_def.restitution = 0.6;
            let handle = body.create_fixture(&shape, &mut fixture_def);
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

    let (level_def, props_def, weapon_defs)  = read_files(&assets);
    load_level(&mut game,
        swingyships::level_loader::Textures{chaser: chaser_tex, default: tex},
        level_def,
        &weapon_defs,
        &props_def);

    while let Some(e) = window.next() {
        game.scene.event(&e);

        for key in game.objects.keys() {
            let handle = game.handle(key).unwrap();
            game.obj_type(key).unwrap().update(&e, &game, handle);
        }

        if let Some(args) = e.render_args() {
            game.world.step(1./60., 20, 20);

            for key in game.objects.keys() {

                let x = game.body(key).unwrap().position().x as f64 * 10.;
                let y = -game.body(key).unwrap().position().y as f64 * 10.;
                let angle = game.body(key).unwrap().angle() as f64;
                let draw_id = game.draw_id(key).unwrap();
                game.scene.child_mut(draw_id).unwrap().set_position(x, y);
                game.scene.run(draw_id, &Action(RotateBy(0., angle)));
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


fn read_files(assets: &std::path::PathBuf) -> (LevelDef, HashMap<String, ColliderProps>, HashMap<String, WeaponDef>) {
    let args: Vec<String> = env::args().collect();
    let level_contents: String = match std::fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            panic!("could not read contents of level file {}", args[1])
        }
    };
    let level_def: LevelDef = match toml::from_str(&level_contents) {
        Ok(def) => def,
        Err(e) => {
            println!("{}", e);
            panic!("could not parse contents of level file")
        }
    };
    let props_contents = match std::fs::read_to_string(&args[2]) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", e);
            panic!("could not read contents of props file {}", args[2])
        }
    };
    let props_def: HashMap<String, ColliderProps> = match toml::from_str(&props_contents) {
        Ok(def) => def,
        Err(e) => {
            println!("{}", e);
            panic!("could not parse contents of props file {}", args[2])
        }
    };

    let mut weapon_defs: HashMap<String, WeaponDef> = HashMap::new();
    for file in std::fs::read_dir(assets.join("weapons")).unwrap() {
        let file = match file {
            Ok(s) => s,
            Err(e) => {
              println!("{}", e);
              panic!("could not read weapon file")
            }
        };
        let weapon_contents = match std::fs::read_to_string(&file.path()) {
            Ok(s) => s,
            Err(e) => {
              println!("{}", e);
              panic!("could not read contents of weapon file {:?}", file.file_name())
            }
        };
        let weapon_def: WeaponDef = match toml::from_str(&weapon_contents) {
            Ok(def) => def,
            Err(e) => {
                println!("{}", e);
                panic!("could not parse contents of weapon file {:?}", &file.file_name())
            }
        };
        weapon_defs.insert(weapon_def.name.clone(), weapon_def);
    }

    (level_def, props_def, weapon_defs)
}
