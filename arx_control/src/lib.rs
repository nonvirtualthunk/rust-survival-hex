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
extern crate arx_game as game;
extern crate arx_gui as gui;
extern crate arx_graphics;
extern crate pathfinding;
extern crate conrod;
extern crate interpolation;
extern crate noisy_float;
#[macro_use]
extern crate itertools;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate arx_gui;
#[macro_use] extern crate arx_gui_macros;

pub mod tactical;

pub mod tactical_gui;

pub mod core;

pub use core::GameMode;

mod tactical_event_handler;

mod control_gui;

mod control_events;