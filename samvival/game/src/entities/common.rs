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


lazy_static! {
    static ref RUNTIME_TAXON_STRS : Mutex<HashSet<Arc<str>>> = Mutex::new(HashSet::new());
}

fn intern_string(string : Str) -> Arc<str> {
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
        static ref CONST_TAXONS: Mutex<HashMap<Str, &'static Taxon>> = Mutex::new(HashMap::new());
    }

    fn register_taxon(taxon : &'static Taxon) {
        if let Taxon::ConstTaxon { name , .. } = taxon {
            CONST_TAXONS.lock().unwrap().insert(*name, taxon);
        } else { error!("Cannot const-register non-const taxons") }
    }
    pub fn register() {
        register_taxon(&Unknown);
    }
    pub fn taxon_by_name(name : Str) -> &'static Taxon {
        CONST_TAXONS.lock().unwrap().get(name).unwrap_or(&&Unknown)
    }
}


#[derive(Clone, Debug, PrintFields, Default)]
pub struct ModifierTrackingData {
    pub modifiers_by_key : HashMap<String, ModifierReference>
}
impl EntityData for ModifierTrackingData {

}