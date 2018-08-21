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
use events::GameEventState;
use std::collections::HashSet;


/// world views are views into the data of a world. The world itself is the ledger of changes, the view is a way of looking at it at a specific time.
/// world views can be made counter-factual by layering modifications on top of them, or by modifying their data directly in place. Once they have been
///

pub struct WorldView {
    pub(crate) entities: Vec<EntityContainer>,
    pub(crate) entity_set: HashSet<Entity>,
    pub(crate) self_entity : Entity,
    pub(crate) constant_data: Map<CloneAny>,
    pub(crate) effective_data: Map<CloneAny>,
    pub(crate) overlay_data: Map<CloneAny>,
    pub current_time: GameEventClock,
    pub(crate) modifier_cursor: ModifierClock,
    pub(crate) modifier_indices: HashMap<TypeId, usize>,
    pub(crate) events: MultiTypeEventContainer,
    pub entity_indices: Map<CloneAny>,
    pub(crate) has_overlay: bool,
}




impl WorldView {
    pub fn data<T: EntityData>(&self, entity: Entity) -> &T {
        if self.has_overlay {
            if let Some(overlay) = self.overlay_data.get::<DataContainer<T>>() {
                if let Some(overlaid) = overlay.storage.get(&entity) {
                    return overlaid;
                }
            }
        }

        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>()
            .unwrap_or_else(|| panic!(format!("Could not retrieve effective data for entity {:?}, looking for data {:?}", entity, unsafe {std::intrinsics::type_name::<T>()})));

        match data.storage.get(&entity) {
            Some(t) => t,
            None => &data.sentinel
        }
    }
    pub fn data_opt<T: EntityData>(&self, entity: Entity) -> Option<&T> {
        if self.has_overlay {
            if let Some(overlay) = self.overlay_data.get::<DataContainer<T>>() {
                if let Some(overlaid) = overlay.storage.get(&entity) {
                    return Some(overlaid);
                }
            }
        }

        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>()
            .unwrap_or_else(|| panic!(format!("Could not retrieve effective data for entity {:?}, looking for data {:?}", entity, unsafe {std::intrinsics::type_name::<T>()})));
        data.storage.get(&entity)
    }

    pub fn data_mut<T: EntityData>(&mut self, entity: Entity) -> &mut T {
        self.has_overlay = true;

        let eff_data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>().unwrap();
        let overlay_data: &mut DataContainer<T> = self.overlay_data.entry::<DataContainer<T>>().or_insert_with(||DataContainer::<T>::new());
        overlay_data.storage.entry(entity).or_insert_with(|| eff_data.storage.get(&entity).cloned().unwrap_or_else(||T::default()))
    }

    pub fn clear_overlay(&mut self) {
        self.overlay_data.clear();
        self.has_overlay = false;
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
        if self.has_overlay {
            if let Some(overlay) = self.overlay_data.get::<DataContainer<T>>() {
                if overlay.storage.contains_key(&entity) {
                    return true;
                }
            }
        }

        let data: &DataContainer<T> = self.effective_data.get::<DataContainer<T>>()
            .unwrap_or_else(|| panic!(format!("Could not retrieve effective data for entity {:?}, looking for data {:?}", entity, unsafe {std::intrinsics::type_name::<T>()})));
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