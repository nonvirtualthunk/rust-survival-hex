use common::prelude::*;

use core::*;
use game::world::WorldView;
use game::Entity;
use game::entities::*;

use game::core::GameEventClock;


use common::hex::AxialCoord;
use common::hex::CartVec;
use common::color::Color;
use common::Rect;


pub struct UnitRenderer {

}

const HEALTH_BAR_WIDTH : f32 = 0.2;
//const STAMINA_WHEEL_WIDTH : f32 = 0.3;

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

            let c = world.character(*id);

            // Top half of selection indicator
            if *id == selected_character {
                let color = world.faction(c.faction).color;
                let pos = c.effective_graphical_pos();
                quads.push(Quad::new(String::from("ui/selectedTop"), pos.0).centered().color(color));
            }

            // Main unit display
            let pos = c.effective_graphical_pos();
            let quad = Quad::new(format!("entities/{}", c.sprite), pos.0).centered().color(c.graphics.color);
            quads.push(quad);

            // Health bar
            let health_fract = c.health.cur_fract().max(0.0);
            let bar_pos = pos - CartVec::new(1.0, 0.4);
            let quad = Quad::new(strf("ui/blank"), bar_pos.0).size(v2(HEALTH_BAR_WIDTH, 1.0));
            quads.push(quad);
            let quad = Quad::new(strf("ui/blank"), bar_pos.0)
                .size(v2(HEALTH_BAR_WIDTH, 1.0 * health_fract as f32))
                .color(Color::new(0.9, 0.1, 0.1, 1.0));
            quads.push(quad);

            // Stamina indicator
            let stamina_fract = c.stamina.cur_fract().max(0.0);
            let stamina_bar_pos = pos - CartVec::new(1.0 - HEALTH_BAR_WIDTH - 0.02,0.4);
            quads.push(Quad::new(strf("ui/blank"), stamina_bar_pos.0 - v2(0.02,0.02)).size(v2(HEALTH_BAR_WIDTH + 0.04, 1.04)).color(Color::black()));
            quads.push(Quad::new(strf("ui/blank"), stamina_bar_pos.0).size(v2(HEALTH_BAR_WIDTH, 1.0)));
            quads.push(Quad::new(strf("ui/blank"), stamina_bar_pos.0)
                           .size(v2(HEALTH_BAR_WIDTH, 1.0 * stamina_fract as f32))
                           .color(Color::new(0.1, 0.6, 0.1, 1.0)));
//            let (whole_stam, part_stam) = c.stamina.cur_value().as_whole_and_parts();
//            let mut stam_wheels = Vec::new();
//            for _i in 0 .. whole_stam { stam_wheels.push(8); }
//            stam_wheels.push(part_stam);
//
//            let mut wheel_pos = pos + CartVec::new(0.9,0.75);
//            for portion in stam_wheels {
//                quads.push(Quad::new(format!("ui/Sext/Sext_{}", portion), wheel_pos.0)
//                    .size(v2(STAMINA_WHEEL_WIDTH, STAMINA_WHEEL_WIDTH))
//                    .color(Color::new(0.1, 0.7, 0.2, 1.0))
//                );
//                wheel_pos = wheel_pos - CartVec::new(0.0,0.2)
//            }

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