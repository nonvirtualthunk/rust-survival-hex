#![allow(unused_imports)]
#![allow(where_clauses_object_safety)]
#![allow(unused_variables)]
#![allow(dead_code)]

#![feature(associated_type_defaults)]
#![feature(box_syntax)]
#![feature(get_type_id)]
#![feature(specialization)]

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
extern crate anymap;
extern crate core;
#[macro_use] extern crate spectral;
extern crate multimap;

pub mod gui;
pub use gui::*;

mod gui_event_handling;
mod gui_rendering;

pub mod widgets;
pub use widgets::*;

pub mod compound_widgets;

pub mod events;
pub use events::*;


pub use piston_window::MouseButton;
pub use piston_window::Key;