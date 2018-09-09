pub mod weapons;

pub use archetypes::weapons::weapon_archetypes;

pub mod characters;

pub use archetypes::characters::character_archetypes;

pub mod tiles;
//pub use archetypes::tiles::tile_archetypes;

use game::EntityBuilder;
use std::collections::HashMap;

mod tests;

pub struct ArchetypeLibrary<T=EntityBuilder> {
    archetypes_by_name: HashMap<String, T>,
    default: T,
}

impl <T> ArchetypeLibrary<T> {
    pub fn with_name<S : ::std::borrow::Borrow<str>>(&self, name: S) -> &T {
        self.archetypes_by_name.get(name.borrow()).unwrap_or(&self.default)
    }
}