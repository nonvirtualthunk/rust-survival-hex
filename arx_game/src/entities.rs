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
use common::color::Color;

#[derive(Clone, Default, Debug)]
pub struct FactionData {
    pub name : String,
    pub color : Color
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

#[derive(Enum, Debug, Clone, Copy)]
pub enum Skill {
    Dodge = 0,
    Melee = 1,
    Ranged = 2,
    MountainSurvival = 3,
    ForestSurvival = 4,
    FireMagic = 5,
    IceMagic = 6
}

impl Skill {
    pub fn xp_required_for_level(lvl : u32) -> u32 {
        let lvl = (lvl + 1) as f64; // shift over by 1 so that getting to level 1 doesn't cost 0 xp
        ((0.5 * lvl.powf(2.0) - 0.5 * lvl) * 10.0) as u32
    }

    pub fn level_for_xp(xp : u32) -> u32 {
        for i in 0 .. 100 {
            if Skill::xp_required_for_level(i) > xp {
                return i - 1;
            }
        }
        100
    }
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
    pub ap_cost : u32, // represents how many ap it costs to perform this attack
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
            ap_cost : 1,
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
pub trait ItemDataStore {
    fn item(&self, ent : Entity) -> &ItemData;
}
impl ItemDataStore for WorldView {
    fn item(&self, ent: Entity) -> &ItemData {
        self.data::<ItemData>(ent)
    }
}


#[derive(Clone, Debug)]
pub struct CharacterData {
    pub faction : Entity,
    pub position: AxialCoord,
    pub graphical_position: Option<CartVec>,
    pub graphical_color: Color,
    pub health: Reduceable<i32>,
    pub action_points: Reduceable<i32>,
    pub move_speed: Oct, // represented in octs
    pub moves: Oct,
    pub stamina: Reduceable<Oct>,
    pub sprite : String,
    pub name : String
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


#[derive(Clone, Debug, Default)]
pub struct InventoryData {
    pub equipped : Vec<Entity>,
    pub inventory : Vec<Entity>,
}
impl EntityData for InventoryData {}

pub trait InventoryDataStore {
    fn inventory(&self, ent : Entity) -> &InventoryData;
}
impl InventoryDataStore for WorldView {
    fn inventory(&self, ent: Entity) -> &InventoryData {
        self.data::<InventoryData>(ent)
    }
}


#[derive(Clone, Debug, Default)]
pub struct CombatData {
    pub natural_attacks : Vec<Attack>,
    pub counters: Reduceable<i32>,
    pub melee_accuracy: f64,
    pub ranged_accuracy: f64,
    pub dodge_chance: f64,
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


#[derive(Clone, Debug, Default)]
pub struct SkillData {
    pub skill_bonuses: EnumMap<Skill, u32>,
    pub skill_xp : EnumMap<Skill, u32>
}
impl EntityData for SkillData {}


pub trait SkillDataStore {
    fn skills(&self, ent : Entity) -> &SkillData;
}
impl SkillDataStore for WorldView {
    fn skills(&self, ent: Entity) -> &SkillData {
        self.data::<SkillData>(ent)
    }
}

/*
    Action points : each AP may be turned into movement, or used for an action
    Health : if it reaches zero, character dies
    Move speed : each point represents an addition eighth movement point when turning an AP into move

*/

impl Default for CharacterData {
    fn default() -> Self {
        CharacterData {
            faction : Entity::default(),
            position : AxialCoord::new(0,0),
            health : Reduceable::new(1),
            action_points : Reduceable::new(8),
            moves : Oct::zero(),
            move_speed : Oct::of(1),
            stamina : Reduceable::new(Oct::of(8)),
            sprite : strf("default/defaultium"),
            name : strf("unnamed"),
            graphical_position : None,
            graphical_color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl CharacterData {
    pub fn effective_graphical_pos(&self) -> CartVec {
        self.graphical_position.unwrap_or_else(|| self.position.as_cart_vec())
    }
    pub fn is_alive(&self) -> bool {
        self.health.cur_value() > 0
    }
    pub fn can_act(&self) -> bool { self.action_points.cur_value() > 0 }

    pub fn max_moves_remaining(&self, multiplier : f64) -> Oct {
        self.moves + Oct::of_rounded(self.move_speed.as_f64() * self.action_points.cur_value() as f64 * multiplier)
    }
    pub fn max_moves_per_turn(&self, multiplier : f64) -> Oct {
        self.move_speed * self.action_points.max_value()
    }
}

impl SkillData {
    pub fn skill_level(&self, skill : Skill) -> u32 {
         self.skill_bonuses[skill] + Skill::level_for_xp(self.skill_xp[skill])
    }
    pub fn skill_xp(&self, skill : Skill) -> u32 {
        self.skill_xp[skill]
    }
    pub fn skill_xp_up(&mut self, skill : Skill, xp : u32) {
        self.skill_xp[skill] = self.skill_xp[skill] + xp;
    }

    pub fn skill_levels(&self) -> Vec<(Skill, u32)> {
        let mut res = Vec::new();
        for (skill,xp) in &self.skill_xp {
            res.push((skill, self.skill_level(skill)));
        }
        res
    }
}


pub fn modify<T : EntityData, CM : ConstantModifier<T>>(world : &mut World, ent : Entity, modifier : CM) {
    world.add_constant_modifier(ent, modifier);
}


pub struct SkillXPMod(pub Skill, pub u32);
impl ConstantModifier<SkillData> for SkillXPMod {
    fn modify(&self, data: &mut SkillData) {
        let SkillXPMod(skill, xp) = *self;
        data.skill_xp[skill] += xp;
    }
}

pub struct SkillMod(pub Skill, pub u32);
impl ConstantModifier<SkillData> for SkillMod {
    fn modify(&self, data: &mut SkillData) {
        data.skill_bonuses[self.0] += self.1;
    }
}

pub struct ReduceActionsMod(pub u32);
impl ConstantModifier<CharacterData> for ReduceActionsMod {
    fn modify(&self, data: &mut CharacterData) {
        data.action_points.reduce_by(self.0 as i32);
    }
}

pub struct ReduceStaminaMod(pub Oct);
impl ConstantModifier<CharacterData> for ReduceStaminaMod {
    fn modify(&self, data: &mut CharacterData) {
        data.stamina.reduce_by(self.0);
    }
}

pub struct DamageMod(pub i32);
impl ConstantModifier<CharacterData> for DamageMod {
    fn modify(&self, data: &mut CharacterData) {
        data.health.reduce_by(self.0);
    }
}

pub struct ReduceMoveMod(pub Oct);
impl ConstantModifier<CharacterData> for ReduceMoveMod {
    fn modify(&self, data: &mut CharacterData) {
        data.moves = data.moves - self.0;
    }
}

pub struct EndMoveMod;
impl ConstantModifier<CharacterData> for EndMoveMod {
    fn modify(&self, data: &mut CharacterData) {
        data.moves = Oct::zero();
    }
}

pub struct ResetCharacterTurnMod;
impl ConstantModifier<CharacterData> for ResetCharacterTurnMod {
    fn modify(&self, data: &mut CharacterData) {
        data.moves = Oct::zero();
        data.action_points.reset();
    }
}
pub struct ResetCombatTurnMod;
impl ConstantModifier<CombatData> for ResetCombatTurnMod {
    fn modify(&self, data: &mut CombatData) {
        data.counters = Reduceable::new(0);
    }
}

pub struct ChangePositionMod(pub AxialCoord);
impl ConstantModifier<CharacterData> for ChangePositionMod {
    fn modify(&self, data: &mut CharacterData) {
        data.position = self.0;
    }
}

pub struct CarryItemMod(pub Entity);
impl ConstantModifier<InventoryData> for CarryItemMod {
    fn modify(&self, data: &mut InventoryData) {
        data.inventory.push(self.0);
    }
}

pub struct EquipItemMod(pub Entity);
impl ConstantModifier<InventoryData> for EquipItemMod {
    fn modify(&self, data: &mut InventoryData) {
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
    pub move_cost: Oct,
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