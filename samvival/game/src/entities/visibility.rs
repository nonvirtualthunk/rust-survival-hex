use common::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;
use prelude::*;
use game::EntityData;

#[derive(Debug,Clone,Default,PrintFields)]
pub struct VisibilityData {
    pub visibility_by_faction : HashMap<Entity, Visibility>
}
impl EntityData for VisibilityData {}


#[derive(Debug,Clone,Default)]
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