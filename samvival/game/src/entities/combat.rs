use common::hex::AxialCoord;
use common::prelude::*;
use game::core::*;
use entities::inventory::EquipmentData;
use entities::item::ItemData;
use rand::StdRng;
use std::fmt::{Display, Error, Formatter};
use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use game::GameDisplayable;
use entities::common::Taxon;
use prelude::*;

use logic;
use entities::selectors::EntitySelectors;

#[derive(Clone,Debug)]
pub enum DerivedAttackKind {
    PiercingStrike,
    None
}

impl DerivedAttackKind {
    pub fn derive_special_attack(&self, world : &WorldView, attacker : Entity, weapon : Entity, attack : Entity) -> Option<Attack> {
        match self {
            DerivedAttackKind::PiercingStrike => {
                let mut new_attack = world.attack(attack).clone();
                new_attack.name = format!("piercing {}", new_attack.name);
                new_attack.ap_cost += 1;
                new_attack.stamina_cost += 1;
                new_attack.to_hit_bonus -= 1;
                new_attack.min_range = 1;
                new_attack.range = 1;
                new_attack.pattern = HexPattern::Line(0, 2);
                Some(new_attack)
            },
            DerivedAttackKind::None => None
        }
    }
}

/// Entity data to represent a special, derived attack. The weapon entity points to the
#[derive(Clone,Debug,PrintFields)]
pub struct DerivedAttackData {
    pub weapon_condition : EntitySelectors,
    pub character_condition : EntitySelectors,
    pub attack_condition : EntitySelectors,
    pub kind : DerivedAttackKind,
}

impl EntityData for DerivedAttackData {}

impl Default for DerivedAttackData {
    fn default() -> Self {
        DerivedAttackData {
            weapon_condition : EntitySelectors::Any,
            character_condition : EntitySelectors::Any,
            attack_condition : EntitySelectors::Any,
            kind : DerivedAttackKind::None
        }
    }
}

///// Representation of an ability that generates special versions of regular attacks based on some sort of condition.
///// E.g. the piercing attack that spearmen get, it acts as less accurate, higher stamina cost version of their weapon's
///// normal attack, and hits an additional target behind the first. If the spearman's weapon attack gets better, then
///// the piercing special version also gets better, so it has be a derived situation. The "kind" determines how the
///// attack is created from the base value, the conditions determine when a SpecialAttackData entity should be created
///// for a weapon.
//#[derive(Clone, Debug)]
//pub struct SpecialAttackCreator {
//    kind : DerivedAttackKind,
//    weapon_condition : EntitySelectors,
//    character_condition : EntitySelectors,
//}

#[derive(Clone, Debug, PrintFields)]
pub struct CombatData {
    pub active_attack : AttackRef,
    pub active_counterattack : AttackRef,
    pub natural_attacks : Vec<Entity>,
    pub counters_remaining: Reduceable<i32>,
    pub counters_per_event: i32,
    pub melee_accuracy_bonus: i32,
    pub ranged_accuracy_bonus: i32,
    pub melee_damage_bonus: i32,
    pub ranged_damage_bonus: i32,
    pub dodge_bonus: i32,
    pub defense_bonus: i32,
    pub block_bonus: i32,
    pub special_attacks: Vec<Entity>,
}
impl EntityData for CombatData {
    fn nested_entities(&self) -> Vec<Entity> {
        self.special_attacks.clone()
    }
}

impl Default for CombatData {
    fn default() -> Self {
        CombatData {
            active_attack : AttackRef::none(),
            active_counterattack : AttackRef::none(),
            natural_attacks : Vec::new(),
            counters_remaining: Reduceable::new(0),
            counters_per_event: 1,
            melee_accuracy_bonus : 0,
            ranged_accuracy_bonus : 0,
            melee_damage_bonus : 0,
            ranged_damage_bonus : 0,
            dodge_bonus: 0,
            defense_bonus: 0,
            block_bonus: 0,
            special_attacks: Vec::new()
        }
    }
}

pub trait CombatDataStore {
    fn combat(&self, ent : Entity) -> &CombatData;
    fn attack(&self, ent : Entity) -> &Attack;
}
impl CombatDataStore for WorldView {
    fn combat(&self, ent: Entity) -> &CombatData {
        self.data::<CombatData>(ent)
    }

    fn attack(&self, ent: Entity) -> &'_ Attack {
        self.data::<Attack>(ent)
    }
}




#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DamageType {
    Untyped,
    Bludgeoning,
    Slashing,
    Piercing,
    Fire,
    Ice
}

impl Display for DamageType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        (self as &std::fmt::Debug).fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttackType {
    Projectile,
    Thrown,
    Melee,
    Reach
}
impl Display for AttackType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        (self as &std::fmt::Debug).fmt(f)
    }
}
impl Default for AttackType {
    fn default() -> Self {
        AttackType::Melee
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum HexPattern {
    Single,
    Line(i32,i32), // start, length
    Arc(i32,i32), // start, length
}
impl Default for HexPattern { fn default() -> Self { HexPattern::Single } }


#[derive(Clone, Debug, PartialEq)]
pub struct Attack {
    pub name : String,
    pub verb : Option<String>,
    pub attack_type : AttackType,
    pub ap_cost : u32, // represents how many ap it costs to perform this attack
    pub damage_dice : DicePool,
    pub damage_bonus : i32,
    pub to_hit_bonus : i32,
    pub primary_damage_type : DamageType,
    pub secondary_damage_type : Option<DamageType>,
    pub range : u32,
    pub min_range : u32,
    pub ammunition_kind: Option<Taxon>,
    pub stamina_cost : u32,
    pub pattern : HexPattern
}

impl EntityData for Attack{}


pub fn create_attack<T : Into<Taxon>>(world: &mut World, name: Str, kinds: Vec<T>, attack: Attack) -> Entity {
    EntityBuilder::new()
        .with(attack)
        .with(IdentityData::of_name_and_kinds(name, kinds))
        .create(world)
}


#[derive(Clone, Debug, PartialEq)]
pub struct StrikeResult {
    pub weapon : Option<Entity>,
    pub damage_done : i32,
    pub hit : bool,
    pub killing_blow : bool,
    pub strike_number : u8,
    pub damage_types : Vec<DamageType>,
}

impl Default for Attack {
    fn default() -> Self {
        Attack {
            name : strf("Nameless attack"),
            verb : None,
            ap_cost : 1,
            attack_type : AttackType::Melee,
            damage_dice : DicePool::default(),
            damage_bonus : 0,
            to_hit_bonus: 0,
            primary_damage_type : DamageType::Untyped,
            secondary_damage_type : None,
            range : 1,
            min_range : 0,
            ammunition_kind: None,
            stamina_cost: 0,
            pattern : HexPattern::Single
        }
    }
}

#[derive(Clone, PartialEq, Hash, Debug)]
pub struct AttackRef {
    pub attack_entity: Entity,
    derived_from: Entity
}
impl AttackRef {
    pub fn new (attack : Entity, derived_from : Entity) -> AttackRef {
        AttackRef { attack_entity: attack, derived_from }
    }

    pub fn none() -> AttackRef {
        AttackRef { attack_entity: Entity::sentinel(), derived_from : Entity::sentinel() }
    }

    pub fn as_option(&self) -> Option<&AttackRef> {
        if self.is_none() {
            None
        } else {
            Some(self)
        }
    }

//    pub fn of_attack(world : &WorldView, character : Entity, attack : Entity) -> AttackReference {
//        if world.combat(character).natural_attacks.contains(&attack) {
//            return AttackReference::new(attack, character)
//        } else if let Some(equip_data) = world.data_opt::<EquipmentData>(character) {
//            for equipped in &equip_data.equipped {
//                if world.data_opt::<ItemData>(*equipped).filter(|i| i.attacks.contains(&attack)).is_some() {
//                    return AttackReference::new(attack, *equipped);
//                }
//            }
//        } else if let Some(combat_data) = world.data_opt::<CombatData>(character) {
//
//        }
//        AttackReference::none()
//    }

    pub fn of_primary_from(world : &WorldView, entity : Entity) -> AttackRef {
        if let Some(combat) = world.data_opt::<CombatData>(entity) {
            if let Some(attack) = combat.natural_attacks.first() {
                return AttackRef::new(*attack, entity);
            }
        } else if let Some(item) = world.data_opt::<ItemData>(entity) {
            if let Some(attack) = item.attacks.first() {
                return AttackRef::new(*attack, entity);
            }
        }
        AttackRef::none()
    }

    pub fn referenced_attack_name_raw<'a, 'b>(&'a self, world: &'b WorldView) -> Option<&'b String> {
        self.as_option().and_then(|a| world.data_opt::<Attack>(a.attack_entity)).map(|a| &a.name)
    }

    pub fn resolve_attack_and_weapon(&self, world: &WorldView, character : Entity) -> Option<(Attack, Entity)> {
        if self.is_none() {
            None
        } else {
            if logic::combat::character_has_access_to_attack(world, character, self.attack_entity) {
                if let Some(weapon) = self.resolve_weapon(world, character) {
                    if let Some(attack) = world.data_opt::<Attack>(self.attack_entity) {
                        Some((attack.clone(), weapon))
                    } else if let Some(derived_attack) = world.data_opt::<DerivedAttackData>(self.attack_entity) {
                        let underlying_attack = self.derived_from;
                        if let Some(weapon) = logic::combat::intern::weapon_attack_derives_from(world, character, underlying_attack) {
                            if let Some(new_attack) = derived_attack.kind.derive_special_attack(world, character, weapon, underlying_attack) {
                                Some((new_attack, weapon))
                            } else {
                                warn!("derived attack could not create actual new attack from the base attack it was given");
                                None
                            }
                        } else {
                            warn!("derived attack is derived from weapon that could not be identified on character");
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    warn!("Attack reference could not be resolved for lack of identifying the weapon it was derived from");
                    None
                }
            } else {
                info!("Character ({}) no longer has access to referenced attack ({})", world.signifier(character), world.signifier(self.attack_entity));
                None
            }
        }
    }

    pub fn resolve(&self, world: &WorldView, character : Entity) -> Option<Attack> {
        self.resolve_attack_and_weapon(world, character).map(|t| t.0)
    }

    pub fn resolve_weapon(&self, world: &WorldView, character : Entity) -> Option<Entity> {
        if world.has_data::<Attack>(self.attack_entity) {
            logic::combat::intern::weapon_attack_derives_from(world, character, self.attack_entity)
        } else if world.has_data::<DerivedAttackData>(self.attack_entity) {
            logic::combat::intern::weapon_attack_derives_from(world, character, self.derived_from)
        } else {
            None
        }
    }

    pub fn is_melee(&self, world: &WorldView, character : Entity) -> bool {
        self.resolve(world, character).map(|a| a.attack_type == AttackType::Melee).unwrap_or(false)
    }

    pub fn is_derived_attack(&self, world: &WorldView) -> bool {
        ! world.has_data::<Attack>(self.attack_entity) && world.has_data::<DerivedAttackData>(self.attack_entity)
    }

    pub fn is_none(&self) -> bool {
        self.attack_entity == Entity::sentinel()
    }
    pub fn is_some(&self) -> bool { ! self.is_none() }
}

impl GameDisplayable for AttackRef {
    fn to_game_str_full(&self, view : &WorldView) -> String {
        match self.as_option() {
            Some(a) => a.referenced_attack_name_raw(view).cloned().unwrap_or_else(||strf("unresolveable attack")),
            None => strf("none")
        }
    }
}

impl Default for AttackRef {
    fn default() -> Self {
        AttackRef::none()
    }
}

pub struct AttackRoll {
    pub damage_roll : DiceRoll,
    pub damage_bonus : i32,
    pub damage_total : u32
}

impl Attack {
    pub fn roll_damage (&self, rng : &mut StdRng) -> AttackRoll {
        let roll = self.damage_dice.roll(rng);
        let roll_total = roll.total_result;
        AttackRoll {
            damage_roll : roll,
            damage_bonus : self.damage_bonus,
            damage_total : (roll_total as i32 + self.damage_bonus).as_u32_or_0()
        }
    }
}

