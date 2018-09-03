use common::color::Color;
use common::hex::AxialCoord;
use common::hex::CartVec;
use common::prelude::v2;
use game::entities::Attack;
use game::entities::AttackType;
use game::entities::CharacterData;
use game::entities::CharacterStore;
use game::entities::GraphicsData;
use game::entities::StrikeResult;
use game::entities::PositionData;
use game::Entity;
use game::entity::EntityData;
use game::GameEvent;
use game::prelude::*;
use game::world::WorldView;
use graphics::animation::*;
use graphics::animation::AnimationElement;
use graphics::animation::TextAnimationElement;
use graphics::core::DrawList;
use graphics::GraphicsResources;
use graphics::interpolation::Interpolateable;
use graphics::interpolation::Interpolation;
use graphics::interpolation::InterpolationType;
use graphics::renderers::ItemRenderer;
use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;
use std::marker::PhantomData;
use game::entities::IdentityData;
use cgmath::InnerSpace;
use std::f64::consts;
use std::collections::HashMap;
use serde::de::DeserializeOwned;
use graphics::FontSize;

pub fn animation_elements_for_new_event(world_view: &WorldView, wrapper: &GameEventWrapper<GameEvent>, resources: &mut GraphicsResources) -> Vec<Box<AnimationElement>> {
    if wrapper.is_starting() {
        match wrapper.event {
            GameEvent::Strike { attacker, ref defenders, ref attack, ref strike_results, .. } => {
                animate_attack(world_view, attacker, defenders, &attack, strike_results, resources)
            }
            GameEvent::Move { character, from, to, cost, .. } => {
                animate_move(world_view, character, from, to, cost)
            }
            GameEvent::DamageTaken { entity, damage_taken, .. } => {
                animate_damage(world_view, entity, damage_taken)
            }
            _ => vec![]
        }
    } else {
        vec![]
    }
}


fn animate_move(world_view: &WorldView, character: Entity, from: AxialCoord, to: AxialCoord, move_cost: Sext) -> Vec<Box<AnimationElement>> {
    let duration = 0.35;
    let from_pos = from.as_cart_vec();
    let to_pos = to.as_cart_vec();
//    let delta = (to_pos - from_pos).normalize() * (1.0 / 60.0);
//    let from_pos = from_pos + delta; // move it forward one frame
    vec![box EntityFieldAnimation::new(
        character,
        Interpolation::linear_from_endpoints(from_pos, to_pos),
        |data: &mut GraphicsData, new_value| data.graphical_position = Some(new_value),
        duration,
    )]
}

fn animate_attack(world_view: &WorldView, attacker: Entity, defenders: &Vec<Entity>, attack: &Attack, strike_results: &HashMap<Entity, StrikeResult>, resources: &mut GraphicsResources) -> Vec<Box<AnimationElement>> {
    if defenders.is_empty() { warn!("Animating attack with no defenders, how did this occur?");return Vec::new(); }
    let defender = defenders[0];

    let attacker_data = world_view.character(attacker);
    let main_defender_data = world_view.character(defender);

    let main_defender_pos = main_defender_data.position.hex.as_cart_vec();
    let attacker_pos = attacker_data.position.hex.as_cart_vec();

    let main_raw_delta: CartVec = main_defender_pos - attacker_pos;
    let main_delta: CartVec = main_raw_delta.normalize_s() * 0.5;

    let base_duration = 0.5;
    let blocking_duration = if strike_results.values().any(|r| r.hit) { base_duration * 0.5 } else { base_duration };


    let mut animation_group = AnimationGroup::new();

    let miss_anim_start_point =
        if attack.attack_type == AttackType::Thrown {
            let throw_body_move_blocking_duration = base_duration * 0.5;
            let throw_at_enemy_body_move = EntityFieldAnimation::new(
                attacker,
                Interpolation::linear_from_delta(attacker_pos, main_delta * 0.5).circular(),
                |data: &mut GraphicsData, new_value| { data.graphical_position = Some(new_value) },
                base_duration,
            ).with_blocking_duration(throw_body_move_blocking_duration);

            animation_group = animation_group.with_animation(throw_at_enemy_body_move, None);

            let mut end_point = 0.0f64;
            for (target, strike_result) in strike_results {
                let this_strike_end_point = if let Some(weapon) = strike_result.weapon {
                    let defender_data = world_view.character(*target);
                    let defender_pos = defender_data.position.hex.as_cart_vec();
                    let raw_delta = defender_pos - attacker_pos;
                    let delta = raw_delta.normalize_s() * 0.5;
                    let dist: f64 = raw_delta.magnitude_s() as f64;

                    let ident = world_view.data::<IdentityData>(weapon);
                    let item_image = ItemRenderer::image_for(resources, &ident.main_kind());

                    let baseline_rotation = consts::PI / 4.0;
                    let rotation = f64::atan2(delta.y as f64, delta.x as f64);
                    let eff_rotation = rotation - baseline_rotation;

                    let throw_weapon = ImageAnimationElement::new(item_image.clone(), attacker_pos + delta, Color::white(), dist * 0.1f64)
                        .with_end_position(defender_pos - delta * 2.0, InterpolationType::Linear)
                        .with_rotation(eff_rotation as f32);

                    let miss_start = throw_body_move_blocking_duration + throw_weapon.blocking_duration();
                    if strike_result.hit {
                        let weapon_stick = ImageAnimationElement::new(item_image.clone(), defender_pos - delta * 2.0, Color::white(), 0.5)
                            .with_rotation(eff_rotation as f32)
                            .with_blocking_duration(0.0);
                        animation_group.add_animation(weapon_stick, Some(miss_start));
                    }

                    animation_group.add_animation(throw_weapon, Some(throw_body_move_blocking_duration));

                    miss_start
                } else {
                    warn!("Time to handle thrown natural weapons");
                    throw_body_move_blocking_duration
                };
                end_point = end_point.max(this_strike_end_point);
            }
            end_point
        } else if attack.attack_type == AttackType::Projectile {
            let base_duration = base_duration * 0.5;
            let fire_weapon_body_move_blocking_duration = base_duration * 0.5;
            let fire_weapon_body_move = EntityFieldAnimation::new(
                attacker,
                Interpolation::linear_from_delta(attacker_pos, main_delta * 0.2).circular(),
                |data: &mut GraphicsData, new_value| { data.graphical_position = Some(new_value) },
                base_duration,
            ).with_blocking_duration(fire_weapon_body_move_blocking_duration);

            animation_group = animation_group.with_animation(fire_weapon_body_move, None);

            let mut end_point = 0.0f64;
            for (target, strike_result) in strike_results {
                let this_end_point = if let Some(projectile_kind) = &attack.ammunition_kind {
                    let defender_data = world_view.character(*target);
                    let defender_pos = defender_data.position.hex.as_cart_vec();
                    let raw_delta = defender_pos - attacker_pos;
                    let delta = raw_delta.normalize_s() * 0.5;
                    let dist: f64 = raw_delta.magnitude_s() as f64;

                    let item_image = ItemRenderer::image_for(resources, projectile_kind);

                    let baseline_rotation = consts::PI / 4.0;
                    let rotation = f64::atan2(delta.y as f64, delta.x as f64);
                    let eff_rotation = rotation - baseline_rotation;

                    let projectile_movement = ImageAnimationElement::new(item_image.clone(), attacker_pos + delta, Color::white(), dist * 0.02f64)
                        .with_end_position(defender_pos - delta * 2.0, InterpolationType::Exponential { power: 0.7 })
                        .with_rotation(eff_rotation as f32);

                    let miss_start = fire_weapon_body_move_blocking_duration + projectile_movement.blocking_duration();
                    if strike_result.hit {
                        let weapon_stick = ImageAnimationElement::new(item_image.clone(), defender_pos - delta, Color::white(), 0.5)
                            .with_rotation(eff_rotation as f32)
                            .with_blocking_duration(0.0)
                            .with_end_color(Color::clear(), InterpolationType::Exponential { power: 1.5 });
                        animation_group.add_animation(weapon_stick, Some(miss_start));
                    }

                    animation_group.add_animation(projectile_movement, Some(fire_weapon_body_move_blocking_duration));

                    miss_start
                } else {
                    warn!("Projectile attack with no ammunition kind");
                    fire_weapon_body_move_blocking_duration
                };
                end_point = end_point.max(this_end_point);
            }
            end_point
        } else {
            let swing_at_enemy = EntityFieldAnimation::new(
                attacker,
                Interpolation::linear_from_delta(attacker_pos, main_delta).circular(),
                |data: &mut GraphicsData, new_value| { data.graphical_position = Some(new_value) },
                base_duration,
            ).with_blocking_duration(blocking_duration);

            let miss_start_point = swing_at_enemy.blocking_duration() * 0.5;
            animation_group.add_animation(swing_at_enemy, None);

            miss_start_point
        };

    for (target, strike_result) in strike_results {
        if !strike_result.hit {
            let defender_pos = world_view.data::<PositionData>(*target).hex.as_cart_vec();
            let (msg, color) = (String::from("miss"), Color::new(0.1, 0.0, 0.0, 1.0));
            let rising_damage_text = TextAnimationElement::new(msg, FontSize::HeadingMajor, defender_pos + CartVec::new(0.0, 0.5), color, 3.0)
                .with_delta(CartVec::new(0.0, 1.0), InterpolationType::Linear)
                .with_end_color(color.with_a(0.0), InterpolationType::Linear)
                .with_blocking_duration(0.0);
            animation_group = animation_group.with_animation(rising_damage_text, Some(miss_anim_start_point));
        }
    }

    vec![box animation_group]
}

fn animate_damage(world_view: &WorldView, entity: Entity, damage_done: u32) -> Vec<Box<AnimationElement>> {
    let entity_data = world_view.character(entity);

    let mut animation_group = AnimationGroup::new();
    if damage_done > 0 {
        let starting_health = world_view.character(entity).health.cur_value();
        let damage_bar_animation = EntityFieldAnimation::new(
            entity,
            Interpolation::linear_from_delta(starting_health as f32, -(damage_done as f32)),
            |data: &mut CharacterData, new_value: f32| { data.health.reduce_to(new_value.floor() as i32); },
            0.5);

        let start_color = entity_data.graphics.color;
        let end_color = Color::new(1f32, 0.1f32, 0.1f32, 1f32);
        let red_tint_animation = EntityFieldAnimation::new(
            entity,
            Interpolation::linear_from_endpoints(start_color, end_color).circular(),
            |data: &mut GraphicsData, new_value| { data.color = new_value; },
            0.5,
        );

        animation_group = animation_group
            .with_animation(damage_bar_animation, None)
            .with_animation(red_tint_animation, None);
    }

    let (msg, color) = (damage_done.to_string(), Color::new(0.9, 0.2, 0.2, 1.0));
    let rising_damage_text = TextAnimationElement::new(msg, FontSize::HeadingMajor, entity_data.position.hex.as_cart_vec() + CartVec::new(0.0, 0.5), color, 3.0)
        .with_delta(CartVec::new(0.0, 1.0), InterpolationType::Linear)
        .with_end_color(color.with_a(0.0), InterpolationType::Linear)
        .with_blocking_duration(0.0);

    animation_group = animation_group.with_animation(rising_damage_text, None);
    vec![box animation_group]
}

struct EntityFieldAnimation<T: EntityData, F: Interpolateable<F>, S: Fn(&mut T, F)> {
    pub entity: Entity,
    pub interpolation: Interpolation<F>,
    pub store_function: S,
    pub duration: f64,
    pub blocking_duration: Option<f64>,
    pub phantom_: PhantomData<T>,
}

impl<T: EntityData, F: Interpolateable<F>, S: Fn(&mut T, F)> EntityFieldAnimation<T, F, S> {
    pub fn new(entity: Entity, interpolation: Interpolation<F>, store_function: S, duration: f64) -> EntityFieldAnimation<T, F, S> {
        EntityFieldAnimation {
            entity,
            interpolation,
            store_function,
            duration,
            blocking_duration: None,
            phantom_: PhantomData::default(),
        }
    }

    pub fn with_blocking_duration(mut self, blocking_duration: f64) -> Self {
        self.blocking_duration = Some(blocking_duration);
        self
    }
}

impl<T: EntityData, F: Interpolateable<F>, S: Fn(&mut T, F)> AnimationElement for EntityFieldAnimation<T, F, S> where T : DeserializeOwned {
    fn draw(&self, view: &mut WorldView, pcnt_elapsed: f64) -> DrawList {
        if pcnt_elapsed > 1.0 || pcnt_elapsed < 0.0 {
            warn!("Unexpected, pcnt was more than 1: {}", pcnt_elapsed);
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

impl<T: EntityData, F: Interpolateable<F>, S: Fn(&mut T, F)> Debug for EntityFieldAnimation<T, F, S> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "EntityFieldAnimation[{:?}]", self.entity)
    }
}