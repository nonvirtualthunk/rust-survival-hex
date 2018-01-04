#![feature(box_syntax)]
#![allow(unused_imports)]

extern crate arx_common as common;
extern crate either;
#[macro_use]
extern crate enum_map;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate noisy_float;
extern crate pathfinding;

pub mod entity_base;

pub mod entities;

pub mod events;

pub mod core;

pub mod world;

pub mod actions;

pub use world::World;