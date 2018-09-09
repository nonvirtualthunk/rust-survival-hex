use common::prelude::*;
use game::prelude::*;

use gui::GameState;
use game::entities::TileStore;
use game::logic;
use graphics::prelude::*;


fn create_move_ui_draw_list(world_view: &WorldView, game_state : &GameState) -> DrawList {
    if game_state.animating || game_state.player_faction_active {
        DrawList::none()
    } else {
        let hovered_tile = world_view.tile_opt(game_state.hovered_hex_coord);
        if let Some(hovered_tile) = hovered_tile {
            let hovered_hex = hovered_tile.position;

            let mut draw_list = DrawList::of_quad(Quad::new_cart(String::from("ui/hoverHex"), hovered_hex.as_cart_vec()).centered());

            if let Some(selected) = game_state.selected_character {
                let sel_c = world_view.character(selected);

                if let Some(hovered_occupant) = hovered_tile.occupied_by {
                    if logic::faction::is_enemy(world_view, hovered_occupant, selected) {
                        if let Some(attack_ref) = logic::combat::primary_attack_ref(world_view, selected) {
                            let path = logic::combat::path_to_attack(world_view, selected, hovered_occupant, &attack_ref, game_state.mouse_cart_vec()).map(|t| t.0)
                                .or_else(|| logic::movement::path_adjacent_to(world_view, selected, hovered_occupant).map(|t| t.0));
//                                    .map(|path| logic::movement::portion_of_path_traversable_this_turn(world_view, selected, &path));

                            if let Some(path) = path {
                                for hex in path {
                                    draw_list = draw_list.with_quad(Quad::new_cart(String::from("ui/feet"), hex.as_cart_vec()).centered());
                                }
                            }
                        }
                    }
                } else {
                    if let Some(path_result) = logic::movement::path(world_view, selected, sel_c.position.hex, hovered_hex) {
                        let path = path_result.0;
                        for hex in path {
                            draw_list = draw_list.with_quad(Quad::new_cart(String::from("ui/feet"), hex.as_cart_vec()).centered());
                        }
                    }
                }
            }

            draw_list
        } else {
            DrawList::none()
        }
    }
}