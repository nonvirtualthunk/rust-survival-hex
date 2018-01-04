use world::World;
use world::WorldView;
use entities::*;
use entities::Skill;
use events::*;
use common::hex::*;
use rand::Rng;
use rand::SeedableRng;
use rand::StdRng;
use entities::Attack;
use noisy_float::prelude::*;
use noisy_float;


pub fn handle_attack(world : &mut World, attacker_ref : CharacterRef, defender_ref : CharacterRef, attack : &Attack) {
    let world_view = world.view();

    // reduce the actions available to the attacker by 1, regardless of how the attack goes
    modify_character(world, attacker_ref, move |c| c.actions.reduce_by(1));

    let mut attacker_strikes = (attack.speed as u32).max(1);
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
}

pub fn handle_strike(world : &mut World, attacker_ref : CharacterRef, defender_ref : CharacterRef, attack : &Attack) {
    let seed : &[_] = &[world.event_clock as usize,13 as usize];
    let mut rng : StdRng = SeedableRng::from_seed(seed);

    let attacker = world.character(attacker_ref);
    let defender = world.character(defender_ref);

    if ! attacker.is_alive() || ! defender.is_alive() {
        return;
    }

    let _attacker_tile = world.tile(&attacker.position);
    let defender_tile = world.tile(&defender.position);

    let (attack_skill_type, accuracy_bonus) = match attack.range {
        i if i <= 1 => (Skill::Melee, attacker.melee_accuracy),
        _ => (Skill::Ranged, attacker.ranged_accuracy)
    };

    let attack_skill = attacker.skill_level(attack_skill_type);

    let dodge_skill = defender.skill_level(Skill::Dodge);
    let dodge_total = combat::dodge_for_skill_level(dodge_skill) + defender.dodge_chance;

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

        modify_character(world, defender_ref, move |c| c.health.reduce_by(damage_total as i32));
        modify_character(world, attacker_ref, move |c| c.moves.reduce_to(0.0));
        modify_character(world, attacker_ref, move |c| c.skill_xp_up(attack_skill_type, 1));

        let killing_blow = !world.character(defender_ref).is_alive();

        world.add_event(GameEvent::Attack {
            attacker : attacker_ref,
            defender : defender_ref,
            damage_done : damage_total,
            hit_chance,
            hit : true,
            killing_blow
        });
    } else {
        world.add_event(GameEvent::Attack {
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

    pub fn counters_for(world_view : &WorldView, defender_ref : CharacterRef, countering_ref : CharacterRef, _countering_attack : &Attack) -> (Attack, u32) {
        let defender = world_view.character(defender_ref);
        let countering = world_view.character(countering_ref);

        // can't counter on ranged attacks
        if defender.position.distance(&countering.position) > 1.0 {
            (Attack::default(), 0)
        } else {
            let possible_attacks = defender.possible_attacks(world_view);
            let possible_attacks = valid_attacks(world_view, defender_ref, &possible_attacks, countering_ref);
            let attack_to_use = best_attack(world_view, defender_ref, &possible_attacks, countering_ref);
            if let Some(attack) = attack_to_use {
                if defender.counters.cur_value() > 0 {
                    (attack.clone(), 1)
                } else {
                    (attack.clone(), 0)
                }
            } else {
                (Attack::default(), 0)
            }
        }
    }

    pub fn valid_attacks<'a, 'b>(world_view: &'a WorldView, attacker : CharacterRef, attacks: &'b Vec<Attack>, defender : CharacterRef) -> Vec<&'b Attack> {
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

    pub fn best_attack<'a, 'b>(world_view: &'a WorldView, attacker : CharacterRef, attacks: &'b Vec<&'b Attack>, defender : CharacterRef) -> Option<&'b Attack> {
        let attacker = world_view.character(attacker);
        let defender = world_view.character(defender);
        attacks.get(0).map(|x| *x)
    }
}

pub mod skills {
    pub fn xp_required_for_level(lvl : u32) -> u32 {
        let lvl = lvl as f64;
        ((0.5 * lvl.powf(2.0) - 0.5 * lvl) * 10.0) as u32
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

pub fn handle_move(world : &mut World, mover : CharacterRef, path : &[AxialCoord]) {
    let start_pos = world.character(mover).position;
    let mut prev_hex = start_pos;
    for hex in path {
        let hex = *hex;
        if hex != start_pos {
            let hex_cost = world.tile(&hex).move_cost as f64;
            if hex_cost <= world.character(mover).moves.cur_value() {
                modify_character(world, mover, move |c| c.moves.reduce_by(hex_cost));
                modify_character(world, mover, move |c| c.position = hex);

                // advance the event clock
                world.add_event(GameEvent::Move { character : mover, from : prev_hex, to : hex });

                prev_hex = hex;
            } else {
                break;
            }
        }
    }
}


pub fn equip_item(world: &mut World, character : CharacterRef, item : ItemRef) {
    modify_character(world, character, move |c| c.equipped.push(item));
    modify_item(world, item, move |i| i.held_by = Some(character));

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