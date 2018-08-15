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
            // draw it if either it is in the inventory of a hex (meaning it's on the hex) or it has no holder and it does have position data
            // so it is acting as an entity of its own
            if let Some(in_inventory) = item_data.in_inventory_of {
                if world.has_data::<TileData>(in_inventory) {
                    self.items_to_draw.push(*id);
                }
            } else if world.has_data::<PositionData>(*id) {
                self.items_to_draw.push(*id);
            }
        }
    }

    pub fn draw_at(world: &WorldView, item : Entity) -> Option<AxialCoord> {
        if let Some(hex) = world.data_opt::<ItemData>(item).and_then(|item_data| item_data.in_inventory_of).and_then(|inv| world.data_opt::<TileData>(inv)).map(|td| td.position) {
            Some(hex)
        } else if let Some(pos_data) = world.data_opt::<PositionData>(item) {
            Some(pos_data.hex)
        } else {
            None
        }
    }

    pub fn render_items(&mut self, world: &WorldView, _bounds : Rect<f32>) -> DrawList {
        if self.cached_game_time != Some(world.current_time) {
            self.identify_relevant_entities(world);
        }

        let mut quads : Vec<Quad> = vec![];

        for ent in &self.items_to_draw {
            let ent = *ent;

            let ident_data = world.data::<IdentityData>(ent);
            let graphics_data = world.data::<GraphicsData>(ent);

            // Main item display
            if let Some(pos) = ItemRenderer::draw_at(world, ent) {
                let pos = pos.as_cart_vec();
                let quad = Quad::new(format!("entities/items/{}", ident_data.kind.name), pos.0).centered().color(graphics_data.color);
                quads.push(quad);
            } else {
                warn!("Tried to draw item {} but could not identify a position to do so", ent);
            }
        }
        DrawList {
            quads,
            ..Default::default()
        }
    }
}