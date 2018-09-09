use common::prelude::*;
use game::prelude::*;


use entities::selectors::EntitySelector::*;
use entities::tile::TileData;

use entities::common_entities::Taxon;

use entities::inventory::InventoryData;
use entities::item::ItemData;
use entities::skill::Skill;
use entities::common_entities::LookupSignifier;

#[derive(PartialEq,Eq,Clone,Debug,Serialize,Deserialize)]
pub enum EntitySelector {
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
    Is(Entity), // test if something is exactly a specific entity
    IsEquivalentTo(Entity), // test if something is, or is the same as (cloned and not modified) the other entity
    And(Box<EntitySelector>, Box<EntitySelector>),
    Or(Box<EntitySelector>, Box<EntitySelector>),
    Any,
    HasStamina(Sext),
    HasAP(i32),
    HasSkillLevel(Skill, i32),
    None
}

impl EntitySelector {
    pub fn friendly_character(of: Entity) -> EntitySelector { Friend { of }.and(IsCharacter) }
    pub fn enemy_of(of: Entity) -> EntitySelector { Enemy { of }.and(IsCharacter) }
    pub fn tile() -> EntitySelector { IsTile }
    pub fn inventory() -> EntitySelector { HasInventory }

    pub fn is_a(taxon : &'static Taxon) -> EntitySelector { IsA(taxon.into()) }
    pub fn has_equipment_kind(taxon : &'static Taxon) -> EntitySelector { HasEquipmentKind(taxon.into()) }
    pub fn has_attack_kind(taxon : &'static Taxon) -> EntitySelector { HasAttackKind(taxon.into()) }

    pub fn within_range(self, hex_range : u32, of : Entity) -> Self {
        self.and(InMoveRange { hex_range, of })
    }

    pub fn and (self, other : EntitySelector) -> EntitySelector {
        EntitySelector::And(box self, box other)
    }
    pub fn or(self, other : EntitySelector) -> EntitySelector {
        EntitySelector::Or(box self, box other)
    }

    pub fn article_string(&self, view: &WorldView) -> String {
        use common::language::*;
        match self {
            EntitySelector::IsA(taxon) => prefix_with_indefinite_article(taxon.name()),
            EntitySelector::Is(ent) => view.signifier(*ent),
            EntitySelector::Friend { of } => format!("a friend of {}", view.signifier(*of)),
            EntitySelector::Enemy { of } => format!("an enemy of {}", view.signifier(*of)),
            EntitySelector::IsCharacter => format!("a character"),
            EntitySelector::HasInventory => format!("something with an inventory"),
            EntitySelector::HasEquipmentKind(taxon) => prefix_with_indefinite_article(taxon.name()),
            EntitySelector::Any => format!("anything"),
            _ => format!("{:?}",self)
        }
    }

//    pub fn as_vec(&self) -> Vec<EntitySelector> {
//        match self {
//            And(a, b) => a.as_vec().extended_by(b.as_vec()),
//            Or(a, b) => a.as_vec().extended_by(b.as_vec()),
//        }
//    }
}
