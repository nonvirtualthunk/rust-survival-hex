use std::collections::HashMap;
use entity::Entity;
use common::multitype::MultiTypeContainer;
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
use events::GameEventState;
use std::collections::HashSet;
use serde::Serialize;
use serde::de::DeserializeOwned;


/// world views are views into the data of a world. The world itself is the ledger of changes, the view is a way of looking at it at a specific time.
/// world views can be made counter-factual by layering modifications on top of them, or by modifying their data directly in place. Once they have been
///

#[derive(Default)]
pub struct WorldView {
    pub(crate) entities: Vec<EntityContainer>,
    pub(crate) entity_set: HashSet<Entity>,
    pub(crate) self_entity : Entity,
    pub(crate) constant_data: MultiTypeContainer,
    pub(crate) effective_data: MultiTypeContainer,
    pub(crate) overlay_data: MultiTypeContainer,
    pub current_time: GameEventClock,
    pub(crate) modifier_cursor: ModifierClock,
    pub(crate) modifier_indices: HashMap<TypeId, usize>,
    pub(crate) events: MultiTypeEventContainer,
    pub entity_indices: MultiTypeContainer,
    pub(crate) has_overlay: bool,
}




impl WorldView {
    pub fn data<T: EntityData>(&self, entity: Entity) -> &T {
        if self.has_overlay {
            if let Some(overlay) = self.overlay_data.get_opt::<DataContainer<T>>() {
                if let Some(overlaid) = overlay.storage.get(&entity) {
                    return overlaid;
                }
            }
        }

        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>();

        match data.storage.get(&entity) {
            Some(t) => t,
            None => &data.sentinel
        }
    }
    pub fn data_opt<T: EntityData>(&self, entity: Entity) -> Option<&T> {
        if self.has_overlay {
            if let Some(overlay) = self.overlay_data.get_opt::<DataContainer<T>>() {
                if let Some(overlaid) = overlay.storage.get(&entity) {
                    return Some(overlaid);
                }
            }
        }

        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>();
        data.storage.get(&entity)
    }

    pub fn data_mut<T: EntityData>(&mut self, entity: Entity) -> &mut T where T : Serialize + DeserializeOwned {
        self.has_overlay = true;

        let eff_data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>();
        let overlay_data: &mut DataContainer<T> = self.overlay_data.register::<DataContainer<T>>();
        overlay_data.storage.entry(entity).or_insert_with(|| eff_data.storage.get(&entity).cloned().unwrap_or_else(||T::default()))
    }

    pub fn clear_overlay(&mut self) {
        self.overlay_data.clear();
        self.has_overlay = false;
    }

    pub fn world_data_opt<T: EntityData>(&self) -> Option<&T> {
        self.data_opt::<T>(self.self_entity)
    }

    pub fn world_data<T: EntityData>(&self) -> &T {
        self.data::<T>(self.self_entity)
    }

    pub fn entities_with_data<T : EntityData>(&self) -> &HashMap<Entity, T> {
        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>();
        &data.storage
    }

    pub fn entity_by_key<I : Hash + Eq + Clone + 'static>(&self, key : &I) -> Option<Entity> {
        let index : &EntityIndex<I> = self.entity_indices.get::<EntityIndex<I>>();
        index.index.get(key).cloned()
    }

    pub fn entity_index<I: Hash + Eq + Clone + 'static>(&self) -> &EntityIndex<I> {
        self.entity_indices.get::<EntityIndex<I>>()
    }

    pub fn has_world_data<T : EntityData>(&self) -> bool {
        self.has_data::<T>(self.self_entity)
    }
    pub fn has_data<T : EntityData>(&self, entity : Entity) -> bool {
        self.has_data_r::<T>(&entity)
    }
    pub fn has_data_r<T : EntityData>(&self, entity : &Entity) -> bool {
        if self.has_overlay {
            if let Some(overlay) = self.overlay_data.get_opt::<DataContainer<T>>() {
                if overlay.storage.contains_key(&entity) {
                    return true;
                }
            }
        }

        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>();
        data.storage.contains_key(&entity)
    }

    pub fn events<E : GameEventType + 'static>(&self) -> impl Iterator<Item=&GameEventWrapper<E>> {
        self.events.events::<E>()
    }
    pub fn events_most_recent_first<E: GameEventType + 'static>(&self) -> impl Iterator<Item=&GameEventWrapper<E>> {
        self.events.revents::<E>()
    }

    pub fn most_recent_event<E: GameEventType + 'static>(&self) -> &GameEventWrapper<E> {
        self.events.most_recent_event::<E>()
    }
}