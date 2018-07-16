use game::events::GameEvent;
use arx_graphics::animation::AnimationElement;
use arx_graphics::animation::TextAnimationElement;

use game::world::WorldView;

use common::hex::CartVec;
use common::hex::AxialCoord;

use common::prelude::v2;

use game::entities::CharacterStore;
use game::entities::CharacterData;
use game::world::Entity;
use game::world::EntityData;
use common::color::Color;
use arx_graphics::interpolation::InterpolationType;
use arx_graphics::interpolation::Interpolateable;
use arx_graphics::interpolation::Interpolation;
use arx_graphics::animation::*;
use std::marker::PhantomData;
use arx_graphics::core::DrawList;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;

pub fn animation_elements_for_new_event(world_view : &WorldView, event : GameEvent) -> Vec<Box<AnimationElement>> {
    match event {
        GameEvent::Attack { attacker, defender, damage_done, hit, .. } => {
            animate_attack(world_view, attacker, defender, damage_done, hit)
        },
        GameEvent::Move { character, from, to, .. } => {
            animate_move(world_view, character, from, to)
        }
        _ => vec![]
    }
}


fn animate_move(world_view : &WorldView, character : Entity, from : AxialCoord, to : AxialCoord) -> Vec<Box<AnimationElement>> {
    let from_pos = from.as_cart_vec();
    let to_pos = to.as_cart_vec();
//    let delta = (to_pos - from_pos).normalize() * (1.0 / 60.0);
//    let from_pos = from_pos + delta; // move it forward one frame
    vec![box EntityFieldAnimation::new(
        character,
        Interpolation::linear_from_endpoints(from_pos, to_pos),
        |data : &mut CharacterData, new_value| data.graphical_position = Some(new_value),
        0.3
    )]
}

fn animate_attack(world_view: &WorldView, attacker : Entity, defender: Entity, damage_done: u32, hit: bool) -> Vec<Box<AnimationElement>> {
    let attacker_data = world_view.character(attacker);
    let defender_data = world_view.character(defender);

    let defender_pos = defender_data.position.as_cart_vec();
    let attacker_pos = attacker_data.position.as_cart_vec();

    let delta : CartVec = (defender_pos - attacker_pos).normalize() * 0.5;

    let swing_at_enemy = EntityFieldAnimation::new(
        attacker,
        Interpolation::linear_from_delta(attacker_pos, delta).circular(),
        |data:&mut CharacterData,new_value| { data.graphical_position = Some(new_value) },
        0.5
    );
    let damage_anim_start_point = swing_at_enemy.raw_duration() / 2.0;

    let mut animation_group = AnimationGroup::new()
        .with_animation(swing_at_enemy, None);

    if damage_done > 0 {
        let starting_health = world_view.character(defender).health.cur_value();
        let damage_bar_animation = EntityFieldAnimation::new(
            defender,
            Interpolation::linear_from_delta(starting_health as f32, -(damage_done as f32)),
            |data : &mut CharacterData, new_value : f32| { data.health.reduce_to(new_value.floor() as i32); },
            0.5);

        let start_color = defender_data.graphical_color;
        let end_color = Color::new(1f32, 0.1f32, 0.1f32, 1f32);
        let red_tint_animation = EntityFieldAnimation::new(
            defender,
            Interpolation::linear_from_endpoints(start_color, end_color).circular(),
            |data:&mut CharacterData,new_value| { data.graphical_color = new_value; },
            0.5
        );

        let text_color = Color::new(0.9, 0.2, 0.2, 1.0);
        let rising_damage_text = TextAnimationElement::new(damage_done.to_string(), 20, defender_pos, text_color, 3.0)
            .with_delta(CartVec::new(0.0, 1.0), InterpolationType::Linear)
            .with_blocking_duration(0.0);

        animation_group = animation_group
            .with_animation(damage_bar_animation, Some(damage_anim_start_point))
            .with_animation(red_tint_animation, Some(damage_anim_start_point));
    }

    let (msg, color) = if hit {
        (damage_done.to_string(), Color::new(0.9, 0.2, 0.2, 1.0))
    } else {
        (String::from("miss"), Color::new(0.1, 0.0, 0.0, 1.0))
    };
    let rising_damage_text = TextAnimationElement::new(msg, 20, defender_pos + CartVec::new(0.0,0.5), color, 3.0)
        .with_delta(CartVec::new(0.0, 1.0), InterpolationType::Linear)
        .with_blocking_duration(0.0);

    let animation_group = animation_group.with_animation(rising_damage_text, Some(damage_anim_start_point));

    vec![box animation_group]
}

struct EntityFieldAnimation<T : EntityData, F : Interpolateable<F>, S : Fn(&mut T, F)> {
    pub entity : Entity,
    pub interpolation : Interpolation<F>,
    pub store_function : S,
    pub duration : f64,
    pub blocking_duration : Option<f64>,
    pub phantom_ : PhantomData<T>
}

impl <T : EntityData, F : Interpolateable<F>, S : Fn(&mut T, F)> EntityFieldAnimation<T,F,S> {
    pub fn new(entity : Entity, interpolation : Interpolation<F>, store_function : S, duration : f64) -> EntityFieldAnimation<T,F,S> {
        EntityFieldAnimation {
            entity, interpolation, store_function, duration, blocking_duration : None, phantom_ : PhantomData::default()
        }
    }

    pub fn with_blocking_duration(mut self, blocking_duration : f64) -> Self {
        self.blocking_duration = Some(blocking_duration);
        self
    }
}

impl <T : EntityData, F : Interpolateable<F>, S : Fn(&mut T, F)> AnimationElement for EntityFieldAnimation<T,F,S> {
    fn draw(&self, view: &mut WorldView, pcnt_elapsed: f64) -> DrawList {
        if pcnt_elapsed > 1.0 || pcnt_elapsed < 0.0 {
            println!("Unexpected, pcnt was more than 1: {}", pcnt_elapsed);
        }
        let mut data = view.data_mut::<T>(self.entity);
        let new_value = self.interpolation.interpolate(pcnt_elapsed);
        (self.store_function)(&mut data, new_value);
        DrawList::none()
    }

    fn raw_duration(&self) -> f64 {
        self.duration
    }

    fn blocking_duration(&self) -> f64 {
        self.blocking_duration.unwrap_or(self.duration)
    }
}

impl <T : EntityData, F : Interpolateable<F>, S : Fn(&mut T, F)> Debug for EntityFieldAnimation<T,F,S> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "EntityFieldAnimation[{:?}]", self.entity)
    }
}