use core::*;
use game::world::WorldView;
use game::entities::*;

use game::core::GameEventClock;


use common::hex::AxialCoord;

pub struct TerrainRenderer {
    pub tile_radius : f32
}

impl TerrainRenderer {
    pub fn render_tiles(&mut self, world : &WorldView, g : &mut GraphicsWrapper, _time: GameEventClock) {
        for q in world.min_tile.q .. world.max_tile.q + 1 {
            for r in world.min_tile.r .. world.max_tile.r + 1 {
                let pos = AxialCoord::new(q,r);
                if let Some(t) = world.tiles.get(&pos) {
                    let cartesian_pos = t.position.as_cartesian(self.tile_radius);
                    let quad = Quad::new(format!("terrain/{}",t.name) , cartesian_pos).centered();
                    g.draw_quad(quad);
                }
            }
        }
//        for t in world.tiles.values() {
//            let cartesian_pos = t.position.as_cartesian(self.tile_radius);
//            let quad = Quad::new(format!("terrain/{}",t.name) , cartesian_pos).centered();
//            g.draw_quad(quad);
//        }
    }
}