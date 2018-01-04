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
use entity_base::*;
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

pub type Faction = Entity<FactionData>;
pub type FactionModifier = Modifier<FactionData>;
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Default)]
pub struct FactionRef(pub usize);
impl PerfectHashable for FactionRef {
    fn hash(&self) -> usize {
        self.0
    }
}

impl Entity<FactionData> {
    pub fn new(data : FactionData) -> Faction {
        Faction {
            id: FACTION_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
            intern_data : data,
            modifiers : vec![]
        }
    }

    pub fn is_sentinel(&self) -> bool {
        self.id == 0
    }
}
impl FactionRef {
    pub fn is_sentinel(&self) -> bool {
        self.0 == 0
    }

    pub fn sentinel() -> FactionRef {
        FactionRef(0)
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
    pub held_by : Option<CharacterRef>,
    pub position : Option<AxialCoord>
}

pub type Item = Entity<ItemData>;
pub type ItemModifier = Modifier<ItemData>;
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Default)]
pub struct ItemRef(pub usize);
impl PerfectHashable for ItemRef {
    fn hash(&self) -> usize {
        self.0
    }
}

impl Entity<ItemData> {
    pub fn new(data : ItemData) -> Item {
        Item {
            id: ITEM_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
            intern_data : data,
            modifiers : vec![]
        }
    }
}


#[derive(Clone, Debug)]
pub struct CharacterData {
    pub faction : FactionRef,
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
    pub equipped : Vec<ItemRef>,
    pub inventory : Vec<ItemData>,
    pub skills : EnumMap<Skill, u32>,
    pub skill_xp : EnumMap<Skill, u32>
}

impl Default for CharacterData {
    fn default() -> Self {
        CharacterData {
            faction : FactionRef::default(),
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

pub type Character = Entity<CharacterData>;
pub type CharacterModifier = Modifier<CharacterData>;

impl Entity<CharacterData> {
    pub fn new(gd: CharacterData) -> Character {
        Character {
            id: CHARACTER_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
            intern_data: gd,
            modifiers: vec!()
        }
    }
    pub fn as_ref(&self) -> CharacterRef {
        CharacterRef(self.id)
    }
    pub fn is_sentinel(&self) -> bool { self.id == 0 }
}

impl CharacterRef {
    pub fn sentinel() -> CharacterRef {
        CharacterRef(0)
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
            let item = world.item(*item_ref);
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

pub fn skill_xp_modifier(skill : Skill, xp : u32) -> Box<Modifier<CharacterData>> {
    Box::new(GenericModifier::new(move |cd : &mut CharacterData,_,_| cd.skill_xp[skill] += xp))
}

pub fn skill_modifier(skill : Skill, level_boost : u32) -> Box<Modifier<CharacterData>> {
    Box::new(GenericModifier::new(move |cd : &mut CharacterData,_,_| cd.skills[skill] += level_boost))
}

pub fn character_modifier<F : Fn(&mut CharacterData) + 'static>(func : F) -> Box<Modifier<CharacterData>> {
    Box::new(GenericModifier::new(move |cd : &mut CharacterData,_,_| func(cd)))
}

pub fn modify_character<F : Fn(&mut CharacterData) + 'static>(world : &mut World, cref : CharacterRef, func : F) {
    world.add_character_modifier(cref, character_modifier(func));
}

pub fn modify_character_turn<F : Fn(&mut CharacterData) + 'static>(world : &mut World, cref : CharacterRef, func : F) {
    world.add_character_modifier(cref, character_modifier(func));
}

pub fn modify_item<F : Fn(&mut ItemData) + 'static>(world: &mut World, iref : ItemRef, func : F) {
    world.add_item_modifier(iref, Box::new(GenericModifier::new(move |id : &mut ItemData,_,_| func(id))))
}

#[derive(Clone, Default)]
pub struct TileData {
    pub name : &'static str,
    pub position: AxialCoord,
    pub move_cost: u32,
    pub cover: f64
}


pub type Tile = Entity<TileData>;

// would be nicer to write as `impl Tile`, but the IDE plugin isn't quite smart enough with auto complete to handle it ideally
impl Entity<TileData> {
    pub fn new(data : TileData) -> Tile {
        Tile {
            id: TILE_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
            intern_data : data,
            modifiers: vec![]
        }
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Default)]
pub struct CharacterRef(pub usize);
impl PerfectHashable for CharacterRef {
    fn hash(&self) -> usize {
        self.0
    }
}


//#[allow(dead_code)]
//fn test_func(world: &mut World, cref1: CharacterRef, _cref2: CharacterRef) {
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
//        faction: FactionRef::sentinel(),
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
//        faction: FactionRef::sentinel(),
//        .. Default::default()
//    });
//
//    let cref1 = world.add_character(raw_char_1);
//
//    let cref2 = world.add_character(raw_char_2);
//
//    test_func(&mut world, cref1, cref2);
//}