use common::prelude::*;
use game::core::Progress;
use entities::*;
use entities::CharacterData;
use entities::TileData;
use entity_util::*;
use game::Entity;
use game::world::WorldView;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::hash::Hash;
use std::hash::Hasher;
use logic;
use entities::actions::EntitySelectors::*;


#[derive(Clone, Debug)]
pub struct Action {
    pub action_type : ActionType,
    pub ap : Progress<u32>,
    pub completed : bool
}


#[derive(PartialEq,Eq,Clone)]
pub enum EntitySelectors {
    Friend { of: Entity },
    Enemy { of: Entity },
    //    Neutral { of: Entity },
    InMoveRange { hex_range: u32, of: Entity },
    IsCharacter,
    IsTile,
    HasInventory
}

pub struct EntitySelector(pub Vec<EntitySelectors>);

impl EntitySelector {
    pub fn friendly_character(of: Entity) -> EntitySelector { EntitySelector(vec![Friend { of }, IsCharacter]) }
    pub fn enemy_of(of: Entity) -> EntitySelector { EntitySelector(vec![Enemy { of }, IsCharacter]) }
    pub fn tile() -> EntitySelector { EntitySelector(vec![IsTile]) }
    pub fn inventory() -> EntitySelector { EntitySelector(vec![HasInventory]) }

    pub fn within_range(mut self, hex_range : u32, of : Entity) -> Self {
        self.0.push(InMoveRange { hex_range, of });
        self
    }

    pub fn matches(&self, entity: Entity, world: &WorldView) -> bool {
        self.0.iter().all(|selector| selector.matches(entity, world))
    }
}

impl EntitySelectors {
    pub fn matches(&self, entity: Entity, world: &WorldView) -> bool {
        match *self {
            IsCharacter => world.has_data::<CharacterData>(entity),
            IsTile => world.has_data::<TileData>(entity),
            Friend { of } =>
                IsCharacter.matches(of, world) &&
                    IsCharacter.matches(entity, world) &&
                    world.character(of).allegiance.faction == world.character(entity).allegiance.faction,
            Enemy { of } =>
                IsCharacter.matches(of, world) &&
                    IsCharacter.matches(entity, world) &&
                    world.character(of).allegiance.faction == world.character(entity).allegiance.faction,
            InMoveRange { hex_range, of } => {
                if let Some(end_point) = position_of(entity, world) {
                    if let Some((_, cost)) = logic::movement::path_to(world, of, end_point) {
                        return cost < hex_range as f64
                    }
                }
                false
            },
            HasInventory => world.has_data::<InventoryData>(entity)
        }
    }
}

#[derive(Clone)]
pub struct ActionType {
    pub target : fn(Entity,&WorldView) -> Vec<EntitySelector>,
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
    pub const MoveAndAttack: ActionType = ActionType {
        target : |actor,world| vec![EntitySelector::enemy_of(actor), EntitySelector::tile().within_range(max_remaining_move(actor, world).as_u32_or_0(), actor)],
        name : "Move and Attack",
        icon : "ui/attack_icon",
        description : "Move (if necessary) and attack an enemy character with your active weapon",
        costs : "1 {Stamina}, 1 {AP} per {Move Speed} moved, {AP} depending on your active weapon"
    };

    pub const Move : ActionType = ActionType {
        target : |actor,world| vec![EntitySelector::tile().within_range(max_remaining_move(actor, world).as_u32_or_0(), actor)],
        name : "Move",
        icon : "ui/move_icon",
        description : "Move across terrain at a normal pace",
        costs : "1 {AP} per {Move Speed} moved"
    };

    pub const Run : ActionType = ActionType {
        target : |actor,world| vec![EntitySelector::tile().within_range((max_remaining_move(actor,world) * 2).as_u32_or_0(), actor)],
        name : "Run",
        icon : "ui/run_icon",
        description : "Run across terrain at a faster pace but at the expense of stamina. Converts all remaining AP to movement points.",
        costs : "2 {Stamina}, all remaining {AP}"
    };

    pub const InteractWithInventory : ActionType = ActionType {
        target : |actor, world| vec![EntitySelector::inventory().within_range(1, actor)],
        name : "Give or Take Items",
        icon : "ui/interact_with_inventory_icon",
        description : "Transfer items from your inventory to or from another. Can be used to drop items on the ground or pick them up.",
        costs : "1 {AP} per item taken or given"
    };
}





