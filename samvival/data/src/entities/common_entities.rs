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
use std::cmp::Ordering;
use either::Either;

#[derive(Default, Clone, Debug, Serialize, Deserialize, Fields)]
pub struct PositionData {
    pub hex: AxialCoord,
}

impl EntityData for PositionData {}

impl PositionData {
    pub fn distance(&self, other: &PositionData) -> R32 {
        self.hex.distance(&other.hex)
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, Fields)]
pub struct IdentityData {
    pub name: Option<String>,
    pub kinds: Vec<Taxon>,
}

impl EntityData for IdentityData {}

impl IdentityData {
    pub fn new<S1: Into<String>, T: Into<Taxon>>(name: S1, kind: T) -> IdentityData {
        IdentityData {
            name: Some(name.into()),
            kinds: vec![kind.into()],
        }
    }

    pub fn of_kind<T: Into<Taxon>>(kind: T) -> IdentityData {
        IdentityData::of_kinds(vec![kind])
    }

    pub fn of_kinds<T: Into<Taxon>>(kinds: Vec<T>) -> IdentityData {
        IdentityData {
            name: None,
            kinds: kinds.into_iter().map(|k| k.into()).collect_vec(),
        }
    }

    pub fn of_name_and_kinds<S1: Into<String>, T: Into<Taxon>>(name: S1, kinds: Vec<T>) -> IdentityData {
        IdentityData {
            name: Some(name.into()),
            kinds: kinds.into_iter().map(|k| k.into()).collect_vec(),
        }
    }

    pub fn set_kind<T: Into<Taxon>>(&mut self, kind: T) { self.kinds = vec![kind.into()] }

    pub fn effective_name(&self) -> &str {
        self.name.as_ref().map(|n| &**n).unwrap_or_else(|| self.main_kind().name())
    }

    pub fn main_kind(&self) -> &Taxon {
        self.kinds.first().unwrap_or(&taxonomy::Unknown)
    }
    pub fn replace_main_kind(&mut self, kind: Taxon) {
        if self.kinds.is_empty() {
            self.kinds.push(kind);
        } else {
            self.kinds[0] = kind;
        }
    }
}

impl Default for IdentityData {
    fn default() -> Self {
        IdentityData {
            name: None,
            kinds: vec![taxonomy::Unknown.clone()],
        }
    }
}

pub trait IdentityDataStore {
    fn identity(&self, entity: Entity) -> &IdentityData;
}

impl IdentityDataStore for WorldView {
    fn identity(&self, entity: Entity) -> &IdentityData {
        self.data::<IdentityData>(entity)
    }
}


#[derive(Clone, Debug, Eq)]
pub enum Taxon {
    ConstTaxon { name: Str, parents: [Option<&'static Taxon>; 4] },
    RuntimeTaxon { name: String, index: usize },
    RuntimeTaxonRef { reference: Arc<Taxon> },
    // we would really only want to do this if it was an Rc, probably, but we can't currently... unless we made const taxons wholly separate...
    ConstTaxonRef { reference: &'static Taxon },
}

impl Default for Taxon {
    fn default() -> Self {
        Taxon::ConstTaxonRef { reference: &taxonomy::Unknown }
    }
}

impl PartialEq<Taxon> for Taxon {
    fn eq(&self, other: &Taxon) -> bool {
        self.name() == other.name()
    }
}

impl PartialOrd for Taxon {
    fn partial_cmp(&self, other: &Taxon) -> Option<Ordering> {
        self.name().partial_cmp(other.name())
    }
}

impl Ord for Taxon {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(other.name())
    }
}

impl Hash for Taxon {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name().as_bytes())
    }
}

impl From<&'static Taxon> for Taxon {
    fn from(a: &'static Taxon) -> Self {
        Taxon::ConstTaxonRef { reference: a }
    }
}

//pub struct Taxon {
//    pub name : Arc<str>,
//    pub parents : [Option<&'static Taxon>;4],
//}
//pub type Taxon = &'static Taxon;

impl Taxon {
    pub fn of(tax: &'static Taxon) -> Taxon {
        Taxon::ConstTaxonRef { reference: tax }
    }

    pub fn new<S: Into<String>, T: Into<Taxon>>(world: &mut World, name: S, parent: T) -> Taxon {
        RuntimeTaxonData::create_new_taxon(world, name, vec![parent.into()])
    }

    pub fn name(&self) -> &str {
        match self {
            Taxon::ConstTaxon { name, .. } => name,
            Taxon::RuntimeTaxon { name, .. } => name,
            Taxon::RuntimeTaxonRef { reference } => reference.name(),
            Taxon::ConstTaxonRef { reference, .. } => reference.name(),
        }
    }

    pub fn is_a(&self, view: &WorldView, other: &Taxon) -> bool {
        if self.name() == other.name() {
            true
        } else {
            self.parents(view).any_match(|p| p.is_a(view, other))
        }
    }

    pub fn parents<'b>(&self, view: &'b WorldView) -> Vec<&'b Taxon> {
        if let Taxon::ConstTaxonRef { reference, .. } = self {
            return reference.parents(view);
        }


        match self {
            Taxon::ConstTaxon { parents, .. } => parents.iter().flat_map(|opt| opt.iter()).map(|t| *t).collect(),
            Taxon::RuntimeTaxon { index, name } => {
                let data = view.world_data::<RuntimeTaxonData>();
                if let Some(parents) = data.runtime_parents.get(*index) {
                    parents.iter().collect()
                } else {
                    error!("Index mismatch for runtime taxon on looking up parents {}[{}]", name, index);
                    vec![]
                }
            }
            Taxon::RuntimeTaxonRef { reference } => reference.parents(view),
            Taxon::ConstTaxonRef { .. } => panic!("this is unreachable"),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Fields)]
pub struct RuntimeTaxonData {
    pub runtime_taxons: Vec<Taxon>,
    pub runtime_parents: Vec<Vec<Taxon>>,
}

impl EntityData for RuntimeTaxonData {}

impl RuntimeTaxonData {
    pub fn create_new_taxon<S: Into<String>>(world: &mut World, name: S, parents: Vec<Taxon>) -> Taxon {
        let name = name.into();
        let view = world.view();
        let data = view.world_data::<RuntimeTaxonData>();
        let existing = data.runtime_taxons.iter().find(|t| if let Taxon::RuntimeTaxon { name: tname, .. } = t { tname == &name } else { false });
        if let Some(existing) = existing {
            existing.clone()
        } else {
            let new_taxon = Taxon::RuntimeTaxon { name, index: data.runtime_taxons.len() };
            world.modify_world(RuntimeTaxonData::runtime_taxons.append(new_taxon.clone()), None);
            world.modify_world(RuntimeTaxonData::runtime_parents.append(parents), None);
//            let mut const_parents : Vec<Taxon> = Vec::new();
//            let mut runtime_parents : Vec<usize> = Vec::new();
//            for parent in &parents {
//                RuntimeTaxonData::add_to_parent_lists(parent, &mut const_parents, &mut runtime_parents);
//            }
//            world.modify_world(RuntimeTaxonData::runtime_parents.append(runtime_parents));
//            world.modify_world(RuntimeTaxonData::const_parents.append(const_parents));
            new_taxon
        }
    }

    fn add_to_parent_lists(parent: &Taxon, const_parents: &mut Vec<&'static Taxon>, runtime_parents: &mut Vec<usize>) {
        match parent {
            Taxon::ConstTaxon { .. } => panic!("you can't pass ownership of a const taxon, how did you even trigger this?"),
            Taxon::ConstTaxonRef { reference } => const_parents.push(reference),
            Taxon::RuntimeTaxon { index, .. } => runtime_parents.push(*index),
            Taxon::RuntimeTaxonRef { reference } => RuntimeTaxonData::add_to_parent_lists(&reference, const_parents, runtime_parents),
        }
    }
}


impl Serialize for Taxon {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
        match self {
            Taxon::ConstTaxon { name, .. } => {
                let empty_parents: Vec<&str> = Vec::new();

//                if serializer.is_human_readable() {
//                    serializer.serialize_str(*name)
//                } else {
                let mut tuple = serializer.serialize_tuple(2)?;
                tuple.serialize_element(&0u8)?;
                tuple.serialize_element(name)?;
                tuple.end()
//                }
            }
            Taxon::RuntimeTaxon { name, index } => {
                let mut tuple = serializer.serialize_tuple(3)?;
                tuple.serialize_element(&1u8)?;
                tuple.serialize_element(name)?;
                tuple.serialize_element(index)?;
                tuple.end()
            }
            Taxon::RuntimeTaxonRef { reference } => {
                reference.serialize(serializer)
            }
            Taxon::ConstTaxonRef { reference } => {
                let empty_parents: Vec<&str> = Vec::new();

//                if serializer.is_human_readable() {
//                    serializer.serialize_str(reference.name())
//                } else {
                let mut tuple = serializer.serialize_tuple(2)?;
                tuple.serialize_element(&2u8)?;
                tuple.serialize_element(reference.name())?;
                tuple.end()
//                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for Taxon {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        struct TaxonVisitor;
        impl<'de> Visitor<'de> for TaxonVisitor {
            type Value = Taxon;

            fn expecting(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
                write!(formatter, "A static taxon")
            }

            fn visit_str<E>(self, v: &str) -> Result<<Self as Visitor<'de>>::Value, E> where E: Error, {
                if let Some(taxon_ref) = taxonomy::taxon_by_name_opt(v) {
                    Ok(Taxon::ConstTaxonRef { reference: taxon_ref })
                } else {
                    warn!("Taxon should have been available by name but was not registered, {}", v);
                    Ok(Taxon::ConstTaxonRef { reference: &taxonomy::Unknown })
                }
            }

            fn visit_string<E>(self, v: String) -> Result<<Self as Visitor<'de>>::Value, E> where E: Error, {
                if let Some(taxon_ref) = taxonomy::taxon_by_name_opt(&v) {
                    Ok(Taxon::ConstTaxonRef { reference: taxon_ref })
                } else {
                    warn!("Taxon should have been available by name but was not registered, {}", v);
                    Ok(Taxon::ConstTaxonRef { reference: &taxonomy::Unknown })
                }
            }


            fn visit_seq<A>(self, mut seq: A) -> Result<<Self as Visitor<'de>>::Value, <A as SeqAccess<'de>>::Error> where A: SeqAccess<'de>, {
                let type_index: u8 = seq.next_element()?.ok_or_else(|| panic!("taxon did not have a taxon type id"))?;
                let name: String = seq.next_element()?.ok_or_else(|| panic!("deserialize taxon name failed"))?;

                // runtime taxons deserialize differently
                if type_index == 1 {
                    let index = seq.next_element()?.ok_or_else(|| panic!("runtime taxon did not have an index"))?;
                    Ok(Taxon::RuntimeTaxon { name, index })
                } else {
                    if let Some(taxon_ref) = taxonomy::taxon_by_name_opt(&name) {
                        Ok(Taxon::ConstTaxonRef { reference: taxon_ref })
                    } else {
                        panic!(format!("Unknown const taxon found: {}", name));
                    }
                }
            }
        }
//        if deserializer.is_human_readable() {
//            deserializer.deserialize_any(TaxonVisitor)
//        } else {
        deserializer.deserialize_tuple(2, TaxonVisitor)
//        }
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

fn intern_string(string: &str) -> Arc<str> {
    let mut strings = RUNTIME_TAXON_STRS.lock().unwrap();
    if let Some(existing) = strings.get(string) {
        return existing.clone();
    }

    let new_arc: Arc<str> = string.into();
    strings.insert(new_arc.clone());
    new_arc
}

pub fn taxon_vec<T: Into<Taxon>>(vec: Vec<T>) -> Vec<Taxon> {
    vec.into_iter().map(|v| v.into()).collect_vec()
}

//pub fn child_taxon(name: &str, parent: &Taxon) -> Taxon {
//    let parent_name = intern_string(parent.name());
//    Taxon::RuntimeChildTaxon { name: intern_string(name), parents: vec![parent_name] }
//}
//
//pub fn taxon(name: &str, parent: &'static Taxon) -> Taxon {
//    Taxon::RuntimeTaxon { name: intern_string(name), parents: [Some(parent), None, None, None] }
//}
//
//pub fn taxon2(name: &str, parent1: &'static Taxon, parent2: &'static Taxon) -> Taxon {
//    Taxon::RuntimeTaxon { name: intern_string(name), parents: [Some(parent1), Some(parent2), None, None] }
//}

pub const fn alias(of: &'static Taxon) -> Taxon { Taxon::ConstTaxonRef { reference: of } }

pub mod taxonomy {
    use super::Taxon;
    use std::sync::Mutex;
    use std::collections::HashMap;
    use common::prelude::Str;
    use super::Rc;
    use super::Arc;
    use super::alias;

    pub static Unknown: Taxon = root_taxon("unknown thing");

    pub static Item: Taxon = root_taxon("item");
    pub static DelicateItem: Taxon = taxon("delicate item", &Item);
    pub static SturdyItem: Taxon = taxon("sturdy item", &Item);

    pub static Axe: Taxon = taxon("axe", &Item);

    pub static Weapon: Taxon = taxon("weapon", &SturdyItem);

    pub mod weapons {
        use super::*;

        pub static MeleeWeapon: Taxon = taxon("melee weapon", &Weapon);
        pub static RangedWeapon: Taxon = taxon("ranged weapon", &Weapon);

        pub static ImprovisedWeapon: Taxon = taxon("improvised weapon", &Weapon);

        pub static StabbingWeapon: Taxon = taxon("stabbing weapon", &MeleeWeapon);
        pub static BladedWeapon: Taxon = taxon("bladed weapon", &MeleeWeapon);
        pub static ProjectileWeapon: Taxon = taxon("projectile weapon", &RangedWeapon);

        pub static ReachWeapon: Taxon = taxon("reach weapon", &MeleeWeapon);

        pub static Sword: Taxon = taxon("sword", &BladedWeapon);
        pub static Bow: Taxon = taxon("bow", &ProjectileWeapon);
        pub static Spear: Taxon = taxon2("spear", &StabbingWeapon, &ReachWeapon);
        pub static BattleAxe: Taxon = taxon2("battle axe", &BladedWeapon, &Axe);

        pub static Longbow: Taxon = taxon("longbow", &Bow);
        pub static Longsword: Taxon = taxon("longsword", &Sword);
    }

    pub static Tool: Taxon = taxon("tool", &Item);

    pub mod tools {
        use super::*;

        pub static SharpTool: Taxon = taxon("bladed tool", &Tool);

        pub static MiningTool: Taxon = taxon("mining tool", &Tool);

        pub static Rod: Taxon = taxon("rod", &Tool);

        pub static ToolAxe: Taxon = taxon2("tool axe", &Tool, &Axe);
        pub static Pickaxe: Taxon = taxon("pickaxe", &MiningTool);
        pub static Scythe: Taxon = taxon("scythe", &SharpTool);
        pub static Hammer: Taxon = taxon("hammer", &Tool);
        pub static Shovel: Taxon = taxon("shovel", &Tool);

        pub static Hatchet: Taxon = taxon2("hatchet", &ToolAxe, &weapons::ImprovisedWeapon);
    }

    pub static Armor: Taxon = taxon("armor", &Item);
    // --------------- armors -------------------------
    pub static PlateArmor: Taxon = taxon("plate armor", &Armor);
    pub static LeatherArmor: Taxon = taxon("leather armor", &Armor);

    pub static Shield: Taxon = taxon("shield", &Armor);
    // --------------- shields -----------------------
    pub static LightShield: Taxon = taxon("light shield", &Shield);
    pub static HeavyShield: Taxon = taxon("heavy shield", &Shield);
    pub static TowerShield: Taxon = taxon("tower shield", &Shield);


    pub static LivingThing: Taxon = root_taxon("living thing");

    pub static Creature: Taxon = taxon("creature", &LivingThing);

    pub static Person: Taxon = taxon("person", &Creature);
    pub static Monster: Taxon = taxon("monster", &Creature);
    pub static Animal: Taxon = taxon("animal", &Creature);


    pub static Projectile: Taxon = taxon("projectile", &Item);

    pub mod projectiles {
        use super::*;

        pub static Arrow: Taxon = taxon("arrow", &Projectile);
        pub static Bolt: Taxon = taxon("bolt", &Projectile);
    }


    pub static Action: Taxon = root_taxon("action");

    pub static Attack: Taxon = taxon("attack", &Action);

    pub mod attacks {
        use super::*;

        pub static MeleeAttack: Taxon = taxon("melee attack", &Attack);
        pub static RangedAttack: Taxon = taxon("ranged attack", &Attack);

        pub static ProjectileAttack: Taxon = taxon("projectile attack", &RangedAttack);
        pub static ThrownAttack: Taxon = taxon("thrown attack", &RangedAttack);

        pub static SlashingAttack: Taxon = taxon("slashing attack", &Attack);

        pub static PiercingAttack: Taxon = taxon("piercing attack", &Attack);
        pub static StabbingAttack: Taxon = taxon("stabbing attack", &PiercingAttack);

        pub static ReachAttack: Taxon = taxon("reach attack", &MeleeAttack);
        pub static BludgeoningAttack: Taxon = taxon("bludgeoning attack", &Attack);
        pub static MagicAttack: Taxon = taxon("magic attack", &Attack);
        pub static NaturalAttack: Taxon = taxon("natural attack", &Attack);

        pub static ImprovisedAttack: Taxon = taxon("improvised attack", &Attack); // an attack with something that isn't really a weapon
    }

    pub static Movement: Taxon = taxon("movement", &Action);


    pub static Plant: Taxon = taxon("plant", &LivingThing);

    pub mod plants {
        use super::*;

        pub static Tree: Taxon = taxon("tree", &Plant);
    }

    pub static Resource: Taxon = taxon("resource", &Item);
    pub static Material: Taxon = taxon("material", &Resource);

    pub static Mineral: Taxon = taxon("mineral", &Resource);
    pub static Metal: Taxon = taxon("metal", &Mineral);

    pub mod resources {
        use super::*;

        pub static PlantResource: Taxon = taxon("plant resource", &Resource);

        pub static Straw: Taxon = taxon2("straw", &PlantResource, &Material);
        pub static Fruit: Taxon = taxon("fruit", &PlantResource);
        pub static Wood: Taxon = taxon2("wood", &PlantResource, &Material);

        pub static Stone: Taxon = taxon2("stone", &Mineral, &Material);
        pub static QuarriedStone: Taxon = taxon("quarried stone", &Stone);
        pub static LooseStone: Taxon = taxon("loose stone", &Stone);

        pub static Dirt: Taxon = taxon("dirt", &Material);
        pub static Iron: Taxon = taxon2("iron", &Metal, &Material);
    }

    pub mod materials {
        use super::*;

        pub static Wood: Taxon = alias(&resources::Wood);
        pub static Stone: Taxon = alias(&resources::Stone);
        pub static Metal: Taxon = alias(&super::Metal);
    }

    pub static Terrain: Taxon = root_taxon("terrain");

    pub mod terrain {
        use super::*;

        pub static Plains: Taxon = taxon("plains", &Terrain);
        pub static Hills: Taxon = taxon("hills", &Terrain);
        pub static Mountains: Taxon = taxon("mountains", &Mountains);
    }

    pub static Vegetation: Taxon = root_taxon("vegetation");

    pub mod vegetation {
        use super::*;

        pub static Grassland: Taxon = taxon("grassland", &Vegetation);
        pub static Forest: Taxon = taxon("forest", &Vegetation);
        pub static PineForest: Taxon = taxon("pine forest", &Forest);
        pub static DeciduousForest: Taxon = taxon("deciduous forest", &Forest);
    }

    pub static IngredientType: Taxon = root_taxon("ingredient type");

    pub mod ingredient_types {
        use super::*;

        pub static WeaponHeadIngredient: Taxon = taxon("weapon head ingredient", &IngredientType);
        pub static WeaponReinforcementIngredient: Taxon = taxon("weapon reinforcement ingredient", &IngredientType);
        pub static HandleIngredient: Taxon = taxon("handle ingredient", &IngredientType);

        pub static Haft: Taxon = taxon("haft", &HandleIngredient);
        pub static Axehead: Taxon = taxon("axehead", &WeaponHeadIngredient);
        pub static Spearhead: Taxon = taxon("spearhead", &WeaponHeadIngredient);
        pub static Binding: Taxon = taxon("binding", &WeaponReinforcementIngredient);
        pub static Blade: Taxon = taxon("blade", &WeaponHeadIngredient);
        pub static Plate: Taxon = taxon("plate", &WeaponReinforcementIngredient);
    }


    lazy_static! {
        static ref CONST_TAXONS: Mutex<HashMap<String, &'static Taxon>> = Mutex::new(HashMap::new());
    }

    pub(crate) fn register_taxon(taxon: &'static Taxon) {
        if let Taxon::ConstTaxon { name, .. } = taxon {
            CONST_TAXONS.lock().unwrap().insert(String::from(*name), taxon);
        } else { error!("Cannot const-register non-const taxons") }
    }

    pub fn register() {
        ::entities::taxonomy_registration::register_taxonomy()
    }

    pub fn taxon_by_name(name: &str) -> &'static Taxon {
        CONST_TAXONS.lock().unwrap().get(name).unwrap_or(&&Unknown)
    }

    pub fn taxon_by_name_opt(name: &str) -> Option<&'static Taxon> {
        CONST_TAXONS.lock().unwrap().get(name).map(|t| *t)
    }


    pub const fn root_taxon(name: Str) -> Taxon {
        Taxon::ConstTaxon { name, parents: [None, None, None, None] }
    }

    pub const fn taxon(name: Str, parent: &'static Taxon) -> Taxon {
        Taxon::ConstTaxon { name, parents: [Some(parent), None, None, None] }
    }

    pub const fn taxon2(name: Str, parent1: &'static Taxon, parent2: &'static Taxon) -> Taxon {
        Taxon::ConstTaxon { name, parents: [Some(parent1), Some(parent2), None, None] }
    }
}


#[derive(Clone, Debug, Fields, Default, Serialize, Deserialize)]
pub struct ModifierTrackingData {
    pub modifiers_by_key: HashMap<String, ModifierReference>
}

impl EntityData for ModifierTrackingData {}


pub trait LookupSignifier {
    fn signifier(&self, entity: Entity) -> String;
}

impl LookupSignifier for WorldView {
    fn signifier(&self, entity: Entity) -> String {
        if let Some(identity) = self.data_opt::<IdentityData>(entity) {
            identity.name.clone().unwrap_or_else(|| String::from(identity.main_kind().name()))
        } else {
            format!("Entity({})", entity.0)
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_serialization() {
        use ron;

        taxonomy::register();

        let serialized = ron::ser::to_string_pretty(&taxonomy::HeavyShield, ron::ser::PrettyConfig::default()).ok().unwrap();
        let deserialized: Taxon = ron::de::from_str(&serialized).expect("could not deserialize");

        assert_eq!(&deserialized, &taxonomy::HeavyShield);

        // TODO: get a full runtime deserialization going on here

//        let runtime_taxon = taxon("test shield", &taxonomy::HeavyShield);
//        let serialized = ron::ser::to_string(&runtime_taxon).ok().unwrap();
//        let deserialized: Taxon = ron::de::from_str(&serialized).expect("could not deserialize");
//
//        assert_eq!(&deserialized, &runtime_taxon);
//        assert_eq!(deserialized.parents(), runtime_taxon.parents());
    }

    #[test]
    pub fn generate_registration_function() {
        if let Ok(mut file) = ::std::fs::File::open("/Users/nvt/code/samvival/samvival/data/src/entities/common_entities.rs") {
            use std::io::Read;
            use regex::Regex;

            let taxon_re = Regex::new(".*pub static ([A-Za-z]*?)\\s*: Taxon.*").unwrap();
            let mod_re = Regex::new(".*pub mod ([A-Za-z_]*?)\\s*\\{").unwrap();

            let mut registration_file = String::new();
            registration_file.push_str("use entities::common_entities::taxonomy;\n");
            registration_file.push_str("pub(crate) fn register_taxonomy() {\n");

            let mut mods = Vec::new();
            let mut contents = String::new();
            match file.read_to_string(&mut contents) {
                Ok(_) => {
                    for line in contents.lines() {
                        if line.contains("alias") {
                            println!("Contains alias, skipping : \"{}\"", line);
                            continue;
                        }
                        if let Some(captures) = taxon_re.captures(line) {
                            let name = &captures[1];
                            let mod_name: String = mods.iter().join("::");
                            registration_file.push_str(format!("\t\ttaxonomy::register_taxon(&{}::{});\n", mod_name, name).as_str());
                        } else if let Some(captures) = mod_re.captures(line) {
                            let mod_name = &captures[1];
                            mods.push(strf(mod_name));
                        } else if line.contains("}") {
                            mods.pop();
                        }
                    }
                }
                Err(err) => println!("Could not read: {:?}", err)
            }

            registration_file.push_str("}");
            println!("Registration file contents\n{}", registration_file);

            if let Ok(mut file) = ::std::fs::File::create("/Users/nvt/code/samvival/samvival/data/src/entities/taxonomy_registration.rs") {
                use std::io::Write;
                if let Ok(_) = file.write_all(registration_file.as_bytes()) {
                    println!("Writing registration file complete");
                } else {
                    println!("Write registration file failed");
                }
            } else {
                println!("Could not create registration file");
            }
        } else {
            println!("Could not open");
        }
    }
}