//use cgmath::prelude::*;
//use cgmath::Vector3;
use std::ops;
//use prelude::*;
use std::marker::PhantomData;
use common::hex::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::RefMut;
use std::hash::Hash;
use std::collections::hash_map;
use events::GameEvent;
use world::*;
use world::World;
use world::WorldView;
use core::*;
use enum_map::EnumMap;
use rand::StdRng;

use common::prelude::*;
use common::datastructures::PerfectHashable;

#[derive(Clone, Default, Debug)]
pub struct FactionData {
    pub name : String,
    pub color : [f32;4]
}

impl EntityData for FactionData {}

pub trait FactionStore {
    fn faction(&self, entity : Entity) -> &FactionData;
}
impl FactionStore for WorldView {
    fn faction(&self, entity: Entity) -> &FactionData {
        self.data::<FactionData>(entity)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DamageType {
    Untyped,
    Bludgeoning,
    Slashing,
    Piercing,
    Fire,
    Ice
}

#[derive(EnumMap, Debug, Clone, Copy)]
pub enum Skill {
    Dodge = 0,
    Melee = 1,
    Ranged = 2,
    MountainSurvival = 3,
    ForestSurvival = 4,
    FireMagic = 5,
    IceMagic = 6
}

#[derive(Debug)]
pub struct SkillInfo {
    pub name : &'static str,
    pub skill_type : Skill
}

static SKILL_INFO : [SkillInfo ; 7] = [
    SkillInfo {
        name : "Dodge",
        skill_type : Skill::Dodge
    },
    SkillInfo {
        name : "Melee",
        skill_type : Skill::Melee
    },
    SkillInfo {
        name : "Ranged",
        skill_type : Skill::Ranged
    },
    SkillInfo {
        name : "Mountain Survival",
        skill_type : Skill::MountainSurvival
    },
    SkillInfo {
        name : "Forest Survival",
        skill_type : Skill::ForestSurvival
    },
    SkillInfo {
        name : "Fire Magic",
        skill_type : Skill::FireMagic
    },
    SkillInfo {
        name : "Ice Magic",
        skill_type : Skill::IceMagic
    }
];

pub fn skill_info(for_skill : Skill) -> &'static SkillInfo {
    &SKILL_INFO[for_skill as usize]
}

#[derive(Clone, Debug)]
pub struct Attack {
    pub speed : f64, // represents how many attacks can be made per round for baseline user (never effectively < 1)
    pub damage_dice : DicePool,
    pub damage_bonus : u32,
    pub relative_accuracy: f64,
    pub primary_damage_type : DamageType,
    pub secondary_damage_type : Option<DamageType>,
    pub range : u32,
    pub min_range : u32
}

impl Default for Attack {
    fn default() -> Self {
        Attack {
            speed : 0.75,
            damage_dice : DicePool::default(),
            damage_bonus : 0,
            relative_accuracy : 0.0,
            primary_damage_type : DamageType::Untyped,
            secondary_damage_type : None,
            range : 1,
            min_range : 0
        }
    }
}

pub struct AttackRoll {
    pub damage_roll : DiceRoll,
    pub damage_bonus : u32,
    pub damage_total : u32
}

impl Attack {
    pub fn roll_damage (&self, rng : &mut StdRng) -> AttackRoll {
        let roll = self.damage_dice.roll(rng);
        let roll_total = roll.total_result;
        AttackRoll {
            damage_roll : roll,
            damage_bonus : self.damage_bonus,
            damage_total : roll_total + self.damage_bonus
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct ItemData {
    pub primary_attack : Option<Attack>,
    pub secondary_attack : Option<Attack>,
    pub held_by : Option<Entity>,
    pub position : Option<AxialCoord>
}
impl EntityData for ItemData {}


#[derive(Clone, Debug)]
pub struct CharacterData {
    pub faction : Entity,
    pub position: AxialCoord,
    pub graphical_position: Option<Vec2f>,
    pub graphical_color: [f32; 4],
    pub health: Reduceable<i32>,
    pub moves: Reduceable<f64>,
    pub stamina: Reduceable<i32>,
    pub actions: Reduceable<i32>,
    pub counters: Reduceable<i32>,
    pub melee_accuracy: f64,
    pub ranged_accuracy: f64,
    pub dodge_chance: f64,
    pub sprite : String,
    pub name : String,
    pub natural_attacks : Vec<Attack>,
    pub equipped : Vec<Entity>,
    pub inventory : Vec<Entity>,
    pub skills : EnumMap<Skill, u32>,
    pub skill_xp : EnumMap<Skill, u32>
}
impl EntityData for CharacterData {}

pub trait CharacterStore {
    fn character(&self, ent : Entity) -> &CharacterData;
}
impl CharacterStore for WorldView {
    fn character(&self, ent: Entity) -> &CharacterData {
        self.data::<CharacterData>(ent)
    }
}

impl Default for CharacterData {
    fn default() -> Self {
        CharacterData {
            faction : Entity::default(),
            position : AxialCoord::new(0,0),
            graphical_position : None,
            graphical_color: [1.0, 1.0, 1.0, 1.0],
            health : Reduceable::new(1),
            moves : Reduceable::new(1.0),
            stamina : Reduceable::new(20),
            actions : Reduceable::new(1),
            counters : Reduceable::new(1),
            melee_accuracy : 0.0,
            ranged_accuracy : 0.0,
            dodge_chance : 0.0,
            sprite : strf("default/defaultium"),
            name : strf("unnamed"),
            natural_attacks : vec![],
            equipped : vec![],
            inventory : vec![],
            skills : EnumMap::default(),
            skill_xp : EnumMap::default()
        }
    }
}

impl CharacterData {
    pub fn skill_level(&self, skill : Skill) -> u32 {
        self.skills[skill]
    }
    pub fn skill_xp(&self, skill : Skill) -> u32 {
        self.skill_xp[skill]
    }
    pub fn skill_xp_up(&mut self, skill : Skill, xp : u32) {
        self.skill_xp[skill] = self.skill_xp[skill] + xp;
    }
    pub fn possible_attacks(&self, world : &WorldView) -> Vec<Attack> {
        let mut res = self.natural_attacks.clone();
        for item_ref in &self.equipped {
            let item : &ItemData = world.data(*item_ref);
            if let Some(ref attack) = item.primary_attack {
                res.push(attack.clone());
            }
            if let Some(ref attack) = item.secondary_attack {
                res.push(attack.clone());
            }
        }
        res
    }
    pub fn effective_graphical_pos(&self, tile_radius : f32) -> Vec2f {
        self.graphical_position.unwrap_or_else(|| self.position.as_cartesian(tile_radius))
    }
    pub fn is_alive(&self) -> bool {
        self.health.cur_value() > 0
    }
    pub fn can_act(&self) -> bool { self.actions.cur_value() > 0 }
}


pub fn modify<T : EntityData, CM : ConstantModifier<T>>(world : &mut World, ent : Entity, modifier : CM) {
    world.add_constant_modifier(ent, modifier);
}


pub struct SkillXPMod(pub Skill, pub u32);
impl ConstantModifier<CharacterData> for SkillXPMod {
    fn modify(&self, data: &mut CharacterData) {
        let SkillXPMod(skill, xp) = *self;
        data.skill_xp[skill] += xp;
    }
}

pub struct SkillMod(pub Skill, pub u32);
impl ConstantModifier<CharacterData> for SkillMod {
    fn modify(&self, data: &mut CharacterData) {
        data.skills[self.0] += self.1;
    }
}

pub struct ReduceActionsMod(pub i32);
impl ConstantModifier<CharacterData> for ReduceActionsMod {
    fn modify(&self, data: &mut CharacterData) {
        data.actions.reduce_by(self.0);
    }
}

pub struct DamageMod(pub i32);
impl ConstantModifier<CharacterData> for DamageMod {
    fn modify(&self, data: &mut CharacterData) {
        data.health.reduce_by(self.0);
    }
}

pub struct ReduceMoveMod(pub f64);
impl ConstantModifier<CharacterData> for ReduceMoveMod {
    fn modify(&self, data: &mut CharacterData) {
        data.moves.reduce_by(self.0);
    }
}

pub struct EndMoveMod;
impl ConstantModifier<CharacterData> for EndMoveMod {
    fn modify(&self, data: &mut CharacterData) {
        data.moves.reduce_to(0.0f64);
    }
}

pub struct ResetCharacterTurnMod;
impl ConstantModifier<CharacterData> for ResetCharacterTurnMod {
    fn modify(&self, data: &mut CharacterData) {
        data.moves.reset();
        data.actions.reset();
        data.counters.reset();
    }
}

pub struct ChangePositionMod(pub AxialCoord);
impl ConstantModifier<CharacterData> for ChangePositionMod {
    fn modify(&self, data: &mut CharacterData) {
        data.position = self.0;
    }
}

pub struct CarryItemMod(pub Entity);
impl ConstantModifier<CharacterData> for CarryItemMod {
    fn modify(&self, data: &mut CharacterData) {
        data.inventory.push(self.0);
    }
}

pub struct EquipItemMod(pub Entity);
impl ConstantModifier<CharacterData> for EquipItemMod {
    fn modify(&self, data: &mut CharacterData) {
        data.equipped.push(self.0);
    }
}

pub struct ItemHeldByMod(pub Option<Entity>);
impl ConstantModifier<ItemData> for ItemHeldByMod{
    fn modify(&self, data: &mut ItemData) {
        data.held_by = self.0;
    }
}

#[derive(Clone, Default, Debug)]
pub struct TileData {
    pub name : &'static str,
    pub position: AxialCoord,
    pub move_cost: u32,
    pub cover: f64
}
impl EntityData for TileData {}


pub trait TileStore {
    fn tile (&self, coord : AxialCoord) -> &TileData;
    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData>;
}

impl TileStore for WorldView {
    fn tile(&self, coord: AxialCoord) -> &TileData {
        let tile_ent = self.entity_by_key(&coord).unwrap();
        self.data::<TileData>(tile_ent)
    }

    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData> {
        self.entity_by_key(&coord).map(|e| self.data::<TileData>(e))
    }
}

#[derive(Clone, Default, Debug)]
pub struct MapData {
    pub min_tile_bound : AxialCoord,
    pub max_tile_bound : AxialCoord
}
impl EntityData for MapData {}


#[derive(Clone, Default, Debug)]
pub struct TimeData {
    pub turn_number : u32
}
impl EntityData for TimeData {}

pub struct SetTurnNumberMod(pub u32);
impl ConstantModifier<TimeData> for SetTurnNumberMod{
    fn modify(&self, data: &mut TimeData) {
        data.turn_number = self.0;
    }
}

//#[allow(dead_code)]
//fn test_func(world: &mut World, cref1: Entity, _cref2: Entity) {
//    let character1 = world.character(cref1);
//    assert_eq!(character1.attack_power, 1);
//    {
//        world.add_character_modifier(cref1, box GenericModifier::new(|data: &mut CharacterData, _world: &World, _at_time : GameEventClock| { data.attack_power += 2 }));
//    }
//
//    let character1 = world.character(cref1);
//    assert_eq!(character1.attack_power, 3);
//
//    {
//        world.add_character_modifier(cref1, box GenericModifier::new(|data: &mut CharacterData, _world: &World, _at_time : GameEventClock| { data.attack_power += 4 }));
//    }
//    let character1 = world.character(cref1);
//    assert_eq!(character1.attack_power, 7);
//}
//
//#[test]
//fn simple() {
//    let mut world = World::new();
//
//    let raw_char_1 = Character::new(CharacterData {
//        position: AxialCoord::new(0, 0),
//        health: Reduceable::new(0),
//        moves: Reduceable::new(0),
//        melee_accuracy: 1.0,
//        ranged_accuracy: 1.0,
//        sprite: "test",
//        name: "test",
//        equipped: Vec![],
//        inventory: Vec![],
//        natural_attacks: Vec![],
//        skills: enum_map!(),
//        skill_xp: enum_map!(),
//        faction: Entity::sentinel(),
//        .. Default::default()
//    });
//
//    let raw_char_2 = Character::new(CharacterData {
//        position: AxialCoord::new(0, 0),
//        health: Reduceable::new(0),
//        moves: Reduceable::new(0),
//        melee_accuracy: 1.0,
//        ranged_accuracy: 1.0,
//        sprite: "test",
//        name: "test",
//        equipped: Vec![],
//        inventory: Vec![],
//        natural_attacks: Vec![],
//        skills: enum_map!(),
//        skill_xp: enum_map!(),
//        faction: Entity::sentinel(),
//        .. Default::default()
//    });
//
//    let cref1 = world.add_character(raw_char_1);
//
//    let cref2 = world.add_character(raw_char_2);
//
//    test_func(&mut world, cref1, cref2);
//}