pub use reflect::*;
pub use events::GameEvent;
pub use entities::character::CharacterData;
pub use entities::character::CharacterStore;
pub use entities::Taxon;
pub use entities::taxonomy;
pub use entities::taxon;
pub use entities::IdentityData;

pub use game::prelude::*;
pub use samvival_core::*;

use std::borrow::Borrow;



pub trait LookupSignifier {
    fn signifier(&self, entity : Entity) -> String;
}

impl LookupSignifier for WorldView {
    fn signifier(&self, entity: Entity) -> String {
        if let Some(identity) = self.data_opt::<IdentityData>(entity) {
            identity.name.clone().unwrap_or_else(||String::from(identity.main_kind().name()))
        } else {
            format!("Entity({})", entity.0)
        }
    }
}