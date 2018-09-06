use std::hash::Hash;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::hash::Hasher;
use common::prelude::*;
use game::prelude::*;
use GameEvent;

use entities::EntitySelector;
use entities::character::CharacterData;
use entities::combat::CombatData;
use entities::common_entities::ModifierTrackingData;
use entities::taxonomy;


pub enum EventTrigger {}

#[derive(Clone,Copy,Hash,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub enum ReactionTypeRef {
    Counterattack,
    Dodge,
    Block,
    Defend
}
impl ReactionTypeRef {
    pub fn resolve(&self) -> &'static ReactionType {
        match self {
            ReactionTypeRef::Counterattack => &reaction_types::Counterattack,
            ReactionTypeRef::Dodge => &reaction_types::Dodge,
            ReactionTypeRef::Block => &reaction_types::Block,
            ReactionTypeRef::Defend => &reaction_types::Defend,
        }
    }
}
impl Default for ReactionTypeRef {
    fn default() -> Self {
        ReactionTypeRef::Defend
    }
}

impl Into<&'static ReactionType> for ReactionTypeRef {
    fn into(self) -> &'static ReactionType {
       self.resolve()
    }
}

#[derive(Clone)]
pub struct ReactionType {
    //    pub react_func: fn(&mut World, Entity, &GameEventWrapper<GameEvent>) -> bool,
    pub icon: Str,
    pub name: Str,
    pub infinitive: Str,
    pub description: Str,
    pub rules_description: Str,
    pub condition_description: Str,
    pub costs: Str,
    pub condition: fn() -> EntitySelector
//    pub condition_func: fn(&WorldView, Entity) -> bool,
//    pub on_event: fn(&mut World, Entity, &GameEventWrapper<GameEvent>),
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
    use entities::character::AllegianceData;

    /// takes care of the boilerplate of turn based reaction
    fn reaction_modifier_boilerplate(world : &mut World, ent : Entity, event : &GameEventWrapper<GameEvent>, modifier_key : Str) -> bool {
        if let GameEvent::FactionTurn { faction, .. } = event.event {
            let view = world.view();
            let allegiance = view.data::<AllegianceData>(ent);

            if faction == allegiance.faction {
                let char_data = view.data::<CharacterData>(ent);
                if event.is_starting() {
                    if let Some(prev_modifier) = view.data::<ModifierTrackingData>(ent).modifiers_by_key.get(modifier_key) {
                        world.disable_modifier(prev_modifier.clone());
                        world.add_event(GameEvent::EffectEnded { entity : None });
                    }
                } else if event.is_ended() {
                    return true;
                }
            }
        }
        false
    }

    pub static Counterattack: ReactionType = ReactionType {
        icon: "ui/counterattack_icon",
        name: "counter attack",
        infinitive: "countering",
        description: "Forgo defense in favor of retribution and opportunistic strikes",
        rules_description: "Any time an enemy leaves themselves open to attack you make a strike at them. You can make as many counter strikes per turn as you could make \
        regular strikes with your active weapon, but you can only make one strike per event. The primary ways enemies leave themselves open to counter strikes are: attacking, \
        and moving into or out of a hex you threaten.",
        costs: "1 stamina for every counter strike made",
        condition_description: "Must have a melee attack to use",
        condition: || EntitySelector::HasStamina(Sext::of_int(1)).and(EntitySelector::has_attack_kind(&taxonomy::attacks::MeleeAttack))
    };

    pub static Dodge: ReactionType = ReactionType {
        icon: "ui/dodge_icon",
        name: "dodge",
        infinitive: "dodging",
        description: "Focus on dodging and evading enemy attacks",
        rules_description: "Gain a +2 dodge bonus to avoid getting hit, or double your dodge bonus, whichever is higher",
        costs: "1 stamina for every 2 strikes against you",
        condition_description: "none",
        condition: || EntitySelector::HasStamina(Sext::of_int(1)),
    };


    pub static Block: ReactionType = ReactionType {
        icon: "ui/block_icon",
        name: "block",
        infinitive: "blocking",
        description: "Focus on blocking enemy attacks with a shield",
        rules_description: "Gain a +2 dodge bonus to avoid getting hit, or double your dodge bonus, whichever is better",
        costs: "1 stamina for every 2 strikes against you",
        condition_description: "none",
        condition: || EntitySelector::Any,
    };


    pub static Defend: ReactionType = ReactionType {
        icon: "ui/defend_icon",
        name: "defend",
        infinitive: "defending",
        description: "Defend yourself normally, without expending too much effort",
        rules_description: "Gain a +1 bonus to defense.",
        costs: "none",
        condition_description: "none",
        condition: || EntitySelector::Any,
    };
}