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