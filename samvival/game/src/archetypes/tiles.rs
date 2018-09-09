//use archetypes::ArchetypeLibrary;
//use common::prelude::*;
//use data::entities::ActionData;
//use data::entities::combat::*;
//use data::entities::EquipmentData;
//use data::entities::GraphicsData;
//use data::entities::IdentityData;
//use data::entities::InventoryData;
//use data::entities::ModifierTrackingData;
//use data::entities::ObserverData;
//use data::entities::PositionData;
//use data::entities::SkillData;
//use data::entities::tile::*;
//use prelude::*;
//use std::collections::HashMap;
//use std::ops::Range;
//use data::entities::effects::*;
//use data::entities::EntitySelector;
//use data::entities::taxonomy;
//use data::entities::skill::Skill;
//
//pub struct TileHarvestable {
//    pub name: String,
//    pub harvestable: Harvestable,
//    pub frequency: f32,
//}
//
//pub struct TileGeology {
//    pub name : String,
//    pub elevation : Range<i32>,
//    pub move_cost : Sext,
//    pub possible_harvestables: Vec<TileHarvestable>,
//}
//
//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct TileVariant {
//    pub frequency: f32,
//    pub harvestables: HashMap<String, Harvestable>,
//    pub tile_data: TileData,
//    pub generation_data: TerrainGenerationData,
//}
//
//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct TileArchetype {
//    primary: TileVariant,
//    variants: Vec<TileVariant>,
//}
//
//#[derive(Clone, Debug, Serialize, Deserialize)]
//pub struct TerrainGenerationData {
//    pub moisture: Range<i32>,
//    pub elevation: Range<i32>,
//    pub temperature: Range<i32>,
//    pub cluster_size: i32,
//}
//
//pub fn harvestable(name: Str, data: Harvestable) -> (String, Harvestable) {
//    (String::from(name), data)
//}
//
//pub fn tile_archetypes(world: &mut World) -> ArchetypeLibrary<TileArchetype> {
//    Resources::init_resources(world);
//
//    let main_resources = &world.view().world_data::<Resources>().main;
//
//    let baseline: EntityBuilder = EntityBuilder::new()
//        .with(InventoryData::default())
//        .with(IdentityData::of_kind(&taxonomy::Person));
//
//    let mut archetypes_by_name = HashMap::new();
//
//
//    archetypes_by_name.insert(strf("grassland"), create_grassland(main_resources));
//
//
//    let dirt_removed_effect = Effects::register_effect(world, Effect::new("dirt removed")
//        .with_modifier(world, TileData::name_modifiers.append(strf("barren")))
//        .with_modifier(world, TileData::fertility.set_to(0)),
//    );
//
//    let stone_mined_effect = Effects::register_effect(world, Effect::new("stone mined")
//        .with_modifier(world, TileData::name_modifiers.append(strf("pitted")))
//        .with_modifier(world, TileData::move_cost::add(Sext::of_parts(0,3))));
//
//
//    let dirt_tile_harvest = |amount : i32| TileHarvestable {
//        name: strf("dirt"),
//        frequency: 1.0,
//        harvestable: Harvestable {
//            action_name: strf("gather dirt"),
//            amount: Reduceable::new(Sext::of(amount)),
//            ap_per_harvest: 8,
//            dice_amount_per_harvest: DicePool::of(1, 2),
//            resource: main_resources.dirt,
//            on_depletion_effects: vec![dirt_removed_effect],
//            tool: EntitySelector::is_a(&taxonomy::tools::Shovel),
//            .. Default::default()
//        },
//    };
//    let stone_tile_harvest = |amount : i32, frequency: f32| TileHarvestable {
//        name: strf("loose stones"),
//        frequency,
//        harvestable: Harvestable {
//            action_name: strf("gather loose stones"),
//            name_modifiers: vec![strf("stony")],
//            amount: Reduceable::new(Sext::of(amount)),
//            ap_per_harvest: 4,
//            fixed_amount_per_harvest: 1,
//            resource: main_resources.loose_stone,
//            .. Default::default()
//        }
//    };
//
//    let dirt_plains = TileGeology {
//        name : strf("plains"),
//        elevation: 0..0,
//        move_cost: Sext::of(1),
//        possible_harvestables: vec![
//            dirt_tile_harvest(8),
//            stone_tile_harvest(2, 0.05)
//        ],
//    };
//
//    let hills = TileGeology {
//        name : strf("hills"),
//        elevation: 1..2,
//        move_cost: Sext::of(1),
//        possible_harvestables: vec![
//            dirt_tile_harvest(2),
//            stone_tile_harvest(4,0.75),
//            TileHarvestable {
//                name: strf("stone"),
//                frequency: 0.05,
//                harvestable: Harvestable {
//                    action_name: strf("quarry stone"),
//                    amount: Reduceable::new(Sext::of(16)),
//                    ap_per_harvest: 16,
//                    dice_amount_per_harvest: DicePool::of(1,4),
//                    resource: main_resources.quarried_stone,
//                    requires_tool: true,
//                    tool: EntitySelector::is_a(&taxonomy::tools::MiningTool),
//                    on_depletion_effects : vec![stone_mined_effect],
//                    skills_used : vec![Skill::Mining],
//                    .. Default::default()
//                }
//            },
//            TileHarvestable {
//                name: strf("iron"),
//                frequency: 0.05,
//                harvestable: Harvestable {
//                    action_name: strf("mine iron"),
//                    name_modifiers: vec![strf("iron")],
//                    amount: Reduceable::new(Sext::of(8)),
//                    ap_per_harvest: 16,
//                    dice_amount_per_harvest: DicePool::of(1,4),
//                    resource: main_resources.iron,
//                    requires_tool: true,
//                    tool: EntitySelector::is_a(&taxonomy::tools::MiningTool),
//                    skills_used : vec![Skill::Mining],
//                    .. Default::default()
//                }
//            }
//        ],
//    };
//
//    ArchetypeLibrary::<TileArchetype> {
//        archetypes_by_name,
//        default: TileArchetype {
//            primary: TileVariant {
//                harvestables: HashMap::new(),
//                generation_data: TerrainGenerationData {
//                    moisture: 0..0,
//                    elevation: 0..0,
//                    temperature: 0..0,
//                    cluster_size: 0,
//                },
//                tile_data: TileData::default(),
//                frequency: 0.0f32,
//            },
//            variants: Vec::new(),
//        },
//    }
//}
//
//fn create_grassland(main_resources: &ConstantResources) -> TileArchetype {
//    let primary = TileVariant {
//        harvestables: [harvestable("grass", Harvestable {
//            action_name: strf("cut grass"),
//            amount: Reduceable::new(Sext::of(3)),
//            ap_per_harvest: 4,
//            dice_amount_per_harvest: DicePool::none(),
//            fixed_amount_per_harvest: 2,
//            renew_rate: Some(Sext::of_parts(0, 3)),
//            resource: main_resources.straw,
//            on_depletion_effects: Vec::new(),
//            tool: EntitySelector::is_a(&taxonomy::tools::SharpTool)
//                .or(EntitySelector::is_a(&taxonomy::weapons::BladedWeapon)),
//            requires_tool: false,
//            skills_used: vec![Skill::Farming],
//            character_requirements: EntitySelector::Any,
//        })].into_iter().cloned().collect(),
//        tile_data: TileData {
//            cover: 0,
//            elevation: 0,
//            main_terrain_name: strf("grass"),
//            secondary_terrain_name: None,
//            move_cost: Sext::of(1),
//            ..Default::default()
//        },
//        generation_data: TerrainGenerationData {
//            moisture: 1..4,
//            elevation: 0..0,
//            temperature: 0..3,
//            cluster_size: 5,
//        },
//        frequency: 0.6,
//    };
//
//    TileArchetype {
//        primary,
//        variants: Vec::new(),
//    }
//}
//
//
///// Geology -> Hydrology -> Biology
///// So maybe we have our baseline tile archetypes be just the geological stuff, plains, hills, mountains, etc
///// Then we could have forestation or other plant based stuff be effects layered onto the baseline tile, which can
///// just be cancelled when they are harvested out.
//
//pub fn create_forested_versions<'a>(world: &mut World, archetypes: impl Iterator<Item=&'a TileArchetype>) -> Vec<TileArchetype> {
//    let main_resources = &world.view().world_data::<Resources>().main;
//
//    let effect = Effect::new(None)
//        .with_modifier(world, HarvestableData::harvestables.remove_key(String::from("trees")))
//        .with_modifier(world, TileData::secondary_terrain_name.set_to(None));
//    let deforest_effect = Effects::register_effect(world, effect);
//
//    let mut new_archetypes = Vec::new();
//    for arch in archetypes {
//        let mut ret: TileArchetype = arch.clone();
//        let tree_harvest = harvestable("trees", Harvestable {
//            action_name: strf("chop trees"),
//            amount: Reduceable::new(Sext::of(4)),
//            ap_per_harvest: 8,
//            dice_amount_per_harvest: DicePool::of(1, 2),
//            fixed_amount_per_harvest: 0,
//            renew_rate: Some(Sext::of_parts(0, 1)),
//            resource: main_resources.wood,
//            on_depletion_effects: vec![deforest_effect.clone()],
//            tool: EntitySelector::is_a(&taxonomy::Axe),
//            requires_tool: true,
//            skills_used: vec![Skill::ForestSurvival, Skill::Axe],
//            character_requirements: EntitySelector::Any,
//        });
//        ret.primary.harvestables.insert(tree_harvest.0, tree_harvest.1);
//        ret.primary.tile_data.secondary_terrain_name = Some(strf("forest"));
//        new_archetypes.push(ret);
//    }
//
//    new_archetypes
//}