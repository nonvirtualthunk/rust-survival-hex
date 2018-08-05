use common::hex::AxialCoord;
use common::prelude::*;
use core::*;
use entities::inventory::InventoryData;
use entities::item::ItemData;
use rand::StdRng;
use std::fmt::{Display, Error, Formatter};
use world::Entity;
use world::EntityData;
use world::WorldView;


#[derive(Clone, Debug, Default)]
pub struct CombatData {
    pub active_attack : Option<AttackReference>,
    pub natural_attacks : Vec<Attack>,
    pub counters: Reduceable<i32>,
    pub melee_accuracy_bonus: i32,
    pub ranged_accuracy_bonus: i32,
    pub melee_damage_bonus: i32,
    pub ranged_damage_bonus: i32,
    pub dodge_bonus: i32,
}
impl EntityData for CombatData {}

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




#[derive(Clone, Debug, PartialEq)]
pub struct Attack {
    pub name : Str,
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

#[derive(Clone, Copy, PartialEq, Hash, Debug)]
pub struct AttackReference {
    pub entity : Entity,
    pub index : usize
}
impl AttackReference {
    fn new (entity : Entity, index : usize) -> AttackReference {
        AttackReference { entity, index }
    }

    pub fn of_attack(world : &WorldView, character : Entity, attack : &Attack) -> Option<AttackReference> {
        if let Some(natural_attack_index) = world.data::<CombatData>(character).natural_attacks.iter().position(|a| a == attack) {
            Some(AttackReference::new(character, natural_attack_index))
        } else {
            if let Some(inv_data) = world.data_opt::<InventoryData>(character) {
                for equipped_item in &inv_data.equipped {
                    if let Some(item_data) = world.data_opt::<ItemData>(*equipped_item) {
                        if item_data.primary_attack.as_ref() == Some(attack) {
                            return Some(AttackReference::new(*equipped_item, 0));
                        } else if item_data.secondary_attack.as_ref() == Some(attack) {
                            return Some(AttackReference::new(*equipped_item, 1));
                        }
                    }
                }
            }
            None
        }
    }

    pub fn referenced_attack<'a,'b>(&'a self, world: &'b WorldView, character : Entity) -> Option<&'b Attack> {
        if character == self.entity {
            world.data::<CombatData>(character).natural_attacks.get(self.index)
        } else {

            if let Some(inv_data) = world.data_opt::<InventoryData>(character) {
                if inv_data.equipped.contains(&self.entity) {
                    if let Some(item_data) = world.data_opt::<ItemData>(self.entity) {
                        match self.index {
                            0 => return item_data.primary_attack.as_ref(),
                            1 => return item_data.secondary_attack.as_ref(),
                            _ => warn!("non 0/1 for index in reference to item attack")
                        }
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

