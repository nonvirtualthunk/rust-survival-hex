use archetypes::ArchetypeLibrary;
use common::prelude::*;
use data::entities::ActionData;
use data::entities::combat::*;
use data::entities::EquipmentData;
use data::entities::GraphicsData;
use data::entities::IdentityData;
use data::entities::InventoryData;
use data::entities::ModifierTrackingData;
use data::entities::ObserverData;
use data::entities::PositionData;
use data::entities::SkillData;
use data::entities::tile::*;
use prelude::*;
use std::collections::HashMap;
use std::ops::Range;
use data::entities::effects::*;
use data::entities::EntitySelector;
use data::entities::taxonomy;
use data::entities::skill::Skill;

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct TileArchetype {
    pub harvestable_data: HarvestableData,
    pub tile_data: TileData,
    pub generation_data: TerrainGenerationData,
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct TerrainGenerationData {
    pub moisture: Range<i32>,
    pub elevation: Range<i32>,
    pub temperature: Range<i32>,
}

pub fn harvestable(world: &mut World, name : Str, data : Harvestable) -> (String, Entity) {
    (String::from(name), EntityBuilder::new()
        .with(data)
        .create(world))
}

pub fn tile_archetypes(world: &mut World) -> ArchetypeLibrary<TileArchetype> {
    Resources::init_resources(world);

    let main_resources = &world.view().world_data::<Resources>().main;

    let baseline: EntityBuilder = EntityBuilder::new()
        .with(InventoryData::default())
        .with(IdentityData::of_kind(&taxonomy::Person));

    let mut archetypes_by_name = HashMap::new();

    archetypes_by_name.insert(strf("grassland"), TileArchetype {
        harvestable_data: HarvestableData {
            harvestables: [harvestable(world, "grass",Harvestable {
                action_name: strf("cut grass"),
                amount: Reduceable::new(Sext::of(3)),
                ap_per_harvest: 4,
                dice_amount_per_harvest: DicePool::none(),
                fixed_amount_per_harvest: 2,
                renew_rate: Some(Sext::of_parts(0, 3)),
                resource: main_resources.straw,
                on_depletion_effects: Vec::new(),
                tool: EntitySelector::is_a(&taxonomy::tools::SharpTool)
                    .or(EntitySelector::is_a(&taxonomy::weapons::BladedWeapon)),
                requires_tool: false,
                skills_used: vec![Skill::Farming],
                character_requirements: EntitySelector::Any,
            })].into_iter().cloned().collect()
        },
        tile_data: TileData {
            cover: 0,
            elevation: 0,
            main_terrain_name: strf("grass"),
            secondary_terrain_name: None,
            move_cost: Sext::of(1),
            ..Default::default()
        },
        generation_data: TerrainGenerationData {
            moisture: 1..4,
            elevation: 0..0,
            temperature: 0..3,
        },
    });

    ArchetypeLibrary::<TileArchetype> {
        archetypes_by_name,
        default: TileArchetype {
            harvestable_data : HarvestableData::default(),
            generation_data : TerrainGenerationData {
                moisture : 0..0,
                elevation : 0..0,
                temperature : 0..0
            },
            tile_data : TileData::default()
        },
    }
}

pub fn create_forested_version<'a> (world: &mut World, archetypes: impl Iterator<Item=&'a TileArchetype>) -> Vec<TileArchetype> {
    let main_resources = &world.view().world_data::<Resources>().main;

    let effect = Effect::new(None)
        .with_modifier(world, HarvestableData::harvestables.remove_key(String::from("trees")))
        .with_modifier(world, TileData::secondary_terrain_name.set_to(None));
    let deforest_effect = Effects::register_effect(world, effect);

    let mut new_archetypes = Vec::new();
    for arch in archetypes {
        let mut ret : TileArchetype = arch.clone();
        let tree_harvest = harvestable(world, "trees", Harvestable {
            action_name: strf("chop trees"),
            amount: Reduceable::new(Sext::of(4)),
            ap_per_harvest: 8,
            dice_amount_per_harvest : DicePool::of(1,2),
            fixed_amount_per_harvest : 0,
            renew_rate: Some(Sext::of_parts(0,1)),
            resource: main_resources.wood,
            on_depletion_effects : vec![deforest_effect],
            tool : EntitySelector::is_a(&taxonomy::Axe),
            requires_tool : true,
            skills_used: vec![Skill::ForestSurvival, Skill::Axe],
            character_requirements: EntitySelector::Any
        });
        ret.harvestable_data.harvestables.insert(tree_harvest.0, tree_harvest.1);
        ret.tile_data.secondary_terrain_name = Some(strf("forest"));
        new_archetypes.push(ret);
    }

    new_archetypes
}