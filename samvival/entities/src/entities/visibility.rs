use common::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;
use prelude::*;
use game::EntityData;

#[derive(Debug,Clone,Default,Serialize, Deserialize, PrintFields)]
pub struct VisibilityData {
    pub visibility_by_faction : HashMap<Entity, Visibility>,
    empty_visibility : Visibility
}
impl EntityData for VisibilityData {}


#[derive(Debug,Clone,Default, Serialize, Deserialize)]
pub struct Visibility {
    pub visible_hexes : HashSet<AxialCoord>,
    pub revealed_hexes : HashSet<AxialCoord>,
}

impl Visibility {
    pub fn new() -> Visibility {
        Visibility {
            visible_hexes : HashSet::new(),
            revealed_hexes : HashSet::new()
        }
    }
}

impl VisibilityData {
    pub fn visibility_for(&self, faction : Entity) -> &Visibility {
        self.visibility_by_faction.get(&faction).unwrap_or(&self.empty_visibility)
    }
}