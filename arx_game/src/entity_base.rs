use std::ops;
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

pub static CHARACTER_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;
pub static ITEM_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;
pub static FACTION_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;
pub static WORLD_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;
pub static TILE_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;


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

pub struct ModifierContainer<GameDataType> {
    pub applied_at : GameEventClock,
    pub modifier : Box<Modifier<GameDataType>>
}