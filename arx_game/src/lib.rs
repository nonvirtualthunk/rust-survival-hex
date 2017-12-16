#![feature(box_syntax)]
#![allow(unused_imports)]

extern crate arx_common as common;

pub mod entities;

pub mod events;

pub mod core;

pub mod world;

pub use world::World;