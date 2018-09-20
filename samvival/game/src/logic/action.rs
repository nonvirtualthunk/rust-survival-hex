use common::prelude::*;
use prelude::*;
use entities::{AllegianceData,ActionData,ActionType,Action};


pub fn continue_ongoing_actions(world : &mut World, event : &GameEventWrapper<GameEvent>) {
    match event.if_starting() {
        Some(GameEvent::FactionTurn { faction , .. }) => {
            let view = world.view();
            let action_data_store = view.all_data_of_type::<ActionData>();
            for (ent,allegiance) in view.entities_with_data::<AllegianceData>() {
                if &allegiance.faction == faction {
                    let action_data = action_data_store.data(*ent);
                    if let Some(active_action) = &action_data.active_action {
                        apply_action(world, *ent, active_action.clone())
                    }
                }
            }
        },
        _ => ()
    }
}



pub fn apply_action(world: &mut World, character : Entity, action : Action) {
    let view = world.view();

    world.modify(character, ActionData::active_action.set_to(None));
    world.add_event(::game::events::CoreEvent::Mark);

    match action.action_type {
        ActionType::Harvest { from, harvestable, preserve_renewable } => {
            ::logic::harvest::harvest(world, character, from, harvestable, preserve_renewable, Some(action.ap.current))
        },
        _ => error!("No action application has been set up for {:?}", action)
    }
}