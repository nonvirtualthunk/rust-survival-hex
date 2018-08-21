use std::hash::Hash;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::hash::Hasher;
use common::prelude::*;
use prelude::*;
use logic;

use entities::CharacterData;
use entities::CombatData;
use entities::ModifierTrackingData;

pub enum EventTrigger {}

#[derive(Clone)]
pub struct ReactionType {
    //    pub react_func: fn(&mut World, Entity, &GameEventWrapper<GameEvent>) -> bool,
    pub icon: Str,
    pub name: Str,
    pub infinitive: Str,
    pub description: Str,
    pub rules_description: Str,
    pub condition_func: fn(&WorldView, Entity) -> bool,
    pub condition_description: Str,
    pub costs: Str,
    pub on_event: fn(&mut World, Entity, &GameEventWrapper<GameEvent>),
}

impl Default for ReactionType {
    fn default() -> Self {
        reaction_types::Defend.clone()
    }
}

impl PartialEq<ReactionType> for ReactionType {
    fn eq(&self, other: &ReactionType) -> bool {
        self.name == other.name
    }
}

impl Eq for ReactionType {}

impl Debug for ReactionType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "ReactionActionType({})", self.name)
    }
}

impl Hash for ReactionType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
    }
}

#[allow(non_upper_case_globals)]
pub mod reaction_types {
    use super::*;
    use entities::combat::AttackType;


    pub const Counterattack: ReactionType = ReactionType {
        icon: "ui/counterattack_icon",
        name: "counter attack",
        infinitive: "countering",
        description: "Forgo defense in favor of retribution and opportunistic strikes",
        rules_description: "Any time an enemy leaves themselves open to attack you make a strike at them. You can make as many counter strikes per turn as you could make \
        regular strikes with your active weapon, but you can only make one strike per event. The primary ways enemies leave themselves open to counter strikes are: attacking, \
        and moving into or out of a hex you threaten.",
        costs: "1 stamina for every counter strike made",
        condition_description: "Must have a melee attack to use",
        condition_func: |view, ent| logic::combat::possible_attacks(view, ent).any_match(|a| a.attack_type == AttackType::Melee) && view.data::<CharacterData>(ent).stamina.cur_value() > Sext::of(0),
        on_event: |world, ent, event| if let GameEvent::FactionTurn { faction, .. } = event.event {
            let view = world.view();
            let char_data = view.data::<CharacterData>(ent);
            let modifier_key = "counter-reaction";
            if faction == char_data.faction {
                if event.is_ended() {
                    if char_data.stamina.cur_value() > Sext::of(0) {
                        if let Some(counter_attack) = logic::combat::counter_attack_ref_to_use(view, ent) {
                            if let Some(counter_attack) = counter_attack.referenced_attack(view, ent) {
                                let increase_counters_by = view.data::<CharacterData>(ent).action_points.max_value() / counter_attack.ap_cost as i32;
                                let modifier = world.modify(ent, CombatData::counters_remaining.increase_by(increase_counters_by), "counterattack reaction");

                                world.modify(ent, ModifierTrackingData::modifiers_by_key.set_key_to(strf(modifier_key), modifier), None);
                                world.add_event(GameEvent::ReactionEffectApplied { entity : ent });
                            }
                        }
                    }
                } else if event.is_starting() {
                    if let Some(prev_modifier) = view.data::<ModifierTrackingData>(ent).modifiers_by_key.get(modifier_key) {
                        world.disable_modifier(prev_modifier.clone());
                        world.add_event(GameEvent::EffectEnded { entity : None });
                    }
                }
            }
        },
    };

    pub const Dodge: ReactionType = ReactionType {
        icon: "ui/dodge_icon",
        name: "dodge",
        infinitive: "dodging",
        description: "Focus on dodging and evading enemy attacks",
        rules_description: "Gain a +2 dodge bonus to avoid getting hit, or double your dodge bonus, whichever is higher",
        costs: "1 stamina for every 2 strikes against you",
        condition_description: "none",
        condition_func: |view, ent| view.data::<CharacterData>(ent).stamina.cur_value() > Sext::of(0),
        on_event: |world, ent, event| if let GameEvent::FactionTurn { faction, .. } = event.event {
            let view = world.view();
            let char_data = view.data::<CharacterData>(ent);
            let modifier_key = "dodge-reaction";
            if faction == char_data.faction {
                if event.is_ended() {
                    if char_data.stamina.cur_value() > Sext::of(0) {
                        let increase_dodge_by = (world.view().data::<CombatData>(ent).dodge_bonus * 2).max(2);
                        let modifier = world.modify(ent, CombatData::dodge_bonus.add(increase_dodge_by), "dodge reaction");

                        world.modify(ent, ModifierTrackingData::modifiers_by_key.set_key_to(strf(modifier_key), modifier), None);
                        world.add_event(GameEvent::ReactionEffectApplied { entity : ent });
                    }
                } else if event.is_starting() {
                    if let Some(prev_modifier) = view.data::<ModifierTrackingData>(ent).modifiers_by_key.get(modifier_key) {
                        world.disable_modifier(prev_modifier.clone());
                        world.add_event(GameEvent::EffectEnded { entity : None });
                    }
                }
            }
        },
    };


    pub const Block: ReactionType = ReactionType {
        icon: "ui/block_icon",
        name: "block",
        infinitive: "blocking",
        description: "Focus on blocking enemy attacks with a shield",
        rules_description: "Gain a +2 dodge bonus to avoid getting hit, or double your dodge bonus, whichever is better",
        costs: "1 stamina for every 2 strikes against you",
        condition_description: "none",
        condition_func: |view, ent| true,
        on_event: |world, ent, event| {},
    };


    pub const Defend: ReactionType = ReactionType {
        icon: "ui/defend_icon",
        name: "defend",
        infinitive: "defending",
        description: "Defend yourself normally, without expending too much effort",
        rules_description: "Gain a +1 bonus to defense.",
        costs: "none",
        condition_description: "none",
        condition_func: |view, ent| true,
        on_event: |world, ent, event| if let Some(GameEvent::FactionTurn { faction, .. }) = event.if_ended() {
            let view = world.view();
            let char_data = view.data::<CharacterData>(ent);
            if faction == &char_data.faction {
                let modifier = world.modify(ent, CombatData::defense_bonus.add(1), "defense reaction");
                world.modify(ent, ModifierTrackingData::modifiers_by_key.set_key_to(strf("defend-reaction"), modifier), None);
                world.add_event(GameEvent::ReactionEffectApplied { entity : ent });
                println!("end turn defense increase modifications applied");
            }
        } else if let Some(GameEvent::FactionTurn { faction, .. }) = event.if_ended() {
            let view = world.view();
            let char_data = view.data::<CharacterData>(ent);
            if faction == &char_data.faction {
                if let Some(prev_modifier) = view.data::<ModifierTrackingData>(ent).modifiers_by_key.get("defend-reaction") {
                    world.disable_modifier(prev_modifier.clone());
                    world.add_event(GameEvent::EffectEnded { entity : None });
                    println!("end turn defense increase modifications removed");
                }
            }
        },
    };
}