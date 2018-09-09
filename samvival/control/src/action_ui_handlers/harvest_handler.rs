use common::prelude::*;


use gui::GUI;
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