use common::prelude::*;

use std::collections::HashMap;

use gui::harvest_detail_widget::*;
use gui::Key;
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
use std::collections::HashSet;
use game::entities::tile::TileStore;
use action_ui_handlers::draw_hex_boundary;
use action_ui_handlers::draw_movement_path;
use gui::events::KeyToNumber;

pub(crate) struct HarvestHandler {
    harvest_summary: HarvestSummaryWidget,
    preferred_harvestable_by_character: HashMap<Entity, Entity>,
    last_harvestable_hexes_key: (GameEventClock, Entity),
    last_harvestable_hexes: HashSet<AxialCoord>,
    last_harvest_overlay_key: (GameEventClock, Entity),
    last_harvest_overlay: DrawList,
}

impl HarvestHandler {
    pub fn new() -> Box<PlayerActionHandler> {
        box HarvestHandler {
            harvest_summary: HarvestSummaryWidget::new(),
            preferred_harvestable_by_character: HashMap::new(),
            last_harvest_overlay: DrawList::none(),
            last_harvest_overlay_key: (0, Entity::sentinel()),
            last_harvestable_hexes: HashSet::new(),
            last_harvestable_hexes_key: (0, Entity::sentinel()),
        }
    }
}

impl PlayerActionHandler for HarvestHandler {
    fn handle_click(&mut self, world: &mut World, game_state: &GameState, player_action: &PlayerActionType, button: MouseButton) -> bool {
        if let Some(selected) = game_state.selected_character {
            if let PlayerActionType::Harvest = player_action {
                let view = world.view();
                let preferred_harvestable = self.preferred_harvestable_by_character.get(&selected);
                let preferred_resource = preferred_harvestable.map(|&h| view.data::<Harvestable>(h).resource);

                let char_data = view.character(selected);
                let pos: AxialCoord = char_data.position.hex;
                let target_hex: AxialCoord = game_state.hovered_hex_coord;
                let range = logic::harvest::harvest_range(view, selected);
                let bad_tile = view.tile_opt(target_hex).map(|t| t.is_occupied()).unwrap_or(true);
                if !bad_tile {
                    if pos.distance(&target_hex) > r32(range as f32) {
                        if let Some((path, cost)) = logic::movement::path_adjacent_to_hex(view, selected, target_hex) {
                            info!("Path found to out-of-range harvest tile, moving");
                            logic::movement::handle_move(world, selected, &path);
                        }
                    }

                    let pos: AxialCoord = char_data.position.hex; // check the new position
                    if pos.distance(&target_hex) <= r32(range as f32) && char_data.action_points.cur_value() > 0 {
                        info!("Within range of harvest");
                        let all_harvestables = harvest::harvestables_sorted_by_desirability_at(world.view(), selected, target_hex);
                        if let Some(first_harvestable) = all_harvestables.first() {
                            let matching_preferred_harvestable =
                                preferred_harvestable.cloned().filter(|h| all_harvestables.contains(h))
                                    .or(preferred_resource.and_then(|pr| {
                                            all_harvestables.find(|h| view.data::<Harvestable>(*h).resource == pr).cloned()
                                        }));

                            use game::entities::ActionType;
                            let chosen_harvestable = matching_preferred_harvestable.unwrap_or(*first_harvestable);

                            info!("Performing harvest");
                            harvest::harvest(world, selected, target_hex, chosen_harvestable, false, None);
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn handle_key_release(&mut self, world: &mut World, game_state: &GameState, player_action: &PlayerActionType, key: Key) -> bool {
        if let (PlayerActionType::Harvest, Some(selected)) = (player_action, game_state.selected_character) {
            if let Some(num) = key.key_to_number() {
                let view = world.view();
                let harvestables = harvest::harvestables_sorted_by_desirability_at(view, selected, game_state.hovered_hex_coord);
                let index = (num - 1) as usize;
                if let Some(at_index) = harvestables.get(index) {
                    self.preferred_harvestable_by_character.insert(selected, *at_index);
                }
                true
            } else {
                false
            }
        } else { false }
    }


    fn draw(&mut self, world_view: &WorldView, game_state: &GameState, player_action: &PlayerActionType) -> DrawList {
        if let (Some(selected), PlayerActionType::Harvest) = (game_state.selected_character, player_action) {
            let key_state = (game_state.display_event_clock, selected);
            let mut draw_list = if self.last_harvest_overlay_key == key_state {
                self.last_harvest_overlay.clone()
            } else {
                let harvestable_hexes = self.harvestable_hexes(world_view, selected);

                let draw_list = draw_hex_boundary(world_view, logic::visibility::faction_visibility_for_character(world_view, selected),
                                                  &harvestable_hexes, ::common::Color::new(0.2, 0.6, 0.2, 0.8));

                self.last_harvest_overlay_key = key_state;
                self.last_harvest_overlay = draw_list.clone();
                draw_list
            };

            if let Some(tile) = world_view.tile_ent_opt(game_state.hovered_hex_coord) {
//                if harvestable_hexes.contains(&game_state.hovered_hex_coord) {
                if let Some((path, cost)) = logic::movement::path_adjacent_to(world_view, selected, tile.entity) {
                    draw_list.extend(draw_movement_path(world_view, selected, path));
                }
//                }
            }

            draw_list
        } else {
            DrawList::none()
        }
    }

    fn update_widgets(&mut self, gui: &mut GUI, grsrc: &mut GraphicsResources, world: &World, world_view: &WorldView, game_state: &GameState, player_action: &PlayerActionType) {
        if let Some(selected) = game_state.selected_character {
            if let PlayerActionType::Harvest = player_action {
                // cannot harvest from your own hex
                if game_state.hovered_hex_coord != logic::movement::position_of(world_view, selected) {
                    let reachable_this_turn = self.harvestable_hexes(world_view, selected).contains(&game_state.hovered_hex_coord);

                    self.harvest_summary.update(gui, world, world.view(), grsrc, game_state.mouse_pixel_pos, selected, game_state.hovered_hex_coord, false, !reachable_this_turn);
                    return;
                }
            }
        }
        self.hide_widgets(gui);
    }

    fn hide_widgets(&mut self, gui: &mut GUI) {
        self.harvest_summary.set_showing(false).reapply(gui);
    }
}

impl HarvestHandler {
    fn harvestable_hexes(&mut self, world_view: &WorldView, selected: Entity) -> HashSet<AxialCoord> {
        let key_state = (world_view.current_time, selected);
        if self.last_harvestable_hexes_key == key_state {
            self.last_harvestable_hexes.clone()
        } else {
            if world_view.data::<CharacterData>(selected).action_points.cur_value() > 0 {
                let hexes = if let Some(movement_type) = logic::movement::default_movement_type(world_view, selected) {
                    let reachable_hexes = logic::movement::hexes_reachable_by_character_this_turn(world_view, selected, movement_type);
                    let mut reachable_hexes_with_neighbors = HashSet::with_capacity((reachable_hexes.len() as f64 * 1.25) as usize);
                    let mut checked = HashSet::with_capacity((reachable_hexes.len() as f64 * 0.2) as usize);
                    for (hex, _) in reachable_hexes {
                        reachable_hexes_with_neighbors.insert(hex);
                        let neighbors = hex.neighbors();
                        for neighbor in &neighbors {
                            if !checked.contains(neighbor) {
                                checked.insert(*neighbor);
                                if let Some(tile) = world_view.tile_opt(*neighbor) {
                                    if tile.occupied_by.is_none() {
                                        reachable_hexes_with_neighbors.insert(*neighbor);
                                    }
                                }
                            }
                        }
                    }
                    reachable_hexes_with_neighbors
                } else { HashSet::new() };

                self.last_harvestable_hexes_key = key_state;
                self.last_harvestable_hexes = hexes.clone();
                hexes
            } else { HashSet::new() }
        }
    }
}