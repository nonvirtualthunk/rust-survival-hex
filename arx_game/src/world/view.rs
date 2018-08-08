use std::collections::HashMap;
use entity::Entity;
use events::GameEventWrapper;
use std::any::TypeId;
use anymap::Map;
use anymap::any::CloneAny;
use entity::EntityData;
use std::hash::Hash;
use std::rc::Rc;
use core::GameEventClock;
use world::storage::*;
use events::GameEventType;

pub struct WorldView {
    pub(crate) entities: Vec<EntityContainer>,
    pub(crate) self_entity : Entity,
    pub(crate) constant_data: Map<CloneAny>,
    pub(crate) effective_data: Map<CloneAny>,
    pub current_time: GameEventClock,
    pub(crate) modifier_cursor: ModifierClock,
    pub(crate) modifier_indices: HashMap<TypeId, usize>,
    pub(crate) events: MultiTypeEventContainer,
    pub entity_indices: Map<CloneAny>
}



impl WorldView {
    pub fn data<T: EntityData>(&self, entity: Entity) -> &T {
        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>()
            .unwrap_or_else(|| panic!(format!("Could not retrieve effective data for entity {:?}, looking for data {:?}", entity, unsafe {std::intrinsics::type_name::<T>()})));

        match data.storage.get(&entity) {
            Some(t) => t,
            None => &data.sentinel
        }
    }
    pub fn data_opt<T: EntityData>(&self, entity: Entity) -> Option<&T> {
        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>()
            .unwrap_or_else(|| panic!(format!("Could not retrieve effective data for entity {:?}, looking for data {:?}", entity, unsafe {std::intrinsics::type_name::<T>()})));
        data.storage.get(&entity)
    }

    pub fn data_mut<T: EntityData>(&mut self, entity: Entity) -> &mut T {
        let data: &mut DataContainer<T> = self.effective_data.get_mut::<DataContainer<T>>().unwrap();
        match data.storage.get_mut(&entity) {
            Some(t) => t,
            None => panic!("Attempted to get mutable reference to non-existent data in view")
        }
    }

    pub fn world_data<T: EntityData>(&self) -> &T {
        self.data::<T>(self.self_entity)
    }

    pub fn entities_with_data<T : EntityData>(&self) -> &HashMap<Entity, T> {
        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>().unwrap();
        &data.storage
    }

    pub fn entity_by_key<I : Hash + Eq + Clone + 'static>(&self, key : &I) -> Option<Entity> {
        let index : &EntityIndex<I> = self.entity_indices.get::<EntityIndex<I>>()
            .unwrap_or_else(|| panic!(format!("Index on {:?} not created", unsafe {std::intrinsics::type_name::<I>()})));
        index.index.get(key).cloned()
    }

    pub fn has_data<T : EntityData>(&self, entity : Entity) -> bool {
        self.has_data_r::<T>(&entity)
    }
    pub fn has_data_r<T : EntityData>(&self, entity : &Entity) -> bool {
        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>()
            .unwrap_or_else(|| panic!(format!("Could not retrieve effective data for entity {:?}, looking for data {:?}", entity, unsafe {std::intrinsics::type_name::<T>()})));
        data.storage.contains_key(&entity)
    }

    pub fn events<E : GameEventType + 'static>(&self) -> impl Iterator<Item=&GameEventWrapper<E>> {
        self.events.events::<E>()
    }
}