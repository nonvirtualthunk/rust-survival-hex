use common::prelude::*;

use core::*;
use game::world::WorldView;
use game::world::Entity;
use game::entities::*;

use game::core::GameEventClock;


use common::hex::AxialCoord;
use common::hex::CartVec;
use common::color::Color;
use common::Rect;


#[derive(Default)]
pub struct TerrainRenderer {
//    last_rendered : GameEventClock,
//    last_draw_list : DrawList
}

impl TerrainRenderer {
    pub fn render_tiles(&mut self, world: &WorldView, _time: GameEventClock, bounds : Rect<f32>) -> DrawList {
        // expand the bounds by a buffer of 2 hex radii in each direction
        let bounds = Rect::new(bounds.x - 2.0, bounds.y - 2.0, bounds.w + 4.0, bounds.h + 4.0);
//        if time != self.last_rendered {
//            self.last_draw_list = {
        let map_data = world.world_data::<MapData>();

        let mut quads : Vec<Quad> = vec![];
        for q in map_data.min_tile_bound.q..map_data.max_tile_bound.q + 1 {
            for r in map_data.min_tile_bound.r..map_data.max_tile_bound.r + 1 {
                let pos = AxialCoord::new(q, r);
                if let Some(t) = world.tile_opt(pos) {
                    let cartesian_pos = t.position.as_cart_vec();
                    if bounds.contains(cartesian_pos.0) {
                        let quad = Quad::new(format!("terrain/{}", t.name), cartesian_pos.0).centered();
                        quads.push(quad);
                    }
                }
            }
        }

        DrawList {
            quads,
            ..Default::default()
        }
//            };
//            self.last_rendered = time;
//        }
//
//        &self.last_draw_list
    }
}