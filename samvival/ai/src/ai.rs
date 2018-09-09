use common::prelude::*;
use game::prelude::*;
use noisy_float::types::r32;
use game::logic;
use game::logic::{faction,combat,movement};
use game::entities::{PositionData, CharacterData, AllegianceData};

pub fn take_ai_actions(world: &mut World, faction : Entity) {
    let world_view = world.view();
    for (cref, cur_data) in world_view.entities_with_data::<CharacterData>() {
        let allegiance = world_view.data::<AllegianceData>(*cref);
        if allegiance.faction == faction && cur_data.is_alive() {
            // these are enemies, now we get to decide what they want to do
            ai_action(&cref, &cur_data, world, world_view);
        }
    }
}


fn ai_action(ai_ref: &Entity, cdata: &CharacterData, world: &mut World, world_view: &WorldView) {

    let ai = world_view.character(*ai_ref);

    if ai.movement.move_speed == Sext::of(0) {

    } else {
        let closest_enemy = world_view.entities_with_data::<CharacterData>()
            .filter(|&(_, c)| c.is_alive())
            .filter(|&(cref, _)| faction::is_enemy(world_view, *ai_ref, *cref))
            .min_by_key(|t| world_view.data::<PositionData>(*t.0).hex.distance(&ai.position.hex));

        if let Some(closest) = closest_enemy {
            let enemy_ref: &Entity = closest.0;
            let enemy_data = world_view.character(*closest.0);

            if enemy_data.position.distance(&ai.position) >= r32(1.5) {
                if let Some(path) = movement::path_any_v(world_view, *ai_ref, ai.position.hex, &enemy_data.position.hex.neighbors_vec(), enemy_data.position.hex) {
                    movement::handle_move(world, *ai_ref, path.0.as_slice())
                } else {
                    println!("No move towards closest enemy, stalling");
                }
            }

            if enemy_data.position.distance(&ai.position) < r32(1.5) {
                let all_possible_attacks = combat::possible_attack_refs(world_view, *ai_ref);
                if let Some(attack) = all_possible_attacks.first() {
                    combat::handle_attack(world, *ai_ref, *enemy_ref, attack);
                }
            }
        } else {
            if let Some(path) = logic::movement::path(world_view, *ai_ref, ai.position.hex, ai.position.hex.neighbor(0)) {
                movement::handle_move(world, *ai_ref, path.0.as_slice());
            }
        }
    }
}