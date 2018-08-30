use common::prelude::*;
use game::core::Progress;
use CharacterData;
use TileData;
use game::Entity;
use game::world::WorldView;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::hash::Hash;
use std::hash::Hasher;
use selectors::EntitySelectors;


#[derive(Clone, Debug)]
pub struct Action {
    pub action_type : ActionType,
    pub ap : Progress<u32>,
    pub completed : bool
}



#[derive(Clone)]
pub struct ActionType {
    pub target : fn(Entity,&WorldView) -> Vec<EntitySelectors>,
    pub icon : Str,
    pub name : Str,
    pub description : Str,
    pub costs : Str
}
impl PartialEq<ActionType> for ActionType {
    fn eq(&self, other: &ActionType) -> bool {
        self.name == other.name
    }
}
impl Eq for ActionType {}
impl Debug for ActionType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "ActionType({})", self.name)
    }
}
impl Hash for ActionType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
    }
}

#[allow(non_upper_case_globals)]
pub mod action_types {
    use super::*;
//    pub const MoveAndAttack: ActionType = ActionType {
//        target : |actor,world| vec![EntitySelectors::enemy_of(actor), EntitySelectors::tile().within_range(logic::movement::max_moves_remaining(world, actor, 1.0).as_u32_or_0(), actor)],
//        name : "Move and Attack",
//        icon : "ui/attack_icon",
//        description : "Move (if necessary) and attack an enemy character with your active weapon",
//        costs : "1 {Stamina}, 1 {AP} per {Move Speed} moved, {AP} depending on your active weapon"
//    };
//
//    pub const Move : ActionType = ActionType {
//        target : |actor,world| vec![EntitySelectors::tile().within_range(logic::movement::max_moves_remaining(world, actor, 1.0).as_u32_or_0(), actor)],
//        name : "Move",
//        icon : "ui/move_icon",
//        description : "Move across terrain at a normal pace",
//        costs : "1 {AP} per {Move Speed} moved"
//    };
//
//    pub const Run : ActionType = ActionType {
//        target : |actor,world| vec![EntitySelectors::tile().within_range((logic::movement::max_moves_remaining(world, actor, 2.0)).as_u32_or_0(), actor)],
//        name : "Run",
//        icon : "ui/run_icon",
//        description : "Run across terrain at a faster pace but at the expense of stamina. Converts all remaining AP to movement points.",
//        costs : "2 {Stamina}, all remaining {AP}"
//    };
//
//    pub const TransferItem: ActionType = ActionType {
//        target : |actor, world| vec![EntitySelectors::inventory().within_range(1, actor)],
//        name : "Transfer Item",
//        icon : "ui/interact_with_inventory_icon",
//        description : "Transfer items from your inventory to or from another. Can be used to drop items on the ground or pick them up.",
//        costs : "1 {AP} per item taken or given"
//    };
}