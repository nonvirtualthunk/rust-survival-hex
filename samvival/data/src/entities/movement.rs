use common::prelude::*;
use game::prelude::*;
use game::EntityData;
use std::hash::Hash;
use std::hash::Hasher;
use game::EntityBuilder;
use entities::common_entities::IdentityData;
use entities::taxonomy;

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct MovementType {
    pub name: String,
    pub move_multiplier: Sext,
    pub move_bonus: Sext,
    pub ap_activation_cost: i32,
    pub stamina_cost: Sext,
}
impl EntityData for MovementType {}

impl PartialEq<MovementType> for MovementType {
    fn eq(&self, other: &MovementType) -> bool {
        self.name == other.name
    }
}
impl Hash for MovementType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes())
    }
}

impl Default for MovementType {
    fn default() -> Self {
        MovementType {
            name : strf("default movement type"),
            move_multiplier: Sext::of(1),
            move_bonus: Sext::of(0),
            ap_activation_cost: 0,
            stamina_cost: Sext::of(0)
        }
    }
}

pub fn create_movement_type(world : &mut World, data : MovementType) -> Entity {
    EntityBuilder::new()
        .with(IdentityData::new(data.name.as_str(), &taxonomy::Movement))
        .with(data)
        .create(world)
}
pub fn create_walk_movement_type(world : &mut World) -> Entity {
    create_movement_type(world, MovementType {
        name : strf("walk"),
        move_multiplier : Sext::of(1),
        move_bonus : Sext::of(0),
        ap_activation_cost: 0,
        stamina_cost: Sext::of(0)
    })
}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug,Serialize,Deserialize)]
pub struct MovementTypeRef { pub movement : Entity, pub mover : Entity }
impl MovementTypeRef {
    pub fn of_movement_and_mover(movement : Entity, mover : Entity) -> MovementTypeRef { MovementTypeRef { movement, mover } }
}

// --------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct MovementData {
    pub active_movement_type: Option<MovementTypeRef>,
    pub move_speed: Sext,
    // represented in sexts
    pub moves: Sext,
    pub movement_types: Vec<Entity>,
}

impl EntityData for MovementData {}

impl Default for MovementData {
    fn default() -> Self {
        MovementData {
            active_movement_type: None,
            move_speed: Sext::of(0),
            moves: Sext::of(0),
            movement_types: vec![],
        }
    }
}