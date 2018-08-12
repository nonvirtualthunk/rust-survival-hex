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
use logic::item;
use logic;
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
use std::collections::HashMap;
use entities::combat::CombatData;

pub enum StrikeIndex {
    Strike(usize),
    Counter(usize),
}

#[derive(Default)]
pub struct AttackBreakdown {
    pub strikes: Vec<StrikeBreakdown>,
    pub counters: Vec<StrikeBreakdown>,
    pub ordering: Vec<StrikeIndex>,
}

impl AttackBreakdown {
    pub fn add_strike(&mut self, strike: StrikeBreakdown) {
        self.strikes.push(strike);
        self.ordering.push(StrikeIndex::Strike(self.strikes.len() - 1));
    }
    pub fn add_counter(&mut self, counter: StrikeBreakdown) {
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
pub struct Breakdown<T: Default> {
    pub total: T,
    pub components: Vec<(String, String)>,
}

impl<T: Default> Breakdown<T> where T: ops::Add<Output=T> + Copy + ToStringWithSign {
    pub fn add_field<S1: Into<String>, E: EntityData, U: ToStringWithSign + Clone>(&mut self, net_value: T, logs: &FieldLogs<E>, field: &'static Field<E, U>, descriptor: S1) {
        self.total = self.total + net_value;
        let base_value = (field.getter)(&logs.base_value);
        let base_value_str = base_value.to_string_with_sign();
        self.components.push((base_value_str, format!("base {}", descriptor.into())));
        for field_mod in logs.modifications_for(field) {
            let mut modification_str = field_mod.modification.to_string();
            modification_str.retain(|c| !c.is_whitespace());
            self.components.push((modification_str, strf(field_mod.description.unwrap_or(""))))
        }
    }

    pub fn add<S1: Into<String>>(&mut self, value: T, descriptor: S1) {
        self.total = self.total + value;
        self.components.push((value.to_string_with_sign(), descriptor.into()));
    }
}

#[derive(Default)]
pub struct StrikeBreakdown {
    pub to_hit_components: Breakdown<i32>,
    pub to_miss_components: Breakdown<i32>,
    pub damage_bonus_components: Breakdown<i32>,
    pub damage_resistance_components: Breakdown<f32>,
    pub damage_absorption_components: Breakdown<i32>,
    //    pub dice_count_components: Vec<(i32, Str)>,
//    pub die_components: Vec<(i32, Str)>,
    pub damage_dice_components: Vec<(DicePool, Str)>,
    pub ap_cost_components: Breakdown<i32>,
    pub damage_types: Vec<DamageType>,
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
    pub fn damage_dice_total<'a>(&'a self) -> impl Iterator<Item=DicePool> + 'a {
//        let dice_count : i32 = self.dice_count_components.iter().map(|c| c.0).sum();
//        let die : i32 = self.die_components.iter().map(|c| c.0).sum();
//        DicePool::of(dice_count.as_u32_or_0(), die.as_u32_or_0())
        self.damage_dice_components.iter().map(|dd| dd.0)
    }
    pub fn ap_cost_total(&self) -> u32 {
        let cost: i32 = self.ap_cost_components.total;
        cost.as_u32_or_0()
    }
}

pub fn within_range(world_view: &WorldView, attacker: Entity, defender: Entity, attack: &Attack, from_position: Option<AxialCoord>, to_position: Option<AxialCoord>) -> bool {
    let from_position = from_position.unwrap_or_else(|| world_view.data::<PositionData>(attacker).hex);
    let to_position = to_position.unwrap_or_else(|| world_view.data::<PositionData>(defender).hex);

    let dist = from_position.distance(&to_position);
    dist <= attack.range as f32 && dist >= attack.min_range as f32
}

/// returns whether it would ever be valid to attack this target, irrespective of current state. Primarily intended for determining if the defender is
/// alive at the moment. Allows for attacking friends, that should be checked separately
pub fn is_valid_attack_target(world_view: &WorldView, attacker_: Entity, defender: Entity, attack_: &Attack) -> bool {
    if !world_view.data::<CharacterData>(defender).is_alive() {
        return false;
    }

    true
}

pub fn can_attack(world_view: &WorldView, attacker: Entity, defender: Entity, attack: &Attack, from_position: Option<AxialCoord>, to_position: Option<AxialCoord>) -> bool {
    if !is_valid_attack_target(world_view, attacker, defender, attack) {
        return false;
    }

    if world_view.data::<CharacterData>(attacker).action_points.cur_value() < attack.ap_cost as i32 {
        return false;
    }

    within_range(world_view, attacker, defender, attack, from_position, to_position)
}

pub fn possible_attack_locations_with_cost(world_view: &WorldView, attacker: Entity, defender: Entity, attack: &Attack) -> HashMap<AxialCoord, f64> {
    let target_pos = world_view.data::<PositionData>(defender).hex;
    let max_move = logic::movement::max_moves_remaining(world_view, attacker, 1.0);
    logic::movement::hexes_in_range(world_view, attacker, max_move).into_iter()
        .filter(|(k, v)| can_attack(world_view, attacker, defender, attack, Some(*k), Some(target_pos))).collect()
}

pub fn does_event_trigger_counterattack(world_view: &WorldView, counterer: Entity, event: &GameEventWrapper<GameEvent>) -> bool {
    if let GameEvent::Strike { attacker, defender, strike_number, .. } = event.event {
        if event.is_ended() && counterer == defender {
            let combat_data = world_view.data::<CombatData>(defender);
            if combat_data.counters_per_event as u8 > strike_number && combat_data.counters_remaining.cur_value() > 0 {
                return true;
            }
        }
    }
    false
}

pub fn closest_attack_location_with_cost(world_view: &WorldView, attacker: Entity, defender: Entity, attack: &Attack) -> Option<(AxialCoord, f64)> {
    possible_attack_locations_with_cost(world_view, attacker, defender, attack).into_iter().min_by_key(|(k, v)| r64(*v))
}

pub fn compute_attack_breakdown(world: &World, world_view: &WorldView, attacker: Entity, defender: Entity, attack: &Attack) -> AttackBreakdown {
    let mut attack_breakdown = AttackBreakdown::default();

    let attacker_data = world_view.character(attacker);

    let mut attacker_strikes = attacker_data.action_points.cur_value() / attack.ap_cost as i32;
    let (defender_counter_attack, mut defender_counters) =
        combat::counters_for(world_view, defender, attacker, attack);

    let mut attacker_turn = true;
    while attacker_strikes > 0 || defender_counters > 0 {
        if attacker_turn && attacker_strikes > 0 {
            attack_breakdown.add_strike(compute_strike_breakdown(world, world_view, attacker, defender, attack));
            attacker_strikes -= 1;
        } else if !attacker_turn && defender_counters > 0 {
            attack_breakdown.add_counter(compute_strike_breakdown(world, world_view, defender, attacker, &defender_counter_attack));
            defender_counters -= 1;
        }
        attacker_turn = !attacker_turn;
    }

    attack_breakdown
}

pub fn compute_strike_breakdown(world: &World, view: &WorldView, attacker_ref: Entity, defender_ref: Entity, attack: &Attack) -> StrikeBreakdown {
    let mut ret = StrikeBreakdown::default();

    ret.damage_types.push(attack.primary_damage_type);
    if let Some(secondary_damage_type) = attack.secondary_damage_type {
        ret.damage_types.push(secondary_damage_type);
    }

    ret.ap_cost_components.add(attack.ap_cost as i32, "weapon ap cost");

    let attacker = view.character(attacker_ref);
    let attacker_combat = view.combat(attacker_ref);
    let attacker_combat_field_log = world.field_logs_for::<CombatData>(attacker_ref);
    let attacker_skills = view.skills(attacker_ref);
    let defender = view.character(defender_ref);
    let defender_combat = view.combat(defender_ref);
    let defender_skills = view.skills(defender_ref);

    let _attacker_tile: &TileData = view.tile(attacker.position.hex);
    let defender_tile: &TileData = view.tile(defender.position.hex);

    match attack.attack_type {
        AttackType::Melee | AttackType::Reach => {
            ret.to_hit_components.add_field(attacker_combat.melee_accuracy_bonus, &attacker_combat_field_log, &CombatData::melee_accuracy_bonus, "melee accuracy");
        }
        AttackType::Projectile | AttackType::Thrown => {
            ret.to_hit_components.add_field(attacker_combat.ranged_accuracy_bonus, &attacker_combat_field_log, &CombatData::ranged_accuracy_bonus, "ranged accuracy")
        }
    }
    ret.to_miss_components.add(defender_combat.defense_bonus, "defense");
    ret.to_miss_components.add(defender_combat.dodge_bonus, "dodge");
    ret.to_miss_components.add(defender_combat.block_bonus, "block");

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

pub fn handle_attack(world: &mut World, attacker_ref: Entity, defender_ref: Entity, attack_ref: &AttackReference) {
    if let Some(attack) = attack_ref.referenced_attack(world.view(), attacker_ref) {
        let world_view = world.view();

        let attack_breakdown = compute_attack_breakdown(world, world_view, attacker_ref, defender_ref, attack);

        world.start_event(GameEvent::Attack { attacker: attacker_ref, defender: defender_ref });

        for strike_index in attack_breakdown.ordering {
            match strike_index {
                StrikeIndex::Strike(i) => handle_strike(world, attacker_ref, defender_ref, &attack_breakdown.strikes[i], i as u8),
                StrikeIndex::Counter(i) => handle_strike(world, defender_ref, attacker_ref, &attack_breakdown.counters[i], i as u8)
            }
        }

        let attack_skill_type = match attack.range {
            i if i <= 1 => Skill::Melee,
            _ => Skill::Ranged
        };
        modify(world, attacker_ref, SkillXPMod(attack_skill_type, 1));
        modify(world, attacker_ref, ReduceStaminaMod(Sext::of(1)));

        world.end_event(GameEvent::Attack { attacker: attacker_ref, defender: defender_ref });

        if attack.attack_type == AttackType::Thrown {
            // thrown natural attacks don't cause you to lose an entity when they occur, but otherwise we want to un-equip the weapon
            // and put it in the world
            if attack_ref.entity != attacker_ref {
                item::unequip_item(world, attacker_ref, attack_ref.entity, true);
                let defender_pos = world_view.data::<PositionData>(defender_ref).hex;
                let attacker_pos = world_view.data::<PositionData>(attacker_ref).hex;

                let drop_pos = *defender_pos.neighbors().iter().min_by_key(|n| n.distance(&attacker_pos)).unwrap();
                item::place_item_in_world(world, attack_ref.entity, drop_pos);
            }
        }
    } else {
        warn!("Attempted an attack with a non-resolveable attack reference");
    }
}


/*
This was originally something else. But let's give it another think. We're doing away with percentages, because fuck percentages, too impersonal.
Okay, if we start from basis of 3d6 that gives us a normal-ish distribution between [3,18]. Various things give bonuses to hit, others give maluses.
Skills don't automatically give a curve, but specific levels give discrete bumps.
*/

pub fn handle_strike(world: &mut World, attacker_ref: Entity, defender_ref: Entity, strike: &StrikeBreakdown, strike_number: u8) {
    let seed = world.random_seed(13);
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    let view = world.view();

    let attacker = view.character(attacker_ref);
    let attacker_combat = view.combat(attacker_ref);
    let attacker_skills = view.skills(attacker_ref);
    let defender = view.character(defender_ref);
    let defender_combat = view.combat(defender_ref);
    let defender_skills = view.skills(defender_ref);

    if !attacker.is_alive() || !defender.is_alive() {
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
        let damage_total: i32 = damage_dice.map(|dd| dd.roll(&mut rng).total_result as i32).sum::<i32>()
            + strike.damage_bonus_total()
            - strike.damage_absorption_total();
        let damage_total: u32 = damage_total.as_u32_or_0();

        world.start_event(GameEvent::Strike {
            attacker: attacker_ref,
            defender: defender_ref,
            damage_done: damage_total,
            hit: true,
            killing_blow: false,
            strike_number,
        });

        logic::character::apply_damage_to_character(world, defender_ref, damage_total, &strike.damage_types);
        world.modify(attacker_ref, CharacterData::moves.set_to(Sext::of(0)), None);

        let killing_blow = !view.data::<CharacterData>(defender_ref).is_alive();

        world.end_event(GameEvent::Strike {
            attacker: attacker_ref,
            defender: defender_ref,
            damage_done: damage_total,
            hit: true,
            killing_blow,
            strike_number,
        });
    } else {
        world.add_event(GameEvent::Strike {
            attacker: attacker_ref,
            defender: defender_ref,
            damage_done: 0,
            hit: false,
            killing_blow: false,
            strike_number,
        });
    }
}

pub fn accuracy_for_skill_level(level: u32) -> f64 {
    // even a totally unskilled individual can hit half the time, a perfectly skilled individual
    // will hit every time even in adverse conditions (> 1.0 base rate)
    0.5 + level_curve(level) * 1.0
}

pub fn dodge_for_skill_level(level: u32) -> f64 {
    // an unskilled individual won't dodge much of anything, a half skilled individual will always
    // dodge
    level_curve(level)
}

pub fn damage_multiplier_for_skill_level(level: u32) -> f64 {
    level_curve(level)
}

pub fn counters_for(world_view: &WorldView, defender_ref: Entity, countering_ref: Entity, _countering_attack: &Attack) -> (Attack, u32) {
    let defender = world_view.character(defender_ref);
    let defender_combat = world_view.combat(defender_ref);
    let countering = world_view.character(countering_ref);

    // can't counter on ranged attacks
    if defender.position.hex.distance(&countering.position.hex) > 1.0 {
        (Attack::default(), 0)
    } else {
        let counter_attack = counter_attack_ref_to_use(world_view, defender_ref).and_then(|ar| ar.referenced_attack(world_view, defender_ref));
        if let Some(attack) = counter_attack {
            if defender_combat.counters_remaining.cur_value() > 0 {
                (attack.clone(), 1)
            } else {
                (attack.clone(), 0)
            }
        } else {
            warn!("Not countering, no counter attack to use found");
            (Attack::default(), 0)
        }
    }
}

pub fn possible_attack_refs(world: &WorldView, attacker: Entity) -> Vec<AttackReference> {
    let mut res = world.combat(attacker).natural_attacks.iter().enumerate().map(|(i, a)| AttackReference::new(attacker, i, a.name)).collect_vec();
    for item_ref in &world.inventory(attacker).equipped {
        let item = world.item(*item_ref);
        res.extend(item.attacks.iter().enumerate().map(|(i, a)| AttackReference::new(*item_ref, i, a.name)));
    }
    res
}

pub fn possible_attacks(world: &WorldView, attacker: Entity) -> Vec<Attack> {
    possible_attack_refs(world, attacker).iter().flat_map(|ar| ar.referenced_attack(world, attacker).cloned()).collect_vec()
}

pub fn default_attack(world: &WorldView, attacker: Entity) -> Option<AttackReference> {
    for item_ref in &world.inventory(attacker).equipped {
        let item = world.item(*item_ref);
        if let Some(attack) = item.attacks.first() {
            return Some(AttackReference::new(*item_ref, 0, attack.name));
        }
    }
    world.combat(attacker).natural_attacks.first().map(|a| AttackReference::new(attacker, 0, a.name))
}

pub fn counter_attack_ref_to_use(world: &WorldView, counter_attacker: Entity) -> Option<AttackReference> {
    primary_attack_ref(world, counter_attacker)
        .filter(|par| par.is_melee(world, counter_attacker))
        .or_else(|| {
            let mut melee_attacks = possible_attack_refs(world, counter_attacker);
            melee_attacks.retain(|a| a.is_melee(world, counter_attacker));
            best_attack(world, counter_attacker, &melee_attacks)
        })
}
pub fn counter_attack_to_use(world: &WorldView, counter_attacker: Entity) -> Option<&Attack> {
    counter_attack_ref_to_use(world, counter_attacker).and_then(|ar| ar.referenced_attack(world, counter_attacker))
}

pub fn primary_attack_ref(world: &WorldView, attacker: Entity) -> Option<AttackReference> {
    let combat_data = world.data::<CombatData>(attacker);
    combat_data.active_attack.as_option().cloned().or(default_attack(world, attacker))
}

pub fn primary_attack(world: &WorldView, attacker: Entity) -> Option<Attack> {
    primary_attack_ref(world, attacker).and_then(|r| r.referenced_attack(world, attacker).cloned())
}

pub fn valid_attacks(world_view: &WorldView, attacker: Entity, attacks_references: &Vec<AttackReference>, defender: Entity) -> Vec<AttackReference> {
    let attacker_c = world_view.character(attacker);
    let defender_c = world_view.character(defender);

    let mut ret = vec![];
    for attack_ref in attacks_references {
        if let Some(attack) = attack_ref.referenced_attack(world_view, attacker) {
            if within_range(world_view, attacker, defender, attack, None, None) {
                ret.push(attack_ref.clone());
            }
        }
    }
    ret
}

pub fn best_attack(view: &WorldView, attacker: Entity, attacks: &Vec<AttackReference>) -> Option<AttackReference> {
    attacks.iter().cloned().max_by_key(|ar| ar.referenced_attack(view, attacker).map(|ra| r32(ra.damage_dice.avg_roll() / ra.ap_cost.max(1) as f32)).unwrap_or(r32(0.0)))
}

pub fn best_attack_against(world_view: &WorldView, attacker: Entity, attacks: &Vec<AttackReference>, defender: Entity) -> Option<AttackReference> {
    let attacker = world_view.character(attacker);
    let defender = world_view.character(defender);
    attacks.get(0).map(|x| x.clone())
}