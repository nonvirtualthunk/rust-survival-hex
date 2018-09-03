#![allow(unused_imports)]
#![allow(where_clauses_object_safety)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#![feature(box_syntax)]
#![feature(nll)]
#![feature(const_vec_new)]

extern crate piston_window;
extern crate gfx_device_gl;
extern crate find_folder;
extern crate gfx_graphics;
extern crate gfx;
extern crate cgmath;
#[macro_use]
extern crate lazy_static;
extern crate image;
extern crate vecmath;
extern crate arx_common as common;
extern crate samvival_game as game;
extern crate arx_gui as gui;
extern crate samvival_graphics as graphics;
extern crate pathfinding;
extern crate interpolation;
extern crate noisy_float;
#[macro_use]
extern crate itertools;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate arx_gui;
#[macro_use] extern crate arx_gui_macros;

pub mod action_bar;
pub use self::action_bar::*;

pub mod reaction_bar;
pub use self::reaction_bar::*;

pub mod character_info;
pub use self::character_info::*;

pub mod attack_descriptions;

pub mod messages_widget;

pub mod state;
pub use state::*;

pub mod control_events;
pub mod inventory_widget;

pub use gui::*;

pub mod escape_menu;


use std::fs::File;
pub fn open_save_file(create: bool) -> Option<File> {
    use common;
    if let Some(load_base_path) = common::file::save_game_path("samvival") {
        println!("Opening save file at base path {:?}", load_base_path);
        if create {
            ::std::fs::create_dir_all(load_base_path.clone()).expect("Could not create necessary save directories");
        }
        let path = load_base_path.join("savegame.ron");
        if create {
            File::create(path).ok()
        } else {
            File::open(path).ok()
        }
    } else { None }
}
