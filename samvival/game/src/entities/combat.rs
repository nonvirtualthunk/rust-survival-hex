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


#[derive(Clone, Debug, PrintFields)]
pub struct CombatData {
    pub active_attack : AttackReference,
    pub active_counterattack : AttackReference,
    pub natural_attacks : Vec<Attack>,
    pub counters_remaining: Reduceable<i32>,
    pub counters_per_event: i32,
    pub melee_accuracy_bonus: i32,
    pub ranged_accuracy_bonus: i32,
    pub melee_damage_bonus: i32,
    pub ranged_damage_bonus: i32,
    pub dodge_bonus: i32,
    pub defense_bonus: i32,
    pub block_bonus: i32
}
impl EntityData for CombatData {}

impl Default for CombatData {
    fn default() -> Self {
        CombatData {
            active_attack : AttackReference::none(),
            active_counterattack : AttackReference::none(),
            natural_attacks : Vec::new(),
            counters_remaining: Reduceable::new(0),
            counters_per_event: 1,
            melee_accuracy_bonus : 0,
            ranged_accuracy_bonus : 0,
            melee_damage_bonus : 0,
            ranged_damage_bonus : 0,
            dodge_bonus: 0,
            defense_bonus: 0,
            block_bonus: 0
        }
    }
}

pub trait CombatDataStore {
    fn combat(&self, ent : Entity) -> &CombatData;
}
impl CombatDataStore for WorldView {
    fn combat(&self, ent: Entity) -> &CombatData {
        self.data::<CombatData>(ent)
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
pub struct Attack {
    pub name : Str,
    pub attack_type : AttackType,
    pub ap_cost : u32, // represents how many ap it costs to perform this attack
    pub damage_dice : DicePool,
    pub damage_bonus : i32,
    pub to_hit_bonus : i32,
    pub primary_damage_type : DamageType,
    pub secondary_damage_type : Option<DamageType>,
    pub range : u32,
    pub min_range : u32
}

impl Default for Attack {
    fn default() -> Self {
        Attack {
            name : "Nameless attack",
            ap_cost : 1,
            attack_type : AttackType::Melee,
            damage_dice : DicePool::default(),
            damage_bonus : 0,
            to_hit_bonus: 0,
            primary_damage_type : DamageType::Untyped,
            secondary_damage_type : None,
            range : 1,
            min_range : 0
        }
    }
}

#[derive(Clone, PartialEq, Hash, Debug)]
pub struct AttackReference {
    pub entity : Entity,
    pub index : usize,
//    pub name : String
}
impl AttackReference {
    pub fn new (entity : Entity, index : usize) -> AttackReference {
        AttackReference { entity, index }
    }

    pub fn none() -> AttackReference {
        AttackReference { entity : Entity::sentinel(), index : 0 }
    }

    pub fn as_option(&self) -> Option<&AttackReference> {
        if self.is_none() {
            None
        } else {
            Some(self)
        }
    }

    pub fn of_attack(world : &WorldView, character : Entity, attack : &Attack) -> AttackReference {
        if let Some(natural_attack_index) = world.data::<CombatData>(character).natural_attacks.iter().position(|a| a == attack) {
            AttackReference::new(character, natural_attack_index)
        } else {
            if let Some(inv_data) = world.data_opt::<EquipmentData>(character) {
                for equipped_item in &inv_data.equipped {
                    if let Some(item_data) = world.data_opt::<ItemData>(*equipped_item) {
                        if let Some(pos) = item_data.attacks.iter().position(|a| a == attack) {
                            return AttackReference::new(*equipped_item, pos);
                        }
                    }
                }
            }
            AttackReference::none()
        }
    }

    pub fn of_primary_from(world : &WorldView, entity : Entity) -> AttackReference {
        if let Some(combat) = world.data_opt::<CombatData>(entity) {
            if let Some(attack) = combat.natural_attacks.first() {
                return AttackReference::new(entity, 0);
            }
        } else if let Some(item) = world.data_opt::<ItemData>(entity) {
            if let Some(attack) = item.attacks.first() {
                return AttackReference::new(entity, 0);
            }
        }
        AttackReference::none()
    }

    pub fn referenced_attack_raw<'a,'b>(&'a self, world: &'b WorldView) -> Option<&'b Attack> {
        if self.is_none() {
            None
        } else {
            if let Some(combat_data) = world.data_opt::<CombatData>(self.entity) {
                combat_data.natural_attacks.get(self.index)
            } else {
                if let Some(item_data) = world.data_opt::<ItemData>(self.entity) {
                    return item_data.attacks.get(self.index);
                } else {
                    warn!("attack reference neither natural nor item based");
                }
                None
            }
        }
    }

    pub fn referenced_attack<'a,'b>(&'a self, world: &'b WorldView, character : Entity) -> Option<&'b Attack> {
        if self.is_none() {
            None
        } else {
            if character == self.entity {
                world.data::<CombatData>(character).natural_attacks.get(self.index)
            } else {

                if let Some(inv_data) = world.data_opt::<EquipmentData>(character) {
                    if inv_data.equipped.contains(&self.entity) {
                        if let Some(item_data) = world.data_opt::<ItemData>(self.entity) {
                            return item_data.attacks.get(self.index);
                        } else {
                            warn!("attack reference neither natural nor item based");
                        }
                    } else {
                        warn!("attack referenced did not belong to any currently equipped entity");
                    }
                }

                None
            }
        }
    }

    pub fn is_melee(&self, world: &WorldView, character : Entity) -> bool {
        self.referenced_attack(world, character).map(|a| a.attack_type == AttackType::Melee).unwrap_or(false)
    }

    pub fn is_none(&self) -> bool {
        self.entity == Entity::sentinel()
    }
    pub fn is_some(&self) -> bool { ! self.is_none() }
}

impl GameDisplayable for AttackReference {
    fn to_game_str_full(&self, view : &WorldView) -> String {
        match self.as_option() {
            Some(a) => strf(a.referenced_attack_raw(view).map(|a| a.name).unwrap_or("unresolveable attack")),
            None => strf("none")
        }
    }
}

impl Default for AttackReference {
    fn default() -> Self {
        AttackReference::none()
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

