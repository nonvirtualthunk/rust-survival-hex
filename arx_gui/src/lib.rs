#![allow(unused_imports)]
#![feature(box_syntax)]
#![allow(where_clauses_object_safety)]
#![allow(unused_variables)]
#![allow(dead_code)]

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
extern crate arx_graphics as graphics;
extern crate interpolation;
extern crate noisy_float;
#[macro_use]
extern crate itertools;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
extern crate backtrace;
extern crate num;
#[macro_use] extern crate derive_more;


pub mod gui;

pub use gui::*;


pub mod widgets;
pub use widgets::*;