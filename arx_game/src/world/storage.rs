use common::prelude::*;
use std::collections::HashSet;
use std::collections::HashMap;
use entity::Entity;
use entity::EntityData;
use modifiers::Modifier;
use core::GameEventClock;
use std::hash::Hash;
use events::GameEventWrapper;
use events::GameEventType;
use anymap::Map;
use anymap::any::CloneAny;
use std::iter;
use core::MAX_GAME_EVENT_CLOCK;
use events::GameEventState;
use world::world::World;
use world::ModifierReference;
use world::ModifierReferenceType;
use std::rc::Rc;
use std::any::TypeId;
use erased_serde;
use common::multitype::MultiTypeContainer;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub type ModifierClock = usize;
pub type EventCallback<E> = fn(&mut World, &GameEventWrapper<E>);

#[derive(Serialize,Deserialize)]
pub(crate) struct MultiTypeEventContainer {
    pub(crate) event_containers: MultiTypeContainer,
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) clone_up_to_time_funcs: Vec<fn(&mut MultiTypeEventContainer, &MultiTypeEventContainer, GameEventClock)>,
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) update_to_time_funcs: Vec<fn(&mut MultiTypeEventContainer, &MultiTypeEventContainer, GameEventClock)>,
}


#[derive(Clone,Serialize,Deserialize)]
pub(crate) struct EventContainer<E: GameEventType> {
    pub(crate) events: Vec<GameEventWrapper<E>>,
    pub(crate) default: GameEventWrapper<E>,
    #[serde(skip_serializing, skip_deserializing)]
    pub(crate) event_listeners: Vec<EventCallback<E>>,
}

impl<E: GameEventType> Default for EventContainer<E> {
    fn default() -> Self {
        EventContainer {
            events: vec![],
            default: GameEventWrapper::event_and_state(E::beginning_of_time_event(), GameEventState::Ended),
            event_listeners: vec![],
        }
    }
}

impl MultiTypeEventContainer {
    pub(crate) fn new() -> MultiTypeEventContainer {
        MultiTypeEventContainer {
            event_containers: MultiTypeContainer::new(),
            clone_up_to_time_funcs: Vec::new(),
            update_to_time_funcs: Vec::new(),
        }
    }
    pub(crate) fn register_event_type<'de, E: GameEventType + 'static + Serialize + Default + DeserializeOwned>(&mut self) {
//        println!("Registering event of type {:?}, nth registered: {:?}", unsafe {std::intrinsics::type_name::<E>()}, self.event_containers.len());
//        println!("MTE: {:?}", (self as *const MultiTypeEventContainer));
        self.event_containers.register::<EventContainer<E>>();

        self.clone_up_to_time_funcs.push(|mte: &mut MultiTypeEventContainer, from: &MultiTypeEventContainer, time: GameEventClock| {
            mte.register_event_type::<E>();

            mte.event_containers.get_mut::<EventContainer<E>>().events =
                from.events::<E>().filter(|e| e.occurred_at <= time).cloned().collect();
        });

        self.update_to_time_funcs.push(|mte: &mut MultiTypeEventContainer, from: &MultiTypeEventContainer, end_time: GameEventClock| {
            let cur_high_time = mte.event_containers.get::<EventContainer<E>>().events
                .last()
                .map(|ec| ec.occurred_at)
                .unwrap_or(0);

            // events are stored in order oldest to newest, so we start from the back, taking newest to oldest, ignore all that are newer
            // than our target time, take all that are newer than our start time, now we have all new events ordered newest to oldest. Take
            // that and reverse it and we can tack it onto the end.
            let mut new_events = from.event_containers.get::<EventContainer<E>>().events.iter()
                .rev()
                .skip_while(|ec| ec.occurred_at > end_time)
                .take_while(|ec| ec.occurred_at > cur_high_time)
                .cloned()
                .collect_vec();
            new_events.reverse();

            mte.event_containers.get_mut::<EventContainer<E>>().events.extend(new_events);
        });
    }
    pub(crate) fn add_callback<E: GameEventType + 'static>(&mut self, callback: EventCallback<E>) {
        let event_container = self.event_containers.get_mut::<EventContainer<E>>();
        event_container.event_listeners.push(callback);
    }
    pub(crate) fn push_event<E: GameEventType + 'static>(&mut self, evt: GameEventWrapper<E>) -> Vec<EventCallback<E>> {
        let event_container = self.event_containers.get_mut::<EventContainer<E>>();
        event_container.events.push(evt);
        event_container.event_listeners.clone()
    }
    pub(crate) fn events<E: GameEventType + 'static>(&self) -> impl Iterator<Item=&GameEventWrapper<E>> {
        self.event_containers.get::<EventContainer<E>>().events.iter()
    }
    pub(crate) fn revents<E: GameEventType + 'static>(&self) -> impl Iterator<Item=&GameEventWrapper<E>> {
        self.event_containers.get::<EventContainer<E>>().events.iter().rev()
    }
    pub(crate) fn most_recent_event<E: GameEventType + 'static>(&self) -> &GameEventWrapper<E> {
        let container = self.event_containers.get::<EventContainer<E>>();
        container.events.iter().rev().next().unwrap_or(&container.default)
    }

    pub(crate) fn clone_events_up_to(&self, at_time: GameEventClock) -> MultiTypeEventContainer {
        let mut ret = MultiTypeEventContainer::new();

        for func in &self.clone_up_to_time_funcs {
            (func)(&mut ret, self, at_time);
        }

        ret
    }

    pub(crate) fn update_events_to(&mut self, from: &MultiTypeEventContainer, at_time: GameEventClock) {
        for func in self.update_to_time_funcs.clone() {
            (func)(self, from, at_time);
        }
    }
}

#[derive(Serialize,Deserialize)]
pub struct ModifierContainer<T: EntityData> {
    pub(crate) modifier: Rc<Modifier<T>>,
    pub(crate) applied_at: GameEventClock,
    pub(crate) disabled_at: Option<GameEventClock>,
    pub(crate) modifier_index: ModifierClock,
    pub(crate) entity: Entity,
    pub(crate) description: Option<String>,
}

#[derive(Serialize,Deserialize)]
pub struct ModifierArchetypeContainer<T: EntityData> {
    pub(crate) modifier: Rc<Modifier<T>>,
}

impl<T: EntityData> ModifierContainer<T> {
    pub fn is_active_at_time(&self, time: GameEventClock) -> bool {
        self.applied_at <= time && self.disabled_at.unwrap_or(MAX_GAME_EVENT_CLOCK) > time
    }
}

#[derive(Serialize,Deserialize)]
pub struct ModifiersContainer<T: EntityData> {
    /// All modifiers that alter data of type T and are permanent, stored in chronological order
    pub(crate) modifiers: Vec<ModifierContainer<T>>,
    /// All modifiers that alter data of type T and are limited, stored in chronological order
//    pub(crate) limited_modifiers: Vec<ModifierContainer<T>>,
    pub(crate) modifiers_by_disabled_at: HashMap<GameEventClock, Vec<usize>>,
    /// All Dynamic modifiers, stored in chronological order
    pub(crate) dynamic_modifiers: Vec<ModifierContainer<T>>,
    /// The full set of entities that have dynamic modifiers for this data type
    pub(crate) dynamic_entity_set: HashSet<Entity>,
    pub(crate) modifier_archetypes: Vec<ModifierArchetypeContainer<T>>,
}


impl<T: EntityData> ModifiersContainer<T> {
    pub fn new() -> ModifiersContainer<T> {
        ModifiersContainer {
            modifiers: vec![],
            dynamic_modifiers: vec![],
            modifiers_by_disabled_at: HashMap::new(),
            dynamic_entity_set: HashSet::new(),
            modifier_archetypes: Vec::new(),
        }
    }

    pub fn constant_modifiers_for_entity<'a>(&'a self, entity: Entity) -> impl Iterator<Item=&ModifierContainer<T>> + 'a {
        self.modifiers.iter().filter(move |mc| mc.entity == entity)
    }
    pub fn dynamic_modifiers_for_entity<'a>(&'a self, entity: Entity) -> impl Iterator<Item=&ModifierContainer<T>> + 'a {
        self.dynamic_modifiers.iter().filter(move |mc| mc.entity == entity)
    }

    pub fn register_modifier_archetype(&mut self, modifier: Rc<Modifier<T>>) -> ModifierReference {
        self.modifier_archetypes.push(ModifierArchetypeContainer { modifier });
        ModifierReference(0, ModifierReferenceType::Archetype, self.modifier_archetypes.len() - 1)
    }
}


#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct DataContainer<T: EntityData> {
    pub(crate) storage: HashMap<Entity, T>,
    pub(crate) sentinel: T,
    pub(crate) entities_with_data: Vec<Entity>,
}


impl<T: EntityData> DataContainer<T> {
    pub fn new() -> DataContainer<T> {
        DataContainer {
            storage: HashMap::new(),
            sentinel: T::default(),
            entities_with_data: Vec::new(),
        }
    }
}


#[derive(Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize, Default)]
pub(crate) struct EntityContainer(pub(crate) Entity, pub(crate) GameEventClock);

#[derive(Clone)]
pub struct EntityIndex<T: Hash + Eq + Clone> {
    pub(crate) index: HashMap<T, Entity>
}

impl<T: Hash + Eq + Clone> EntityIndex<T> {
    pub fn new() -> EntityIndex<T> {
        EntityIndex {
            index: HashMap::new()
        }
    }

    pub fn update_from(&mut self, other: &EntityIndex<T>) {
        if other.index.len() > self.index.len() {
            self.index = other.index.clone();
        }
    }

    #[inline]
    pub fn get(&self, k: &T) -> Option<&Entity> {
        self.index.get(k)
    }
}


#[cfg(test)]
mod test {
    use serde;
    use super::*;

    use ron;
    use world::world::World;
    use entity::FieldVisitor;
    use modifiers::Modifier;
    use entity::EntityData;
    use std;

    use super::super::super::entity;
    use common::reflect::Field;



    #[derive(Serialize,Deserialize,Clone,Debug)]
    enum EventType2 {
        Foo,
        Bar,
        Default
    }
    impl Default for EventType2 {
        fn default() -> Self {
            EventType2::Default
        }
    }
    impl GameEventType for EventType2 {
        fn beginning_of_time_event() -> Self {
            EventType2::Foo
        }
    }

    use events::CoreEvent;

    #[test]
    pub fn test_multi_type_ser () {
        let mut container = MultiTypeEventContainer::new();

        container.register_event_type::<CoreEvent>();
        container.register_event_type::<EventType2>();

        container.push_event(GameEventWrapper::new(EventType2::Foo, GameEventState::Started, 1));
        container.push_event(GameEventWrapper::new(EventType2::Bar, GameEventState::Ended, 2));
        container.push_event(GameEventWrapper::new(CoreEvent::WorldInitialized, GameEventState::Started, 3));

        let serialized_str = ron::ser::to_string_pretty(&container, ron::ser::PrettyConfig::default()).expect("serialization failed");
        println!("Serialized======================\n{}", serialized_str);


        let mut container : MultiTypeEventContainer = ron::de::from_str(&serialized_str).expect("failed to read mte");
        container.register_event_type::<CoreEvent>();
        container.register_event_type::<EventType2>();

        println!("CoreEvents:\n{:?}", container.events::<CoreEvent>().collect_vec());
        println!("EventType2:\n{:?}", container.events::<EventType2>().collect_vec());

    }


    struct Foo<T> {
        t: T
    }

    #[derive(Clone, Default, PrintFields, Debug)]
    pub struct Bar {
        pub f: f32,
        pub b: i32,
    }

    impl Bar {
        pub const f: Field<Bar, f32> = Field::new(stringify!( f ), |t| &t.f, |t| &mut t.f, |t, v| { t.f = v; });
        pub const b: Field<Bar, i32> = Field::new(stringify!( b ), |t| &t.b, |t| &mut t.b, |t, v| { t.b = v; });
    }
    impl EntityData for Bar {}

    #[derive(Serialize, Deserialize)]
    pub struct Container {
        pub modifiers: Vec<Box<Modifier<Bar>>>
    }

    #[test]
    pub fn test_serialization_of_field_modifier_vec() {
        use reflect::*;
//
        let pretty_config = ron::ser::PrettyConfig {
            ..Default::default()
        };

        let mut test_values: Vec<Box<Modifier<Bar>>> = Vec::new();
        test_values.push(Bar::f.set_to(3.0));
        test_values.push(Bar::b.add(5));

        let value = Container {
            modifiers: test_values
        };

        let serialized_str = ron::ser::to_string_pretty(&value, pretty_config).expect("serialization failed");
        println!("Serialized======================\n{}", serialized_str);
//        let deserialized: Container<Bar> = ron::de::from_str(&serialized_str).unwrap();
        let deserialized: Container = ron::de::from_str(&serialized_str).unwrap();

        let mut bar = Bar { f: 0.0, b: -1 };
        let world = World::new();
        for wrapped in deserialized.modifiers {
            wrapped.modify(&mut bar, world.view());
        }
        println!("Bar: {:?}", bar)
    }
}