#![feature(box_syntax)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![feature(get_type_id)]
#![feature(entry_and_modify)]
#![feature(core_intrinsics)]
#![allow(where_clauses_object_safety)]
#![allow(dead_code)]

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
#[macro_use] extern crate derive_more;
extern crate num;

pub mod world;

pub mod entities;

pub mod events;

pub mod core;

pub mod world_util;

pub mod actions;

pub use world::World;

pub use world::*;
pub use actions::*;
pub use core::*;
pub use entities::*;
pub use world_util::*;