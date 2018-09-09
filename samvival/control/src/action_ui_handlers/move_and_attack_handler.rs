use common::prelude::*;
use game::prelude::*;
use graphics::prelude::*;
use gui::GameState;
use common::Color;
use game::logic;
use game::entities::combat::*;
use game::entities::TurnData;
use game::entities::VisibilityData;
use game::entities::TileStore;
use game::entities::PositionData;
use game::entities::Character;
use game::entities::Visibility;
use gui::attack_descriptions::AttackDetailsWidget;

#[derive(Default)]
struct MoveAndAttackHandler {
    attack_details_widget : AttackDetailsWidget
}


pub fn draw_move_and_attack_overlay(view: &WorldView, game_state: &GameState, attack_ref: AttackRef) -> DrawList {
    if let Some(selected) = game_state.selected_character {
        if let Some(movement_type) = logic::movement::default_movement_type(view, selected) {
            let cdata = view.character(selected);
            // if it's not the player's turn, don't display UI
            if view.world_data::<TurnData>().active_faction != cdata.allegiance.faction {
                return DrawList::none();
            }

            let current_position = cdata.position.hex;

            let visibility = view.world_data::<VisibilityData>().visibility_for(game_state.player_faction);

            let hexes = logic::movement::hexes_reachable_by_character_this_turn(view, selected, movement_type);

            let mut draw_list = DrawList::none();

            if let Some(attack) = attack_ref.resolve(view, selected) {
                draw_strike_boundaries(view, &cdata, visibility, &hexes, &mut draw_list, &attack);

                let targeted_char = targeted_character(view, game_state, selected);

                if targeted_char == None && ! game_state.animating {
                    draw_in_range_markers(view, selected, visibility, &mut draw_list, &attack);
                }

                if let Some(hovered_char) = targeted_char {
                    if ! game_state.animating {
                        draw_attack_enemy_overlay(view, game_state, attack_ref, selected, &current_position, hexes, &mut draw_list, &attack, hovered_char)
                    }
                } else if let Some(hovered_tile) = view.tile_opt(game_state.hovered_hex_coord) {
                    if let Some((path, cost)) = logic::movement::path_to(view, selected, hovered_tile.position) {
                        for hex in path {
                            draw_list = draw_list.with_quad(Quad::new_cart(String::from("ui/feet"), hex.as_cart_vec()).centered());
                        }
                    }
                }
            } else {
                warn!("No attack possible, whatsoever, for entity {}", selected);
            }

            draw_list
        } else {
            DrawList::none()
        }
    } else {
        DrawList::none()
    }
}

fn draw_attack_enemy_overlay(view: &WorldView, game_state: &GameState, attack_ref: AttackRef, selected: Entity,
                             current_position: &AxialCoord,
                             hexes: HashMap<AxialCoord, f64>,
                             draw_list: &mut DrawList,
                             attack: &Attack,
                             hovered_char: Entity) -> () {

    let attack_from = if logic::combat::can_attack(view, selected, hovered_char, attack, None, None) {
        Some(current_position.clone())
    } else if let Some((attack_from, cost)) = logic::combat::closest_attack_location_with_cost(view, hexes, selected, hovered_char, &attack, game_state.mouse_cart_vec()) {
        Some(attack_from)
    } else {
        None
    };

    if let Some(attack_from) = attack_from {
        let logic::combat::AttackTargets { hexes, characters } = logic::combat::targets_for_attack(view, selected, &attack_ref, hovered_char, Some(attack_from));
        let character_hexes = characters.map(|c| view.data::<PositionData>(*c).hex);
        for hex in hexes {
            let (img_base, color, char_hex) = if character_hexes.contains(&hex) {
                ("ui/hex/hex_edge", Color::new(0.7, 0.1, 0.1, 1.0), true)
            } else {
                ("ui/hex/hex_edge_narrow", Color::new(0.9, 0.5, 0.4, 0.8), false)
            };

            let closest_q = hex.side_closest_to(&attack_from, current_position);
            for q in 0..6 {
                if q == closest_q || !char_hex {
                    draw_list.add_quad(Quad::new(format!("{}_{}", img_base, q), hex.as_cart_vec().0).color(color).centered());
                }
            }
        }

        if let Some((path, cost)) = logic::movement::path_to(view, selected, attack_from) {
            for hex in path {
                draw_list.add_quad(Quad::new_cart(String::from("ui/feet"), hex.as_cart_vec()).centered());
            }
        }

        //                let possible_locs = logic::combat::possible_attack_locations_with_cost(view, selected, hovered_char, &attack);
        //                let max_cost = possible_locs.iter().max_by_key(|(l,c)| r64(**c)).map(|t| *t.1).unwrap_or(1.0);
        //                let min_cost = possible_locs.iter().min_by_key(|(l,c)| r64(**c)).map(|t| *t.1).unwrap_or(1.0);
        //                for (attack_from, cost) in &possible_locs {
        //                    if (cost - min_cost).abs() < 0.0001 {
        //                        let pcnt = (cost / min_cost) as f32;
        //                        draw_list = draw_list.add_quad(Quad::new(format!("ui/hoverHex"), attack_from.as_cart_vec().0).color(Color::greyscale(pcnt)).centered());
        //                    }
        //                }
    }
}

fn draw_in_range_markers(view: &WorldView, selected: Entity, visibility: &Visibility, draw_list: &mut DrawList, attack: &Attack) {
    for (entity, cdata) in view.entities_with_data::<CharacterData>() {
        let character = view.character(*entity);
        if visibility.visible_hexes.contains(&character.position.hex) {
            if logic::combat::can_attack(view, selected, *entity, &attack, None, None) {
                if logic::faction::is_enemy(view, *entity, selected) {
                    let hex = character.position.hex;
                    for q in 0..6 {
                        draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(Color::new(0.7, 0.1, 0.1, 0.6)).centered());
                    }
                }
            }
        }
    }
}

fn draw_strike_boundaries(view: &WorldView, cdata: &Character, visibility: &Visibility, hexes: &HashMap<AxialCoord, f64>, draw_list: &mut DrawList, attack: &Attack) {
    let ap_per_strike = attack.ap_cost;
    let ap_remaining = cdata.action_points.cur_value();
    let cur_move = cdata.movement.moves.as_f64();
    let move_speed = cdata.movement.move_speed.as_f64();
    let max_possible_strikes = cdata.action_points.max_value() / ap_per_strike as i32;
    let is_thrown = attack.attack_type == AttackType::Thrown;
    let strikes_at_cost = |move_cost: &f64| -> i32 {
        let additional_move_required = *move_cost - cur_move;
        let additional_ap_required = (additional_move_required / move_speed).ceil() as i32;
        let raw = (ap_remaining - additional_ap_required) / ap_per_strike as i32;
        if is_thrown {
            raw.min(1)
        } else {
            raw
        }
    };
    let strike_count_colors = [
        Color::new(0.2, 0.6, 0.2, 0.8),
        Color::new(0.25, 0.4, 0.15, 0.8),
        Color::new(0.3, 0.35, 0.1, 0.8),
        Color::new(0.35, 0.35, 0.1, 0.8)];
    let color_for_strike_count = |sc: i32| if sc == 0 { Color::new(0.4, 0.4, 0.4, 0.8) } else { strike_count_colors[(max_possible_strikes - sc).min(3) as usize] };
    for (hex, cost) in hexes {
        if visibility.visible_hexes.contains(&hex) {
            if let Some(tile) = view.entity_by_key(hex) {
                let strikes_in_this_tile = strikes_at_cost(cost);
                let neighbors = hex.neighbors_vec();
                for q in 0..6 {
                    let mut draw_color = None;
                    if let Some(neighbor_cost) = hexes.get(&neighbors[q]) {
                        let neighbor_strikes = strikes_at_cost(neighbor_cost);
                        if neighbor_strikes < strikes_in_this_tile {
                            draw_color = Some(color_for_strike_count(strikes_in_this_tile));
                        }
                    } else if view.tile_opt(neighbors[q]).map(|h| h.occupied_by.is_none()).unwrap_or(true) || strikes_in_this_tile > 0 {
                        draw_color = Some(color_for_strike_count(strikes_in_this_tile));
                    }

                    if let Some(color) = draw_color {
                        draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(color).centered());
                    }
                }
            }
        }
    }
}


fn targeted_character(view: &WorldView, game_state: &GameState, selected: Entity) -> Option<Entity> {
    if let Some(hovered_char) = view.tile_opt(game_state.hovered_hex_coord).and_then(|e| e.occupied_by) {
        if logic::faction::is_enemy(view, selected, hovered_char) {
            return Some(hovered_char);
        }
    }
    None
}

use gui::GUI;
use noisy_float::types::r64;
use std::collections::HashMap;

pub fn update_move_and_attack_widgets(attack_details_widget: &mut AttackDetailsWidget,
                                      world: &World, view: &WorldView,
                                      gui: &mut GUI, game_state: &GameState,
                                      selected: Entity, attack_ref: AttackRef) {
    use gui::*;

    if let Some(hovered_char) = targeted_character(view, game_state, selected) {
        if let Some(attack) = attack_ref.resolve(view, selected) {
            attack_details_widget.set_position(Positioning::constant((game_state.mouse_pixel_pos.x + 20.0).px()), Positioning::constant((game_state.mouse_pixel_pos.y + 20.0).px()));
            attack_details_widget.set_showing(true);

            let attack_from_and_cost = if logic::combat::can_attack(view, selected, hovered_char, &attack, None, None) {
                Some((view.data::<PositionData>(selected).hex, 0.0))
            } else {
                let hexes = logic::movement::hexes_reachable_by_character_this_turn_default(view, selected);
                logic::combat::closest_attack_location_with_cost(view, hexes, selected, hovered_char, &attack, game_state.mouse_cart_vec())
            };

            if let Some((attack_from, cost)) = attack_from_and_cost {
                let current_ap = view.data::<CharacterData>(selected).action_points.cur_value();
                let ap_remaining_after_move = (current_ap - logic::movement::ap_cost_for_move_cost(view, selected, Sext::of_rounded_up(cost)) as i32).max(0);
                attack_details_widget.update(gui, world, view, selected, hovered_char, &attack_ref, Some(attack_from), ap_remaining_after_move);
            } else {
                attack_details_widget.update(gui, world, view, selected, hovered_char, &attack_ref, None, 0);
            }
        }
        return;
    }
    attack_details_widget.hide(gui);
}