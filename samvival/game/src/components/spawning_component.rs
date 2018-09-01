use common::prelude::*;
use prelude::*;
use data::entities::*;
use logic;
use rand::Rng;
use game::DebugData;
use archetypes::characters::character_archetypes;

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
                                            let entity_archetype = match &spawn.entity {
                                                SpawnEntity::Character(archetype) => character_archetypes().with_name(archetype.as_str()).clone()
                                                    .with(AllegianceData { faction : allegiance.faction })
                                                    .with(DebugData { name : format!("spawned creature: {:?}", archetype) })
                                            };

                                            let spawned_entity = entity_archetype
                                                .with(PositionData { hex : *spawn_point})
                                                .create(world);
                                            logic::movement::place_entity_in_world(world, spawned_entity, *spawn_point);
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