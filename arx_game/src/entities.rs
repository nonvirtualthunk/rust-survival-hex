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
use world::World;
use core::*;

static ENTITY_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

pub trait Modifier<GameData: Clone> {
    fn apply(&self, gamedata: &mut GameData, world: &World, at_time: GameEventClock);
}

pub struct ModifierRef(usize);

pub struct Entity<GameData: Clone> {
    pub id: usize,
    pub intern_data: GameData,
    pub modifiers: Vec<ModifierContainer<GameData>>
}

impl<GameData: Clone> Entity<GameData> {
    pub fn raw_data(&mut self) -> &mut GameData {
        &mut self.intern_data
    }

    pub fn data(&self, world: &World) -> GameData {
        let mut cur = self.intern_data.clone();
        for modifier in &self.modifiers {
            modifier.modifier.apply(&mut cur, world, world.event_clock);
        }
        return cur;
    }

    pub fn data_at_time(&self, world: &World, at_time : GameEventClock) -> GameData {
        let mut cur = self.intern_data.clone();
        for modifier in &self.modifiers {
            if modifier.applied_at <= at_time {
                modifier.modifier.apply(&mut cur, world, at_time);
            }
        }
        return cur;
    }

    pub fn add_modifier(&mut self, modifier: ModifierContainer<GameData>) {
        self.modifiers.push(modifier);
    }
}


#[derive(Clone, Default, Debug)]
pub struct CharacterData {
    pub position: AxialCoord,
    pub health: Reduceable<i32>,
    pub moves: Reduceable<f64>,
    pub attack_power: i32,
    pub melee_accuracy: f64,
    pub ranged_accuracy: f64,
    pub sprite : String,
    pub name : String
}

pub type Character = Entity<CharacterData>;
pub type CharacterModifier = Modifier<CharacterData>;

impl Entity<CharacterData> {
    pub fn new(gd: CharacterData) -> Character {
        Character {
            id: ENTITY_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
            intern_data: gd,
            modifiers: vec!()
        }
    }

    pub fn as_ref(&self) -> CharacterRef {
        CharacterRef(self.id)
    }

    pub fn is_sentinel(&self) -> bool { self.id == 0 }
}

pub struct AttackBonus {
    bonus: i32
}

pub struct GenericModifier<GameData: Clone, F: Fn(&mut GameData, &World, GameEventClock)> {
    modify: F,
    _marker: PhantomData<GameData>
}

impl<'a, F: Fn(&mut GameData, &World, GameEventClock), GameData: 'a + Clone> Modifier<GameData> for GenericModifier<GameData, F> {
    fn apply(&self, gamedata: &mut GameData, world: &World, at_time : GameEventClock) {
        (self.modify)(gamedata, world, at_time)
    }
}

impl<'a, GameData: 'a + Clone, F: Fn(&mut GameData, &World, GameEventClock)> GenericModifier<GameData, F> {
    pub fn new(func: F) -> GenericModifier<GameData, F> {
        GenericModifier {
            modify: func,
            _marker: PhantomData
        }
    }
}



pub struct GenericCharacterModifier<F: Fn(&mut CharacterData, &World, GameEventClock)> {
    modify: F
}

pub fn character_modifier<F : Fn(&mut CharacterData) + 'static>(func : F) -> Box<Modifier<CharacterData>> {
    Box::new(GenericCharacterModifier::new(move |cd : &mut CharacterData,_,_| func(cd)))
}

impl<'a, F: Fn(&mut CharacterData, &World, GameEventClock)> Modifier<CharacterData> for GenericCharacterModifier<F> {
    fn apply(&self, gamedata: &mut CharacterData, world: &World, at_time : GameEventClock) {
        (self.modify)(gamedata, world, at_time)
    }
}

impl<F: Fn(&mut CharacterData, &World, GameEventClock)> GenericCharacterModifier<F> {
    pub fn new(func: F) -> GenericCharacterModifier<F> {
        GenericCharacterModifier {
            modify: func
        }
    }
}


impl Modifier<CharacterData> for AttackBonus {
    fn apply(&self, gamedata: &mut CharacterData, _world: &World, _at_time : GameEventClock) {
        gamedata.attack_power += self.bonus;
    }
}


#[derive(Clone)]
pub struct TileData {
    pub name : &'static str,
    pub position: AxialCoord,
    pub move_cost: u32
}

//impl Default for TileData {
//    fn default() -> Self {
//        TileData {
//            name : "Default",
//            position: AxialCoord::new(0,0)
//        }
//    }
//}

pub type Tile = Entity<TileData>;

// would be nicer to write as `impl Tile`, but the IDE plugin isn't quite smart enough with auto complete to handle it ideally
impl Entity<TileData> {
    pub fn new(data : TileData) -> Tile {
        Tile {
            id: ENTITY_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1,
            intern_data : data,
            modifiers: vec![]
        }
    }
}


#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CharacterRef(pub usize);


pub struct ModifierContainer<GameDataType> {
    pub applied_at : GameEventClock,
    pub modifier : Box<Modifier<GameDataType>>
}

pub struct EntityContainer<KeyType : Eq + Hash, GameDataType: Clone> {
    pub entities: HashMap<KeyType, Entity<GameDataType>>,
    //    modifiers : HashMap<usize, Box<Modifier<GameDataType>>>,
    //    modifier_counter : usize,
//    modifiers: Vec<ModifierContainer<GameDataType>>,
    pub sentinel: Entity<GameDataType>
}

impl<KeyType : Eq + Hash, GameDataType: Clone> EntityContainer<KeyType, GameDataType> {
    pub fn new(sentinel: Entity<GameDataType>) -> EntityContainer<KeyType, GameDataType> {
        EntityContainer {
            sentinel,
//            modifiers: vec![],
            entities: HashMap::new()
        }
    }
}


#[allow(dead_code)]
fn test_func(world: &mut World, cref1: CharacterRef, _cref2: CharacterRef) {
    let character1 = world.character(cref1);
    assert_eq!(character1.attack_power, 1);
    {
        world.add_character_modifier(cref1, box AttackBonus { bonus: 2 });
    }

    let character1 = world.character(cref1);
    assert_eq!(character1.attack_power, 3);

    {
        world.add_character_modifier(cref1, box GenericModifier::new(|data: &mut CharacterData, _world: &World, _at_time : GameEventClock| { data.attack_power += 4 }));
    }
    let character1 = world.character(cref1);
    assert_eq!(character1.attack_power, 7);
}

#[test]
fn simple() {
    let mut world = World::new();

    let raw_char_1 = Character::new(CharacterData {
        position: AxialCoord::new(0, 0),
        health: Reduceable::new(0),
        moves: Reduceable::new(0),
        attack_power: 1,
        melee_accuracy: 1.0,
        ranged_accuracy: 1.0,
        sprite: "test",
        name: "test"
    });

    let raw_char_2 = Character::new(CharacterData {
        position: AxialCoord::new(0, 0),
        health: Reduceable::new(0),
        moves: Reduceable::new(0),
        attack_power: 1,
        melee_accuracy: 1.0,
        ranged_accuracy: 1.0,
        sprite: "test",
        name: "test"
    });

    let cref1 = world.add_character(raw_char_1);

    let cref2 = world.add_character(raw_char_2);

    test_func(&mut world, cref1, cref2);
}