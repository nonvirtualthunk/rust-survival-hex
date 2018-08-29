use common::prelude::*;
use prelude::*;
use anymap::AnyMap;
use game::Modifier;
use game::EntityData;

pub struct Effect {
    pub modifiers : AnyMap
}


impl Effect {
    pub fn add_modifier<T : EntityData, U : Modifier<T> + Clone>(modifier : U) {

    }
}