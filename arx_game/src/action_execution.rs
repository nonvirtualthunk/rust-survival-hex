use world::World;
use world::WorldView;
use world::Entity;
use entities::*;
use entities::Skill;
use events::*;
use common::hex::*;
use common::flood_search;
use rand::Rng;
use rand::SeedableRng;
use rand::StdRng;
use entities::Attack;
use noisy_float::prelude::*;
use noisy_float;
use core::Oct;

pub fn handle_attack(world : &mut World, attacker_ref : Entity, defender_ref : Entity, attack : &Attack) {
    let world_view = world.view();

    let attacker_data = world_view.character(attacker_ref);

    let mut attacker_strikes = attacker_data.action_points.cur_value() / attack.ap_cost as i32;
    let (defender_counter_attack, mut defender_counters) =
        combat::counters_for(world_view, defender_ref, attacker_ref, attack);

    let mut attacker_turn = true;
    while attacker_strikes > 0 || defender_counters > 0 {
        if attacker_turn && attacker_strikes > 0 {
            handle_strike(world, attacker_ref, defender_ref, attack);
            attacker_strikes -= 1;
        } else if ! attacker_turn && defender_counters > 0 {
            handle_strike(world, defender_ref, attacker_ref, &defender_counter_attack);
            defender_counters -= 1;
        }
        attacker_turn = ! attacker_turn;
    }

    let attack_skill_type = match attack.range {
        i if i <= 1 => Skill::Melee,
        _ => Skill::Ranged
    };
    modify(world, attacker_ref, SkillXPMod(attack_skill_type, 1));
    modify(world, attacker_ref, ReduceStaminaMod(Oct::of(1)));

    world.add_event(GameEvent::Attack { attacker : attacker_ref, defender : defender_ref });
}

pub fn handle_strike(world : &mut World, attacker_ref : Entity, defender_ref : Entity, attack : &Attack) {
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
    world.add_constant_modifier(attacker_ref, ReduceActionsMod(attack.ap_cost));

    let _attacker_tile : &TileData = view.tile(attacker.position);
    let defender_tile : &TileData = view.tile(defender.position);

    let (attack_skill_type, accuracy_bonus) = match attack.range {
        i if i <= 1 => (Skill::Melee, attacker_combat.melee_accuracy),
        _ => (Skill::Ranged, attacker_combat.ranged_accuracy)
    };

    let attack_skill = attacker_skills.skill_level(attack_skill_type);

    let dodge_skill = defender_skills.skill_level(Skill::Dodge);
    let dodge_total = combat::dodge_for_skill_level(dodge_skill) + defender_combat.dodge_chance;

    let accuracy_total = combat::accuracy_for_skill_level(attack_skill) + accuracy_bonus;

    // 0.0 indicates standard accuracy, no change, 1.0 indicates, effectively, always hits
    // unless there is some mitigating factor (dodge, cover, etc)
    let base_accuracy = attack.relative_accuracy;

    let hit_chance = base_accuracy +
        accuracy_total -
        dodge_total -
        defender_tile.cover;

    let is_hit = rng.gen_range(0.0,1.0) < hit_chance;
    if is_hit {
        let attack_result = attack.roll_damage(&mut rng);
        let damage_total = attack_result.damage_total;

        modify(world, defender_ref, DamageMod(damage_total as i32));
        modify(world, attacker_ref, EndMoveMod);
//        modify(world, attacker_ref, SkillXPMod(attack_skill_type, 1));

        let killing_blow = !view.data::<CharacterData>(defender_ref).is_alive();

        world.add_event(GameEvent::Strike {
            attacker : attacker_ref,
            defender : defender_ref,
            damage_done : damage_total,
            hit_chance,
            hit : true,
            killing_blow
        });
    } else {
        world.add_event(GameEvent::Strike {
            attacker : attacker_ref,
            defender : defender_ref,
            damage_done : 0,
            hit_chance,
            hit : false,
            killing_blow : false
        });
    }
}

pub mod combat {
    use super::*;

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
        if defender.position.distance(&countering.position) > 1.0 {
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

    pub fn valid_attacks<'a, 'b>(world_view: &'a WorldView, attacker : Entity, attacks: &'b Vec<Attack>, defender : Entity) -> Vec<&'b Attack> {
        let attacker = world_view.character(attacker);
        let defender = world_view.character(defender);
        let dist = attacker.position.distance(&defender.position).raw() as u32;

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
}


pub mod movement {
    use super::*;
    use std::collections::HashMap;
    use noisy_float::prelude::*;
    use noisy_float;
    use pathfinding::prelude::astar;

    pub struct MovementTarget {
        hex : AxialCoord,
        move_cost : Oct
    }


    pub fn move_cost(world_view : &WorldView, from : &AxialCoord, to : &AxialCoord) -> f64 {
        world_view.tile_opt(*to).map(|t| t.move_cost.as_f64()).unwrap_or(100000.0)
    }

    pub fn hexes_in_range(world_view : &WorldView, mover : Entity, range : Oct) -> HashMap<AxialCoord, f64> {
        let start_position = world_view.character(mover).position;
        flood_search(start_position, range.as_f64(), |from, to| move_cost(world_view, from, to), |&from| from.neighbors())
    }

    pub fn path_to(world_view: &WorldView, mover : Entity, to : AxialCoord) -> Option<(Vec<AxialCoord>, f64)> {
        let from = world_view.character(mover).position;
        astar(&from, |c| c.neighbors().into_iter().map(|c| (c, r32(move_cost(world_view, &c, &c) as f32))), |c| c.distance(&to), |c| *c == to)
            .map(|(vec, cost)| (vec, cost.raw() as f64))
    }


    pub fn hex_ap_cost(world : &WorldView, mover : Entity, hex : AxialCoord) -> u32 {
        let mover = world.character(mover);
        let hex_cost = world.tile(hex).move_cost;
        let mut moves = mover.moves;
        let mut ap_cost = 0;
        while moves < hex_cost {
            moves += mover.move_speed;
            ap_cost += 1;
        }
        ap_cost
    }

}


/*
    Computes the [0,1] multiplier for a given level. This is a logarithmic curve mixed with a
    linear curve, such that advancement is fastest at the beginning, going from 0.0->0.28 in the
    first two levels, from 0.28->0.5 by level 5, 0.5->0.75 by level 10, 0.75 -> 1.0 by level 20.
    (log[4.5](x+1) * 0.5 * 0.7) + ((x+1)/20) * 0.3 - 0.015
    the 0.7/0.3 is the weighting between log and linear, the sub 0.015 shifts down to 0 at lvl 0
*/
pub fn level_curve(level : u32) -> f64 {
    let x = level as f64 + 1.0;
    f64::log(x, 4.5) * 0.5 * 0.7 + (x/20.0) * 0.3 - 0.015
}

pub fn handle_move(world : &mut World, mover : Entity, path : &[AxialCoord]) {
    let start_pos = world.view().character(mover).position;
    let mut prev_hex = start_pos;
    for hex in path {
        let hex = *hex;
        if hex != start_pos {
            let hex_cost = world.view().tile(hex).move_cost;
            // how many ap must be changed to move points in order to enter the given hex
            let ap_required = movement::hex_ap_cost(world.view(), mover, hex);
            if ap_required as i32 <= world.view().character(mover).action_points.cur_value() {
                let moves_converted = world.view().character(mover).move_speed * ap_required;
                let net_moves_lost = hex_cost - moves_converted;
                modify(world, mover, ReduceActionsMod(ap_required));
                modify(world, mover, ReduceMoveMod(net_moves_lost));
                modify(world, mover, ChangePositionMod(hex));

                modify(world, mover, SkillXPMod(Skill::ForestSurvival, 1));
                // advance the event clock
                world.add_event(GameEvent::Move { character : mover, from : prev_hex, to : hex });

                prev_hex = hex;
            } else {
                break;
            }
        }
    }
}


pub fn equip_item(world: &mut World, character : Entity, item : Entity) {
    modify(world, character, EquipItemMod(item));
    modify(world, item, ItemHeldByMod(Some(character)));

    world.add_event(GameEvent::Equip { character, item });
}












// thoughts:

// one question is what do we want the range to be on skills, [0,20]? [0,9]?
// I think [0,20], but 20 is godlike and virtually unattainable, that would be someone naturally gifted
// with an appropriate background, who you keep with you, focused on the relevant job for the entire
// campaign. So [0,9] is the common range. In keeping with our focus gives increasing benefits idea though
// it ought to give ever increasing benefit in order to make it tempting... but we still assume that it
// takes 2x as much xp to get each subsequent level? Or d20 Style
// levels  1  2  3  4   5   6   7   8    9   10  11  12  13  14
// that's: 1, 2, 4, 8,  16, 32, 64, 128, 256
// vs    : 1, 3, 6, 10, 15, 21, 28, 36,  45, 55, 66, 78, 91, 105
// (note, (0.5 * level^2 - 0.5 * level) gives you the above advancement)
// I think that the d20 style leveling advancement makes more sense, it continues to get harder
// but it doesn't have that same exponential curve to infinity. Ok, so we've got a < x^2 difficulty
// to advance each level, but realistically I don't think we can go much more than linear improvement
// in outcomes or it becomes ridiculous, if you got an x^2 damage bonus it would get bonkers. One
// thing I would like to avoid, I think, is utterly rescaling later levels relative to early ones.
// The situations I find unsatisfying there are 1) when numbers get stupid large they become difficult
// to reason about usefully and feel unsatisfying. When you're dealing 1244423 damage, it just becomes
// noise, 2) when enemies scale up to meet you without changing substance it feels like you're on a
// treadmill and advancement is useless, if you get to the last level and there's a lvl 20 rabbit
// with 10x as much hp as a rabbit in the first level, it seems like, why bother leveling?
// So I can't just scale up enemies arbitrarily, we want the dynamic range to be important, but not
// insane. Currently we're looking at a logistic curve, or s curve or what have you, starts of
// exponentially improving up through about 7 at which point it's linear up through about 12 where
// it becomes more logarithmic. The raw curve is a bit too extreme, so we're going to mix it with
// a straight linear, such that we still get some progress up in the 17-20 range, though diminished
// and the advancement in the 1-4 range isn't so pronounced...actually the s curve is _not_ what
// we want here, that has slowest advancement at beginning and end, fastest in the middle. We
// probably do want a plain old logarithmic advancement.

/*
    Now here's a crazy one, not saying we should actually do it, but just a thought exercise. What
    if we didn't increase any of the scales or multipliers directly with level, but rather gave
    specific perks per-level. I.e. gaining a level of melee grants you one of n perks, like,
    +1 damage, or +1 to-hit, or +2 to-hit against enemies in this condition, etc. It's similar
    to the approach that that custom wesnoth campaign uses, I think. The goal though is to make
    each of the levels in some way meaningful, rather than just 15% better or whatever. The other
    way to go about that is through the class leveling system, so skills level up quickly-ish
    and grant continuously increasing gradient bonuses, but class levels are gained more infrequently
    and the class choices are determined by skills and offer more meaningful choices. If you level
    up ranged attack and forest survival you gain access to the ranger class, which grants more
    interesting perks.

    Or both, I suppose, that's the other possibility. But if we were going to go that route it would
    just be a lot of overhead, choosing a perk at every skill up, and choosing classes at every
    class level up, and so on. We could scale between by doing a continuous gradient improvement,
    with perk choices at intervals, i.e. odd levels, or 5,10,15 or whatever. So you get continuously
    a little better at everything, but you get to make distinct choices periodically. You could go
    into a weird meta-loop by also making perk choices dependent on other skills or classes, so it
    all interlinks together. You could also have the perks be hardcoded and not up for choice, so
    you get continuously a little better, then gain noteworthy perks at intervals, without the
    need for concrete choice. Leveling melee attack to 5 grants the ability to do a power attack,
    ranged to 3 gets you aimed shot, or dodge 5 gives you a passive chance to reflex save away from
    area of effect attacks. I feel like that's probably the best approach, otherwise the choices
    become overwhelming. You're choosing which skills to level (by usage or training), which directly
    determines which perks you get (since they're fixed), and determines the options when class
    leveling occurs, which then is a secondary choice.
*/