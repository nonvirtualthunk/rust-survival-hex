use common::prelude::*;

use core::*;
use game::prelude::*;
use game::entities::*;

use game::core::*;


use common::hex::AxialCoord;
use common::hex::CartVec;
use common::color::Color;
use common::Rect;


pub struct ItemRenderer {
    items_to_draw: Vec<Entity>,
    cached_game_time : Option<GameEventClock>
}

impl ItemRenderer {
    pub fn new() -> ItemRenderer {
        ItemRenderer {
            items_to_draw: Vec::new(),
            cached_game_time : None
        }
    }

    fn identify_relevant_entities(&mut self, world : &WorldView) {
        self.items_to_draw.clear();
        for (id, item_data) in world.entities_with_data::<ItemData>() {
            // it must not be held by anyone, and it must have a position in the world
            if item_data.held_by.is_none() && world.has_data::<PositionData>(*id) {
                self.items_to_draw.push(*id);
            }
        }
    }

    pub fn render_items(&mut self, world: &WorldView, _bounds : Rect<f32>) -> DrawList {
        if self.cached_game_time != Some(world.current_time) {
            self.identify_relevant_entities(world);
        }

        let mut quads : Vec<Quad> = vec![];

        for ent in &self.items_to_draw {
            let ent = *ent;
//            let item_data = world.item(ent);
            let position_data = world.data::<PositionData>(ent);
            let ident_data = world.data::<IdentityData>(ent);
            let graphics_data = world.data::<GraphicsData>(ent);

            // Main item display
            let pos = position_data.hex.as_cart_vec();
            let quad = Quad::new(format!("entities/items/{}", ident_data.kind.name), pos.0).centered().color(graphics_data.color);
            quads.push(quad);
        }
        DrawList {
            quads,
            ..Default::default()
        }
    }
}