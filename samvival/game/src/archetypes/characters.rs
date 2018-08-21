use archetypes::ArchetypeLibrary;
use prelude::*;
use common::prelude::*;

use entities::*;
use std::collections::HashMap;

pub fn character_archetypes() -> ArchetypeLibrary {

    let baseline : EntityBuilder = EntityBuilder::new()
        .with(SkillData::default())
        .with(InventoryData {
            items : Vec::new(),
            inventory_size : Some(5),
        })
        .with(EquipmentData::default())
        .with(PositionData::default())
        .with(GraphicsData::default())
        .with(ActionData::default())
        .with(ModifierTrackingData::default())
        .with(IdentityData::of_kind(taxonomy::Person));


    let human = baseline.clone();

    let mut archetypes_by_name = HashMap::new();
    archetypes_by_name.insert(strf("human"), human);

    ArchetypeLibrary {
        archetypes_by_name,
        default : baseline
    }
}