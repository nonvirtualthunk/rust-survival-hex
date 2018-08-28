use common::prelude::*;
use prelude::*;

use logic::test::testbed::*;
use archetypes::*;
use logic;
use std::time::Instant;
use std::time::Duration;



#[test]
pub fn flood_search_performance_test() {
    let config = TestbedConfig {
        map_radius : 70,
        .. Default::default()
    };
    in_custom_testbed(config, |world, testbed| {
        let character = character_archetypes().with_name("human").create(world);

        let start = Instant::now();
        let hexes = logic::movement::hexes_in_range(world, character, Sext::of(8));

        let duration = Instant::now().duration_since(start);

        println!("Took {:?} to identify {} hexes", duration, hexes.len());
    });
}