use game::world::World;
use game::world::WorldView;
use game::Entity;
use entities::*;
use entities::Skill;
use entities::modifiers::*;
use game::events::*;
use common::hex::*;
use common::flood_search;
use rand::Rng;
use rand::SeedableRng;
use rand::StdRng;
use entities::Attack;
use noisy_float::prelude::*;
use noisy_float;
use game::core::Sext;
use logic::combat;
use logic::experience::level_curve;
use game::core::DicePool;
use common::prelude::*;
use game::reflect::ReduceableField;
use game::reflect::SettableField;
use std::ops;
use game::entity::EntityData;
use game::modifiers::FieldLogs;
use std::fmt::Display;
use common::reflect::Field;
use events::GameEvent;

pub enum StrikeIndex {
    Strike(usize),
    Counter(usize)
}

#[derive(Default)]
pub struct AttackBreakdown {
    pub strikes : Vec<StrikeBreakdown>,
    pub counters : Vec<StrikeBreakdown>,
    pub ordering : Vec<StrikeIndex>,
    pub damage_types : Vec<DamageType>
}
impl AttackBreakdown {
    pub fn add_strike(&mut self, strike : StrikeBreakdown) {
        self.strikes.push(strike);
        self.ordering.push(StrikeIndex::Strike(self.strikes.len() - 1));
    }
    pub fn add_counter(&mut self, counter : StrikeBreakdown) {
        self.counters.push(counter);
        self.ordering.push(StrikeIndex::Counter(self.counters.len() - 1));
    }
}
//
//pub struct BreakdownComponent<T> {
//    contribution : T,
//    description : Str,
//    details : Vec<String>
//}
#[derive(Default)]
pub struct Breakdown<T : Default> {
    pub total : T,
    pub components : Vec<(String, String)>
}
impl <T : Default> Breakdown<T> where T : ops::Add<Output=T> + Copy + ToStringWithSign {
    pub fn add_field<S1 : Into<String>, E : EntityData, U : ToStringWithSign + Clone>(&mut self, net_value : T, logs : &FieldLogs<E>, field : &'static Field<E,U>, descriptor : S1) {
        self.total = self.total + net_value;
        let base_value = (field.getter)(&logs.base_value);
        let base_value_str = base_value.to_string_with_sign();
        self.components.push((base_value_str, format!("base {}", descriptor.into())));
        for field_mod in logs.modifications_for(field) {
            let mut modification_str = field_mod.modification.to_string();
            modification_str.retain(|c| ! c.is_whitespace());
            self.components.push((modification_str, strf(field_mod.description.unwrap_or(""))))
        }
    }

    pub fn add<S1 : Into<String>>(&mut self, value : T, descriptor : S1) {
        self.total = self.total + value;
        self.components.push((value.to_string_with_sign(), descriptor.into()));
    }

}

#[derive(Default)]
pub struct StrikeBreakdown {
    pub to_hit_components : Breakdown<i32>,
    pub to_miss_components: Breakdown<i32>,
    pub damage_bonus_components : Breakdown<i32>,
    pub damage_resistance_components : Breakdown<f32>,
    pub damage_absorption_components: Breakdown<i32>,
//    pub dice_count_components: Vec<(i32, Str)>,
//    pub die_components: Vec<(i32, Str)>,
    pub damage_dice_components: Vec<(DicePool, Str)>,
    pub ap_cost_components: Breakdown<i32>
}
impl StrikeBreakdown {
//    pub fn to_hit_total(&self) -> i32 { self.to_hit_components.iter().map(|c| c.0).sum() }
//    pub fn to_miss_total(&self) -> i32 { self.to_miss_components.iter().map(|c| c.0).sum() }
//    pub fn damage_bonus_total(&self) -> i32 { self.damage_bonus_components.iter().map(|c| c.0).sum() }
//    pub fn damage_resistance_total(&self) -> f32 { self.damage_resistance_components.iter().map(|c| c.0).sum() }
//    pub fn damage_absorption_total(&self) -> i32 { self.damage_absorption_components.iter().map(|c| c.0).sum() }
//    pub fn damage_dice_total<'a>(&'a self) -> impl Iterator<Item = DicePool> + 'a {
////        let dice_count : i32 = self.dice_count_components.iter().map(|c| c.0).sum();
////        let die : i32 = self.die_components.iter().map(|c| c.0).sum();
////        DicePool::of(dice_count.as_u32_or_0(), die.as_u32_or_0())
//        self.damage_dice_components.iter().map(|dd| dd.0)
//    }
//    pub fn ap_cost_total(&self) -> u32 {
//        let cost : i32 = self.ap_cost_components.iter().map(|c| c.0).sum();
//        cost.as_u32_or_0()
//    }
pub fn to_hit_total(&self) -> i32 { self.to_hit_components.total }
    pub fn to_miss_total(&self) -> i32 { self.to_miss_components.total }
    pub fn damage_bonus_total(&self) -> i32 { self.damage_bonus_components.total }
    pub fn damage_resistance_total(&self) -> f32 { self.damage_resistance_components.total }
    pub fn damage_absorption_total(&self) -> i32 { self.damage_absorption_components.total }
    pub fn damage_dice_total<'a>(&'a self) -> impl Iterator<Item = DicePool> + 'a {
//        let dice_count : i32 = self.dice_count_components.iter().map(|c| c.0).sum();
//        let die : i32 = self.die_components.iter().map(|c| c.0).sum();
//        DicePool::of(dice_count.as_u32_or_0(), die.as_u32_or_0())
        self.damage_dice_components.iter().map(|dd| dd.0)
    }
    pub fn ap_cost_total(&self) -> u32 {
        let cost : i32 = self.ap_cost_components.total;
        cost.as_u32_or_0()
    }
}


pub fn compute_attack_breakdown(world : &World, world_view : &WorldView, attacker : Entity, defender : Entity, attack : &Attack) -> AttackBreakdown {

    let mut attack_breakdown = AttackBreakdown::default();

    let attacker_data = world_view.character(attacker);
    attack_breakdown.damage_types.push(attack.primary_damage_type);
    if let Some(secondary_damage_type) = attack.secondary_damage_type {
        attack_breakdown.damage_types.push(secondary_damage_type);
    }

    let mut attacker_strikes = attacker_data.action_points.cur_value() / attack.ap_cost as i32;
    let (defender_counter_attack, mut defender_counters) =
        combat::counters_for(world_view, defender, attacker, attack);

    let mut attacker_turn = true;
    while attacker_strikes > 0 || defender_counters > 0 {
        if attacker_turn && attacker_strikes > 0 {
            attack_breakdown.add_strike(compute_strike_breakdown(world, world_view, attacker, defender, attack));
            attacker_strikes -= 1;
        } else if ! attacker_turn && defender_counters > 0 {
            attack_breakdown.add_counter(compute_strike_breakdown(world, world_view, defender, attacker, &defender_counter_attack));
            defender_counters -= 1;
        }
        attacker_turn = ! attacker_turn;
    }

    attack_breakdown
}

pub fn compute_strike_breakdown(world : &World, view : &WorldView, attacker_ref : Entity, defender_ref : Entity, attack : &Attack) -> StrikeBreakdown {
    let mut ret = StrikeBreakdown::default();

    ret.ap_cost_components.add(attack.ap_cost as i32, "weapon ap cost");

    let attacker = view.character(attacker_ref);
    let attacker_combat = view.combat(attacker_ref);
    let attacker_combat_field_log = world.field_logs_for::<CombatData>(attacker_ref);
    let attacker_skills = view.skills(attacker_ref);
    let defender = view.character(defender_ref);
    let defender_combat = view.combat(defender_ref);
    let defender_skills = view.skills(defender_ref);

    let _attacker_tile : &TileData = view.tile(attacker.position.hex);
    let defender_tile : &TileData = view.tile(defender.position.hex);

    match attack.range {
        i if i <= 1 => {
            ret.to_hit_components.add_field(attacker_combat.melee_accuracy_bonus, &attacker_combat_field_log, &CombatData::melee_accuracy_bonus, "melee accuracy");
//            ret.to_hit_components.push((attacker_combat_field_log.base_value.melee_accuracy_bonus, "base melee accuracy"));
//            for field_mod in attacker_combat_field_log.modifications_for(&CombatData::melee_accuracy_bonus) {
//                ret.to_hit_components.push((field_mod.modification, field_mod.description.unwrap_or("")))
//            }
        },
        _ => ret.to_hit_components.add_field(attacker_combat.ranged_accuracy_bonus, &attacker_combat_field_log, &CombatData::ranged_accuracy_bonus, "ranged accuracy")
            //ret.to_hit_components.add(attacker_combat.ranged_accuracy_bonus, "base ranged accuracy")
    }

    ret.to_miss_components.add(defender_combat.dodge_bonus, "dodge");

    ret.to_hit_components.add(attack.to_hit_bonus, "weapon accuracy");

    ret.to_miss_components.add(defender_tile.cover as i32, "terrain defense");

//    ret.dice_count_components.push((attack.damage_dice.count as i32, "weapon dice"));
//    ret.die_components.push((attack.damage_dice.die as i32, "weapon die size"));
    ret.damage_dice_components.push((attack.damage_dice, "base weapon damage"));


    match attack.range {
        i if i <= 1 => ret.damage_bonus_components.add(attacker_combat.melee_damage_bonus, "base melee damage bonus"),
        _ => ret.damage_bonus_components.add(attacker_combat.ranged_damage_bonus, "base ranged damage bonus")
    }
    ret.damage_bonus_components.add(attack.damage_bonus, "weapon damage bonus");

    ret
}

pub fn handle_attack(world : &mut World, attacker_ref : Entity, defender_ref : Entity, attack : &Attack) {
    let world_view = world.view();

    let attack_breakdown = compute_attack_breakdown(world, world_view, attacker_ref, defender_ref, attack);

    for strike_index in attack_breakdown.ordering {
        match strike_index {
            StrikeIndex::Strike(i) => handle_strike(world, attacker_ref, defender_ref, &attack_breakdown.strikes[i]),
            StrikeIndex::Counter(i) => handle_strike(world, defender_ref, attacker_ref, &attack_breakdown.counters[i])
        }
    }

    let attack_skill_type = match attack.range {
        i if i <= 1 => Skill::Melee,
        _ => Skill::Ranged
    };
    modify(world, attacker_ref, SkillXPMod(attack_skill_type, 1));
    modify(world, attacker_ref, ReduceStaminaMod(Sext::of(1)));

    world.add_event(GameEvent::Attack { attacker : attacker_ref, defender : defender_ref });
}


/*
This was originally something else. But let's give it another think. We're doing away with percentages, because fuck percentages, too impersonal.
Okay, if we start from basis of 3d6 that gives us a normal-ish distribution between [3,18]. Various things give bonuses to hit, others give maluses.
Skills don't automatically give a curve, but specific levels give discrete bumps.
*/

pub fn handle_strike(world : &mut World, attacker_ref : Entity, defender_ref : Entity, strike : &StrikeBreakdown) {
    let seed = world.random_seed(13);
    let mut rng : StdRng = SeedableRng::from_seed(seed);

    let view = world.view();

    let attacker = view.character(attacker_ref);
    let attacker_combat = view.combat(attacker_ref);
    let attacker_skills = view.skills(attacker_ref);
    let defender = view.character(defender_ref);
    let defender_combat = view.combat(defender_ref);
    let defender_skills = view.skills(defender_ref);

    if ! attacker.is_alive() || ! defender.is_alive() {
        return;
    }

    // reduce the actions available to the attacker by the cost of the attack, regardless of how the attack goes
    world.add_modifier(attacker_ref, CharacterData::action_points.reduce_by(strike.ap_cost_total() as i32), "attack");

    let to_miss_total = strike.to_miss_total();
    let to_hit_total = strike.to_hit_total();

    // base number needed to be hit with no modifiers one way or another. With a base value of 8, 85% of attacks will hit
    // given no modifiers one way or another. We probably want to shift that a bit, and give a noticeable bump in the early
    // levels of dodging/attacking such that unskilled commoners are pretty useless at attacking until they get a bit of experience
    // 62.5% will have at least a 10, so it's still a decent chance to hit
    let base_to_hit = 10;

    let to_hit_modifiers = to_hit_total - to_miss_total;

    let dice = DicePool::of(3, 6);
    let is_hit = dice.roll(&mut rng).total_result as i32 + to_hit_modifiers >= base_to_hit;
    if is_hit {
        let damage_dice = strike.damage_dice_total();
        let damage_total = damage_dice.map(|dd| dd.roll(&mut rng).total_result).sum();

        world.modify(defender_ref, CharacterData::health.reduce_by(damage_total as i32), "attack damage");
        world.modify(attacker_ref, CharacterData::moves.set_to(Sext::of(0)), None);

        let killing_blow = !view.data::<CharacterData>(defender_ref).is_alive();

        world.add_event(GameEvent::Strike {
            attacker : attacker_ref,
            defender : defender_ref,
            damage_done : damage_total,
            hit : true,
            killing_blow
        });
    } else {
        world.add_event(GameEvent::Strike {
            attacker : attacker_ref,
            defender : defender_ref,
            damage_done : 0,
            hit : false,
            killing_blow : false
        });
    }
}

pub fn accuracy_for_skill_level(level : u32) -> f64 {
    // even a totally unskilled individual can hit half the time, a perfectly skilled individual
    // will hit every time even in adverse conditions (> 1.0 base rate)
    0.5 + level_curve(level) * 1.0
}

pub fn dodge_for_skill_level(level : u32) -> f64 {
    // an unskilled individual won't dodge much of anything, a half skilled individual will always
    // dodge
    level_curve(level)
}

pub fn damage_multiplier_for_skill_level(level : u32) -> f64 {
    level_curve(level)
}

pub fn counters_for(world_view : &WorldView, defender_ref : Entity, countering_ref : Entity, _countering_attack : &Attack) -> (Attack, u32) {
    let defender = world_view.character(defender_ref);
    let defender_combat = world_view.combat(defender_ref);
    let countering = world_view.character(countering_ref);

    // can't counter on ranged attacks
    if defender.position.hex.distance(&countering.position.hex) > 1.0 {
        (Attack::default(), 0)
    } else {
        let possible_attacks = possible_attacks(world_view, defender_ref);
        let possible_attacks = valid_attacks(world_view, defender_ref, &possible_attacks, countering_ref);
        let attack_to_use = best_attack(world_view, defender_ref, &possible_attacks, countering_ref);
        if let Some(attack) = attack_to_use {
            if defender_combat.counters.cur_value() > 0 {
                (attack.clone(), 1)
            } else {
                (attack.clone(), 0)
            }
        } else {
            (Attack::default(), 0)
        }
    }
}

pub fn possible_attacks(world : &WorldView, attacker : Entity) -> Vec<Attack> {
    let mut res = world.combat(attacker).natural_attacks.clone();
    for item_ref in &world.inventory(attacker).equipped {
        let item = world.item(*item_ref);
        if let Some(ref attack) = item.primary_attack {
            res.push(attack.clone());
        }
        if let Some(ref attack) = item.secondary_attack {
            res.push(attack.clone());
        }
    }
    res
}

pub fn default_attack(world : &WorldView, attacker : Entity) -> Option<Attack> {
    for item_ref in &world.inventory(attacker).equipped {
        let item = world.item(*item_ref);
        if let Some(ref attack) = item.primary_attack {
            return Some(attack.clone());
        }
        if let Some(ref attack) = item.secondary_attack {
            return Some(attack.clone());
        }
    }
    return world.combat(attacker).natural_attacks.first().cloned();
}

pub fn primary_attack(world : &WorldView, attacker : Entity) -> Option<Attack> {
    let combat_data = world.data::<CombatData>(attacker);
    if let Some(attack) = combat_data.active_attack.referenced_attack(world, attacker) {
        Some(attack.clone())
    } else {
        default_attack(world, attacker)
    }
}

pub fn valid_attacks<'a, 'b>(world_view: &'a WorldView, attacker : Entity, attacks: &'b Vec<Attack>, defender : Entity) -> Vec<&'b Attack> {
    let attacker = world_view.character(attacker);
    let defender = world_view.character(defender);
    let dist = attacker.position.hex.distance(&defender.position.hex).raw() as u32;

    let mut ret = vec![];
    for attack in attacks {
        if attack.range >= dist && attack.min_range <= dist {
            ret.push(attack);
        }
    }
    ret
}

pub fn best_attack<'a, 'b>(world_view: &'a WorldView, attacker : Entity, attacks: &'b Vec<&'b Attack>, defender : Entity) -> Option<&'b Attack> {
    let attacker = world_view.character(attacker);
    let defender = world_view.character(defender);
    attacks.get(0).map(|x| *x)
}