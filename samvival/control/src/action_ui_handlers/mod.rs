pub mod move_and_attack_handler;
pub use self::move_and_attack_handler::*;

pub mod move_handler;

pub mod player_action_handler;

pub mod harvest_handler;


use common::Color;
use std::collections::HashSet;
use game::entities::TileStore;
use game::prelude::*;
use game::entities::Visibility;
use graphics::prelude::*;
use common::prelude::*;

pub(crate) fn draw_hex_boundary(view: &WorldView, visibility: &Visibility,
                     hexes: &HashSet<AxialCoord>,
                     color : Color) -> DrawList {
    let mut draw_list = DrawList::none();
    for hex in hexes {
        if visibility.visible_hexes.contains(&hex) {
            if let Some(tile) = view.entity_by_key(hex) {
                let neighbors = hex.neighbors_vec();
                for q in 0..6 {
                    let mut draw_color = None;
                    if ! hexes.contains(&neighbors[q]) {
                        draw_color = Some(color);
                    }

                    if let Some(color) = draw_color {
                        draw_list.add_quad(Quad::new(format!("ui/hex/hex_edge_{}", q), hex.as_cart_vec().0).color(color).centered());
                    }
                }
            }
        }
    }
    draw_list
}

pub(crate) fn draw_movement_path(view: &WorldView, mover : Entity, path : Vec<AxialCoord>) -> DrawList {
    let mut draw_list = DrawList::none();
    for hex in path.iter().skip(1) { // skip the starting position
        draw_list.add_quad(Quad::new_cart(String::from("ui/feet"), hex.as_cart_vec()).centered());
    }
    draw_list
}
