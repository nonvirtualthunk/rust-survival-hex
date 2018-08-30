#![feature(box_syntax)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![feature(get_type_id)]
#![feature(core_intrinsics)]
#![allow(where_clauses_object_safety)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#![feature(extern_prelude)]
#![feature(const_fn)]
#![feature(type_ascription)]

extern crate arx_common as common;
extern crate samvival_data as data;
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
extern crate itertools;
#[macro_use] extern crate arx_macros;
extern crate arx_game as game;
extern crate noise;
#[macro_use] extern crate erased_serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde;
extern crate ron;

extern crate num;
extern crate cgmath;

pub mod world_util;

pub mod archetypes;

pub mod prelude;

pub mod terrain;
//
//pub mod core;
//
//pub mod world_util;
//
pub mod logic;
//
pub mod entity_util;

pub mod components;
//
//pub mod prelude;
//
//pub mod reflect;
//
//pub mod modifiers;
//
//pub use game::world::World;
//
//mod world_playground;

//pub use game::world::*;
//pub use game::core::*;
pub use world_util::*;
//pub use actions::*;
pub use entity_util::*;
//pub use reflect::*;
//pub use modifiers::*;


pub use game::*;
pub use data::*;

pub mod samvival_core;
pub use samvival_core::*;