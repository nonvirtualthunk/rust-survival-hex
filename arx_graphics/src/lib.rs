#![allow(unused_imports)]
#![allow(where_clauses_object_safety)]

#![feature(box_syntax)]
#![feature(trait_alias)]

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
#[macro_use] extern crate itertools;
extern crate rusttype;

pub mod core;
pub use core::*;

pub mod camera;
pub use camera::*;

pub mod renderers;
pub use renderers::*;

pub mod animation;
pub use animation::*;

pub mod interpolation;
pub use interpolation::*;

pub mod text;
pub use text::*;