use common::prelude::*;

#[derive(Serialize,Deserialize,PartialEq,Debug,Clone)]
pub enum EntityArchetype {
    CopyEnitity(Entity),
    Weapon(String),
    Character(String)
}