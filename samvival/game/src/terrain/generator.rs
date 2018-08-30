use common::prelude::*;
use common::hex::*;
use prelude::*;
use entities::InventoryData;
use entities::TileData;


use noise::RidgedMulti;
use noise::OpenSimplex;
use noise::Worley;
use cgmath::InnerSpace;
use noise::NoiseFn;


pub fn generate(radius: i32) -> Vec<EntityBuilder> {
    let r = radius;

    let mut ret = Vec::new();

    let mut height_noise = RidgedMulti::new();
    height_noise.octaves = 3;
    height_noise.frequency = 0.7;
    height_noise.persistence = 1.1;

    let forest_noise = OpenSimplex::new();
    let forest_worley = Worley::new();

    for x in -r .. r {
        for y in -r.. r {
            let coord = AxialCoord::new(x, y);
            if coord.as_cart_vec().magnitude2() < 60.0 * 60.0 {
                let mut tile_data = TileData {
                    position: coord,
                    main_terrain_name: strf("grass"),
                    secondary_terrain_name: None,
                    move_cost: Sext::of(1),
                    cover: 0,
                    occupied_by: None,
                    elevation: 0,
                };

                let raw = forest_noise.get([x as f64 * 0.074, y as f64 * 0.074]);
                let forest_value = if forest_worley.get([x as f64 * 0.5, y as f64 * 0.5]) > -0.2 {
                    raw
                } else {
                    -1.0
                };

                let noise = height_noise.get([x as f64 * 0.05, y as f64 * 0.05]);
                if noise > 0.35 {
                    tile_data.main_terrain_name = strf("mountains");
                    tile_data.move_cost = Sext::of(3);
                    tile_data.elevation = 2;
                } else if noise > 0.0 {
                    tile_data.main_terrain_name = strf("hills");
                    tile_data.move_cost = Sext::of(2);
                    tile_data.elevation = 1;
                    if forest_value > 0.25 {
                        tile_data.secondary_terrain_name = Some(strf("pine-forest-sparse"));
                        tile_data.move_cost = Sext::of_parts(1, 2);
                        tile_data.cover = 1;
                    }
                } else {
                    tile_data.main_terrain_name = strf("grass");
                    if forest_value > 0.1 {
                        tile_data.secondary_terrain_name = Some(strf("forest"));
                        tile_data.move_cost = Sext::of_parts(1, 3);
                        tile_data.cover = 2;
                    }
                }

                let tile = EntityBuilder::new()
                    .with(tile_data)
                    .with(InventoryData::default());
                ret.push(tile);
            }
        }
    }

    ret
}