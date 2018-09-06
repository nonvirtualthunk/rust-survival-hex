use common::prelude::*;
use game::core::Progress;
use entities::character::CharacterData;
use entities::tile::TileData;
use game::Entity;
use game::world::WorldView;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::hash::Hash;
use std::hash::Hasher;
use entities::selectors::EntitySelector;
use entities::movement::MovementTypeRef;
use entities::combat::AttackRef;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub action_type : ActionType,
    pub ap : Progress<i32>
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Move { from : AxialCoord , to : AxialCoord, movement_type : MovementTypeRef },
    Attack { targets : Vec<Entity>, attack : AttackRef },
    TransferItem { item : Entity, from : Entity, to : Entity },
    Harvest { from : AxialCoord, harvestable : Entity, preserve_renewable : bool }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Fields)]
pub struct ActionData {
    pub active_action : Option<Action>,
    pub active_reaction: ReactionTypeRef,
}
impl EntityData for ActionData {}