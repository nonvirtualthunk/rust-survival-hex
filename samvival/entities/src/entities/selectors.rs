use common::prelude::*;
use prelude::*;


use entities::EntitySelectors::*;
use entities::tile::TileData;
use entity_util::position_of;

use logic;
use entities::inventory::InventoryData;
use entities::item::ItemData;

#[derive(PartialEq,Eq,Clone,Debug,Serialize,Deserialize)]
pub enum EntitySelectors {
    Friend { of: Entity },
    Enemy { of: Entity },
    //    Neutral { of: Entity },
    InMoveRange { hex_range: u32, of: Entity },
    IsCharacter,
    IsTile,
    HasInventory,
    IsA(Taxon),
    And(Box<EntitySelectors>, Box<EntitySelectors>),
    Or(Box<EntitySelectors>, Box<EntitySelectors>),
    Any
}

impl EntitySelectors {
    pub fn friendly_character(of: Entity) -> EntitySelectors { Friend { of }.and(IsCharacter) }
    pub fn enemy_of(of: Entity) -> EntitySelectors { Enemy { of }.and(IsCharacter) }
    pub fn tile() -> EntitySelectors { IsTile }
    pub fn inventory() -> EntitySelectors { HasInventory }

    pub fn is_a(taxon : &'static Taxon) -> EntitySelectors{ IsA(taxon.into()) }

    pub fn within_range(self, hex_range : u32, of : Entity) -> Self {
        self.and(InMoveRange { hex_range, of })
    }

    pub fn and (self, other : EntitySelectors) -> EntitySelectors {
        EntitySelectors::And(box self, box other)
    }
    pub fn or(self, other : EntitySelectors) -> EntitySelectors {
        EntitySelectors::Or(box self, box other)
    }


    pub fn matches(&self, world: &WorldView, entity: Entity) -> bool {
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
