use common::prelude::*;

use core::*;
use game::world::WorldView;
use game::world::Entity;
use game::entities::*;

use game::core::GameEventClock;


use common::hex::AxialCoord;

pub struct TerrainRenderer {
    pub tile_radius: f32
}

impl TerrainRenderer {
    pub fn render_tiles(&mut self, world: &WorldView, g: &mut GraphicsWrapper, _time: GameEventClock) {
        let map_data = world.world_data::<MapData>();

        for q in map_data.min_tile_bound.q..map_data.max_tile_bound.q + 1 {
            for r in map_data.min_tile_bound.r..map_data.max_tile_bound.r + 1 {
                let pos = AxialCoord::new(q, r);
                if let Some(t) = world.tile_opt(pos) {
                    let cartesian_pos = t.position.as_cartesian(self.tile_radius);
                    let quad = Quad::new(format!("terrain/{}", t.name), cartesian_pos).centered();
                    g.draw_quad(quad);
                }
            }
        }
    }
}

pub struct UnitRenderer {
    pub tile_radius: f32
}

const HEALTH_BAR_WIDTH : f32 = 6.0;

impl UnitRenderer {
    pub fn render_units(&mut self, world: &WorldView,
                        g: &mut GraphicsWrapper,
                        _time: GameEventClock,
                        selected_character: Option<Entity>) {
        let selected_character = selected_character.unwrap_or(Entity::sentinel());
        for (id, c) in world.entities_with_data::<CharacterData>() {
            if !c.is_alive() {
                continue;
            }

            // Top half of selection indicator
            if *id == selected_character {
                let color = world.faction(c.faction).color;
                let pos = c.effective_graphical_pos(self.tile_radius);
                g.draw_quad(Quad::new(String::from("ui/selectedTop"), pos).centered().color(color));
            }

            // Main unit display
            let cartesian_pos = c.position.as_cartesian(self.tile_radius);
            let pos = c.graphical_position.unwrap_or_else(|| cartesian_pos);
            let quad = Quad::new(format!("entities/{}", c.sprite), pos).centered().color(c.graphical_color);
            g.draw_quad(quad);

            // Health bar
            let health_fract = c.health.cur_fract().max(0.0);
            let bar_pos = pos - v2(self.tile_radius, 0.0);
            let quad = Quad::new(strf("ui/blank"), bar_pos).size(v2(HEALTH_BAR_WIDTH, self.tile_radius));
            g.draw_quad(quad);
            let quad = Quad::new(strf("ui/blank"), bar_pos)
                .size(v2(HEALTH_BAR_WIDTH, self.tile_radius * health_fract as f32))
                .color([0.9, 0.1, 0.1, 1.0]);
            g.draw_quad(quad);

            // Action circle
            let color = if c.actions.cur_value() > 0 && c.moves.cur_value() > 0.0 {
                [0.3, 1.0, 0.3, 1.0]
            } else if c.actions.cur_value() > 0 || c.moves.cur_value() > 0.0 {
                [0.8, 0.6, 0.0, 1.0]
            } else {
                [0.2, 0.2, 0.2, 1.0]
            };
            let circle_pos = pos + v2(self.tile_radius, self.tile_radius * 0.8);
            let quad = Quad::new(strf("ui/circle_small"), circle_pos).size(v2(10.0, 10.0)).centered().color(color);
            g.draw_quad(quad);


            // Bottom half of selection indicator
            if *id == selected_character {
                let color = world.faction(c.faction).color;
                let pos = c.effective_graphical_pos(self.tile_radius);
                g.draw_quad(Quad::new(String::from("ui/selectedBottom"), pos).centered().color(color));
            }
        }
    }
}