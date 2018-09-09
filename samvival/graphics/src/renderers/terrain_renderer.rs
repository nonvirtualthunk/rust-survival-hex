use common::prelude::*;

use core::*;
use game::world::WorldView;
use game::Entity;
use game::entities::*;

use game::core::GameEventClock;

use heck::SnakeCase;

use common::hex::CubeCoord;
use common::hex::CartVec;
use common::color::Color;
use common::Rect;


#[derive(Default)]
pub struct TerrainRenderer {
//    last_rendered : GameEventClock,
//    last_draw_list : DrawList
}

impl TerrainRenderer {
    pub fn render_tiles(&mut self, world: &WorldView, player_faction: Entity, _time: GameEventClock, bounds: Rect<f32>) -> DrawList {
        if let Some(visibility) = world.world_data::<VisibilityData>().visibility_by_faction.get(&player_faction) {
            let visible_hexes = &visibility.visible_hexes;
            let revealed_hexes = &visibility.revealed_hexes;

            // expand the bounds by a buffer of 2 hex radii in each direction
            let bounds = Rect::new(bounds.x - 2.0, bounds.y - 2.0, bounds.w + 4.0, bounds.h + 4.0);
    //        if time != self.last_rendered {
    //            self.last_draw_list = {
//            let map_data = world.world_data::<MapData>();

            let mut quads: Vec<Quad> = vec![];
            let mut over_quads: Vec<Quad> = vec![];

            let bounds_center = bounds.center();
            let center = AxialCoord::from_cart_coord(CartVec::new(bounds_center.x, bounds_center.y)).as_cube_coord();
            let corner = AxialCoord::from_cart_coord(CartVec::new(bounds.min_x(), bounds.min_y())).as_cube_coord();
            let dist = corner.distance(&center) as i32;

            let accessor = TileAccessor::new(world);

//            for q in map_data.min_tile_bound.q..map_data.max_tile_bound.q + 1 {
//                for r in map_data.min_tile_bound.r..map_data.max_tile_bound.r + 1 {
            for x in -dist ..= dist {
                for y in (-dist).max(-x - dist) ..= dist.min(-x + dist) {
                    let pos = CubeCoord::new(center.x + x,center.y + y,center.z - x - y).as_axial_coord();
                    let cartesian_pos = pos.as_cart_vec();
                    if bounds.contains(cartesian_pos.0) {
                        if revealed_hexes.contains(&pos) {
                            let visible = visible_hexes.contains(&pos);

                            let color = if visible {
                                Color::white()
                            } else {
                                Color::greyscale(0.5)
                            };
                            if let Some(t) = accessor.tile_opt(pos) {
                                let terrain = accessor.terrain(&t);
                                let vegetation = accessor.vegetation_opt(&t);
                                let quad = Quad::new(format!("terrain/{}", terrain.kind.name().to_snake_case()), cartesian_pos.0)
                                    .centered()
                                    .color(color);
                                quads.push(quad);

                                if let Some(vegetation) = vegetation {
                                    let quad = Quad::new(format!("terrain/{}", vegetation.kind.name().to_snake_case()), cartesian_pos.0)
                                        .centered()
                                        .color(color);
                                    over_quads.push(quad);
                                }
                            }
                        } else {
                            quads.push(Quad::new(strf("terrain/fog1"), cartesian_pos.0).centered());
                        }
                    }
                }
            }

            quads.extend(over_quads);
            DrawList {
                quads,
                ..Default::default()
            }
        } else {
            error!("No visibility data for player faction");
            DrawList::none()
        }
//            };
//            self.last_rendered = time;
//        }
//
//        &self.last_draw_list
    }
}