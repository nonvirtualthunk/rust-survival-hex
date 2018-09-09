#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(where_clauses_object_safety)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]

#![feature(extern_prelude)]
#![feature(const_fn)]
#![feature(type_ascription)]
#![feature(get_type_id)]
#![feature(core_intrinsics)]
#![feature(box_syntax)]
#![feature(nll)]

#[macro_use] extern crate arx_common as common;
extern crate samvival_data as data;
#[macro_use] extern crate lazy_static;
extern crate rand;
extern crate noisy_float;
extern crate pathfinding;
#[macro_use] extern crate spectral;
extern crate pretty_env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate derive_more;
extern crate itertools;
extern crate samvival_game as game;
extern crate noise;
//#[macro_use] extern crate erased_serde;
//#[macro_use] extern crate serde_derive;
//#[macro_use] extern crate serde;
//extern crate ron;

extern crate num;
extern crate cgmath;

pub mod ai;