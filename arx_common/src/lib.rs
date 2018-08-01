#![allow(unused_imports)]
#![feature(vec_resize_default)]
#![allow(where_clauses_object_safety)]

extern crate cgmath;
extern crate interpolation;
extern crate noisy_float;
extern crate num;
#[macro_use]
extern crate derive_more;
#[macro_use] extern crate itertools;
extern crate anymap;
#[macro_use] extern crate spectral;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

pub mod hex;
pub use hex::*;

pub mod prelude;
pub use prelude::*;

pub mod datastructures;
pub use datastructures::*;

pub mod color;
pub use color::*;

pub mod math;
pub use math::*;

pub mod event_bus;
pub use event_bus::*;

pub mod search;
pub use search::*;