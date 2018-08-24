use common::prelude::*;
use prelude::*;
use entities::*;
use logic;
use rand::Rng;
use game::DebugData;


pub struct SpawningComponent {

}

impl SpawningComponent {
    pub fn register(world : &mut World) {
        world.add_callback(|world: &mut World, evt : &GameEventWrapper<GameEvent>| {
            let view = world.view();
            if let Some(GameEvent::FactionTurn { turn_number, faction }) = evt.if_starting() {
                for (ent,spawner_data) in view.entities_with_data::<MonsterSpawnerData>() {
                    if let Some(char_data) = view.data_opt::<CharacterData>(*ent) {
                        let allegiance = view.data::<AllegianceData>(*ent);
                        if &allegiance.faction == faction {
                            if let Some(spawner_pos) = view.data_opt::<PositionData>(*ent) {
                                for spawn in &spawner_data.spawns {
                                    let turn_offset = *turn_number as i32 - spawn.start_spawn_turn;
                                    if turn_offset % (spawn.turns_between_spawns+1) == 0 {
                                        let possible_spawn_points = spawner_pos.hex.neighbors_vec();
                                        let valid_spawn_points = possible_spawn_points.iter()
                                            .filter(|p| view.tile_ent_opt(**p).map(|t| t.occupied_by.is_none()).unwrap_or(false))
                                            .collect_vec();

                                        if valid_spawn_points.non_empty() {
                                            let mut rand = world.random(144);

                                            let spawn_point = valid_spawn_points[rand.gen_range(0,valid_spawn_points.len())];
                                            let monster = spawn.entity.clone()
                                                .with(DebugData { name : strf("spawned monster") })
                                                .with(PositionData { hex : *spawn_point})
                                                .with(AllegianceData { faction : allegiance.faction })
                                                .create(world);
                                            logic::movement::place_entity_in_world(world, monster, *spawn_point);
                                        } else {
                                            warn!("No valid spawn locations");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
    }
}