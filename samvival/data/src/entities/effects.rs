use common::prelude::*;
use game::prelude::*;
use anymap::AnyMap;
use game::Modifier;
use game::EntityData;
use game::ModifierReference;

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
    pub fn new(description : Option<String>) -> Effect {
        Effect { modifiers : Vec::new(), description }
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

    pub fn apply(&self, world: &mut World, to_entity : Entity) {
        for modifier in &self.modifiers {
            world.apply_modifier_archetype(to_entity, *modifier, self.description.clone());
        }
    }
}

#[derive(Clone,Copy,Serialize,Deserialize,Hash,PartialEq,Eq,Debug)]
pub struct EffectReference(usize);

impl EffectReference {
    pub fn resolve<'a, 'b>(&'a self, world: &'b WorldView) -> &'b Effect {
        let effects_data = world.world_data::<Effects>();
        if let Some(effect) = effects_data.effects.get(self.0) {
            effect
        } else {
            warn!("Effect reference could not be resolved, returning empty effect");
            &EMPTY_EFFECT
        }
    }
}

#[derive(Clone,Serialize,Deserialize,Debug,Default,Fields)]
pub struct Effects {
    pub(crate) effects : Vec<Effect>
}
impl EntityData for Effects {}

impl Effects {
    pub fn register_effect(world: &mut World, effect : Effect) -> EffectReference {
        let effects = world.view().world_data::<Effects>();
        let index = effects.effects.len();
        world.modify_world(Effects::effects.append(effect), None);
        EffectReference(index)
    }
}