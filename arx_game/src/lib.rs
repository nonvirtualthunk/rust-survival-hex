#![feature(box_syntax)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![feature(get_type_id)]
#![feature(entry_and_modify)]
#![feature(core_intrinsics)]

extern crate arx_common as common;
extern crate either;
#[macro_use] extern crate enum_map;
#[macro_use] extern crate lazy_static;
extern crate rand;
extern crate noisy_float;
extern crate pathfinding;
extern crate anymap;
#[macro_use] extern crate spectral;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

pub mod world;

pub mod entities;

pub mod events;

pub mod core;

pub mod world_util;

pub mod actions;

pub use world::World;