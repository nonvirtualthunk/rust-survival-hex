use std::fmt::Debug;
use std::any::Any;
use std::fmt;
use world::World;
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};

pub static ENTITY_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

type EntityId = usize;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Default)]
pub struct Entity(pub EntityId);

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Ent({})", self.0)
    }
}

impl Entity {
    pub fn sentinel() -> Entity {
        Entity(0)
    }
}

pub trait EntityData: Clone + Any + Default + Debug {}


pub struct EntityBuilder {
    initializations: Vec<Box<Fn(&mut World, Entity)>>
}

impl EntityBuilder {
    pub fn new() -> EntityBuilder {
        EntityBuilder {
            initializations: vec![]
        }
    }

    pub fn with<T: EntityData>(mut self, new_data: T) -> Self {
        self.initializations.push(box move |world: &mut World, entity: Entity| {
            world.attach_data(entity, &new_data)
        });
        self
    }

    pub fn create(self, world: &mut World) -> Entity {
        let entity = World::create_entity();
        for initialization in self.initializations {
            initialization(world, entity);
        }
        entity
    }
}