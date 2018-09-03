use std::ops;
use std::marker::PhantomData;
use common::hex::*;
use common::prelude::*;
use common::multitype::MultiTypeContainer;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::RefMut;
use std::hash::Hash;
use std::collections::hash_map;
use std::collections::hash_set;
use events::GameEventWrapper;
use core::*;
use std::any::Any;
use anymap::any::CloneAny;
use std::any::TypeId;
use anymap::AnyMap;
use anymap::Map;
use std::cell::UnsafeCell;
use std::fmt;
use std::fmt::Debug;
use common::Field;
use modifiers::*;
use world::storage::*;
use world::WorldView;
use entity::Entity;
use entity::EntityData;
use storage::MultiTypeEventContainer;
use events::GameEventType;
use events::CoreEvent;
use events::GameEventState;
use std::intrinsics::type_name;
use std::ops::Deref;
use entity::DebugData;
use rand::StdRng;
use rand::SeedableRng;
use backtrace::Backtrace;
use events::CoreEvent::DataRegistered;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde::Deserializer;
use serde::de::Visitor;
use serde::Serializer;
use serde::de::SeqAccess;
use serde::de;
use serde::de::MapAccess;

pub struct ModifiersApplication {
    disable_func: fn(&mut World, ModifierReference),
    reset_func: fn(&World, &mut WorldView),
    recompute_for_disabled_modifiers: fn(&World, &mut WorldView, GameEventClock, GameEventClock),
    apply_func: fn(&World, &mut WorldView, usize, ModifierClock, GameEventClock, bool) -> Option<usize>,
    remove_entity_func: fn(&mut WorldView, Entity),
    bootstrap_entity_func: fn(&World, &mut WorldView, Entity),
    register_func: fn(&mut WorldView),
    registered_at: GameEventClock,
}

pub struct IndexApplication {
    index_func: Rc<Fn(&World, &mut WorldView)>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierReferenceType {
    Permanent,
    Dynamic,
    Archetype,
    Sentinel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifierReference(pub(crate) usize, pub(crate) ModifierReferenceType, pub(crate) usize);

impl ModifierReference {
    pub fn sentinel() -> ModifierReference { ModifierReference(0, ModifierReferenceType::Sentinel, 0) }
    pub fn as_opt(&self) -> Option<&ModifierReference> {
        if self.is_sentinel() {
            None
        } else {
            Some(self)
        }
    }
    pub fn is_sentinel(&self) -> bool {
        match self.1 {
            ModifierReferenceType::Sentinel => true,
            _ => false
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct World {
    pub(crate) entities: Vec<EntityContainer>,
    pub self_entity: Entity,
    pub data: MultiTypeContainer,
    pub modifiers: MultiTypeContainer,
    pub total_modifier_count: ModifierClock,
    pub total_dynamic_modifier_count: ModifierClock,
    pub next_time: GameEventClock,
    pub(crate) events: MultiTypeEventContainer,
    pub entity_indices: MultiTypeContainer,
    pub entity_id_counter : usize,
    // runtime only -----------------------------------------------------------------------
    #[serde(skip_serializing, skip_deserializing)]
    pub view: UnsafeCell<WorldView>,
    #[serde(skip_serializing, skip_deserializing)]
    pub modifier_application_by_type: hash_map::HashMap<TypeId, ModifiersApplication>,
    #[serde(skip_serializing, skip_deserializing)]
    pub index_applications: Vec<IndexApplication>,
}


impl World {
    pub fn new() -> World {
        let self_ent = Entity(1);

        let mut world = World {
            entities: vec![],
            self_entity: self_ent,
            data: MultiTypeContainer::new(),
            modifiers: MultiTypeContainer::new(),
            total_modifier_count: 0,
            total_dynamic_modifier_count: 0,
            next_time: 0,
            events: MultiTypeEventContainer::new(),
            view: UnsafeCell::new(WorldView {
                entities: vec![],
                entity_set: HashSet::new(),
                self_entity: self_ent,
                constant_data: MultiTypeContainer::new(),
                effective_data: MultiTypeContainer::new(),
                overlay_data: MultiTypeContainer::new(),
                current_time: 0,
                events: MultiTypeEventContainer::new(),
                modifier_cursor: 0,
                modifier_indices: hash_map::HashMap::new(),
                entity_indices: MultiTypeContainer::new(),
                has_overlay: false,
            }),
            modifier_application_by_type: hash_map::HashMap::new(),
            entity_indices: MultiTypeContainer::new(),
            index_applications: vec![],
            entity_id_counter: 2
        };

        world.register_core_types();

        world
    }

    /// must be called after deserializing a world for it to work properly
    pub fn initialize_loaded_world(&mut self){
        self.initialize_internal_view();
        self.register_core_types();
    }

    pub(crate) fn register_core_types(&mut self) {
        self.register_event_type::<CoreEvent>();
        self.register::<DebugData>();
    }

    pub fn current_time(&self) -> GameEventClock {
        let raw = self.next_time as i64 - 1;
        if raw < 0 { 0 } else { raw as u64 }
    }

    pub fn register_index<I: Default + Hash + Eq + Clone + 'static + Serialize + DeserializeOwned>(&mut self) {
        self.entity_indices.register::<EntityIndex<I>>();
        self.mut_view().entity_indices.register::<EntityIndex<I>>();

        let index_func = |world: &World, view: &mut WorldView| {
            let world_index: &EntityIndex<I> = world.entity_indices.get::<EntityIndex<I>>();
            let view_index: &mut EntityIndex<I> = view.entity_indices.get_mut::<EntityIndex<I>>();
            view_index.update_from(world_index);
        };

        self.index_applications.push(IndexApplication {
            index_func: Rc::new(index_func)
        });
    }

    pub fn register_event_type<E: GameEventType + 'static + Serialize + Default>(&mut self) where for<'de> E: serde::Deserialize<'de> {
        self.events.register_event_type::<E>();
        self.mut_view().events.register_event_type::<E>();
    }

    pub(crate) fn modifiers_container<T: EntityData>(&self) -> &ModifiersContainer<T> {
        self.modifiers.get::<ModifiersContainer<T>>()
    }

    pub(crate) fn initialize_internal_view(&mut self) {
        self.view = UnsafeCell::new(WorldView {
            entities: vec![],
            entity_set: HashSet::new(),
            self_entity: self.self_entity,
            constant_data: MultiTypeContainer::new(),
            effective_data: MultiTypeContainer::new(),
            overlay_data: MultiTypeContainer::new(),
            current_time: 0,
            events: MultiTypeEventContainer::new(),
            modifier_cursor: 0,
            modifier_indices: hash_map::HashMap::new(),
            entity_indices: MultiTypeContainer::new(),
            has_overlay: false,
        });
        let mut_view = self.mut_view();
        self.update_view_to_time_intern(mut_view, self.next_time-1, true);
    }

    pub fn register<T: EntityData>(&mut self) where T: DeserializeOwned {
        self.data.register::<DataContainer<T>>();
        self.modifiers.register::<ModifiersContainer<T>>();

        let register_func = |view: &mut WorldView| {
            if view.constant_data.contains::<DataContainer<T>>() {
                error!("Registered {} twice, that's not good, we're hitting it more often than expected", typename::<T>());
            }
            view.constant_data.register::<DataContainer<T>>();
            view.effective_data.register::<DataContainer<T>>();
        };

        let disable_func = |world: &mut World, modifier_ref: ModifierReference| {
            let all_modifiers: &mut ModifiersContainer<T> = world.modifiers.get_mut::<ModifiersContainer<T>>();
            let ModifierReference(modifier_clock, modifier_type, index) = modifier_ref;
            match modifier_type {
                ModifierReferenceType::Dynamic => {
                    panic!("Disabling dynamic modifiers not re-implemented yet");
//                    all_modifiers.dynamic_modifiers.get_mut(index).expect("cannot disable a non-existent modifier").disabled_at = Some(world.next_time);
                }
                ModifierReferenceType::Permanent => {
                    if let Some((index, modifier)) = all_modifiers.modifiers.iter_mut().enumerate().find(|(i, e)| e.modifier_index == modifier_clock) {
                        modifier.disabled_at = Some(world.next_time);
                        all_modifiers.modifiers_by_disabled_at.entry(world.next_time).or_insert_with(|| Vec::new()).push(index);
                    }
//                    trace!("Disabling modifier with reference {:?} and marking disabled at to {:?}", modifier_ref, world.next_time);
//                    let modifier = all_modifiers.modifiers.get_mut(index).expect("cannot disable a non-existent modifier").disabled_at = Some(world.next_time);
//                    all_modifiers.modifiers_by_disabled_at.entry(world.next_time).or_insert_with(|| Vec::new()).push(index);
                }
                ModifierReferenceType::Archetype => { warn!("it makes no sense to attempt to disable a modifier archetype") }
                ModifierReferenceType::Sentinel => { warn!("removing a sentinel reference is a no-op") }
            }
        };

        let recompute_for_disabled_modifiers_between = |world: &World, view: &mut WorldView, start: GameEventClock, end: GameEventClock| {
            let all_modifiers: &ModifiersContainer<T> = world.modifiers.get::<ModifiersContainer<T>>();

            let mut entities_to_recompute = HashSet::new();
            let empty_vec = Vec::new();
            for time in start..=end {
                for modifier_index in all_modifiers.modifiers_by_disabled_at.get(&time).unwrap_or(&empty_vec) {
                    let entity = all_modifiers.modifiers.get(*modifier_index).expect("modifier referenced by disabled at must exist").entity;
                    entities_to_recompute.insert(entity);
                }
            }

            if entities_to_recompute.len() > 0 {
                trace!("Entities to recompute due to disabled modifiers : {:?}", entities_to_recompute);
            }

            for entity in entities_to_recompute {
                let mut raw_data: T = world.data.get::<DataContainer<T>>()
                    .storage
                    .get(&entity)
                    .unwrap_or_else(|| panic!(format!("Attempt to recompute data that has not been attached to entity: {:?}", entity)))
                    .clone();

                trace!("Raw data for recomputation: {:?}", raw_data);
                // check if this entity has dynamic modifiers for this data type
                let is_dynamic = all_modifiers.dynamic_entity_set.contains(&entity);
                // if it does, we will write to constant data, then top up with the dynamics afterwards. If it's not dynamic we can write straight to the effective data

                // NB: right now, when recomputing it will be operating against the unchanged world view. Non-dynamic modifiers are not allowed to _look_ at the world view,
                // so that should be fine, but if they do, they'll get weird results
                for modifier in all_modifiers.constant_modifiers_for_entity(entity) {
                    // if the modifier in question has not yet been disabled
                    if modifier.applied_at <= end && modifier.disabled_at.unwrap_or(MAX_GAME_EVENT_CLOCK) > end {
                        modifier.modifier.modify(&mut raw_data, view);
                    }
                }
                trace!("Data after relevant modifiers applied: {:?}", raw_data);

                if is_dynamic {
                    // clone off what we have so far for the constant data section and insert it
                    let constant_data = raw_data.clone();
                    let constant_data_storage = &mut view.constant_data.get_mut::<DataContainer<T>>().storage;
                    constant_data_storage.insert(entity, constant_data);

                    // then recompute all the dynamics. For the moment this is pretty much just the same as the non-dynamic modifiers
                    for dyn_modifier in all_modifiers.dynamic_modifiers_for_entity(entity) {
                        if dyn_modifier.applied_at <= end && dyn_modifier.disabled_at.unwrap_or(MAX_GAME_EVENT_CLOCK) > end {
                            dyn_modifier.modifier.modify(&mut raw_data, view);
                        }
                    }

                    // insert into the effective data storage
                    let effective_data_storage = &mut view.effective_data.get_mut::<DataContainer<T>>().storage;
                    effective_data_storage.insert(entity, raw_data);
                } else {
                    // no need for a clone here, just insert the raw data and we're done
                    let effective_data_storage = &mut view.effective_data.get_mut::<DataContainer<T>>().storage;
                    effective_data_storage.insert(entity, raw_data);
                }
            }
        };

        let reset_func = |world: &World, view: &mut WorldView| {
            let all_modifiers: &ModifiersContainer<T> = world.modifiers.get::<ModifiersContainer<T>>();

            // everything remains in effective_data_storage only, until such time as there is a dynamic modifier on that data, then effective is copied into constant,
            // and all further non-dynamic modifications are made there, all dynamic modifications are made to the effective data, which is reset from constant at each
            // recomputation
            let constant_data_storage = &mut view.constant_data.get_mut::<DataContainer<T>>().storage;
            let effective_data_storage = &mut view.effective_data.get_mut::<DataContainer<T>>().storage;

            let world_data: &DataContainer<T> = world.data.get::<DataContainer<T>>();
            let new_entities = world_data.entities_with_data.iter().rev().take_while(|e| !effective_data_storage.contains_key(*e)).collect_vec();
            for new_ent in new_entities {
                effective_data_storage.insert(*new_ent, world_data.storage.get(new_ent).expect("entities with data did not align with actual storage").clone());
            }

            for entity_id in &all_modifiers.dynamic_entity_set {
                if constant_data_storage.contains_key(entity_id) {
                    let existing_data = constant_data_storage.get(entity_id).expect("existing constant data not present").clone();
                    effective_data_storage.insert(*entity_id, existing_data);
                } else {
                    // this handles the initial case when we switch from non-dynamic to dynamic. Entirely non-dynamic entities only use the effective_data
                    // storage, so that has the up to date constant-modified data. We need to pull that _into_ constant data to keep track of it now that
                    // we're going to be tracking those separately
                    constant_data_storage.insert(*entity_id, effective_data_storage.get(entity_id).expect("could not instantiate constant from effective").clone());
                };
            }
        };

        let apply_func = |world: &World, view: &mut WorldView, i: usize, modifier_cursor: ModifierClock, at_time: GameEventClock, is_dynamic: bool| {
            let all_modifiers: &ModifiersContainer<T> = world.modifiers.get::<ModifiersContainer<T>>();

            let relevant_modifiers = match is_dynamic {
                true => &all_modifiers.dynamic_modifiers,
                false => &all_modifiers.modifiers
            };


            match relevant_modifiers.get(i) {
                None => None, // out of bounds, we're done
                Some(wrapper) => {
                    trace!("[{:?}] Examining relevant modifier {:?}, {:?}    {:?}, {:?}", (if is_dynamic { "dynamic" } else { "constant" }), wrapper.modifier_index, modifier_cursor, wrapper.applied_at, at_time);
                    if wrapper.modifier_index != modifier_cursor {
                        trace!("In bounds, but current modifier in this set is not the one we're looking for: {}, {}, {}", wrapper.modifier_index, modifier_cursor, is_dynamic);
                        // in bounds, but the current modifier in this set is not the one we're looking for in our total monotonic order, so don't advance
                        Some(i)
                    } else if wrapper.applied_at > at_time {
                        trace!("In bounds, at right cursor point, but past time, so done: {}", is_dynamic);
                        // in bounds, and the modifier at the right cursor point, but past time, so we're done
                        None
                    } else {
                        // this is the correct one in total monotonic order, and not past time, so process it.
                        // because of the borrowing rules, we can't modify it in place, since the modify function needs a reference to the world view itself
                        // so instead we make a clone, modify it, then overwrite it into the map. This is unlikely to be very efficient, we will probably
                        // want to improve it. It may be moot though, depending on the extent to which we can just use incrementals

                        if wrapper.disabled_at.unwrap_or(MAX_GAME_EVENT_CLOCK) > at_time {
                            trace!("Active and ready: {}", is_dynamic);
                            let ent_has_dynamic_data = all_modifiers.dynamic_entity_set.contains(&wrapper.entity);

                            // pull from effective data if we're calculating dynamics, or if there is no dynamic data on this entity at all, in which case we always look at
                            // effective

                            let effective_data = &mut view.effective_data;
                            let constant_data = &mut view.constant_data;

                            let mut ent_data: T = match is_dynamic || !ent_has_dynamic_data {
                                true => effective_data.get_mut::<DataContainer<T>>().storage
                                    //.entry(wrapper.entity.0).or_insert(T::default()).clone(),
                                    .entry(wrapper.entity)
                                    .or_insert_with(|| {
                                        if is_dynamic {
                                            constant_data.get::<DataContainer<T>>().storage
                                                .get(&wrapper.entity).expect("dynamic modifier could not pull baseline constant data to work from").clone()
                                        } else {
                                            world.raw_data::<T>(wrapper.entity).clone()
                                        }
                                    })
                                    .clone(),
                                false => constant_data.get_mut::<DataContainer<T>>().storage
                                    //.entry(wrapper.entity).or_insert(T::default()).clone()
//                                    .get(&wrapper.entity).unwrap_or_else(|| panic!(format!("Could not retrieve constant data for modified entity {}", wrapper.entity))).clone()
                                    .entry(wrapper.entity)
                                    .or_insert_with(|| { world.raw_data::<T>(wrapper.entity).clone() })
                                    .clone(),
                            };

                            wrapper.modifier.modify(&mut ent_data, view);
                            trace!("[{:?}] Modified ent data, new value is {:?}", (if is_dynamic { "dynamic" } else { "constant" }), ent_data);

                            match is_dynamic || !ent_has_dynamic_data {
                                true => view.effective_data.get_mut::<DataContainer<T>>().storage.entry(wrapper.entity).and_modify(|e| { *e = ent_data.clone() }),
                                false => view.constant_data.get_mut::<DataContainer<T>>().storage.entry(wrapper.entity).and_modify(|e| { *e = ent_data.clone() })
                            };
                        } else {
                            trace!("Not active, no action: {}", is_dynamic);
                        }

                        Some(i + 1)
                    }
                }
            }
        };

        let remove_entity_func = |view: &mut WorldView, entity: Entity| {
            view.effective_data.get_mut::<DataContainer<T>>().storage.remove(&entity);
            view.constant_data.get_mut::<DataContainer<T>>().storage.remove(&entity);
        };

        let bootstrap_entity_func = |world: &World, view: &mut WorldView, entity: Entity| {
            if let Some(existing_data) = world.raw_data_opt::<T>(entity) {
                view.effective_data.get_mut::<DataContainer<T>>().storage.insert(entity, existing_data.clone());
            }
        };

        self.modifier_application_by_type.insert(TypeId::of::<T>(), ModifiersApplication {
            disable_func: (disable_func),
            reset_func: (reset_func),
            recompute_for_disabled_modifiers: (recompute_for_disabled_modifiers_between),
            apply_func: (apply_func),
            remove_entity_func,
            bootstrap_entity_func,
            register_func,
            registered_at: self.next_time,
        });

        self.add_event(DataRegistered);
    }

    /// Returns a view of this world that will be kept continuously up to date
    pub fn view<'a, 'b>(&'a self) -> &'b WorldView {
        unsafe { &*self.view.get() }
    }

    fn mut_view(&self) -> &mut WorldView {
        unsafe { &mut *self.view.get() }
    }

    pub fn view_at_time(&self, at_time: GameEventClock) -> WorldView {
        let entities = self.entities.iter().filter(|e| e.1 <= at_time).cloned().collect_vec();
        let entity_set: HashSet<Entity> = entities.iter().map(|e| e.0).collect();
        let mut new_view = WorldView {
            entity_set,
            entities,
            self_entity: self.self_entity,
            constant_data: self.data.clone(),
            effective_data: self.data.clone(),
            overlay_data: MultiTypeContainer::new(),
            current_time: 0,
            events: self.events.clone_events_up_to(at_time),
            modifier_cursor: 0,
            modifier_indices: hash_map::HashMap::new(),
            entity_indices: self.entity_indices.clone(),
            has_overlay: false,
        };

        for EntityContainer(entity, time) in self.entities.iter().skip_while(|e| e.1 <= at_time) {
            for (type_id, application_capability) in &self.modifier_application_by_type {
                trace!("Removing entity that was created after {:?}, [{:?}]", at_time, time);
                (application_capability.remove_entity_func)(&mut new_view, *entity);
            }
        }

        self.update_view_to_time_intern(&mut new_view, at_time, true);

        new_view
    }

    pub fn update_view_to_time(&self, view: &mut WorldView, at_time: GameEventClock) {
        self.update_view_to_time_intern(view, at_time, false);
    }

    fn update_view_to_time_intern(&self, view: &mut WorldView, at_time: GameEventClock, is_init: bool) {
        let cur_time = view.current_time;
        trace!("{:?} view-------------------------------------------", (if is_init { "Initializing" } else { "Updating" }));
        if cur_time >= at_time {
            trace!("\tShort circuit");
            return;
        }

        self.index_applications.iter().for_each(|idx| (idx.index_func)(self, view));

        if !is_init {
            for (type_id, application_capability) in &self.modifier_application_by_type {
                if application_capability.registered_at <= at_time && (application_capability.registered_at > view.current_time || view.current_time == 0) {
                    (application_capability.register_func)(view);
                }
            }
        }

        view.events.update_events_to(&self.events, at_time);

        let existing_set = &view.entity_set;
        let new_entities: Vec<EntityContainer> = self.entities.iter().rev()
            .skip_while(|e| e.1 >= at_time)
            .take_while(|e| e.1 >= cur_time)
            .filter(|e| !existing_set.contains(&e.0))
            .cloned()
            .collect();

        new_entities.into_iter().rev().for_each(|e| {
            trace!("Bootstrapping entity that was created after {:?}, id: {:?}, [{:?}]", cur_time, e.0, e.1);
            for (type_id, application_capability) in &self.modifier_application_by_type {
                if application_capability.registered_at <= at_time {
                    (application_capability.bootstrap_entity_func)(self, view, e.0);
                }
            }
            view.entity_set.insert(e.0);
            view.entities.push(e);
        });


        // we need to keep track of where we are in each modifier type, as well as the global modifier cursor.
        // we continuously iterate the modifier cursor, asking each to apply the active modifier cursor. At
        // each point only one will actually do so, since there is only one modifier at a given cursor point.
        // If we reach a point where none applied anything, then we can assume we have reached the end and are
        // done.

        // set up a vector of walkers, each being an application function and a current index
        let mut walkers = vec![];
        for (type_id, application_capability) in &self.modifier_application_by_type {
            if application_capability.registered_at <= at_time {
                let current_index = view.modifier_indices.get(type_id).map(|i| *i as usize).unwrap_or(0);
                trace!("Pulling current_index from past run: {}", current_index);
                walkers.push((application_capability.apply_func.clone(), Some(current_index), type_id));
            }
        }

        loop {
            let mut any_found = false;
            for walker in &mut walkers {
                let start_i = walker.1;
                match start_i {
                    Some(i) => {
                        let func = &walker.0;
                        let cur_cursor = view.modifier_cursor;
                        let new_i: Option<usize> = func(self, view, i, cur_cursor, at_time, false);
                        // if the new index is different than the previous index we actually processed something
                        // and we can mark as having found something, as well as advance that walker
                        if new_i != start_i {
                            walker.1 = new_i;
                            match new_i {
                                Some(i) => {
                                    trace!("Modified a thing {:?}", i);
                                    view.modifier_indices.insert(*walker.2, i);
                                    any_found = true;
                                }
                                None => ()
                            };
                        }
                    }
                    None => ()
                }
            }
            // if none processed the event that means we're either past the maximum modifier cursor or
            // the modifier at that cursor is past our time point
            if !any_found {
                break;
            } else {
                view.modifier_cursor += 1;
            }
        }


        let mut walkers = vec![];
        for (type_id, application_capability) in &self.modifier_application_by_type {
            if application_capability.registered_at <= at_time {
                let current_index = 0;
                walkers.push((application_capability.apply_func.clone(), Some(current_index), type_id));
                if !is_init {
                    (application_capability.reset_func)(self, view);
                }
            }
        }

        let mut dynamic_cursor = 0;
        loop {
            let mut any_found = false;
            for walker in &mut walkers {
                let start_i = walker.1;
                match start_i {
                    Some(i) => {
                        let func = &walker.0;
                        let new_i: Option<usize> = func(self, view, i, dynamic_cursor, at_time, true);
                        // if the new index is different than the previous index we actually processed something
                        // and we can mark as having found something, as well as advance that walker
                        if new_i != start_i {
                            walker.1 = new_i;
                            any_found = true;
                        }
                    }
                    None => ()
                }
            }
            // if none processed the event that means we're either past the maximum modifier cursor or
            // the modifier at that cursor is past our time point
            if !any_found {
                break;
            } else {
                dynamic_cursor += 1;
            }
        }

        for (type_id, application_capability) in &self.modifier_application_by_type {
            if application_capability.registered_at <= at_time {
                (application_capability.recompute_for_disabled_modifiers)(self, view, view.current_time, at_time);
            }
        }

        view.current_time = at_time;
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(EntityContainer(entity, self.next_time));
        self.add_event(CoreEvent::EntityAdded(entity));
    }

    pub fn index_entity<I: Hash + Eq + Clone + 'static>(&mut self, entity: Entity, key: I) {
        let index: &mut EntityIndex<I> = self.entity_indices.get_mut::<EntityIndex<I>>();
        index.index.insert(key, entity);
    }

    pub fn modify<T: EntityData, S: Into<Option<Str>>>(&mut self, entity: Entity, modifier: Box<Modifier<T>>, description: S) -> ModifierReference {
        self.add_modifier(entity, modifier, description)
    }
    pub fn modify_world<T: EntityData, S: Into<Option<Str>>>(&mut self, modifier: Box<Modifier<T>>, description: S) -> ModifierReference {
        self.add_world_modifier(modifier, description)
    }

    pub fn add_modifier<T: EntityData, S: Into<Option<Str>>>(&mut self, entity: Entity, modifier: Box<Modifier<T>>, description: S) -> ModifierReference {
        let all_modifiers: &mut ModifiersContainer<T> = self.modifiers.get_mut::<ModifiersContainer<T>>();
        if modifier.modifier_type() == ModifierType::Dynamic {
            let index = all_modifiers.dynamic_modifiers.len();
            all_modifiers.dynamic_modifiers.push(ModifierContainer {
                modifier: modifier.into(),
                applied_at: self.next_time,
                disabled_at: None,
                modifier_index: self.total_dynamic_modifier_count,
                entity,
                description: description.into().map(|s| String::from(s)),
            });
            all_modifiers.dynamic_entity_set.insert(entity);
            self.total_dynamic_modifier_count += 1;
            ModifierReference(self.total_dynamic_modifier_count - 1, ModifierReferenceType::Dynamic, index)
        } else {
            let index = all_modifiers.modifiers.len();
            all_modifiers.modifiers.push(ModifierContainer {
                modifier: modifier.into(),
                applied_at: self.next_time,
                disabled_at: None,
                modifier_index: self.total_modifier_count,
                entity,
                description: description.into().map(|s| String::from(s)),
            });
            trace!("Creating modifier with count {}, incrementing", self.total_modifier_count);
            self.total_modifier_count += 1;
            ModifierReference(self.total_modifier_count - 1, ModifierReferenceType::Permanent, index)
        }
    }

    pub fn disable_modifier(&mut self, modifier_ref: ModifierReference) {
        for disable_func in self.modifier_application_by_type.values().map(|c| c.disable_func).clone().collect_vec() {
            (disable_func)(self, modifier_ref.clone());
        }
//        let application_capabilities = self.modifier_application_by_type.get(&modifier_ref.0).expect("attempted to disable modifier of unregistered data type, should be impossible");
//        (application_capabilities.disable_func)(self, modifier_ref);
    }

    pub fn add_world_modifier<T: EntityData, S: Into<Option<Str>>>(&mut self, modifier: Box<Modifier<T>>, description: S) -> ModifierReference {
        let tmp = self.self_entity;
        self.add_modifier::<T, Option<Str>>(tmp, modifier, description.into())
    }

//    pub fn add_constant_modifier<T: EntityData, CM: ConstantModifier<T> + 'static>(&mut self, entity: Entity, constant_modifier: CM) {
//        self.add_modifier(entity, box ConstantModifierWrapper { inner: constant_modifier, _ignored: PhantomData }, None);
//    }
//
//    pub fn add_limited_modifier<T: EntityData, CM: LimitedModifier<T> + 'static>(&mut self, entity: Entity, limited_modifier: CM) {
//        self.add_modifier(entity, box LimitedModifierWrapper { inner: limited_modifier, _ignored: PhantomData }, None);
//    }
//
//    pub fn add_dynamic_modifier<T: EntityData, CM: DynamicModifier<T> + 'static>(&mut self, entity: Entity, dynamic_modifier: CM) {
//        self.add_modifier(entity, box DynamicModifierWrapper { inner: dynamic_modifier, _ignored: PhantomData }, None);
//    }
//
//    pub fn add_constant_world_modifier<T: EntityData, CM: ConstantModifier<T> + 'static>(&mut self, constant_modifier: CM) {
//        let entity = self.self_entity;
//        self.add_modifier(entity, box ConstantModifierWrapper { inner: constant_modifier, _ignored: PhantomData }, None);
//    }
//
//    pub fn add_limited_world_modifier<T: EntityData, CM: LimitedModifier<T> + 'static>(&mut self, limited_modifier: CM) {
//        let entity = self.self_entity;
//        self.add_modifier(entity, box LimitedModifierWrapper { inner: limited_modifier, _ignored: PhantomData }, None);
//    }
//
//    pub fn add_dynamic_world_modifier<T: EntityData, CM: DynamicModifier<T> + 'static>(&mut self, dynamic_modifier: CM) {
//        let entity = self.self_entity;
//        self.add_modifier(entity, box DynamicModifierWrapper { inner: dynamic_modifier, _ignored: PhantomData }, None);
//    }

    pub fn add_callback<E: GameEventType + 'static>(&mut self, event_callback: EventCallback<E>) {
        self.events.add_callback(event_callback);
    }

    pub fn push_event<E: GameEventType + 'static>(&mut self, event: E, state: GameEventState) {
        let wrapper = GameEventWrapper::new(event, state, self.next_time);

        let callbacks = self.events.push_event(wrapper.clone());
        self.update_view_to_time(self.mut_view(), self.next_time);
        self.next_time += 1;

        for callback in callbacks {
            callback(self, &wrapper);
        }
    }

    pub fn add_event<E: GameEventType + 'static>(&mut self, event: E) {
        self.push_event(event.clone(), GameEventState::StartedAndEnded);
    }
    pub fn start_event<E: GameEventType + 'static>(&mut self, event: E) {
        self.push_event(event, GameEventState::Started);
    }
    pub fn end_event<E: GameEventType + 'static>(&mut self, event: E) {
        self.push_event(event, GameEventState::Ended);
    }
    pub fn continue_event<E: GameEventType + 'static>(&mut self, event: E) {
        self.push_event(event, GameEventState::Continuing);
    }

    pub fn event_at<E: GameEventType + 'static>(&self, time: GameEventClock) -> Option<&GameEventWrapper<E>> {
        self.events.events::<E>().find(|e| e.occurred_at == time)
    }

    pub fn ensure_data<T: EntityData>(&mut self, entity: Entity) {
        if !self.has_data::<T>(entity) {
            self.attach_data::<T>(entity, T::default());
        }
    }

    pub fn ensure_world_data<T: EntityData>(&mut self) {
        self.ensure_data::<T>(self.self_entity);
    }

    pub fn attach_data<T: EntityData>(&mut self, entity: Entity, data: T) {
        let self_data: &mut DataContainer<T> = self.data.get_mut::<DataContainer<T>>();
        self_data.entities_with_data.push(entity);
        if let Some(prev) = self_data.storage.insert(entity, data) {
            error!("Attached data <{:?}> multiple times, that's going to super-break stuff {:?}", typename::<T>(), Backtrace::new());
        }
    }

    pub fn attach_world_data<T: EntityData>(&mut self, data: T) {
        let ent = self.self_entity;
        self.attach_data(ent, data);
    }

    pub fn create_entity(&mut self) -> Entity {
        let id = self.entity_id_counter;
        self.entity_id_counter += 1;
        Entity(id)
    }


    pub fn random_seed(&self, extra: usize) -> Vec<usize> {
        vec![extra, self.next_time as usize]
    }

    pub fn random(&self, extra: usize) -> StdRng {
        let seed = self.random_seed(extra);
        let rng: StdRng = SeedableRng::from_seed(seed.as_slice());
        rng
    }

    pub fn raw_data<T: EntityData>(&self, entity: Entity) -> &T {
        self.raw_data_opt::<T>(entity).unwrap_or_else(|| panic!(format!("Attempted to get raw data of type {:?}, but entity {:?} had none", unsafe { std::intrinsics::type_name::<T>() }, entity)))
    }
    pub fn raw_data_opt<T: EntityData>(&self, entity: Entity) -> Option<&T> {
        self.data.get::<DataContainer<T>>().storage.get(&entity)
    }

    pub fn world_data_mut<T: EntityData>(&mut self) -> &T {
        self.data.get_mut::<DataContainer<T>>().storage.entry(self.self_entity).or_insert_with(|| T::default())
    }
}


impl World {
    pub fn permanent_field_logs_for<T: EntityData>(&self, ent: Entity) -> FieldLogs<T> {
        self.field_logs_with_condition_for::<T>(ent, |m| m.modifier.modifier_type() == ModifierType::Permanent, self.next_time)
    }
    pub fn non_permanent_field_logs_for<T: EntityData>(&self, ent: Entity) -> FieldLogs<T> {
        self.field_logs_with_condition_for::<T>(ent, |m| m.modifier.modifier_type() != ModifierType::Permanent, self.next_time)
    }
    pub fn field_logs_for<T: EntityData>(&self, ent: Entity) -> FieldLogs<T> {
        self.field_logs_with_condition_for::<T>(ent, |m| true, self.next_time)
    }

    pub fn field_logs_with_condition_for<T: EntityData>(&self, ent: Entity, condition: fn(&ModifierContainer<T>) -> bool, at_time: GameEventClock) -> FieldLogs<T> {
        let container = self.modifiers_container::<T>();
        let data_container: &DataContainer<T> = self.data.get::<DataContainer<T>>();
        let raw_data = data_container.storage.get(&ent).unwrap_or_else(|| &data_container.sentinel).clone();
        FieldLogs {
            field_modifications: container.modifiers.iter()
                .filter(move |m| m.is_active_at_time(at_time) && m.entity == ent)
                .filter(|m| (condition)(m))
                .flat_map(|m| {
                    let mut field_modifications = m.modifier.modified_fields();
                    if m.description.is_some() {
                        for field_mod in &mut field_modifications {
                            if field_mod.description.is_none() {
                                field_mod.description = m.description.clone();
                            }
                        }
                    }
                    field_modifications
                })
                .collect(),
            base_value: raw_data,
        }
    }
}

impl Deref for World {
    type Target = WorldView;

    fn deref(&self) -> &WorldView {
        self.view()
    }
}