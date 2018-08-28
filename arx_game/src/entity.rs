use std::fmt::Debug;
use std::any::Any;
use std::fmt;
use world::World;
use std::sync::atomic::{ATOMIC_USIZE_INIT, AtomicUsize, Ordering};
use std::rc::Rc;
use std::fmt::Formatter;
use std::fmt::Error;
use std::any::TypeId;
use std::collections::HashMap;
use world::view::WorldView;

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

pub trait EntityData: Clone + Any + Default + Debug {
    fn nested_entities(&self) -> Vec<Entity> {
        Vec::new()
    }

    fn parent_entity(&self) -> Option<Entity> {
        None
    }
}


#[derive(Clone)]
pub struct EntityBuilder {
    initializations_by_type_id: HashMap<TypeId, Rc<Fn(&mut World, Entity)>>
}
impl Debug for EntityBuilder {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "EntityBuilder()")
    }
}

impl EntityBuilder {
    pub fn new() -> EntityBuilder {
        EntityBuilder {
            initializations_by_type_id: HashMap::new()
        }
    }

    pub fn with<T: EntityData>(mut self, new_data: T) -> Self {
        self.initializations_by_type_id.insert(TypeId::of::<T>(), Rc::new(move |world: &mut World, entity: Entity| {
            world.attach_data(entity, new_data.clone());
        }));
        self
    }

    pub fn with_creator<T: EntityData, F : Fn(&mut World) -> T + 'static>(mut self, new_data_func: F) -> Self {
        self.initializations_by_type_id.insert(TypeId::of::<T>(), Rc::new(move |world: &mut World, entity: Entity| {
            let new_data = (new_data_func)(world);
            world.attach_data(entity, new_data);
        }));
        self
    }

    pub fn create(&self, world: &mut World) -> Entity {
        let entity = World::create_entity();
        for initialization in self.initializations_by_type_id.values() {
            (initialization)(world, entity);
        }
        world.add_entity(entity);
        entity
    }
}


#[derive(Clone,Debug,Default)]
pub struct DebugData {
    pub name : String
}
impl EntityData for DebugData {}