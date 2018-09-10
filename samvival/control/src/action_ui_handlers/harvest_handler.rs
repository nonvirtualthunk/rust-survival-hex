use common::prelude::*;

use std::collections::HashMap;

use gui::harvest_detail_widget::*;
use action_ui_handlers::player_action_handler::PlayerActionHandler;
use gui::MouseButton;
use gui::PlayerActionType;
use gui::GUI;
use gui::DelegateToWidget;
use gui::GUILayer;
use gui::WidgetContainer;
use gui::GameState;
use graphics::GraphicsResources;
use graphics::prelude::*;

use game::prelude::*;

use game::logic::harvest;
use game::entities::Harvestable;
use game::entities::Resources;
use noisy_float::types::r32;

pub(crate) struct HarvestHandler {
    harvest_summary : HarvestSummaryWidget,
    preferred_resource_by_character : HashMap<Entity, Entity>,
}

impl HarvestHandler {
    pub fn new() -> Box<PlayerActionHandler> {
        box HarvestHandler {
            harvest_summary : HarvestSummaryWidget::new(),
            preferred_resource_by_character : HashMap::new(),
        }
    }
}

impl PlayerActionHandler for HarvestHandler {
    fn handle_click(&mut self, world: &mut World, game_state: &GameState, player_action: &PlayerActionType, button: MouseButton) -> bool {
        if let Some(selected) = game_state.selected_character {
            if let PlayerActionType::Harvest = player_action {
                let view = world.view();
                let preferred_resource = self.preferred_resource_by_character.get(&selected);

                let char_data = world.view().character(selected);
                let pos : AxialCoord = char_data.position.hex;
                let target_hex : AxialCoord = game_state.hovered_hex_coord;
                if pos.distance(&target_hex) <= r32(1.0) {
                    let all_harvestables = harvest::harvestables_at(world.view(), target_hex);
                    if let Some(first_harvestable) = all_harvestables.first() {
                        let matching_preferred_harvestable = preferred_resource.and_then(|pr| {
                            all_harvestables.find(|h| &view.data::<Harvestable>(*h).resource == pr)
                        }).cloned();

                        use game::entities::ActionType;
                        let chosen_harvestable = matching_preferred_harvestable.unwrap_or(*first_harvestable);

                        println!("attempting to harvest");
                        harvest::harvest(world, selected, target_hex, chosen_harvestable, false, None);
                        return true;
                    }
                }
            }
        }
        false
    }

    fn draw(&mut self, world_view: &WorldView, game_state: &GameState, player_action: &PlayerActionType) -> DrawList {
        DrawList::none()
    }

    fn update_widgets(&mut self, gui: &mut GUI, grsrc: &mut GraphicsResources, world: &World, world_view: &WorldView, game_state: &GameState, player_action: &PlayerActionType) {
        if let Some(selected) = game_state.selected_character {
            if let PlayerActionType::Harvest = player_action {
                self.harvest_summary.update(gui, world, world.view(), grsrc, game_state.mouse_pixel_pos, selected, game_state.hovered_hex_coord, false);
                return;
            }
        }
        self.hide_widgets(gui);
    }

    fn hide_widgets(&mut self, gui: &mut GUI) {
        self.harvest_summary.set_showing(false).reapply(gui);
    }
}