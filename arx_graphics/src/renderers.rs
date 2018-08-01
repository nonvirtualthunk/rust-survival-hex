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

pub struct UnitRenderer {

}

const HEALTH_BAR_WIDTH : f32 = 0.2;
const STAMINA_WHEEL_WIDTH : f32 = 0.3;

impl UnitRenderer {
    pub fn render_units(&mut self, world: &WorldView,
                        _time: GameEventClock,
                        selected_character: Option<Entity>) -> DrawList {
        let mut quads : Vec<Quad> = vec![];

        let selected_character = selected_character.unwrap_or(Entity::sentinel());
        for (id, c) in world.entities_with_data::<CharacterData>() {
            if !c.is_alive() {
                continue;
            }

            // Top half of selection indicator
            if *id == selected_character {
                let color = world.faction(c.faction).color;
                let pos = c.effective_graphical_pos();
                quads.push(Quad::new(String::from("ui/selectedTop"), pos.0).centered().color(color));
            }

            // Main unit display
            let cartesian_pos = c.position.as_cart_vec();
            let pos = c.graphical_position.unwrap_or_else(|| cartesian_pos);
            let quad = Quad::new(format!("entities/{}", c.sprite), pos.0).centered().color(c.graphical_color);
            quads.push(quad);

            // Health bar
            let health_fract = c.health.cur_fract().max(0.0);
            let bar_pos = pos - CartVec::new(1.0, 0.5);
            let quad = Quad::new(strf("ui/blank"), bar_pos.0).size(v2(HEALTH_BAR_WIDTH, 1.0));
            quads.push(quad);
            let quad = Quad::new(strf("ui/blank"), bar_pos.0)
                .size(v2(HEALTH_BAR_WIDTH, 1.0 * health_fract as f32))
                .color(Color::new(0.9, 0.1, 0.1, 1.0));
            quads.push(quad);

            // Stamina indicator
            let (whole_stam, part_stam) = c.stamina.cur_value().as_whole_and_parts();
            let mut stam_wheels = Vec::new();
            for _i in 0 .. whole_stam { stam_wheels.push(8); }
            stam_wheels.push(part_stam);

            let mut wheel_pos = pos + CartVec::new(0.9,0.75);
            for portion in stam_wheels {
                quads.push(Quad::new(format!("ui/oct/oct_{}", portion), wheel_pos.0)
                    .size(v2(STAMINA_WHEEL_WIDTH, STAMINA_WHEEL_WIDTH))
                    .color(Color::new(0.1, 0.7, 0.2, 1.0))
                );
                wheel_pos = wheel_pos - CartVec::new(0.0,0.2)
            }

            // Action circle
            let color = if c.action_points.cur_fract() > 0.5 {
                Color::new(0.3, 1.0, 0.3, 1.0)
            } else if c.action_points.cur_fract() > 0.0 {
                Color::new(0.8, 0.6, 0.0, 1.0)
            } else {
                Color::new(0.2, 0.2, 0.2, 1.0)
            };
            let circle_pos = pos + CartVec::new(-0.9, 0.65);
            let quad = Quad::new(strf("ui/circle_small"), circle_pos.0).size(v2(0.3,0.3)).centered().color(color);
            quads.push(quad);


            // Bottom half of selection indicator
            if *id == selected_character {
                let color = world.faction(c.faction).color;
                let pos = c.effective_graphical_pos();
                quads.push(Quad::new(String::from("ui/selectedBottom"), pos.0).centered().color(color));
            }
        }
        DrawList {
            quads,
            ..Default::default()
        }
    }
}