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
use std::collections::hash_set;
use events::GameEvent;
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

pub static ENTITY_ID_COUNTER: AtomicUsize = ATOMIC_USIZE_INIT;

type EntityId = usize;

#[derive(Clone, Copy, Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Default)]
pub struct Entity(EntityId);

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

/// conceptually, we're breaking up modifiers into several broad types: permanent (movement, damage, temperature),
/// limited (fixed duration spell, poison), and dynamic (+1 attacker per adjacent ally, -1 move at night). Permanent
/// modifiers we can apply in order and forget about, they compact easily. Dynamic modifiers effectively need to be
/// applied last, since they are always dependent on the most current data. Limited modifiers basically act as
/// permanent modifiers until they run out, at which point they need to trigger a recalculation of their entity.
/// So, we need to monitor for the most recent state of limited modifiers and see when it toggles. It might be
/// useful to require that a Limited modifier never switches from off->on, but I think just watching for a toggle
/// is sufficient.
///
/// In order to keep things from spiraling out of control, non-Dynamic modifiers will not be able to look at the
/// current world state when determining their effects, which means that they must reify in anything that varies.
/// I.e. a spell that gives +3 Health if in forest at time of casting or +1 Health otherwise would need to bake
/// in whether the effect was +3 or +1 at creation time rather than looking at current world state to determine it.
/// Anything that does depend on the current world state must necessarily be Dynamic. For general stability
/// purposes it is recommended that Dynamic modifiers only depend on things that are unlikely to have Dynamic
/// modifiers themselves, since ordering among Dynamics may or may not be constant.
///
/// We _could_ make it constant, but it would mean recalculating all of them every tick and baking them in. Every
/// view would basically have two copies of all data, the constant/limited data and the post-dynamic data. Every
/// tick, everything with at least one dynamic modifier would have their post-dynamic data set to a copy of the
/// constant/limited data, then all dynamic modifiers would be applied in-order cross-world. The alternative
/// would be to only calculate the effective post-dynamic data on-demand, when it is actually requested, but
/// since that calculation would be occurring in isolation, every Dynamic effect would be unable to see the effects
/// of any other, or it would have to avoid infinite loops by some other means. I think I'm in favor of the
/// constant recalculation of the dynamic effects in views, it shouldn't be _that_ expensive unless we get a
/// massive number of dynamic effects going on, and I think that should be avoidable except for the really
/// interesting spells.
///
/// So, implementation-wise, where does that put us? We need views to maintain two copies of data
///
#[derive(Eq, PartialEq)]
pub enum ModifierType {
    Permanent,
    Limited,
    Dynamic
}

pub trait ConstantModifier<T: EntityData>: Sized + 'static {
    fn modify(&self, data: &mut T);

    fn apply_to(self, entity: Entity, world: &mut World) {
        world.add_constant_modifier(entity, self);
    }
    fn apply_to_world(self, world : &mut World) {
        world.add_constant_world_modifier(self);
    }

    fn wrap(self) -> Box<Modifier<T>> {
        box ConstantModifierWrapper {
            inner: self,
            _ignored: PhantomData
        }
    }
}

struct ConstantModifierWrapper<T: EntityData, CM: ConstantModifier<T>> {
    inner: CM,
    _ignored: PhantomData<T>
}

impl<T: EntityData, CM: ConstantModifier<T>> Modifier<T> for ConstantModifierWrapper<T, CM> {
    fn modify(&self, data: &mut T, world: &WorldView) {
        self.inner.modify(data);
    }

    fn is_active(&self, world: &WorldView) -> bool {
        true
    }

    fn modifier_type(&self) -> ModifierType {
        ModifierType::Permanent
    }
}

pub trait LimitedModifier<T: EntityData>: Sized + 'static {
    fn modify(&self, data: &mut T);

    fn is_active(&self, world: &WorldView) -> bool;
}

struct LimitedModifierWrapper<T: EntityData, LM: LimitedModifier<T>> {
    inner: LM,
    _ignored: PhantomData<T>
}

impl<T: EntityData, LM: LimitedModifier<T>> Modifier<T> for LimitedModifierWrapper<T, LM> {
    fn modify(&self, data: &mut T, world: &WorldView) {
        self.inner.modify(data);
    }

    fn is_active(&self, world: &WorldView) -> bool {
        self.inner.is_active(world)
    }

    fn modifier_type(&self) -> ModifierType {
        ModifierType::Limited
    }
}

pub trait DynamicModifier<T: EntityData> {
    fn modify(&self, data: &mut T, world: &WorldView);

    fn is_active(&self, world: &WorldView) -> bool;
}

struct DynamicModifierWrapper<T: EntityData, DM: DynamicModifier<T>> {
    inner: DM,
    _ignored: PhantomData<T>
}

impl<T: EntityData, LM: DynamicModifier<T>> Modifier<T> for DynamicModifierWrapper<T, LM> {
    fn modify(&self, data: &mut T, world: &WorldView) {
        self.inner.modify(data, world);
    }

    fn is_active(&self, world: &WorldView) -> bool {
        self.inner.is_active(world)
    }

    fn modifier_type(&self) -> ModifierType {
        ModifierType::Dynamic
    }
}

pub trait Modifier<T: EntityData> {
    fn modify(&self, data: &mut T, world: &WorldView);

    fn is_active(&self, world: &WorldView) -> bool;

    fn modifier_type(&self) -> ModifierType;
}


type ModifierClock = usize;

pub struct ModifierContainer<T: EntityData> {
    modifier: Box<Modifier<T>>,
    applied_at: GameEventClock,
    modifier_index: ModifierClock,
    entity: Entity
}

#[derive(Clone)]
pub struct DataContainer<T: EntityData> {
    storage: hash_map::HashMap<Entity, T>,
    sentinel: T
}

pub struct ModifiersContainer<T: EntityData> {
    /// All modifiers that alter data of type T and are not Dynamic, stored in chronological order
    modifiers: Vec<ModifierContainer<T>>,
    /// All Dynamic modifiers, stored in chronological order
    dynamic_modifiers: Vec<ModifierContainer<T>>,
    /// Tracks what the most recent activation state was for any given Limited type modifier, can be used to determine if recalculation is necessary
    limited_modifier_activation_states: hash_map::HashMap<Entity, bool>,
    // map from index in `modifiers` to last activation state
    /// The full set of entities that have dynamic modifiers for this data type
    dynamic_entity_set: hash_set::HashSet<Entity>
}

impl<T: EntityData> DataContainer<T> {
    pub fn new() -> DataContainer<T> {
        DataContainer {
            storage: hash_map::HashMap::new(),
            sentinel: T::default()
        }
    }
}

impl<T: EntityData> ModifiersContainer<T> {
    pub fn new() -> ModifiersContainer<T> {
        ModifiersContainer {
            modifiers: vec![],
            dynamic_modifiers: vec![],
            limited_modifier_activation_states: hash_map::HashMap::new(),
            dynamic_entity_set: hash_set::HashSet::new()
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct EntityContainer(Entity, GameEventClock);

#[derive(Clone)]
pub struct EntityIndex<T : Hash + Eq + Clone> {
    index : hash_map::HashMap<T, Entity>
}

impl <T : Hash + Eq + Clone> EntityIndex<T> {
    pub fn new() -> EntityIndex<T> {
        EntityIndex {
            index : hash_map::HashMap::new()
        }
    }

    pub fn update_from(&mut self, other : &EntityIndex<T>) {
        if other.index.len() > self.index.len() {
            self.index = other.index.clone();
        }
    }
}

pub struct World {
    pub entities: Vec<EntityContainer>,
    pub self_entity: Entity,
    pub data: Map<CloneAny>,
    pub modifiers: AnyMap,
    pub total_modifier_count: ModifierClock,
    pub total_dynamic_modifier_count: ModifierClock,
    pub current_time: GameEventClock,
    pub events: Vec<Rc<GameEventWrapper>>,
    pub view: UnsafeCell<WorldView>,
    pub modifier_application_by_type: hash_map::HashMap<TypeId, ModifiersApplication>,
    pub entity_indices: Map<CloneAny>,
    pub index_applications: Vec<IndexApplication>
}

pub struct WorldView {
    entities: Vec<EntityContainer>,
    self_entity : Entity,
    constant_data: Map<CloneAny>,
    effective_data: Map<CloneAny>,
    pub current_time: GameEventClock,
    modifier_cursor: ModifierClock,
    modifier_indices: hash_map::HashMap<TypeId, usize>,
    events: Vec<Rc<GameEventWrapper>>,
    pub entity_indices: Map<CloneAny>
}

pub struct ModifiersApplication {
    reset_func: Rc<Fn(&World, &mut WorldView)>,
    apply_func: Rc<Fn(&World, &mut WorldView, usize, ModifierClock, GameEventClock, bool) -> Option<usize>>
}

pub struct IndexApplication {
    index_func: Rc<Fn(&World, &mut WorldView)>
}



impl World {
    pub fn new() -> World {
        let self_ent = World::create_entity();

        let world = World {
            entities: vec![],
            self_entity: self_ent,
            data: Map::<CloneAny>::new(),
            modifiers: AnyMap::new(),
            total_modifier_count: 0,
            total_dynamic_modifier_count: 0,
            current_time: 0,
            events: vec![],
            view: UnsafeCell::new(WorldView {
                entities: vec![],
                self_entity: self_ent,
                constant_data: Map::<CloneAny>::new(),
                effective_data: Map::<CloneAny>::new(),
                current_time: 0,
                events: vec![],
                modifier_cursor: 0,
                modifier_indices: hash_map::HashMap::new(),
                entity_indices: Map::<CloneAny>::new()
            }),
            modifier_application_by_type: hash_map::HashMap::new(),
            entity_indices: Map::<CloneAny>::new(),
            index_applications: vec![]
        };

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

    pub fn register<T: EntityData>(&mut self) {
        self.data.insert(DataContainer::<T>::new());
        self.modifiers.insert(ModifiersContainer::<T>::new());
        self.mut_view().constant_data.insert(DataContainer::<T>::new());
        self.mut_view().effective_data.insert(DataContainer::<T>::new());

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

                        if wrapper.modifier.is_active(view) {
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

        self.modifier_application_by_type.insert(T::get_type_id(&T::default()), ModifiersApplication {
            reset_func: Rc::new(reset_func),
            apply_func: Rc::new(apply_func)
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
            events: self.events.iter().filter(|e| Rc::as_ref(e).occurred_at <= at_time).cloned().collect(),
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


        // we need to keep track of where we are in each modifier type, as well as the global modifier cursor
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

        view.current_time = at_time;
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(EntityContainer(entity, self.current_time));
    }

    pub fn index_entity<I: Hash + Eq + Clone + 'static>(&mut self, entity : Entity, key : I) {
        let index : &mut EntityIndex<I> = self.entity_indices.get_mut::<EntityIndex<I>>().unwrap();
        index.index.insert(key, entity);
    }

    pub fn add_modifier<T: EntityData>(&mut self, entity: Entity, modifier: Box<Modifier<T>>) {
        let all_modifiers: &mut ModifiersContainer<T> = self.modifiers.get_mut::<ModifiersContainer<T>>().unwrap();
        if modifier.modifier_type() == ModifierType::Dynamic {
            all_modifiers.dynamic_modifiers.push(ModifierContainer {
                modifier,
                applied_at: self.current_time,
                modifier_index: self.total_dynamic_modifier_count,
                entity
            });
            all_modifiers.dynamic_entity_set.insert(entity);
            self.total_dynamic_modifier_count += 1;
        } else if modifier.modifier_type() == ModifierType::Permanent {
            all_modifiers.modifiers.push(ModifierContainer {
                modifier,
                applied_at: self.current_time,
                modifier_index: self.total_modifier_count,
                entity
            });
            trace!("Creating modifier with count {}, incrementing", self.total_modifier_count);
            self.total_modifier_count += 1;
        }
    }

    pub fn add_world_modifier<T: EntityData>(&mut self, modifier: Box<Modifier<T>>) {
        let tmp = self.self_entity;
        self.add_modifier::<T>(tmp, modifier);
    }

    pub fn add_constant_modifier<T: EntityData, CM: ConstantModifier<T> + 'static>(&mut self, entity: Entity, constant_modifier: CM) {
        self.add_modifier(entity, box ConstantModifierWrapper { inner: constant_modifier, _ignored: PhantomData });
    }

    pub fn add_limited_modifier<T: EntityData, CM: LimitedModifier<T> + 'static>(&mut self, entity: Entity, limited_modifier: CM) {
        self.add_modifier(entity, box LimitedModifierWrapper { inner: limited_modifier, _ignored: PhantomData });
    }

    pub fn add_dynamic_modifier<T: EntityData, CM: DynamicModifier<T> + 'static>(&mut self, entity: Entity, dynamic_modifier: CM) {
        self.add_modifier(entity, box DynamicModifierWrapper { inner: dynamic_modifier, _ignored: PhantomData });
    }

    pub fn add_constant_world_modifier<T: EntityData, CM: ConstantModifier<T> + 'static>(&mut self, constant_modifier: CM) {
        let entity = self.self_entity;
        self.add_modifier(entity, box ConstantModifierWrapper { inner: constant_modifier, _ignored: PhantomData });
    }

    pub fn add_limited_world_modifier<T: EntityData, CM: LimitedModifier<T> + 'static>(&mut self, limited_modifier: CM) {
        let entity = self.self_entity;
        self.add_modifier(entity, box LimitedModifierWrapper { inner: limited_modifier, _ignored: PhantomData });
    }

    pub fn add_dynamic_world_modifier<T: EntityData, CM: DynamicModifier<T> + 'static>(&mut self, dynamic_modifier: CM) {
        let entity = self.self_entity;
        self.add_modifier(entity, box DynamicModifierWrapper { inner: dynamic_modifier, _ignored: PhantomData });
    }


    pub fn add_event(&mut self, event: GameEvent) {
        self.events.push(Rc::new(GameEventWrapper {
            data: event,
            occurred_at: self.current_time
        }));
        self.current_time += 1;
        self.update_view_to_time(self.mut_view(), self.current_time);
    }

    pub fn event_at(&self, time : GameEventClock) -> Option<GameEvent> {
        self.events.iter().find(|e| e.occurred_at == time).map(|e| e.data)
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

    pub fn entities_with_data<T : EntityData>(&self) -> &hash_map::HashMap<Entity, T> {
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

    pub fn events(&self) -> &Vec<Rc<GameEventWrapper>> {
        &self.events
    }
}


mod test {
    use super::*;

    #[derive(Clone, Default, PartialEq, Debug)]
    struct FooData {
        a: i32,
        b: Vec<f32>
    }

    #[derive(Clone, Default, PartialEq, Debug)]
    struct BarData {
        x: f32
    }

    impl EntityData for FooData {}

    impl EntityData for BarData {}

    struct AddToAModifier {
        delta_a: i32
    }

    impl ConstantModifier<FooData> for AddToAModifier {
        fn modify(&self, data: &mut FooData) {
            data.a += self.delta_a;
        }
    }

    pub struct MultiplyByOtherEntityModifier {
        other_entity: Entity
    }

    impl DynamicModifier<FooData> for MultiplyByOtherEntityModifier {
        fn modify(&self, data: &mut FooData, world: &WorldView) {
            data.a = data.a * world.data::<FooData>(self.other_entity).a;
        }

        fn is_active(&self, world: &WorldView) -> bool {
            true
        }
    }

    pub struct AddBarDataModifier {
        delta: f32
    }

    impl ConstantModifier<BarData> for AddBarDataModifier {
        fn modify(&self, data: &mut BarData) {
            data.x += self.delta;
        }
    }

    use spectral::prelude::*;

    pub struct AddFooBValueModifier {}

    impl ConstantModifier<FooData> for AddFooBValueModifier {
        fn modify(&self, data: &mut FooData) {
            data.b.push(1.0);
        }
    }

    #[test]
    pub fn test_new_world() {
        let mut world = World::new();

        world.register::<FooData>();

        let initial_data = FooData {
            a: 1,
            b: vec![]
        };

        let ent1 = EntityBuilder::new()
            .with(initial_data.clone())
            .create(&mut world);

        let ent2 = EntityBuilder::new()
            .with(FooData {
                a: 4,
                b: vec![]
            }).create(&mut world);

        let view = world.view();

        let test_data_1 = view.data::<FooData>(ent1);
        let test_data_2 = view.data::<FooData>(ent2);

        assert_eq!(*test_data_1, initial_data);
        assert_eq!(test_data_1.a, 1);

        world.add_constant_modifier(ent1, AddToAModifier { delta_a: 4 });
        world.add_event(GameEvent::TurnStart { turn_number: 1 });

        assert_eq!(test_data_1.a, 5);
        assert_eq!(test_data_2.a, 4);

        world.add_dynamic_modifier(ent1, MultiplyByOtherEntityModifier { other_entity: ent2 });
        world.add_event(GameEvent::TurnStart { turn_number: 2 });

        assert_eq!(test_data_1.a, 20);

        world.add_constant_modifier(ent2, AddToAModifier { delta_a: 1 });
        world.add_event(GameEvent::TurnStart { turn_number: 3 });

        assert_that!(&test_data_2.a).is_equal_to(5);
        assert_that!(&test_data_1.a).is_equal_to(25);
    }

    #[test]
    pub fn test_multiple_data_types() {
        let mut world = World::new();

        world.register::<FooData>();
        world.register::<BarData>();

        let ent1 = EntityBuilder::new()
            .with(FooData {
                a: 1,
                b: vec![]
            })
            .with(BarData {
                x: 1.0
            })
            .create(&mut world);

        let ent2 = EntityBuilder::new()
            .with(FooData {
                a: 2,
                b: vec![]
            })
            .with(BarData {
                x: 1.0
            })
            .create(&mut world);

        world.add_entity(ent1);
        world.add_entity(ent2);

        let view = world.view();

        let foo_data_1 = view.data::<FooData>(ent1);
        let foo_data_2 = view.data::<FooData>(ent2);

        let bar_data_1 = view.data::<BarData>(ent1);
        let bar_data_2 = view.data::<BarData>(ent2);

        assert_that(&foo_data_1.a).is_equal_to(1);
        assert_that(&bar_data_1.x).is_equal_to(bar_data_2.x);

        world.add_modifier(ent1, AddBarDataModifier { delta: 2.0 }.wrap());
        world.add_event(GameEvent::TurnStart { turn_number: 1 });

        // show up in reverse chronological order, last created first in list
        assert_that(&view.entities.get(0).unwrap().0).is_equal_to(ent2);
        assert_that(&view.entities.get(1).unwrap().0).is_equal_to(ent1);

        // now that it's been modified they should not be the same
        assert_that(&bar_data_1.x).is_not_equal_to(bar_data_2.x);
        assert_that(&bar_data_1.x).is_equal_to(3.0);

        world.add_dynamic_modifier(ent1, MultiplyByOtherEntityModifier { other_entity: ent2 });
        world.add_event(GameEvent::TurnStart { turn_number: 2 });

        assert_that(&foo_data_1.a).is_equal_to(2);

        world.add_constant_modifier(ent2, AddFooBValueModifier {});
        world.add_event(GameEvent::TurnStart { turn_number: 3 });

        assert_that(&bar_data_1.x).is_equal_to(3.0);
        assert_that(&foo_data_1.a).is_equal_to(2);
        assert_that(&foo_data_2.b).is_equal_to(vec![1.0]);
    }

    #[test]
    pub fn test_entity_index() {
        let mut world = World::new();

        world.register::<FooData>();
        world.register::<BarData>();
        world.register_index::<AxialCoord>();

        let ent1 = EntityBuilder::new()
            .with(FooData {
                a: 1,
                b: vec![]
            })
            .with(BarData {
                x: 1.0
            })
            .create(&mut world);

        let ent2 = EntityBuilder::new()
            .with(FooData {
                a: 2,
                b: vec![]
            })
            .with(BarData {
                x: 1.0
            })
            .create(&mut world);

        world.add_entity(ent1);
        world.index_entity(ent1, AxialCoord::new(2,2));
        world.add_entity(ent2);
        world.index_entity(ent2, AxialCoord::new(3,4));

        world.add_event(GameEvent::WorldStart);

        let view = world.view();

        assert_that!(view.entity_by_key(&AxialCoord::new(2,2))).is_equal_to(Some(ent1));
        assert_that!(view.entity_by_key(&AxialCoord::new(3,4))).is_equal_to(Some(ent2));
        assert_that!(view.entity_by_key(&AxialCoord::new(4,5))).is_equal_to(None);
    }
}