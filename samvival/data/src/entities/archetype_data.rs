use common::prelude::*;

use entities::{IdentityData, Attack, ToolData, AttributeData};
use game::prelude::*;
use game::EntityData;
use entities::selectors::EntitySelector;
use entities::item::Worth;
use entities::item::StackWith;

#[derive(Debug,Clone,Default,Serialize,Deserialize,Fields)]
pub struct EntityMetadata {
    pub archetype : Entity,
}
impl EntityData for EntityMetadata {}


#[derive(Debug,Clone,Serialize,Deserialize,Fields)]
pub struct ItemArchetype {
    pub attacks : Vec<(IdentityData, Attack)>,
    pub stack_limit : i32,
    pub stack_with : StackWith,
    pub worth : Worth,
    pub tool_data : Option<ToolData>,
    pub attributes : AttributeData,
}
impl EntityData for ItemArchetype {}


impl Default for ItemArchetype {
    fn default() -> Self {
        ItemArchetype {
            attacks : Vec::new(),
            stack_limit : 1,
            stack_with : StackWith::SameArchetype,
            worth : Worth::low(-1),
            tool_data : None,
            attributes : AttributeData::default(),
        }
    }
}