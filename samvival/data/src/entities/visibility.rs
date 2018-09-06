use common::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;
use game::prelude::*;
use game::EntityData;

#[derive(Debug,Clone,Default,Serialize, Deserialize, Fields)]
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

impl ::std::ops::Add<Visibility> for Visibility {
    type Output = Visibility;

    fn add(mut self, rhs: Visibility) -> Visibility {
        self.visible_hexes.extend(rhs.visible_hexes);
        self.revealed_hexes.extend(rhs.revealed_hexes);
        self
    }
}
impl ::std::ops::Sub<Visibility> for Visibility {
    type Output = Visibility;

    fn sub(mut self, rhs: Visibility) -> Visibility {
        self.visible_hexes.retain(|h| ! rhs.visible_hexes.contains(h));
        self.revealed_hexes.retain(|h| ! rhs.revealed_hexes.contains(h));
        self
    }
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