#![allow(unused_imports)]
#![allow(where_clauses_object_safety)]

#![feature(box_syntax)]
#![feature(trait_alias)]
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
extern crate arx_graphics as graphics;
#[macro_use] extern crate itertools;
extern crate rusttype;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate heck;

//pub mod core;
//pub use core::*;
//
//pub mod camera;
//pub use camera::*;

pub mod renderers;
pub use renderers::*;

//pub mod animation;
//pub use animation::*;
//
//pub mod interpolation;
//pub use interpolation::*;
//
//pub mod text;
//pub use text::*;
//
//mod resources;

// re-export all of the core graphics
pub use graphics::*;