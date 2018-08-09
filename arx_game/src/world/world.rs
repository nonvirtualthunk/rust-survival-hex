use std::ops;
use std::marker::PhantomData;
use common::hex::*;
use common::prelude::*;
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
use entity::ENTITY_ID_COUNTER;
use storage::MultiTypeEventContainer;
use events::GameEventType;
use events::CoreEvent;

pub struct ModifiersApplication {
    disable_func: fn(&mut World, ModifierReference),
    reset_func: fn(&World, &mut WorldView),
    recompute_for_disabled_modifiers: fn(&World, &mut WorldView, GameEventClock, GameEventClock),
    apply_func: fn(&World, &mut WorldView, usize, ModifierClock, GameEventClock, bool) -> Option<usize>
}

pub struct IndexApplication {
    index_func: Rc<Fn(&World, &mut WorldView)>
}

#[derive(Debug)]
pub struct ModifierReference(TypeId, bool, usize);

pub struct World {
    pub(crate) entities: Vec<EntityContainer>,
    pub self_entity: Entity,
    pub data: Map<CloneAny>,
    pub modifiers: AnyMap,
    pub total_modifier_count: ModifierClock,
    pub total_dynamic_modifier_count: ModifierClock,
    pub current_time: GameEventClock,
    pub(crate) events: MultiTypeEventContainer,
    pub view: UnsafeCell<WorldView>,
    pub modifier_application_by_type: hash_map::HashMap<TypeId, ModifiersApplication>,
    pub entity_indices: Map<CloneAny>,
    pub index_applications: Vec<IndexApplication>
}



impl World {
    pub fn new() -> World {
        let self_ent = World::create_entity();

        let mut world = World {
            entities: vec![],
            self_entity: self_ent,
            data: Map::<CloneAny>::new(),
            modifiers: AnyMap::new(),
            total_modifier_count: 0,
            total_dynamic_modifier_count: 0,
            current_time: 0,
            events: MultiTypeEventContainer::new(),
            view: UnsafeCell::new(WorldView {
                entities: vec![],
                self_entity: self_ent,
                constant_data: Map::<CloneAny>::new(),
                effective_data: Map::<CloneAny>::new(),
                current_time: 0,
                events: MultiTypeEventContainer::new(),
                modifier_cursor: 0,
                modifier_indices: hash_map::HashMap::new(),
                entity_indices: Map::<CloneAny>::new()
            }),
            modifier_application_by_type: hash_map::HashMap::new(),
            entity_indices: Map::<CloneAny>::new(),
            index_applications: vec![]
        };

        world.register_event_type::<CoreEvent>();

        world
    }

    pub fn register_index<I : Hash + Eq + Clone + 'static>(&mut self) {
        self.entity_indices.insert(EntityIndex::<I>::new());
        self.mut_view().entity_indices.insert(EntityIndex::<I>::new());

        let index_func = |world : &World, view : &mut WorldView| {
            let world_index : &EntityIndex<I> = world.entity_indices.get::<EntityIndex<I>>().unwrap();
            let view_index : &mut EntityIndex<I> = view.entity_indices.get_mut::<EntityIndex<I>>().unwrap();
            view_index.update_from(world_index);
        };

        self.index_applications.push(IndexApplication {
            index_func : Rc::new(index_func)
        });
    }

    pub fn register_event_type<E : GameEventType + 'static>(&mut self) {
        self.events.register_event_type::<E>();
    }

    pub(crate) fn modifiers_container<T : EntityData>(&self) -> &ModifiersContainer<T> {
        self.modifiers.get::<ModifiersContainer<T>>().expect("modifiers are expected to be present, you may not have registered all your entity data types")
    }

    pub fn register<T: EntityData>(&mut self) {
        self.data.insert(DataContainer::<T>::new());
        self.modifiers.insert(ModifiersContainer::<T>::new());
        self.mut_view().constant_data.insert(DataContainer::<T>::new());
        self.mut_view().effective_data.insert(DataContainer::<T>::new());


        let disable_func = |world: &mut World, modifier_ref: ModifierReference| {
            let all_modifiers: &mut ModifiersContainer<T> = world.modifiers.get_mut::<ModifiersContainer<T>>().unwrap();
            let ModifierReference(_, dynamic, index) = modifier_ref;
            if dynamic {
                all_modifiers.dynamic_modifiers.get_mut(index).expect("cannot disable a non-existent modifier").disabled_at = Some(world.current_time);
            } else {
                info!("Disabling modifier with reference {:?} and marking disabled at to {:?}", modifier_ref, world.current_time);
                let modifier = all_modifiers.modifiers.get_mut(index).expect("cannot disable a non-existent modifier").disabled_at = Some(world.current_time);
                all_modifiers.modifiers_by_disabled_at.entry(world.current_time).or_insert_with(||Vec::new()).push(index);
            }
        };

        let recompute_for_disabled_modifiers_between = |world: &World, view: &mut WorldView, start : GameEventClock, end : GameEventClock| {
            let all_modifiers: &ModifiersContainer<T> = world.modifiers.get::<ModifiersContainer<T>>().expect("modifiers not present");

            let mut entities_to_recompute = HashSet::new();
            let empty_vec = Vec::new();
            for time in start .. end {
                for modifier_index in all_modifiers.modifiers_by_disabled_at.get(&time).unwrap_or(&empty_vec) {
                    let entity = all_modifiers.modifiers.get(*modifier_index).expect("modifier referenced by disabled at must exist").entity;
                    entities_to_recompute.insert(entity);
                }
            }

            info!("Entities to recompute due to disabled modifiers : {:?}", entities_to_recompute);
            for entity in entities_to_recompute {
                let mut raw_data: T = world.data.get::<DataContainer<T>>()
                    .unwrap_or_else(||panic!(format!("Attempt to recompute unregistered data type for entity: {:?}", entity)))
                    .storage
                    .get(&entity)
                    .unwrap_or_else(||panic!(format!("Attempt to recompute data that has not been attached to entity: {:?}", entity)))
                    .clone();

                info!("Raw data for recomputation: {:?}", raw_data);
                // check if this entity has dynamic modifiers for this data type
                let is_dynamic = all_modifiers.dynamic_entity_set.contains(&entity);
                // if it does, we will write to constant data, then top up with the dynamics afterwards. If it's not dynamic we can write straight to the effective data

                // NB: right now, when recomputing it will be operating against the unchanged world view. Non-dynamic modifiers are not allowed to _look_ at the world view,
                // so that should be fine, but if they do, they'll get weird results
                for modifier in all_modifiers.constant_modifiers_for_entity(entity) {
                    // if the modifier in question has not yet been disabled
                    if modifier.disabled_at.unwrap_or(MAX_GAME_EVENT_CLOCK) >= end {
                        modifier.modifier.modify(&mut raw_data, view);
                    }
                }
                info!("Data after relevant modifiers applied: {:?}", raw_data);

                if is_dynamic {
                    // clone off what we have so far for the constant data section and insert it
                    let constant_data = raw_data.clone();
                    let constant_data_storage = &mut view.constant_data.get_mut::<DataContainer<T>>().expect("constant data not present").storage;
                    constant_data_storage.insert(entity, constant_data);

                    // then recompute all the dynamics. For the moment this is pretty much just the same as the non-dynamic modifiers
                    for dyn_modifier in all_modifiers.dynamic_modifiers_for_entity(entity) {
                        if dyn_modifier.disabled_at.unwrap_or(MAX_GAME_EVENT_CLOCK) >= end {
                            dyn_modifier.modifier.modify(&mut raw_data, view);
                        }
                    }

                    // insert into the effective data storage
                    let effective_data_storage = &mut view.effective_data.get_mut::<DataContainer<T>>().expect("dynamic data not present").storage;
                    effective_data_storage.insert(entity, raw_data);
                } else {
                    // no need for a clone here, just insert the raw data and we're done
                    let effective_data_storage = &mut view.effective_data.get_mut::<DataContainer<T>>().expect("constant data not present").storage;
                    effective_data_storage.insert(entity, raw_data);
                }
            }
        };

        let reset_func = |world: &World, view: &mut WorldView| {
            let all_modifiers: &ModifiersContainer<T> = world.modifiers.get::<ModifiersContainer<T>>().expect("modifiers not present");

            // everything remains in effective_data_storage only, until such time as there is a dynamic modifier on that data, then effective is copied into constant,
            // and all further non-dynamic modifications are made there, all dynamic modifications are made to the effective data, which is reset from constant at each
            // recomputation
            let constant_data_storage = &mut view.constant_data.get_mut::<DataContainer<T>>().expect("constant data not present").storage;
            let effective_data_storage = &mut view.effective_data.get_mut::<DataContainer<T>>().expect("dynamic data not present").storage;
            for entity_id in &all_modifiers.dynamic_entity_set {
                if constant_data_storage.contains_key(entity_id) {
                    let existing_data = constant_data_storage.get(entity_id).expect("existing constant data not present").clone();
                    effective_data_storage.insert(*entity_id, existing_data);
                } else {
                    constant_data_storage.insert(*entity_id, effective_data_storage.get(entity_id).expect("could not instantiate constant from effective").clone());
                };
            }
        };

        let apply_func = |world: &World, view: &mut WorldView, i: usize, modifier_cursor: ModifierClock, at_time: GameEventClock, is_dynamic: bool| {
            let all_modifiers: &ModifiersContainer<T> = world.modifiers.get::<ModifiersContainer<T>>().expect("modifiers not present");

            let relevant_modifiers = match is_dynamic {
                true => &all_modifiers.dynamic_modifiers,
                false => &all_modifiers.modifiers
            };


            match relevant_modifiers.get(i) {
                None => None, // out of bounds, we're done
                Some(wrapper) => {
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

                        if wrapper.disabled_at.unwrap_or(MAX_GAME_EVENT_CLOCK) >= at_time {
                            trace!("Active and ready: {}", is_dynamic);
                            let ent_has_dynamic_data = all_modifiers.dynamic_entity_set.contains(&wrapper.entity);

                            // pull from effective data if we're calculating dynamics, or if there is no dynamic data on this entity at all, in which case we always look at
                            // effective
                            let mut ent_data: T = match is_dynamic || !ent_has_dynamic_data {
                                true => view.effective_data.get_mut::<DataContainer<T>>().expect("modifier's dynamic data not present").storage
                                    //.entry(wrapper.entity.0).or_insert(T::default()).clone(),
                                    .get(&wrapper.entity).unwrap_or_else(||panic!(format!("Could not retrieve dynamic data for modified entity {}", wrapper.entity))).clone(),
                                false => view.constant_data.get_mut::<DataContainer<T>>().expect("modifier's constant data not present").storage
                                    //.entry(wrapper.entity).or_insert(T::default()).clone()
                                    .get(&wrapper.entity).unwrap_or_else(||panic!(format!("Could not retrieve constant data for modified entity {}", wrapper.entity))).clone()
                            };

                            wrapper.modifier.modify(&mut ent_data, view);

                            match is_dynamic || !ent_has_dynamic_data {
                                true => view.effective_data.get_mut::<DataContainer<T>>().unwrap().storage.entry(wrapper.entity).and_modify(|e| { *e = ent_data.clone() }),
                                false => view.constant_data.get_mut::<DataContainer<T>>().unwrap().storage.entry(wrapper.entity).and_modify(|e| { *e = ent_data.clone() })
                            };
                        } else {
                            trace!("Not active, no action: {}", is_dynamic);
                        }

                        Some(i + 1)
                    }
                }
            }
        };

        self.modifier_application_by_type.insert(TypeId::of::<T>(), ModifiersApplication {
            disable_func: (disable_func),
            reset_func: (reset_func),
            recompute_for_disabled_modifiers: (recompute_for_disabled_modifiers_between),
            apply_func: (apply_func),
        });
    }

    /// Returns a view of this world that will be kept continuously up to date
    pub fn view<'a, 'b>(&'a self) -> &'b WorldView {
        unsafe { &*self.view.get() }
    }

    fn mut_view(&self) -> &mut WorldView {
        unsafe { &mut *self.view.get() }
    }

    pub fn view_at_time(&self, at_time: GameEventClock) -> WorldView {
        let mut new_view = WorldView {
            entities: self.entities.iter().filter(|e| e.1 <= at_time).cloned().collect(),
            self_entity: self.self_entity,
            constant_data: self.data.clone(),
            effective_data: self.data.clone(),
            current_time: 0,
            events: self.events.clone_events_up_to(at_time),
            modifier_cursor: 0,
            modifier_indices: hash_map::HashMap::new(),
            entity_indices: self.entity_indices.clone()
        };

        self.update_view_to_time(&mut new_view, at_time);

        new_view
    }

    pub fn update_view_to_time(&self, view: &mut WorldView, at_time: GameEventClock) {
        self.index_applications.iter().for_each(|idx| (idx.index_func)(self, view));

        trace!("Updating view-------------------------------------------");
        if view.current_time >= at_time {
            trace!("\tShort circuit");
            return;
        }

        // TODO: deal with events

        let new_entities: Vec<EntityContainer> = self.entities.iter().take_while(|e| e.1 <= at_time).cloned().collect();
        new_entities.iter().rev().for_each(|e| view.entities.push(e.clone()));


        // we need to keep track of where we are in each modifier type, as well as the global modifier cursor.
        // we continuously iterate the modifier cursor, asking each to apply the active modifier cursor. At
        // each point only one will actually do so, since there is only one modifier at a given cursor point.
        // If we reach a point where none applied anything, then we can assume we have reached the end and are
        // done.

        // set up a vector of walkers, each being an application function and a current index
        let mut walkers = vec![];
        for (type_id, application_capability) in &self.modifier_application_by_type {
            let current_index = view.modifier_indices.get(type_id).map(|i| *i as usize).unwrap_or(0);
            trace!("Pulling current_index from past run: {}", current_index);
            walkers.push((application_capability.apply_func.clone(), Some(current_index), type_id));
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
            let current_index = 0;
            walkers.push((application_capability.apply_func.clone(), Some(current_index), type_id));
            (application_capability.reset_func)(self, view);
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

        // TODO: should this be ..= or .. at_time? Only relevant when we're catching up, most likely, but probably worth determining if there's odd behavior
        for (type_id, application_capability) in &self.modifier_application_by_type {
            (application_capability.recompute_for_disabled_modifiers)(self, view, view.current_time, at_time);
        }

        view.current_time = at_time;
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(EntityContainer(entity, self.current_time));
    }

    pub fn index_entity<I: Hash + Eq + Clone + 'static>(&mut self, entity : Entity, key : I) {
        let index : &mut EntityIndex<I> = self.entity_indices.get_mut::<EntityIndex<I>>().unwrap();
        index.index.insert(key, entity);
    }

    pub fn modify<T: EntityData, S : Into<Option<Str>>>(&mut self, entity: Entity, modifier: Box<Modifier<T>>, description : S) -> ModifierReference {
        self.add_modifier(entity, modifier, description)
    }
    pub fn add_modifier<T: EntityData, S : Into<Option<Str>>>(&mut self, entity: Entity, modifier: Box<Modifier<T>>, description : S) -> ModifierReference {
        let all_modifiers: &mut ModifiersContainer<T> = self.modifiers.get_mut::<ModifiersContainer<T>>().unwrap();
        if modifier.modifier_type() == ModifierType::Dynamic {
            let index = all_modifiers.dynamic_entity_set.len();
            all_modifiers.dynamic_modifiers.push(ModifierContainer {
                modifier,
                applied_at: self.current_time,
                disabled_at: None,
                modifier_index: self.total_dynamic_modifier_count,
                entity,
                description : description.into()
            });
            all_modifiers.dynamic_entity_set.insert(entity);
            self.total_dynamic_modifier_count += 1;
            ModifierReference(TypeId::of::<T>(), true, index)
        } else {
            let index = all_modifiers.modifiers.len();
            all_modifiers.modifiers.push(ModifierContainer {
                modifier,
                applied_at: self.current_time,
                disabled_at: None,
                modifier_index: self.total_modifier_count,
                entity,
                description : description.into()
            });
            trace!("Creating modifier with count {}, incrementing", self.total_modifier_count);
            self.total_modifier_count += 1;
            ModifierReference(TypeId::of::<T>(), false, index)
        }
    }

    pub fn disable_modifier(&mut self, modifier_ref: ModifierReference) {
        let application_capabilities = self.modifier_application_by_type.get(&modifier_ref.0).expect("attempted to disable modifier of unregistered data type, should be impossible");
        (application_capabilities.disable_func)(self, modifier_ref);
    }

    pub fn add_world_modifier<T: EntityData>(&mut self, modifier: Box<Modifier<T>>) {
        let tmp = self.self_entity;
        self.add_modifier::<T,Option<Str>>(tmp, modifier, None);
    }

    pub fn add_constant_modifier<T: EntityData, CM: ConstantModifier<T> + 'static>(&mut self, entity: Entity, constant_modifier: CM) {
        self.add_modifier(entity, box ConstantModifierWrapper { inner: constant_modifier, _ignored: PhantomData }, None);
    }

    pub fn add_limited_modifier<T: EntityData, CM: LimitedModifier<T> + 'static>(&mut self, entity: Entity, limited_modifier: CM) {
        self.add_modifier(entity, box LimitedModifierWrapper { inner: limited_modifier, _ignored: PhantomData }, None);
    }

    pub fn add_dynamic_modifier<T: EntityData, CM: DynamicModifier<T> + 'static>(&mut self, entity: Entity, dynamic_modifier: CM) {
        self.add_modifier(entity, box DynamicModifierWrapper { inner: dynamic_modifier, _ignored: PhantomData }, None);
    }

    pub fn add_constant_world_modifier<T: EntityData, CM: ConstantModifier<T> + 'static>(&mut self, constant_modifier: CM) {
        let entity = self.self_entity;
        self.add_modifier(entity, box ConstantModifierWrapper { inner: constant_modifier, _ignored: PhantomData }, None);
    }

    pub fn add_limited_world_modifier<T: EntityData, CM: LimitedModifier<T> + 'static>(&mut self, limited_modifier: CM) {
        let entity = self.self_entity;
        self.add_modifier(entity, box LimitedModifierWrapper { inner: limited_modifier, _ignored: PhantomData }, None);
    }

    pub fn add_dynamic_world_modifier<T: EntityData, CM: DynamicModifier<T> + 'static>(&mut self, dynamic_modifier: CM) {
        let entity = self.self_entity;
        self.add_modifier(entity, box DynamicModifierWrapper { inner: dynamic_modifier, _ignored: PhantomData }, None);
    }


    pub fn add_event<E : GameEventType + 'static>(&mut self, event: E) {
        self.events.push_event(GameEventWrapper {
            data: event,
            occurred_at: self.current_time
        });
        self.current_time += 1;
        self.update_view_to_time(self.mut_view(), self.current_time);
    }

    pub fn event_at<E : GameEventType + 'static>(&self, time : GameEventClock) -> Option<E> {
        self.events.events::<E>().find(|e| e.occurred_at == time).map(|e| e.data)
    }

    pub fn attach_data<T: EntityData>(&mut self, entity: Entity, data: &T) {
        {
            let self_data: &mut DataContainer<T> = self.data.get_mut::<DataContainer<T>>()
                .unwrap_or_else(||panic!(format!("Attempt to attach unregistered data: {:?}", data)));
            self_data.storage.insert(entity, data.clone());
        }

        {
            let mut_view_data: &mut DataContainer<T> = self.mut_view().effective_data.get_mut::<DataContainer<T>>().unwrap();
            mut_view_data.storage.insert(entity, data.clone());
        }
    }

    pub fn attach_world_data<T: EntityData>(&mut self, data: &T) {
        let ent = self.self_entity;
        self.attach_data(ent, data);
    }

    pub fn create_entity() -> Entity {
        let id = ENTITY_ID_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        Entity(id)
    }


    pub fn random_seed(&self, extra : u8) -> [u8;32] {
        use std::mem;

        let time_bytes : [u8;8] = unsafe {
            mem::transmute(self.current_time)
        };

        [time_bytes[0],time_bytes[1],time_bytes[2],time_bytes[3],time_bytes[4],time_bytes[5],time_bytes[6],time_bytes[7],0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,extra]
    }
}



impl World {
    pub fn permanent_field_logs_for<T: EntityData>(&self, ent: Entity) -> FieldLogs<T> {
        self.field_logs_with_condition_for::<T>(ent, |m| m.modifier.modifier_type() == ModifierType::Permanent, self.current_time)
    }
    pub fn non_permanent_field_logs_for<T: EntityData>(&self, ent: Entity) -> FieldLogs<T> {
        self.field_logs_with_condition_for::<T>(ent, |m| m.modifier.modifier_type() != ModifierType::Permanent, self.current_time)
    }
    pub fn field_logs_for<T: EntityData>(&self, ent: Entity) -> FieldLogs<T> {
        self.field_logs_with_condition_for::<T>(ent, |m| true, self.current_time)
    }

    pub fn field_logs_with_condition_for<T: EntityData>(&self, ent: Entity, condition : fn(&ModifierContainer<T>) -> bool, at_time : GameEventClock) -> FieldLogs <T>{
        let container = self.modifiers_container::<T>();
        let data_container : &DataContainer<T> = self.data.get::<DataContainer<T>>().expect("Data kind must exist");
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
            base_value : raw_data
        }
    }
}
