use common::prelude::*;
use game::prelude::*;


use EntitySelectors::*;
use tile::TileData;

use Taxon;

use inventory::InventoryData;
use item::ItemData;

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
}
