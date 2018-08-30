use prelude::CharacterData;
use entity_util::position_of;
use logic;
use prelude::IdentityData;
use data::selectors::EntitySelectors;
use data::selectors::EntitySelectors::*;
use data::entities::*;
use game::prelude::*;

pub trait SelectorMatches {
    fn matches(&self, world: &WorldView, entity: Entity) -> bool;
}


use common::ExtendedCollection;
impl SelectorMatches for EntitySelectors {
    fn matches(&self, world: &WorldView, entity: Entity) -> bool {
        match *self {
            IsCharacter => world.has_data::<CharacterData>(entity),
            IsTile => world.has_data::<TileData>(entity),
            Friend { of } =>
                IsCharacter.matches(world, of) &&
                    IsCharacter.matches(world, entity) &&
                    world.character(of).allegiance.faction == world.character(entity).allegiance.faction,
            Enemy { of } =>
                IsCharacter.matches(world, of) &&
                    IsCharacter.matches(world, entity) &&
                    world.character(of).allegiance.faction == world.character(entity).allegiance.faction,
            InMoveRange { hex_range, of } => {
                if let Some(end_point) = position_of(entity, world) {
                    if let Some((_, cost)) = logic::movement::path_to(world, of, end_point) {
                        return cost < hex_range as f64
                    }
                }
                false
            },
            HasInventory => world.has_data::<InventoryData>(entity),
            IsA(ref taxon) => world.data_opt::<IdentityData>(entity).filter(|i| i.kinds.any_match(|k| k.is_a(&taxon))).is_some(),
            And(ref a,ref b) => a.matches(world, entity) && b.matches(world, entity),
            Or(ref a,ref b) => a.matches(world, entity) || b.matches(world, entity),
            Any => true
        }
    }
}