pub use reflect::*;
pub use data::events::GameEvent;
pub use data::entities::character::CharacterData;
pub use data::entities::character::CharacterStore;
pub use data::entities::Taxon;
pub use data::entities::taxonomy;
pub use data::entities::taxon;
pub use data::entities::IdentityData;

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


pub use logic::selection::SelectorMatches;
pub use logic::combat::ResolveableAttackRef;
pub use logic::movement::ResolveMovementType;