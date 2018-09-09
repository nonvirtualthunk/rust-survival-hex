use common::prelude::*;
use prelude::*;
use terrain;
use common::Color;
use data::entities::faction::FactionData;
use data::entities::time::TurnData;
use data::entities::tile::TileData;
use game::DebugData;


pub struct Testbed {
    player_faction : Entity
}

pub struct TestbedConfig {
    pub map_radius : i32
}
impl Default for TestbedConfig {
    fn default() -> Self {
        TestbedConfig {
            map_radius : 10
        }
    }
}

pub fn in_testbed<F : Fn(&mut World, Testbed)>(func : F) {
    in_custom_testbed(TestbedConfig::default(), func)
}

pub fn in_custom_testbed<F : Fn(&mut World, Testbed)>(config : TestbedConfig, func : F) {
    let mut world = create_world();

    for tile in terrain::generator::generate(&mut world, config.map_radius) {
        let tile = tile.with(DebugData { name : strf("world tile") }).create(&mut world);
        let pos = world.data::<TileData>(tile).position;
        world.index_entity(tile, pos);
    }

    let player_faction = EntityBuilder::new()
        .with(FactionData {
            name: String::from("Player"),
            color: Color::new(1.1, 0.3, 0.3, 1.0),
            player_faction: true,
        })
        .with(DebugData { name : strf("player faction") })
        .create(&mut world);

    world.attach_world_data(TurnData {
        turn_number: 0,
        active_faction: player_faction,
    });


    let enemy_faction = EntityBuilder::new()
        .with(FactionData {
            name: String::from("Enemy"),
            color: Color::new(0.3, 0.3, 0.9, 1.0),
            player_faction: false,
        })
        .with(DebugData { name : strf("enemy faction") })
        .create(&mut world);

    func(&mut world, Testbed { player_faction });
}