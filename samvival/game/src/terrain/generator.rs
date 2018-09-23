use common::prelude::*;
use common::hex::*;
use prelude::*;
use data::entities::InventoryData;
use data::entities::TileData;


use noise::RidgedMulti;
use noise::OpenSimplex;
use noise::Worley;
use cgmath::InnerSpace;
use noise::NoiseFn;

use entities::VegetationData;
use entities::TerrainData;
use entities::Resources;
use entities::Effects;
use entities::Effect;
use entities::EntitySelector;
use entities::Skill;
use entities::Harvestable;
use entities::DepletionBehavior;
use entities::ToolUse;

use std::collections::HashMap;
use rand::distributions::Sample;

pub fn generate(world : &mut World, radius: i32) -> Vec<EntityBuilder> {
    let r = radius;

    let mut ret = Vec::new();

    let mut height_noise = RidgedMulti::new();
    height_noise.octaves = 3;
    height_noise.frequency = 0.7;
    height_noise.persistence = 1.1;

    let forest_noise = OpenSimplex::new();
    let forest_worley = Worley::new();

    let mut rng = world.random(2344109);
    let mut normal_dist = ::rand::distributions::Normal::new(0.5,0.15);

    let main_resources = &world.view().world_data::<Resources>().main;

    let dirt_removed_effect_contents = Effect::new("dirt removed")
        .with_modifier(world, TileData::name_modifiers.append(strf("barren")))
        .with_modifier(world, TerrainData::fertility.set_to(0));
    let dirt_removed_effect = Effects::register_effect(world, dirt_removed_effect_contents);

    let stone_mined_effect_contents = Effect::new("stone mined")
        .with_modifier(world, TileData::name_modifiers.append(strf("pitted")))
        .with_modifier(world, TerrainData::move_cost.add(Sext::of_parts(0,3)));
    let stone_mined_effect = Effects::register_effect(world, stone_mined_effect_contents);

    let dirt_tile_harvest = |world : &mut World, amount : i32| EntityBuilder::new().with(Harvestable {
        action_name: strf("gather dirt"),
        amount: Reduceable::new(Sext::of(amount)),
        ap_per_harvest: 8,
        dice_amount_per_harvest: DicePool::of(1, 2),
        resource: main_resources.dirt,
        on_depletion : DepletionBehavior::Custom(vec![dirt_removed_effect]),
        tool: EntitySelector::is_a(&taxonomy::tools::Shovel),
        .. Default::default()
    }).create(world);
    let stone_tile_harvest = |world : &mut World, amount : i32, frequency: f32| EntityBuilder::new().with(Harvestable {
        action_name: strf("gather loose stones"),
        name_modifiers: vec![strf("stony")],
        amount: Reduceable::new(Sext::of(amount)),
        ap_per_harvest: 4,
        fixed_amount_per_harvest: 1,
        resource: main_resources.loose_stone,
        .. Default::default()
    }).create(world);
    let quarry_stone_harvest = |world : &mut World, amount : i32| EntityBuilder::new().with(Harvestable {
        action_name: strf("quarry stone"),
        amount: Reduceable::new(Sext::of(amount)),
        ap_per_harvest: 16,
        dice_amount_per_harvest: DicePool::of(1,4),
        resource: main_resources.quarried_stone,
        tool_use: ToolUse::Required,
        tool: EntitySelector::is_a(&taxonomy::tools::MiningTool),
        on_depletion : DepletionBehavior::Custom(vec![stone_mined_effect]),
        skills_used : vec![Skill::Mining],
        .. Default::default()
    }).create(world);

    for x in -r .. r {
        for y in -r.. r {
            let coord = AxialCoord::new(x, y);
            if coord.as_cart_vec().magnitude2() < 60.0 * 60.0 {
                let mut vegetation_data : Option<VegetationData> = None;
                let mut terrain_data = TerrainData::default();


                let raw = forest_noise.get([x as f64 * 0.074, y as f64 * 0.074]);
                let forest_value = if forest_worley.get([x as f64 * 0.5, y as f64 * 0.5]) > -0.2 {
                    raw
                } else {
                    -1.0
                };

                let noise = height_noise.get([x as f64 * 0.05, y as f64 * 0.05]);
                if noise > 0.35 {
                    terrain_data.kind = Taxon::of(&taxonomy::terrain::Mountains);
                    terrain_data.move_cost = Sext::of(3);
                    terrain_data.elevation = 2;
                } else if noise > 0.0 {
                    terrain_data.kind = Taxon::of(&taxonomy::terrain::Hills);
                    terrain_data.move_cost = Sext::of(2);
                    terrain_data.elevation = 1;
                    let stone_amount = 3 + (normal_dist.sample(&mut rng) * 10.0) as i32;
                    terrain_data.harvestables.insert(strf("quarry stone"), (quarry_stone_harvest.clone())(world, stone_amount));
                    if forest_value > 0.25 {
                        vegetation_data = Some(VegetationData {
                            harvestables : HashMap::new(),
                            move_cost : Sext::of_parts(0,3),
                            cover : 2,
                            kind : Taxon::of(&taxonomy::vegetation::PineForest)
                        });
                    }
                } else {
                    terrain_data.kind = Taxon::of(&taxonomy::terrain::Plains);
                    terrain_data.move_cost = Sext::of(1);
                    terrain_data.elevation = 0;
                    terrain_data.harvestables.insert(strf("dirt"), (dirt_tile_harvest.clone())(world, 6 + (normal_dist.sample(&mut rng) * 3.0) as i32));
                    if forest_value > 0.1 {
                        let mut harvestables = HashMap::new();
                        harvestables.insert(strf("wood"), EntityBuilder::new().with(Harvestable {
                            action_name: strf("chop wood"),
                            amount: Reduceable::new(Sext::of(((normal_dist.sample(&mut rng) * 5 as f64) as i32).max(0))),
                            ap_per_harvest: 8,
                            dice_amount_per_harvest: DicePool::of(1,4),
                            resource: main_resources.wood,
                            tool_use: ToolUse::DifficultWithout { amount_limit : Some(1), ap_increase : Some(8) },
                            tool: EntitySelector::is_a(&taxonomy::Axe),
                            on_depletion: DepletionBehavior::RemoveLayer,
                            skills_used : vec![Skill::Axe, Skill::ForestSurvival],
                            .. Default::default()
                        }).create(world));

                        vegetation_data = Some(VegetationData {
                            harvestables,
                            move_cost : Sext::of_parts(0,3),
                            cover : 2,
                            kind : Taxon::of(&taxonomy::vegetation::DeciduousForest)
                        });
                    } else {
                        vegetation_data = Some(VegetationData {
                            harvestables : HashMap::new(),
                            move_cost : Sext::of(0),
                            cover : 0,
                            kind : Taxon::of(&taxonomy::vegetation::Grassland)
                        });
                    }
                }


                let tile_data = TileData {
                    position: coord,
                    occupied_by: None,
                    name_modifiers : Vec::new()
                };

                let tile = EntityBuilder::new()
                    .with(tile_data)
                    .with(terrain_data)
                    .with_opt(vegetation_data);
//                    .with(InventoryData::default());
                ret.push(tile);
            }
        }
    }

    ret
}