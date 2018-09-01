use common::prelude::*;
use game::prelude::*;


use entities::selectors::EntitySelectors::*;
use entities::tile::TileData;

use entities::common_entities::Taxon;

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
    HasEquipmentKind(Taxon),
    HasAttackKind(Taxon),
    IsA(Taxon),
    And(Box<EntitySelectors>, Box<EntitySelectors>),
    Or(Box<EntitySelectors>, Box<EntitySelectors>),
    Any,
    HasStamina(Sext),
    HasAP(i32)
}

impl EntitySelectors {
    pub fn friendly_character(of: Entity) -> EntitySelectors { Friend { of }.and(IsCharacter) }
    pub fn enemy_of(of: Entity) -> EntitySelectors { Enemy { of }.and(IsCharacter) }
    pub fn tile() -> EntitySelectors { IsTile }
    pub fn inventory() -> EntitySelectors { HasInventory }

    pub fn is_a(taxon : &'static Taxon) -> EntitySelectors{ IsA(taxon.into()) }
    pub fn has_equipment_kind(taxon : &'static Taxon) -> EntitySelectors { HasEquipmentKind(taxon.into()) }
    pub fn has_attack_kind(taxon : &'static Taxon) -> EntitySelectors { HasAttackKind(taxon.into()) }

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
