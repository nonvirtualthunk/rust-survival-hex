#![allow(unused_imports)]

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
extern crate arx_graphics;
extern crate pathfinding;
extern crate conrod;

pub mod tactical;

pub mod core;

pub use core::GameMode;