use common::prelude::*;

use core::*;
use game::world::WorldView;
use game::world::Entity;
use game::entities::*;

use game::core::GameEventClock;
use core::GraphicsWrapper;
use piston_window::types::Color;


use common::hex::AxialCoord;


pub trait AnimationElement {
    fn draw (&self, view : &WorldView, pcnt_elapsed : f32, g: &mut GraphicsWrapper);

    fn raw_duration(&self) -> f64;
}


pub struct TextAnimationElement <'a> {
    pub text : &'a str,
    pub text_size : u32,
    pub start_position : Vec2f,
    pub movement : Vec2f,
    pub duration : f64,
    pub color : Color
}

impl <'a> AnimationElement for TextAnimationElement<'a> {
    fn draw(&self, _world_view: &WorldView, pcnt_elapsed: f32, g: &mut GraphicsWrapper) {
        let pos = self.start_position;
        g.draw_text(Text::new(self.text, self.text_size)
            .offset(v2(pos.x + self.movement.x * pcnt_elapsed, pos.y + self.movement.y * pcnt_elapsed))
            .color(self.color));
    }

    fn raw_duration(&self) -> f64 {
        self.duration
    }
}