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
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::hash::Hash;
use std::hash::Hasher;
use serde::Serialize;
use serde::Serializer;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::Visitor;
use std::fmt::Formatter;
use std::fmt;
use std::error::Error;
use entities::reactions::ReactionTypeRef;
use serde::de::EnumAccess;
use serde::de::SeqAccess;
use serde::ser::SerializeTuple;

#[derive(Default, Clone, Debug, Serialize, Deserialize, PrintFields)]
pub struct PositionData {
    pub hex: AxialCoord,
}
impl EntityData for PositionData {}

impl PositionData {
    pub fn distance(&self, other : &PositionData) -> R32 {
        self.hex.distance(&other.hex)
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, PrintFields)]
pub struct IdentityData {
    pub name : Option<String>,
    pub kinds : Vec<Taxon>
}
impl EntityData for IdentityData {}

impl IdentityData {
    pub fn new<S1 : Into<String>, T : Into<Taxon>> (name : S1, kind : T) -> IdentityData {
        IdentityData {
            name : Some(name.into()),
            kinds : vec![kind.into()]
        }
    }

    pub fn of_kind<T : Into<Taxon>>(kind : T) -> IdentityData {
        IdentityData::of_kinds(vec![kind])
    }

    pub fn of_kinds<T : Into<Taxon>>(kinds : Vec<T>) -> IdentityData {
        IdentityData {
            name : None,
            kinds : kinds.into_iter().map(|k| k.into()).collect_vec()
        }
    }

    pub fn of_name_and_kinds<S1 : Into<String>, T : Into<Taxon>> (name : S1, kinds : Vec<T>) -> IdentityData {
        IdentityData {
            name : Some(name.into()),
            kinds : kinds.into_iter().map(|k| k.into()).collect_vec()
        }
    }

    pub fn effective_name(&self) -> &str {
        self.name.as_ref().map(|n| &**n).unwrap_or_else(|| self.main_kind().name())
    }

    pub fn main_kind(&self) -> &Taxon {
        self.kinds.first().unwrap_or(&taxonomy::Unknown)
    }
}

impl Default for IdentityData {
    fn default() -> Self {
        IdentityData {
            name : None,
            kinds : vec![taxonomy::Unknown.clone()],
        }
    }
}


#[derive(Clone, Debug, Default, Serialize, Deserialize, PrintFields)]
pub struct ActionData {
    // SERIALIZATION PASS, re-enable actions
//    pub active_action : Option<Action>,
    pub active_reaction: ReactionTypeRef,
//    pub available_action_types : HashSet<ActionType>
}
impl EntityData for ActionData {}

//impl Default for ActionData {
//
//}



#[derive(Clone, Debug, Eq)]
pub enum Taxon {
    ConstTaxon { name : Str, parents : [Option<&'static Taxon>;4] },
    RuntimeTaxon { name : Arc<str>, parents : [Option<&'static Taxon>;4] },
    ConstTaxonRef { reference : &'static Taxon },
}

impl PartialEq<Taxon> for Taxon {
    fn eq(&self, other: & Taxon) -> bool {
        self.name() == other.name()
    }
}
impl Hash for Taxon {
    fn hash<H: Hasher>(&self, state: & mut H) {
        state.write(self.name().as_bytes())
    }
}

impl From<&'static Taxon> for Taxon {
    fn from(a: &'static Taxon) -> Self {
        Taxon::ConstTaxonRef { reference : a }
    }
}

//pub struct Taxon {
//    pub name : Arc<str>,
//    pub parents : [Option<&'static Taxon>;4],
//}
//pub type Taxon = &'static Taxon;

impl Taxon {
    pub fn name(&self) -> &str {
        match self {
            Taxon::ConstTaxon { name, .. } => name,
            Taxon::RuntimeTaxon { name, .. } => name,
            Taxon::ConstTaxonRef { reference, .. } => reference.name(),
        }
    }

    pub fn is_a(&self, other : &Taxon) -> bool {
        if self.name() == other.name() {
            true
        } else {
            self.parents().any_match(|p| p.is_a(other))
        }
    }

    pub fn parents(&self) -> Vec<&Taxon> {
        if let Taxon::ConstTaxonRef { reference, .. } = self {
            return reference.parents();
        }

        let mut ret = Vec::new();
        let raw_parents = match self {
            Taxon::ConstTaxon { parents , .. } => parents,
            Taxon::RuntimeTaxon { parents, .. } => parents,
            Taxon::ConstTaxonRef { .. } => panic!("this is unreachable")
        };
        for parent in raw_parents {
            if let Some(parent) = parent {
                ret.push(*parent);
            }
        }
        ret
    }
}


impl Serialize for Taxon {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
        match self {
            Taxon::ConstTaxon { name, .. } => {
                let empty_parents : Vec<&str> = Vec::new();

                let mut tuple = serializer.serialize_tuple(2)?;
                tuple.serialize_element(name)?;
                tuple.serialize_element(&empty_parents)?;
                tuple.end()
            },
            Taxon::RuntimeTaxon { name, parents } => {
                let mut tuple = serializer.serialize_tuple(2)?;
                tuple.serialize_element(name)?;
                let parent_vec = parents.iter().flat_map(|p| p.map(|i| i.name())).collect_vec();
                tuple.serialize_element(&parent_vec)?;
                tuple.end()
            },
            Taxon::ConstTaxonRef { reference } => {
                let empty_parents : Vec<&str> = Vec::new();

                let mut tuple = serializer.serialize_tuple(2)?;
                tuple.serialize_element(reference.name())?;
                tuple.serialize_element(&empty_parents)?;
                tuple.end()
            }
        }
    }
}
impl <'de> Deserialize<'de> for Taxon {

    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        struct TaxonVisitor;
        impl <'de> Visitor<'de> for TaxonVisitor {
            type Value = Taxon;

            fn expecting(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
                write!(formatter, "A static taxon")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<<Self as Visitor<'de>>::Value, <A as SeqAccess<'de>>::Error> where A: SeqAccess<'de>, {
                let name : String = seq.next_element()?.ok_or_else(||panic!("deserialize taxon name failed"))?;
                let parent_names : Vec<String> = seq.next_element()?.ok_or_else(||panic!("deserialize taxon parents failed"))?;
                if let Some(taxon_ref) = taxonomy::taxon_by_name_opt(&name) {
                    Ok(Taxon::ConstTaxonRef { reference : taxon_ref } )
                } else {
                    let mut parents : [Option<&'static Taxon>;4] = [None,None,None,None];
                    let mut parent_count = 0;
                    for parent_name in parent_names.iter() {
                        if let Some(parent_ref) = taxonomy::taxon_by_name_opt(parent_name) {
                            parents[parent_count] = Some(parent_ref);
                            parent_count += 1;
                        }
                    }
                    Ok(Taxon::RuntimeTaxon { name : intern_string(&name), parents })
                }
            }
        }
        deserializer.deserialize_tuple(2, TaxonVisitor)
    }
}

//static TAXON_VARIANTS : [Str;3] = ["ConstTaxon","RuntimeTaxon","ConstTaxonRef"];
//impl <'de> Deserialize<'de> for Taxon {
//
//    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
//        struct TaxonVisitor;
//        impl <'de> Visitor<'de> for TaxonVisitor {
//            type Value = &'static Taxon;
//
//            fn expecting(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
//                write!(formatter, "A static taxon")
//            }
//
//            fn visit_str<E>(self, v: &str) -> Result<<Self as Visitor<'de>>::Value, E> where E: Error, {
//                Ok(taxonomy::taxon_by_name(v))
//            }
//
//            fn visit_string<E>(self, v: String) -> Result<<Self as Visitor<'de>>::Value, E> where E: Error, {
//                Ok(taxonomy::taxon_by_name(v.as_str()))
//            }
//
//            fn visit_enum<A>(self, data: A) -> Result<<Self as Visitor<'de>>::Value, <A as EnumAccess<'de>>::Error> where A: EnumAccess<'de>, {
//                data.variant()
//            }
//        }
//        deserializer.deserialize_enum("Taxon",&TAXON_VARIANTS,TaxonVisitor)
//    }
//}

//impl Serialize for Taxon {
//    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
//        match self {
//            Taxon::ConstTaxon { name, .. } => {
//                let variant_serializer = serializer.serialize_struct_variant("Taxon",0,"ConstTaxon",1)?;
//                variant_serializer.serialize_field("name", name)?;
//                variant_serializer.end()
//            },
//            Taxon::RuntimeTaxon { name, parents } => {
//                let variant_serializer = serializer.serialize_struct_variant("Taxon",1,"RuntimeTaxon",1)?;
//                variant_serializer.serialize_field("name", name)?;
//                variant_serializer.serialize_field("parents", parents)?;
//                variant_serializer.end()
//            },
//            Taxon::ConstTaxonRef { name, .. } => {
//                let variant_serializer = serializer.serialize_struct_variant("Taxon",2,"ConstTaxonRef",1)?;
//                variant_serializer.serialize_field("name", name)?;
//                variant_serializer.end()
//            }
//        }
//    }
//}
//
//static TAXON_VARIANTS : [Str;3] = ["ConstTaxon","RuntimeTaxon","ConstTaxonRef"];
//impl <'de> Deserialize<'de> for Taxon {
//
//    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
//        struct TaxonVisitor;
//        impl <'de> Visitor<'de> for TaxonVisitor {
//            type Value = &'static Taxon;
//
//            fn expecting(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
//                write!(formatter, "A static taxon")
//            }
//
//            fn visit_str<E>(self, v: &str) -> Result<<Self as Visitor<'de>>::Value, E> where E: Error, {
//                Ok(taxonomy::taxon_by_name(v))
//            }
//
//            fn visit_string<E>(self, v: String) -> Result<<Self as Visitor<'de>>::Value, E> where E: Error, {
//                Ok(taxonomy::taxon_by_name(v.as_str()))
//            }
//
//            fn visit_enum<A>(self, data: A) -> Result<<Self as Visitor<'de>>::Value, <A as EnumAccess<'de>>::Error> where A: EnumAccess<'de>, {
//                data.variant()
//            }
//        }
//        deserializer.deserialize_enum("Taxon",&TAXON_VARIANTS,TaxonVisitor)
//    }
//}


//impl Serialize for &'static Taxon {
//    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
//        serializer.serialize_str(self.name())
//    }
//}
//impl <'de> Deserialize<'de> for &'static Taxon {
//    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
//        struct TaxonVisitor;
//        impl <'de> Visitor<'de> for TaxonVisitor {
//            type Value = &'static Taxon;
//
//            fn expecting(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
//                write!(formatter, "A static taxon")
//            }
//
//            fn visit_str<E>(self, v: &str) -> Result<<Self as Visitor<'de>>::Value, E> where E: Error, {
//                Ok(taxonomy::taxon_by_name(v))
//            }
//
//            fn visit_string<E>(self, v: String) -> Result<<Self as Visitor<'de>>::Value, E> where E: Error, {
//                Ok(taxonomy::taxon_by_name(v.as_str()))
//            }
//        }
//        deserializer.deserialize_str(TaxonVisitor)
//    }
//}

lazy_static! {
    static ref RUNTIME_TAXON_STRS : Mutex<HashSet<Arc<str>>> = Mutex::new(HashSet::new());
}

fn intern_string(string : &str) -> Arc<str> {
    let mut strings = RUNTIME_TAXON_STRS.lock().unwrap();
    if let Some(existing) = strings.get(string) {
        return existing.clone()
    }

    let new_arc : Arc<str> = string.into();
    strings.insert(new_arc.clone());
    new_arc
}

pub fn taxon_vec<T : Into<Taxon>>(vec : Vec<T>) -> Vec<Taxon> {
    vec.into_iter().map(|v| v.into()).collect_vec()
}
pub fn taxon(name : Str, parent : &'static Taxon) -> Taxon {
    Taxon::RuntimeTaxon { name : intern_string(name), parents : [Some(parent),None,None,None] }
}
pub fn taxon2(name : Str, parent1 : &'static Taxon, parent2 : &'static Taxon) -> Taxon {
    Taxon::RuntimeTaxon { name : intern_string(name), parents : [Some(parent1), Some(parent2), None, None] }}


pub mod taxonomy {
    pub const fn root_taxon(name : Str) -> Taxon {
        Taxon::ConstTaxon { name, parents : [None,None,None,None] }
    }
    pub const fn taxon(name : Str, parent : &'static Taxon) -> Taxon {
        Taxon::ConstTaxon { name, parents : [Some(parent),None,None,None] }
    }
    pub const fn taxon2(name : Str, parent1 : &'static Taxon, parent2 : &'static Taxon) -> Taxon {
        Taxon::ConstTaxon { name, parents : [Some(parent1), Some(parent2), None, None] }}

    use super::Taxon;
    use std::sync::Mutex;
    use std::collections::HashMap;
    use common::prelude::Str;
    use super::Rc;
    use super::Arc;

    pub static Unknown : Taxon = root_taxon("unknown thing");

    pub static Item : Taxon = root_taxon("item");

    pub static Weapon : Taxon = taxon("weapon", &Item);
    pub mod weapons {
        use super::*;
        pub static StabbingWeapon: Taxon = taxon("stabbing weapon", &Weapon);
        pub static BladedWeapon: Taxon = taxon("bladed weapon", &Weapon);
        pub static ProjectileWeapon : Taxon = taxon("projectile weapon", &Weapon);

        pub static ReachWeapon : Taxon = taxon("reach weapon", &Weapon);

        pub static Sword : Taxon = taxon("sword", &BladedWeapon);
        pub static Bow : Taxon = taxon("bow", &ProjectileWeapon);
        pub static Spear : Taxon = taxon2("spear", &StabbingWeapon, &ReachWeapon);
    }


    pub static Armor : Taxon = taxon("armor", &Item);
    // --------------- armors -------------------------
    pub static PlateArmor : Taxon = taxon("plate armor", &Armor);
    pub static LeatherArmor : Taxon = taxon("leather armor", &Armor);

    pub static Shield : Taxon = taxon("shield", &Armor);
    // --------------- shields -----------------------
    pub static LightShield : Taxon = taxon("light shield", &Shield);
    pub static HeavyShield : Taxon = taxon("heavy shield", &Shield);
    pub static TowerShield : Taxon = taxon("tower shield", &Shield);


    pub static LivingThing : Taxon = root_taxon("living thing");

    pub static Creature : Taxon = taxon("creature", &LivingThing);

    pub static Person : Taxon = taxon("person", &Creature);
    pub static Monster : Taxon = taxon("monster", &Creature);
    pub static Animal : Taxon = taxon("animal", &Creature);


    pub static Projectile : Taxon = taxon("projectile", &Item);
    pub mod projectiles {
        use super::*;
        pub static Arrow : Taxon = taxon("arrow", &Projectile);
        pub static Bolt : Taxon = taxon("bolt", &Projectile);
    }


    pub static Action : Taxon = root_taxon("action");

    pub static Attack : Taxon = taxon("attack", &Action);
    pub mod attacks {
        use super::*;
        pub static RangedAttack : Taxon = taxon("ranged attack", &Attack);

        pub static ProjectileAttack : Taxon = taxon("projectile attack", &RangedAttack);
        pub static ThrownAttack : Taxon = taxon("thrown attack", &RangedAttack);

        pub static SlashingAttack : Taxon = taxon("slashing attack", &Attack);

        pub static PiercingAttack : Taxon = taxon("piercing attack", &Attack);
        pub static StabbingAttack : Taxon = taxon("stabbing attack", &PiercingAttack);

        pub static ReachAttack : Taxon = taxon("reach attack", &Attack);
        pub static BludgeoningAttack : Taxon = taxon("bludgeoning attack", &Attack);
        pub static MagicAttack : Taxon = taxon("magic attack", &Attack);
        pub static NaturalAttack : Taxon = taxon("natural attack", &Attack);
    }

    pub static Movement : Taxon = taxon("movement", &Action);


    pub static Plant : Taxon = taxon("plant", &LivingThing);
    pub mod plants {
        use super::*;
        pub static Tree : Taxon = taxon("tree", &Plant);
    }

    pub static Resource : Taxon = taxon("resource", &Item);
    pub static Material : Taxon = taxon("material", &Resource);

    pub static Mineral : Taxon = taxon("mineral", &Resource);
    pub static Metal : Taxon = taxon("metal", &Mineral);

    pub mod resources {
        use super::*;

        pub static PlantResource: Taxon = taxon("plant resource", &Resource);

        pub static Straw : Taxon = taxon2("straw", &PlantResource, &Material);
        pub static Fruit : Taxon = taxon("fruit", &PlantResource);
        pub static Wood : Taxon = taxon2("wood", &PlantResource, &Material);

        pub static Stone : Taxon = taxon2("stone", &Mineral, &Material);
        pub static Iron : Taxon = taxon2("iron", &Metal, &Material);
    }

    lazy_static! {
        static ref CONST_TAXONS: Mutex<HashMap<String, &'static Taxon>> = Mutex::new(HashMap::new());
    }

    fn register_taxon(taxon : &'static Taxon) {
        if let Taxon::ConstTaxon { name , .. } = taxon {
            CONST_TAXONS.lock().unwrap().insert(String::from(*name), taxon);
        } else { error!("Cannot const-register non-const taxons") }
    }
    pub fn register() {
        use super::taxonomy::weapons::*;
        use super::taxonomy::projectiles::*;
        use super::taxonomy::resources::*;
        use super::taxonomy::plants::*;
        use super::taxonomy::attacks::*;

        register_taxon(&Unknown);
        register_taxon(&Item);

        register_taxon(&Weapon);
        register_taxon(&StabbingWeapon);
        register_taxon(&BladedWeapon);
        register_taxon(&ProjectileWeapon);

        register_taxon(&ReachWeapon);

        register_taxon(&Sword);
        register_taxon(&Bow);
        register_taxon(&Spear);


        register_taxon(&Armor);
        register_taxon(&PlateArmor);
        register_taxon(&LeatherArmor);

        register_taxon(&Shield);
        register_taxon(&LightShield);
        register_taxon(&HeavyShield);
        register_taxon(&TowerShield);


        register_taxon(&LivingThing);

        register_taxon(&Creature);

        register_taxon(&Person);
        register_taxon(&Monster);
        register_taxon(&Animal);


        register_taxon(&Projectile);
        register_taxon(&Arrow);
        register_taxon(&Bolt);

        register_taxon(&Action);

        register_taxon(&Attack);
        register_taxon(&RangedAttack);

        register_taxon(&ProjectileAttack);
        register_taxon(&ThrownAttack);

        register_taxon(&SlashingAttack);

        register_taxon(&PiercingAttack);
        register_taxon(&StabbingAttack);

        register_taxon(&ReachAttack);
        register_taxon(&BludgeoningAttack);
        register_taxon(&MagicAttack);
        register_taxon(&NaturalAttack);

        register_taxon(&Movement);


        register_taxon(&Plant);
        register_taxon(&Tree);

        register_taxon(&Resource);
        register_taxon(&Material);

        register_taxon(&Mineral);
        register_taxon(&Metal);

        register_taxon(&PlantResource);

        register_taxon(&Straw);
        register_taxon(&Fruit);
        register_taxon(&Wood);

        register_taxon(&Stone);
        register_taxon(&Iron);
    }
    pub fn taxon_by_name(name : &str) -> &'static Taxon {
        CONST_TAXONS.lock().unwrap().get(name).unwrap_or(&&Unknown)
    }
    pub fn taxon_by_name_opt(name : &str) -> Option<&'static Taxon> {
        CONST_TAXONS.lock().unwrap().get(name).map(|t| *t)
    }
}


#[derive(Clone, Debug, PrintFields, Default)]
pub struct ModifierTrackingData {
    pub modifiers_by_key : HashMap<String, ModifierReference>
}
impl EntityData for ModifierTrackingData {

}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_serialization() {
        use ron;

        taxonomy::register();

        let serialized = ron::ser::to_string_pretty(&taxonomy::HeavyShield, ron::ser::PrettyConfig::default()).ok().unwrap();
        let deserialized : Taxon = ron::de::from_str(&serialized).expect("could not deserialize");

        assert_eq!(&deserialized, &taxonomy::HeavyShield);


        let runtime_taxon = taxon("test shield", &taxonomy::HeavyShield);
        let serialized = ron::ser::to_string(&runtime_taxon).ok().unwrap();
        let deserialized : Taxon = ron::de::from_str(&serialized).expect("could not deserialize");

        assert_eq!(&deserialized, &runtime_taxon);
        assert_eq!(deserialized.parents(), runtime_taxon.parents());
    }
}