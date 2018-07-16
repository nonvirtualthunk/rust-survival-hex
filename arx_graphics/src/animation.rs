use common::prelude::*;

use core::*;
use game::world::WorldView;
use game::world::Entity;
use game::entities::*;

use game::core::GameEventClock;
use core::GraphicsWrapper;


use common::hex::AxialCoord;
use common::hex::CartVec;
use common::color::*;
use interpolation::*;
use std::fmt::Debug;


pub trait AnimationElement : Debug {
    fn draw(&self, view: &mut WorldView, pcnt_elapsed: f64) -> DrawList;

    /// The full amount of time over which this animation will occur, expressed in seconds
    /// not accounting for any sort of time dilation. The percentage elapsed is always in
    /// terms of this duration, regardless of what the blocking duration is
    fn raw_duration(&self) -> f64;

    /// The amount of time this animation should be allowed to run before other animations
    /// may begin. In seconds. Defaults to the full raw_duration
    fn blocking_duration(&self) -> f64 {
        self.raw_duration()
    }
}

#[derive(Debug)]
pub struct AnimationGroupElement {
    pub delay : f64,
    pub animation : Box<AnimationElement>
}

#[derive(Debug)]
pub struct AnimationGroup {
    pub elements : Vec<AnimationGroupElement>
}

impl AnimationGroup {
    pub fn new() -> AnimationGroup {
        AnimationGroup {
            elements : Vec::new()
        }
    }

    pub fn with_animation<T : AnimationElement + 'static> (mut self, elem : T, delay : Option<f64>) -> Self {
        self.elements.push(
            AnimationGroupElement {
                delay : delay.unwrap_or(0.0),
                animation : box elem
            }
        );
        self
    }
}

impl AnimationElement for AnimationGroup {
    fn draw(&self, view: &mut WorldView, pcnt_elapsed: f64) -> DrawList {
        let total_elapsed = pcnt_elapsed * self.raw_duration();

        let mut quads = Vec::new();
        let mut text = Vec::new();

        for element in &self.elements {
            let effective_elapsed = total_elapsed - element.delay;
            let effective_pcnt_elapsed = effective_elapsed / element.animation.raw_duration();
            if effective_pcnt_elapsed < 1.0 && effective_pcnt_elapsed > 0.0 {
                let mut draw_list = element.animation.draw(view, effective_pcnt_elapsed);
                quads.append(&mut draw_list.quads);
                text.append(&mut draw_list.text);
            }
        }

        DrawList {
            quads,
            text,
            ..Default::default()
        }
    }

    fn raw_duration(&self) -> f64 {
        let mut maximum = 0.0f64;
        for element in &self.elements {
            maximum = maximum.max(element.delay + element.animation.raw_duration());
        }
        maximum
    }

    fn blocking_duration(&self) -> f64 {
        let mut maximum = 0.0f64;
        for element in &self.elements {
            maximum = maximum.max(element.delay + element.animation.blocking_duration());
        }
        maximum
    }
}


#[derive(Debug)]
pub struct TextAnimationElement {
    pub text: String,
    pub text_size: u32,
    pub duration: f64,
    pub blocking_duration: Option<f64>,
    pub position_interpolation : Interpolation<CartVec>,
    pub color_interpolation : Interpolation<Color>
}

impl TextAnimationElement {
    pub fn new (text : String, text_size : u32, position : CartVec, color : Color, duration : f64) -> TextAnimationElement {
        TextAnimationElement {
            text,
            text_size,
            position_interpolation : Interpolation::constant(position),
            color_interpolation : Interpolation::constant(color),
            duration,
            blocking_duration : None
        }
    }

    pub fn with_end_color(mut self, end_color : Color, interpolation_type : InterpolationType) -> Self {
        self.color_interpolation.delta = end_color - self.color_interpolation.start.clone();
        self.position_interpolation.interpolation_type = interpolation_type;
        self
    }
    pub fn with_end_position(mut self, end_pos : CartVec, interpolation_type : InterpolationType) -> Self {
        self.position_interpolation.delta = end_pos - self.position_interpolation.start.clone();
        self.position_interpolation.interpolation_type = interpolation_type;
        self
    }
    pub fn with_delta(mut self, delta : CartVec, interpolation_type : InterpolationType) -> Self {
        self.position_interpolation.delta = delta;
        self.position_interpolation.interpolation_type = interpolation_type;
        self
    }
    pub fn with_blocking_duration(mut self, blocking_duration : f64) -> Self {
        self.blocking_duration = Some(blocking_duration);
        self
    }
}

impl AnimationElement for TextAnimationElement {
    fn draw(&self, _world_view: &mut WorldView, pcnt_elapsed: f64) -> DrawList {
        let pos = self.position_interpolation.interpolate(pcnt_elapsed);
        DrawList::of_text(
            Text::new(self.text.clone(), self.text_size)
                .offset(pos.0)
                .color(self.color_interpolation.interpolate(pcnt_elapsed)))
    }


    fn raw_duration(&self) -> f64 {
        self.duration
    }

    fn blocking_duration(&self) -> f64 {
        self.blocking_duration.unwrap_or(self.duration)
    }
}