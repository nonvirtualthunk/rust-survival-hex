use common::prelude::*;
use game::prelude::*;
use entities::{MovementData, CharacterData, TurnData, FactionData};
use prelude::GameEvent;


pub fn end_faction_turn(world : &mut World) {
    let world_view = world.view();
    let turn_data = world_view.world_data::<TurnData>();
    let current_turn = turn_data.turn_number;

    let prev_faction = turn_data.active_faction;
    let all_factions = world_view.entities_with_data::<FactionData>().map(|(faction,_)| faction).collect_vec();
    let cur_index = all_factions.iter().position(|f| *f == &prev_faction).map(|p| p as i32).unwrap_or(-1);
    let next_index = (cur_index + 1) % (all_factions.len() as i32);
    let next_faction = *all_factions[next_index as usize];
    let next_faction_data = world_view.data::<FactionData>(next_faction);


    // if we're back around at the beginning, start a new overall turn
    if next_index == 0 {
        for (cref, cdat) in world_view.entities_with_data::<CharacterData>() {
            world.modify_with_desc(*cref, MovementData::moves.set_to(Sext::of(0)), None);
            world.modify_with_desc(*cref, CharacterData::action_points.reset(), None);
            world.modify_with_desc(*cref, CharacterData::stamina.recover_by(cdat.stamina_recovery), None);
        }

        let turn_number = current_turn + 1;
        world.modify_world(TurnData::turn_number.set_to(turn_number), None);

        world.add_event(GameEvent::TurnStart { turn_number });
    }


    world.modify_world(TurnData::active_faction.set_to(next_faction), None);
    world.end_event(GameEvent::FactionTurn { turn_number : current_turn, faction : prev_faction });
    world.start_event(GameEvent::FactionTurn { turn_number : current_turn, faction : next_faction });
}