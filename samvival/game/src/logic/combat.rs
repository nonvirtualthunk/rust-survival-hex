use game::world::World;
use game::world::WorldView;
use game::Entity;
use data::entities::*;
use data::entities::modifiers::*;
use data::events::*;
use common::flood_search;
use common::hex::CubeCoord;
use rand::Rng;
use rand::SeedableRng;
use rand::StdRng;
use data::entities::Attack;
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
use data::events::GameEvent;
use std::collections::HashMap;
use data::entities::combat::CombatData;
use cgmath::InnerSpace;
use common::hex::CartVec;
use game::reflect::*;
use prelude::*;

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
            self.components.push((modification_str, field_mod.description.clone().unwrap_or_else(||String::from(""))))
        }
    }

    pub fn add<S1: Into<String>>(&mut self, value: T, descriptor: S1) {
        self.total = self.total + value;
        self.components.push((value.to_string_with_sign(), descriptor.into()));
    }
}

#[derive(Default)]
pub struct StrikeTargetBreakdown {
    pub target : Entity,
    pub to_hit_components: Breakdown<i32>,
    pub to_miss_components: Breakdown<i32>,
    pub damage_bonus_components: Breakdown<i32>,
    pub damage_resistance_components: Breakdown<f32>,
    pub damage_absorption_components: Breakdown<i32>,
    //    pub dice_count_components: Vec<(i32, Str)>,
//    pub die_components: Vec<(i32, Str)>,
    pub damage_dice_components: Vec<(DicePool, Str)>,
}

impl StrikeTargetBreakdown {
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
}

#[derive(Default)]
pub struct StrikeBreakdown {
    pub attack: Attack,
    pub weapon: Entity,
    pub ap_cost_components: Breakdown<i32>,
    pub damage_types: Vec<DamageType>,
    pub per_target_breakdowns : Vec<StrikeTargetBreakdown>,
}

impl StrikeBreakdown {
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


pub fn possible_attack_locations_with_cost(world_view: &WorldView, hexes: HashMap<AxialCoord,f64>, attacker: Entity, defender: Entity, attack: &Attack) -> HashMap<AxialCoord, f64> {
    let target_pos = world_view.data::<PositionData>(defender).hex;
    hexes.into_iter().filter(|(k, v)| can_attack(world_view, attacker, defender, attack, Some(*k), Some(target_pos))).collect()
}

//pub fn does_event_trigger_counterattack(world_view: &WorldView, counterer: Entity, event: &GameEventWrapper<GameEvent>) -> bool {
//    if let GameEvent::Strike { attacker, defender, ref strike_result, .. } = event.event {
//        if event.is_ended() && counterer == defender {
//            let combat_data = world_view.data::<CombatData>(defender);
//            if combat_data.counters_per_event as u8 > strike_result.strike_number && combat_data.counters_remaining.cur_value() > 0 {
//                return true;
//            }
//        }
//    }
//    false
//}


pub fn closest_attack_location_with_cost(world_view: &WorldView, hexes : HashMap<AxialCoord,f64>, attacker: Entity, defender: Entity, attack: &Attack, nudge_towards: CartVec) -> Option<(AxialCoord, f64)> {
    let defender_pos = world_view.data::<PositionData>(defender).hex.as_cart_vec();
    let nudge_delta = (nudge_towards - defender_pos).normalize_s();

    // do the min by cost, adjusted ever so slightly by the actual hex in question, this is necessary to make this a stable selection, otherwise there might be many equidistant
    // locations to use
    possible_attack_locations_with_cost(world_view, hexes, attacker, defender, attack).into_iter()
        .min_by_key(|(k, v)| {
            let distance = *v;
            let stabilizer_epsilon = ((k.r as f32 * 0.000001) + (k.q as f32 * 0.0000013)) as f64;
            let angle_distance = (k.as_cart_vec() - defender_pos).normalize_s().0.dot(nudge_delta.0).acos().abs() as f64;

            if attack.range > 2 {
                r64(distance + angle_distance + stabilizer_epsilon)
            } else {
                r64(distance * 0.05 + angle_distance + stabilizer_epsilon)
            }
        })
}


pub fn compute_attack_breakdown(world: &World, world_view: &WorldView, attacker: Entity, defender: Entity, attack_ref: &AttackRef, attack_from: Option<AxialCoord>, ap_remaining: Option<i32>) -> AttackBreakdown {
    let mut attack_breakdown = AttackBreakdown::default();


    if let Some((attack, weapon)) = attack_ref.resolve_attack_and_weapon(world_view, attacker) {
        let attacker_data = world_view.character(attacker);
        let targets = targets_for_attack(world_view, attacker, attack_ref, defender, attack_from);

        let attack_from = attack_from.unwrap_or(attacker_data.position.hex);

        let counter_targets = AttackTargets { hexes : vec![attack_from], characters : vec![attacker] };
        let mut attacker_strikes = max_strikes_remaining(world_view, attacker, &attack, ap_remaining);

        let (defender_counter_attack, defender_weapon, mut defender_counters) =
            combat::counters_for(world_view, defender, attacker, &attack);

        let mut attacker_turn = true;
        while attacker_strikes > 0 || defender_counters > 0 {
            if attacker_turn && attacker_strikes > 0 {
                attack_breakdown.add_strike(compute_strike_breakdown(world, world_view, attacker, defender, &attack, weapon, &targets));
                attacker_strikes -= 1;
            } else if !attacker_turn && defender_counters > 0 {
                attack_breakdown.add_counter(compute_strike_breakdown(world, world_view, defender, attacker, &defender_counter_attack, defender_weapon, &counter_targets));
                defender_counters -= 1;
            }
            attacker_turn = !attacker_turn;
        }
    } else {
        warn!("Computing empty attack breakdown, attack reference was not valid");
    }

    attack_breakdown
}

pub fn max_strikes_remaining(world_view: &WorldView, attacker: Entity, attack: &Attack, ap_remaining : Option<i32>) -> i32 {

    let attacker_data = world_view.character(attacker);
    let ap_remaining = ap_remaining.unwrap_or(attacker_data.action_points.cur_value());
    let base_attacker_strikes = ap_remaining / attack.ap_cost as i32;
    match attack.attack_type {
        AttackType::Thrown => base_attacker_strikes.min(1),
        _ => base_attacker_strikes
    }
}

pub fn compute_strike_breakdown(world: &World, view: &WorldView, attacker_ref: Entity, primary_defender_ref: Entity, attack: &Attack, weapon: Entity, targets : &AttackTargets) -> StrikeBreakdown {
    let mut ret = StrikeBreakdown::default();

    ret.attack = attack.clone();
    ret.weapon = weapon;
    ret.damage_types.push(attack.primary_damage_type);
    if let Some(secondary_damage_type) = attack.secondary_damage_type {
        ret.damage_types.push(secondary_damage_type);
    }

    ret.ap_cost_components.add(attack.ap_cost as i32, "weapon ap cost");

    let attacker = view.character(attacker_ref);
    let attacker_combat = view.combat(attacker_ref);
    let attacker_combat_field_log = world.field_logs_for::<CombatData>(attacker_ref);
    let attacker_skills = view.skills(attacker_ref);

    for defender_ref in &targets.characters {
        let defender_ref = *defender_ref;
        let mut target_breakdown = StrikeTargetBreakdown::default();
        target_breakdown.target = defender_ref;

        let defender = view.character(defender_ref);
        let defender_combat = view.combat(defender_ref);
        let defender_skills = view.skills(defender_ref);

        let _attacker_tile: &TileData = view.tile(attacker.position.hex);
        let defender_tile: &TileData = view.tile(defender.position.hex);

        match attack.attack_type {
            AttackType::Melee | AttackType::Reach => {
                target_breakdown.to_hit_components.add_field(attacker_combat.melee_accuracy_bonus, &attacker_combat_field_log, &CombatData::melee_accuracy_bonus, "melee accuracy");
            }
            AttackType::Projectile | AttackType::Thrown => {
                target_breakdown.to_hit_components.add_field(attacker_combat.ranged_accuracy_bonus, &attacker_combat_field_log, &CombatData::ranged_accuracy_bonus, "ranged accuracy")
            }
        }
        target_breakdown.to_miss_components.add(defender_combat.defense_bonus, "defense");
        target_breakdown.to_miss_components.add(defender_combat.dodge_bonus, "dodge");
        target_breakdown.to_miss_components.add(defender_combat.block_bonus, "block");

        target_breakdown.to_hit_components.add(attack.to_hit_bonus, "weapon accuracy");

        target_breakdown.to_miss_components.add(defender_tile.cover as i32, "terrain defense");

    //    ret.dice_count_components.push((attack.damage_dice.count as i32, "weapon dice"));
    //    ret.die_components.push((attack.damage_dice.die as i32, "weapon die size"));
        target_breakdown.damage_dice_components.push((attack.damage_dice, "base weapon damage"));


        match attack.range {
            i if i <= 1 => target_breakdown.damage_bonus_components.add(attacker_combat.melee_damage_bonus, "base melee damage bonus"),
            _ => target_breakdown.damage_bonus_components.add(attacker_combat.ranged_damage_bonus, "base ranged damage bonus")
        }
        target_breakdown.damage_bonus_components.add(attack.damage_bonus, "weapon damage bonus");
        ret.per_target_breakdowns.push(target_breakdown);
    }

    ret
}

pub fn handle_attack(world: &mut World, attacker: Entity, defender_ref: Entity, attack_ref: &AttackRef) {
    if let Some(attack) = attack_ref.resolve(world.view(), attacker) {
        let world_view = world.view();

        let attack_breakdown = compute_attack_breakdown(world, world_view, attacker, defender_ref, attack_ref, None, None);

        world.start_event(GameEvent::Attack { attacker: attacker, defender: defender_ref });


        for strike_index in attack_breakdown.ordering {
            match strike_index {
                StrikeIndex::Strike(i) => handle_strike(world, attacker, defender_ref, &attack_breakdown.strikes[i], i as u8),
                StrikeIndex::Counter(i) => handle_strike(world, defender_ref, attacker, &attack_breakdown.counters[i], i as u8)
            }
        }

        let attack_skill_type = match attack.range {
            i if i <= 1 => Skill::Melee,
            _ => Skill::Ranged
        };
        world.modify(attacker, SkillData::skill_xp.add_to_key(attack_skill_type, 1), None);
        world.modify(attacker, CharacterData::stamina.reduce_by(Sext::of(1)), None);

        world.end_event(GameEvent::Attack { attacker: attacker, defender: defender_ref });

        if attack.attack_type == AttackType::Thrown {
            // thrown natural attacks don't cause you to lose an entity when they occur, but otherwise we want to un-equip the weapon
            // and put it in the world
            if let Some(weapon) = attack_ref.resolve_weapon(world, attacker) {
                if weapon != attacker {
                    item::unequip_item(world, weapon, attacker, true);
                    item::remove_item_from_inventory(world, weapon, attacker);

                    let defender_pos = world_view.data::<PositionData>(defender_ref).hex;
                    let attacker_pos = world_view.data::<PositionData>(attacker).hex;

                    let drop_pos = *defender_pos.neighbors().iter().min_by_key(|n| n.distance(&attacker_pos)).unwrap();
                    item::place_item_in_world(world, weapon, drop_pos);
                }
            } else { warn!("Thrown weapon, but could not identify an originating weapon from which the attack came") }
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

pub fn handle_strike(world: &mut World, attacker_ref: Entity, primary_defender: Entity, strike: &StrikeBreakdown, strike_number: u8) {
    let mut rng = world.random(13);

    let view = world.view();

    let weapon = strike.weapon;
    let attack = &strike.attack;

    let attacker = view.character(attacker_ref);
    let attacker_combat = view.combat(attacker_ref);
    let attacker_skills = view.skills(attacker_ref);

    // reduce the actions available to the attacker by the cost of the attack, regardless of how the attack goes
    world.add_modifier(attacker_ref, CharacterData::action_points.reduce_by(strike.ap_cost_total() as i32), "attack");


    let mut strike_results = HashMap::new();

    for target_breakdown in &strike.per_target_breakdowns {
        let defender_ref = target_breakdown.target;
        let defender = view.character(defender_ref);

        if !attacker.is_alive() || !defender.is_alive() {
            continue;
        }

        let to_miss_total = target_breakdown.to_miss_total();
        let to_hit_total = target_breakdown.to_hit_total();

        // base number needed to be hit with no modifiers one way or another. With a base value of 8, 85% of attacks will hit
        // given no modifiers one way or another. We probably want to shift that a bit, and give a noticeable bump in the early
        // levels of dodging/attacking such that unskilled commoners are pretty useless at attacking until they get a bit of experience
        // 62.5% will have at least a 10, so it's still a decent chance to hit
        let base_to_hit = 10;

        let to_hit_modifiers = to_hit_total - to_miss_total;

        let dice = DicePool::of(3, 6);
        let is_hit = dice.roll(&mut rng).total_result as i32 + to_hit_modifiers >= base_to_hit;
        if is_hit {
            let damage_dice = target_breakdown.damage_dice_total();
            let damage_total: i32 = damage_dice.map(|dd| dd.roll(&mut rng).total_result as i32).sum::<i32>()
                + target_breakdown.damage_bonus_total()
                - target_breakdown.damage_absorption_total();
            let damage_total: u32 = damage_total.as_u32_or_0();

            strike_results.insert(defender_ref, StrikeResult {
                damage_types: strike.damage_types.clone(),
                damage_done: damage_total as i32,
                hit: true,
                killing_blow: damage_total as i32 > defender.health.cur_value(),
                strike_number,
                weapon: if weapon == attacker_ref { None } else { Some(weapon) },
            });
        } else {
            strike_results.insert(defender_ref, StrikeResult {
                damage_types: Vec::new(),
                damage_done: 0,
                hit: false,
                killing_blow: false,
                strike_number,
                weapon: if weapon == attacker_ref { None } else { Some(weapon) },
            });
        }
    }

    let defenders = strike.per_target_breakdowns.map(|t| t.target);
    let strike_event = GameEvent::Strike {
        attacker : attacker_ref,
        attack : box attack.clone(),
        defenders,
        strike_results : strike_results.clone(),
    };
    world.start_event(strike_event.clone());

    for (target, strike_result) in &strike_results {
        if strike_result.hit {
            logic::character::apply_damage_to_character(world, *target, strike_result.damage_done as u32, &strike_result.damage_types);
        }
    }
    world.modify(attacker_ref, MovementData::moves.set_to(Sext::of(0)), None);

    world.end_event(strike_event);
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

pub fn counters_for(world_view: &WorldView, defender_ref: Entity, countering_ref: Entity, _countering_attack: &Attack) -> (Attack, Entity, u32) {
    let defender = world_view.character(defender_ref);
    let defender_combat = world_view.combat(defender_ref);
    let countering = world_view.character(countering_ref);

    // can't counter on ranged attacks
    if defender.position.hex.distance(&countering.position.hex) > 1.0 {
        (Attack::default(), defender_ref, 0)
    } else {
        let counter_attack = counter_attack_ref_to_use(world_view, defender_ref);
        if let Some(counter_attack) = counter_attack {
            if let Some((attack, weapon)) = counter_attack.resolve_attack_and_weapon(world_view, defender_ref) {
                if defender_combat.counters_remaining.cur_value() > 0 {
                    (attack.clone(), weapon, 1)
                } else {
                    (attack.clone(), weapon, 0)
                }
            } else {
                trace!("Not countering, attack did not resolve");
                (Attack::default(), defender_ref, 0)
            }
        } else {
            trace!("Not countering, no counter attack to use found");
            (Attack::default(), defender_ref, 0)
        }
    }
}

pub fn possible_attack_refs(world: &WorldView, attacker: Entity) -> Vec<AttackRef> {
    let combat = world.combat(attacker);
    let mut res = combat.natural_attacks.map(|a| AttackRef::new(*a, attacker));
    let equipment = world.equipment(attacker);
    for item_ref in &equipment.equipped {
        let item = world.item(*item_ref);
        res.extend(item.attacks.iter().map(|a| AttackRef::new(*a, *item_ref)));
    }
    for special_attack in &combat.special_attacks {
        // if the special attack is a raw attack, then the reference can just be to itself, and the derived_from is the attacker
        if world.has_data::<Attack>(*special_attack) {
            res.push(AttackRef::new(*special_attack, attacker))
        } else if let Some(derived) = world.data_opt::<DerivedAttackData>(*special_attack) {
            // it's a derived attack, so see what we can make of it based on its criteria
            if derived.character_condition.matches(world, attacker) {
                // if the character is valid, look at all of their equipment
                for equipped in &equipment.equipped {
                    // if any piece of equipment matches the weapon condition
                    if derived.weapon_condition.matches(world, *equipped) {
                        // look at its item data if any
                        if let Some(item) = world.data_opt::<ItemData>(*equipped) {
                            // and examine its attacks
                            for attack in &item.attacks {
                                // if any of them match the attack condition, create a new attack reference that links
                                // the derived attack -> underlying attack
                                if derived.attack_condition.matches(world, *attack) {
                                    res.push(AttackRef::new(*special_attack, *attack));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    res
}

pub fn character_has_access_to_attack(world: &WorldView, attacker: Entity, attack: Entity) -> bool {
    world.combat(attacker).natural_attacks.iter().any(|a| a == &attack) ||
        world.equipment(attacker).equipped.iter().any(|e| world.item(*e).attacks.contains(&attack)) ||
        world.combat(attacker).special_attacks.iter().any(|a| a == &attack)
}

pub fn possible_attacks(world: &WorldView, attacker: Entity) -> Vec<Attack> {
    possible_attack_refs(world, attacker).iter().flat_map(|ar| ar.resolve(world, attacker)).collect_vec()
}

pub fn default_attack(world: &WorldView, attacker: Entity) -> AttackRef {
    for item_ref in &world.equipment(attacker).equipped {
        let item = world.item(*item_ref);
        if let Some(attack) = item.attacks.first() {
            return AttackRef::new(*attack, *item_ref);
        }
    }
    world.combat(attacker).natural_attacks.first().map(|a| AttackRef::new(*a, attacker)).unwrap_or(AttackRef::none())
}

pub fn is_valid_counter_attack(world: &WorldView, counter_attacker: Entity, attack_ref: &AttackRef) -> bool {
    !attack_ref.is_derived_attack(world) && attack_ref.is_melee(world, counter_attacker)
}

pub fn counter_attack_ref_to_use(world: &WorldView, counter_attacker: Entity) -> Option<AttackRef> {
    let combat_data = world.data::<CombatData>(counter_attacker);
    combat_data.active_counterattack.as_option().cloned()
        .or_else(|| primary_attack_ref(world, counter_attacker)
            .filter(|par| is_valid_counter_attack(world, counter_attacker, par))
            .or_else(|| {
                let mut melee_attacks = possible_attack_refs(world, counter_attacker);
                melee_attacks.retain(|a| is_valid_counter_attack(world, counter_attacker, a));
                best_attack(world, counter_attacker, &melee_attacks)
            }))
}

pub fn primary_attack_ref(world: &WorldView, attacker: Entity) -> Option<AttackRef> {
    let combat_data = world.data::<CombatData>(attacker);
    combat_data.active_attack.as_option().cloned().or(default_attack(world, attacker).as_option().cloned())
}

pub fn primary_attack(world: &WorldView, attacker: Entity) -> Option<Attack> {
    primary_attack_ref(world, attacker).and_then(|r| r.resolve(world, attacker))
}

pub fn valid_attacks(world_view: &WorldView, attacker: Entity, attacks_references: &Vec<AttackRef>, defender: Entity) -> Vec<AttackRef> {
    let attacker_c = world_view.character(attacker);
    let defender_c = world_view.character(defender);

    let mut ret = vec![];
    for attack_ref in attacks_references {
        if let Some(attack) = attack_ref.resolve(world_view, attacker) {
            if within_range(world_view, attacker, defender, &attack, None, None) {
                ret.push(attack_ref.clone());
            }
        }
    }
    ret
}

pub(crate) mod intern {
    use super::*;

    pub(crate) fn weapon_attack_derives_from(world: &WorldView, attacker: Entity, attack: Entity) -> Option<Entity> {
        world.combat(attacker).natural_attacks.iter().find(|a| *a == &attack)
            .or(world.equipment(attacker).equipped.iter().find(|e| world.item(**e).attacks.contains(&attack)))
            .cloned()
    }
}

pub fn best_attack(view: &WorldView, attacker: Entity, attacks: &Vec<AttackRef>) -> Option<AttackRef> {
    attacks.iter().cloned().max_by_key(|ar| ar.resolve(view, attacker).map(|ra| r32(ra.damage_dice.avg_roll() / ra.ap_cost.max(1) as f32)).unwrap_or(r32(0.0)))
}

pub fn best_attack_against(world_view: &WorldView, attacker: Entity, attacks: &Vec<AttackRef>, defender: Entity) -> Option<AttackRef> {
    let attacker = world_view.character(attacker);
    let defender = world_view.character(defender);
    attacks.get(0).map(|x| x.clone())
}

/// when selecting between multiple equivalent paths, will choose the one that is closest to the nudge_towards parameter
pub fn path_to_attack(world_view: &WorldView, attacker: Entity, defender: Entity, attack_ref: &AttackRef, nudge_towards : CartVec) -> Option<(Vec<AxialCoord>, f64)> {
    if let Some(attack) = attack_ref.resolve(world_view, attacker) {
        if logic::combat::can_attack(world_view, attacker, defender, &attack, None, None) {
            Some((Vec::new(), 0.0))
        } else if let Some(movement_type) = logic::movement::default_movement_type(world_view, attacker) {
            let possible_moves = logic::movement::hexes_reachable_by_character_this_turn(world_view, attacker, movement_type);
            if let Some((attack_from, cost_to)) = logic::combat::closest_attack_location_with_cost(world_view, possible_moves, attacker, defender, &attack, nudge_towards) {
                logic::movement::path_to(world_view, attacker, attack_from)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        warn!("path_to_attack used with invalid attack reference");
        None
    }
}

pub struct AttackTargets {
    pub hexes : Vec<AxialCoord>,
    pub characters : Vec<Entity>
}
impl AttackTargets {
    pub fn none() -> AttackTargets {
        AttackTargets { hexes : Vec::new(), characters : Vec::new() }
    }
}

pub fn targets_for_attack(world: &WorldView, attacker: Entity, attack: &AttackRef, originating_on: Entity, attack_from : Option<AxialCoord>) -> AttackTargets {
    let attack_from = attack_from.unwrap_or_else(|| world.data::<PositionData>(attacker).hex);

    let hex = if let Some(tile) = world.data_opt::<TileData>(originating_on) {
        tile.position
    } else if let Some(pos) = world.data_opt::<PositionData>(originating_on) {
        pos.hex
    } else {
        return AttackTargets::none();
    };

    let attacker_hex : AxialCoord = attack_from;
    let attacker_hex_f : Vec3f = attacker_hex.as_cube_coord().as_v3f();
    let hex_f = hex.as_cube_coord().as_v3f();
    let delta : Vec3f = hex_f - attacker_hex_f;
    let delta_n = if delta.magnitude2() > 0.0 {
        delta.normalize()
    } else {
        delta
    };

    if let Some(attack) = attack.resolve(world, attacker) {
        let all_hexes = match attack.pattern {
            HexPattern::Single => vec![hex],
            HexPattern::Line(start_d, length) => {
                let start_f = hex_f + delta_n * start_d as f32;
                let start = CubeCoord::rounded(start_f.x, start_f.y, start_f.z);
                let end_f = start_f + delta_n * length as f32;
                let end = CubeCoord::rounded(end_f.x, end_f.y, end_f.z);

                let hexes = CubeCoord::hexes_between(start, end).map(|c| c.as_axial_coord());
                hexes
            },
            HexPattern::Arc(start_d, length) => {
                error!("Arc hex patterns not yet implemented");
                Vec::new()
            },
        };

        let mut characters = Vec::new();
        for hex in &all_hexes {
            if let Some(tile) = world.tile_opt(*hex) {
                if let Some(occupied) = tile.occupied_by {
                    characters.push(occupied);
                }
            }
        }

        AttackTargets { hexes : all_hexes, characters }
    } else {
        return AttackTargets::none();
    }
}

pub trait ResolveableAttackRef {
    fn resolve_attack_and_weapon(&self, world: &WorldView, character : Entity) -> Option<(Attack, Entity)>;
    fn resolve(&self, world: &WorldView, character : Entity) -> Option<Attack>;

    fn resolve_weapon(&self, world: &WorldView, character : Entity) -> Option<Entity>;
    fn is_melee(&self, world: &WorldView, character : Entity) -> bool;
}

impl ResolveableAttackRef for AttackRef {
    fn resolve_attack_and_weapon(&self, world: &WorldView, character : Entity) -> Option<(Attack, Entity)> {
        if self.is_none() {
            None
        } else {
            if logic::combat::character_has_access_to_attack(world, character, self.attack_entity) {
                if let Some(weapon) = self.resolve_weapon(world, character) {
                    if let Some(attack) = world.data_opt::<Attack>(self.attack_entity) {
                        Some((attack.clone(), weapon))
                    } else if let Some(derived_attack) = world.data_opt::<DerivedAttackData>(self.attack_entity) {
                        let underlying_attack = self.derived_from;
                        if let Some(weapon) = logic::combat::intern::weapon_attack_derives_from(world, character, underlying_attack) {
                            if let Some(new_attack) = derived_attack.kind.derive_special_attack(world, character, weapon, underlying_attack) {
                                Some((new_attack, weapon))
                            } else {
                                warn!("derived attack could not create actual new attack from the base attack it was given");
                                None
                            }
                        } else {
                            warn!("derived attack is derived from weapon that could not be identified on character");
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    warn!("Attack reference could not be resolved for lack of identifying the weapon it was derived from");
                    None
                }
            } else {
                info!("Character ({}) no longer has access to referenced attack ({})", world.signifier(character), world.signifier(self.attack_entity));
                None
            }
        }
    }

    fn resolve(&self, world: &WorldView, character : Entity) -> Option<Attack> {
        self.resolve_attack_and_weapon(world, character).map(|t| t.0)
    }

    fn resolve_weapon(&self, world: &WorldView, character : Entity) -> Option<Entity> {
        if world.has_data::<Attack>(self.attack_entity) {
            logic::combat::intern::weapon_attack_derives_from(world, character, self.attack_entity)
        } else if world.has_data::<DerivedAttackData>(self.attack_entity) {
            logic::combat::intern::weapon_attack_derives_from(world, character, self.derived_from)
        } else {
            None
        }
    }

    fn is_melee(&self, world: &WorldView, character : Entity) -> bool {
        self.resolve(world, character).map(|a| a.attack_type == AttackType::Melee).unwrap_or(false)
    }
}