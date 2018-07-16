use common::prelude::*;

use core::*;
use game::world::WorldView;
use game::world::Entity;
use game::entities::*;

use game::core::GameEventClock;


use common::hex::AxialCoord;
use common::hex::CartVec;
use common::color::Color;

pub struct TerrainRenderer {

}

impl TerrainRenderer {
    pub fn render_tiles(&mut self, world: &WorldView, _time: GameEventClock) -> DrawList {
        let map_data = world.world_data::<MapData>();

        let mut quads : Vec<Quad> = vec![];
        for q in map_data.min_tile_bound.q..map_data.max_tile_bound.q + 1 {
            for r in map_data.min_tile_bound.r..map_data.max_tile_bound.r + 1 {
                let pos = AxialCoord::new(q, r);
                if let Some(t) = world.tile_opt(pos) {
                    let cartesian_pos = t.position.as_cart_vec();
                    let quad = Quad::new(format!("terrain/{}", t.name), cartesian_pos.0).centered();
                    quads.push(quad);
                }
            }
        }

        DrawList {
            quads,
            ..Default::default()
        }
    }
}

pub struct UnitRenderer {

}

const HEALTH_BAR_WIDTH : f32 = 0.2;

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
            let bar_pos = pos - CartVec::new(1.0, 0.0);
            let quad = Quad::new(strf("ui/blank"), bar_pos.0).size(v2(HEALTH_BAR_WIDTH, 1.0));
            quads.push(quad);
            let quad = Quad::new(strf("ui/blank"), bar_pos.0)
                .size(v2(HEALTH_BAR_WIDTH, 1.0 * health_fract as f32))
                .color(Color::new(0.9, 0.1, 0.1, 1.0));
            quads.push(quad);

            // Action circle
            let color = if c.action_points.cur_fract() > 0.5 {
                Color::new(0.3, 1.0, 0.3, 1.0)
            } else if c.action_points.cur_fract() > 0.0 {
                Color::new(0.8, 0.6, 0.0, 1.0)
            } else {
                Color::new(0.2, 0.2, 0.2, 1.0)
            };
            let circle_pos = pos + CartVec::new(1.0, 0.8);
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