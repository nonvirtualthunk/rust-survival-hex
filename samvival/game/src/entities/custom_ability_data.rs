use noisy_float::types::R32;
use common::hex::*;
use game::prelude::*;
use game::EntityData;
use game::ModifierReference;
use common::prelude::*;
use std::collections::HashSet;

#[derive(Clone,Debug)]
pub struct Spawn {
    pub entity : EntityBuilder,
    pub turns_between_spawns : i32,
    pub start_spawn_turn : i32
}

#[derive(Default,Clone,Debug)]
pub struct MonsterSpawnerData {
    pub spawns : Vec<Spawn>
}

impl EntityData for MonsterSpawnerData {}










pub fn register_custom_ability_data(world : &mut World) {
    world.register::<MonsterSpawnerData>();
}