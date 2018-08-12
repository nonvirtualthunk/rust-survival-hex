use noisy_float::types::R32;
use common::hex::*;
use game::prelude::*;
use game::EntityData;
use game::ModifierReference;
use common::prelude::*;
use std::collections::HashSet;
use entities::actions::ActionType;
use entities::actions::Action;
use entities::reactions::ReactionType;
use std::collections::HashMap;

#[derive(Default, Clone, Debug, PrintFields)]
pub struct PositionData {
    pub hex: AxialCoord,
}
impl EntityData for PositionData {}

impl PositionData {
    pub fn distance(&self, other : &PositionData) -> R32 {
        self.hex.distance(&other.hex)
    }
}


#[derive(Clone, Debug, PrintFields)]
pub struct IdentityData {
    pub name : Option<String>,
    pub kind : Taxon
}
impl EntityData for IdentityData {}

impl IdentityData {
    pub fn new<S1 : Into<String>> (name : S1, kind : Taxon) -> IdentityData {
        IdentityData {
            name : Some(name.into()),
            kind
        }
    }

    pub fn of_kind(kind : Taxon) -> IdentityData {
        IdentityData {
            name : None,
            kind
        }
    }
}

impl Default for IdentityData {
    fn default() -> Self {
        IdentityData {
            name : None,
            kind : taxonomy::Unknown
        }
    }
}


#[derive(Clone, Debug, PrintFields, Default)]
pub struct ActionData {
    pub active_action : Option<Action>,
    pub active_reaction: ReactionType,
    pub available_action_types : HashSet<ActionType>
}
impl EntityData for ActionData {}

//impl Default for ActionData {
//
//}


#[derive(Clone, Copy, Debug)]
pub struct Taxon {
    pub name : Str,
    pub parent : Option<&'static Taxon>
}

pub const fn taxon(name : Str, parent : &'static Taxon) -> Taxon {
    Taxon { name, parent : Some(parent) }
}

pub mod taxonomy {
    use super::Taxon;
    use super::taxon;

    pub const Unknown : Taxon = Taxon { name : "unknown thing", parent : None };

    pub const Item : Taxon = Taxon { name : "item", parent : None };

    pub const Weapon : Taxon = taxon("weapon", &Item);

    pub const Sword : Taxon = taxon("sword", &Weapon);
    pub const Bow : Taxon = taxon("bow", &Weapon);
    pub const Spear : Taxon = taxon("spear", &Weapon);


    pub const Armor : Taxon = taxon("armor", &Item);
    // --------------- armors -------------------------
    pub const PlateArmor : Taxon = taxon("plate armor", &Armor);
    pub const LeatherArmor : Taxon = taxon("leather armor", &Armor);

    pub const Shield : Taxon = taxon("shield", &Armor);
    // --------------- shields -----------------------
    pub const LightShield : Taxon = taxon("light shield", &Shield);
    pub const HeavyShield : Taxon = taxon("heavy shield", &Shield);
    pub const TowerShield : Taxon = taxon("tower shield", &Shield);


    pub const Creature : Taxon = Taxon { name : "creature", parent : None };

    pub const Person : Taxon = taxon("person", &Creature);
    pub const Monster : Taxon = taxon("monster", &Creature);
    pub const Animal : Taxon = taxon("animal", &Creature);
}


#[derive(Clone, Debug, PrintFields, Default)]
pub struct ModifierTrackingData {
    pub modifiers_by_key : HashMap<String, ModifierReference>
}
impl EntityData for ModifierTrackingData {

}