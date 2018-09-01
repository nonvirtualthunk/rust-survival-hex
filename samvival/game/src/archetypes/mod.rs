pub mod weapons;

pub use archetypes::weapons::weapon_archetypes;

pub mod characters;

pub use archetypes::characters::character_archetypes;

use game::EntityBuilder;


use std::collections::HashMap;

pub struct ArchetypeLibrary {
    archetypes_by_name: HashMap<String, EntityBuilder>,
    default: EntityBuilder,
}

impl ArchetypeLibrary {
    pub fn with_name<S : ::std::borrow::Borrow<str>>(&self, name: S) -> &EntityBuilder {
        self.archetypes_by_name.get(name.borrow()).unwrap_or(&self.default)
    }
}