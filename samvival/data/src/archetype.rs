use common::prelude::*;
use game::Entity;

#[derive(Serialize,Deserialize,PartialEq,Debug,Clone)]
pub enum EntityArchetype {
    CopyEnitity(Entity),
    Weapon(String),
    Character(String),
    Sentinel
}
impl Default for EntityArchetype {
    fn default() -> Self {
        EntityArchetype::Sentinel
    }
}