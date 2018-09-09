use common::prelude::*;
use game::prelude::*;
use anymap::AnyMap;
use game::Modifier;
use game::EntityData;
use game::ModifierReference;
use std::collections::HashMap;
use events::GameEvent;

#[derive(Clone,Serialize,Deserialize, Default,Debug, PartialEq, Eq)]
pub struct Effect {
    pub modifiers : Vec<ModifierReference>,
    pub description : Option<String>
}

static EMPTY_EFFECT : Effect = Effect {
    modifiers : Vec::new(),
    description: None
};

impl Effect {
    pub fn new<S : OptionalStringArg>(description : S) -> Effect {
        Effect { modifiers : Vec::new(), description : description.into_string_opt() }
    }
    pub fn add_modifier<T : EntityData>(&mut self, world: &mut World, modifier : Box<Modifier<T>>) {
        let mod_ref = world.register_modifier_archetype(modifier);
        self.modifiers.push(mod_ref);
    }

    pub fn with_modifier<T : EntityData>(mut self, world: &mut World, modifier : Box<Modifier<T>>) -> Self {
        let mod_ref = world.register_modifier_archetype(modifier);
        self.modifiers.push(mod_ref);
        self
    }

    pub fn apply(&self, world: &mut World, to_entity : Entity) -> EffectApplication {
        let mut ret = Vec::new();
        for modifier in &self.modifiers {
            ret.push(world.apply_modifier_archetype(to_entity, *modifier, self.description.clone()));
        }
        EffectApplication {
            applied_modifiers : ret
        }
    }
}


#[derive(Clone,Serialize,Deserialize,PartialEq,Debug,Default)]
pub struct EffectApplication {
    pub applied_modifiers : Vec<ModifierReference>
}
impl EffectApplication {
    pub fn disable(&self, world: &mut World, for_entity : Entity) {
        for modifier in &self.applied_modifiers{
            world.disable_modifier(*modifier);
        }
    }
}


#[derive(Clone,Serialize,Deserialize,Hash,PartialEq,Eq,Debug)]
pub enum EffectReference {
    Index(usize),
    Name(String),
}

impl EffectReference {
    pub fn resolve<'a, 'b>(&'a self, world: &'b WorldView) -> &'b Effect {
        let effects_data = world.world_data::<Effects>();

        let index = match self {
            EffectReference::Index(index) => Some(index),
            EffectReference::Name(name) => match effects_data.named_effects.get(name) {
                Some(index) => Some(index),
                None => {
                    error!("Attempted to resolve dangling effect reference {}, this should not be possible", name);
                    None
                }
            }
        };

        if let Some(index) = index {
            if let Some(effect) = effects_data.effects.get(*index) {
                effect
            } else {
                warn!("Effect reference could not be resolved, returning empty effect");
                &EMPTY_EFFECT
            }
        } else {
            warn!("Effect's index could not be determined, returning empty effect");
            &EMPTY_EFFECT
        }
    }
}

#[derive(Clone,Serialize,Deserialize,Debug,Default,Fields)]
pub struct Effects {
    pub(crate) effects : Vec<Effect>,
    pub(crate) named_effects : HashMap<String, usize>,
    pub(crate) applied_effects : HashMap<(Entity,EffectReference), EffectApplication>,
}
impl EntityData for Effects {}

impl Effects {
    pub fn init_effects(world: &mut World) {
        world.ensure_world_data::<Effects>();
    }

    pub fn register_effect(world: &mut World, effect : Effect) -> EffectReference {
        let effects = world.view().world_data::<Effects>();
        let index = effects.effects.len();
        world.modify_world(Effects::effects.append(effect), None);
        world.add_event(GameEvent::EffectRegistered);
        EffectReference::Index(index)
    }

    pub fn register_named_effect<S : Into<String>>(world: &mut World, name : S, effect : Effect) -> EffectReference {
        let effects = world.view().world_data::<Effects>();
        let index = effects.effects.len();
        world.modify_world(Effects::effects.append(effect), None);
        world.modify_world(Effects::named_effects.set_key_to(name.into(), index), None);
        world.add_event(GameEvent::EffectRegistered);
        EffectReference::Index(index)
    }
}