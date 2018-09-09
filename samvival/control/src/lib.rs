#![allow(unused_imports)]
#![allow(where_clauses_object_safety)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#![feature(box_syntax)]
#![feature(nll)]

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
extern crate samvival_graphics as graphics;
extern crate samvival_ai as ai;
extern crate pathfinding;
extern crate interpolation;
extern crate noisy_float;
#[macro_use]
extern crate itertools;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate samvival_gui as gui;
#[macro_use] extern crate arx_gui_macros;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde;
extern crate ron;
extern crate bincode;

pub mod tactical;

pub mod tactical_gui;

pub mod core;

pub use core::GameMode;

mod tactical_event_handler;

mod action_ui_handlers;

pub mod main_menu;