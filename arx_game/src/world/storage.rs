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

pub type ModifierClock = usize;
pub type EventCallback<E> = fn(&mut World, &GameEventWrapper<E>);


pub(crate) struct MultiTypeEventContainer {
    pub(crate) event_containers: Map<CloneAny>,
    pub(crate) clone_up_to_time_funcs: Vec<fn(&mut MultiTypeEventContainer, &MultiTypeEventContainer, GameEventClock)>,
    pub(crate) update_to_time_funcs: Vec<fn(&mut MultiTypeEventContainer, &MultiTypeEventContainer, GameEventClock)>,
}

#[derive(Clone)]
pub(crate) struct EventContainer<E: GameEventType> {
    pub(crate) events: Vec<GameEventWrapper<E>>,
    pub(crate) default: GameEventWrapper<E>,
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
            event_containers: Map::new(),
            clone_up_to_time_funcs: Vec::new(),
            update_to_time_funcs: Vec::new(),
        }
    }
    pub(crate) fn register_event_type<E: GameEventType + 'static>(&mut self) {
//        println!("Registering event of type {:?}, nth registered: {:?}", unsafe {std::intrinsics::type_name::<E>()}, self.event_containers.len());
//        println!("MTE: {:?}", (self as *const MultiTypeEventContainer));
        let evt_container: EventContainer<E> = EventContainer::default();
        self.event_containers.insert::<EventContainer<E>>(evt_container);

        self.clone_up_to_time_funcs.push(|mte: &mut MultiTypeEventContainer, from: &MultiTypeEventContainer, time: GameEventClock| {
            mte.register_event_type::<E>();

            mte.event_containers.get_mut::<EventContainer<E>>().expect("just created, can't not exist").events =
                from.events::<E>().filter(|e| e.occurred_at <= time).cloned().collect();
        });

        self.update_to_time_funcs.push(|mte: &mut MultiTypeEventContainer, from: &MultiTypeEventContainer, end_time: GameEventClock| {
            let cur_high_time = mte.event_containers.get::<EventContainer<E>>().expect("event type must be registered").events
                .last()
                .map(|ec| ec.occurred_at)
                .unwrap_or(0);

            // events are stored in order oldest to newest, so we start from the back, taking newest to oldest, ignore all that are newer
            // than our target time, take all that are newer than our start time, now we have all new events ordered newest to oldest. Take
            // that and reverse it and we can tack it onto the end.
            let mut new_events = from.event_containers.get::<EventContainer<E>>().expect("other must have event type registered").events.iter()
                .rev()
                .skip_while(|ec| ec.occurred_at > end_time)
                .take_while(|ec| ec.occurred_at > cur_high_time)
                .cloned()
                .collect_vec();
            new_events.reverse();

            mte.event_containers.get_mut::<EventContainer<E>>().expect("event type must be registered").events.extend(new_events);
        });
    }
    pub(crate) fn add_callback<E: GameEventType + 'static>(&mut self, callback: EventCallback<E>) {
        let event_container = self.event_containers.get_mut::<EventContainer<E>>().expect("attempted to push event of non-recognized event type");
        event_container.event_listeners.push(callback);
    }
    pub(crate) fn push_event<E: GameEventType + 'static>(&mut self, evt: GameEventWrapper<E>) -> Vec<EventCallback<E>> {
        let event_container = self.event_containers.get_mut::<EventContainer<E>>().expect("attempted to push event of non-recognized event type");
        event_container.events.push(evt);
        event_container.event_listeners.clone()
    }
    pub(crate) fn events<E: GameEventType + 'static>(&self) -> impl Iterator<Item=&GameEventWrapper<E>> {
//        println!("Retrieving event of type {:?} [{:?} total event types registered]", unsafe {std::intrinsics::type_name::<E>()}, self.event_containers.len());
//        println!("MTE: {:?}", (self as *const MultiTypeEventContainer));

        self.event_containers.get::<EventContainer<E>>().map(|e| e.events.iter()).expect("attempted to retrieve events of a non-recognized event type")
    }
    pub(crate) fn revents<E: GameEventType + 'static>(&self) -> impl Iterator<Item=&GameEventWrapper<E>> {
        self.event_containers.get::<EventContainer<E>>().map(|e| e.events.iter().rev()).expect("attempted to retrieve events of a non-recognized event type")
    }
    pub(crate) fn most_recent_event<E: GameEventType + 'static>(&self) -> &GameEventWrapper<E> {
        let container = self.event_containers.get::<EventContainer<E>>().expect("attempted to retrieve most recent event of a non-recognized event type");
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

pub struct ModifierContainer<T: EntityData> {
    pub(crate) modifier: Rc<Modifier<T>>,
    pub(crate) applied_at: GameEventClock,
    pub(crate) disabled_at: Option<GameEventClock>,
    pub(crate) modifier_index: ModifierClock,
    pub(crate) entity: Entity,
    pub(crate) description: Option<Str>,
}

pub struct ModifierArchetypeContainer<T: EntityData> {
    pub(crate) modifier: Rc<Modifier<T>>,
}

impl<T: EntityData> ModifierContainer<T> {
    pub fn is_active_at_time(&self, time: GameEventClock) -> bool {
        self.applied_at <= time && self.disabled_at.unwrap_or(MAX_GAME_EVENT_CLOCK) > time
    }
}

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
        ModifierReference(TypeId::of::<T>(), ModifierReferenceType::Archetype, self.modifier_archetypes.len() - 1)
    }
}


#[derive(Clone)]
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


#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
    use ron::ser;
    use ron;
    use erased_serde;

//    pub trait TestTrait: erased_serde::Serialize {
//        fn moo(&self);
//    }
//
//    #[derive(Serialize, Deserialize)]
//    pub struct Foo {
//        pub i: i32
//    }
//
//    impl TestTrait for Foo {
//        fn moo(&self) { println!("Foo with {}", self.i) }
//    }
//
//    #[derive(Serialize, Deserialize, PrintFields)]
    #[derive(Clone,Debug,Default,PrintFields)]
    pub struct Bar {
        pub f: f32,
        pub b: i32,
    }
//
//    impl TestTrait for Bar {
//        fn moo(&self) { println!("Bar with {}", self.f) }
//    }



    use EntityData;
    use common::reflect::Field;
    use serde::Serializer;
    use std::fmt;
    use serde::Serialize;
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::ser::SerializeStruct;
    use serde::ser::SerializeTuple;
    use serde::de::Visitor;
    use std::fmt::Formatter;
    use std::fmt::Error;
    use serde::de::SeqAccess;

    impl Bar {
        pub const f: Field<Bar, f32> = Field::new(stringify!( f ), |t| &t.f, |t| &mut t.f, |t, v| { t.f = v; });
        pub const b: Field<Bar, i32> = Field::new(stringify!( b ), |t| &t.b, |t| &mut t.b, |t, v| { t.b = v; });

//        pub fn visit_all_fields<T>
    }
    impl EntityData for Bar {}


    pub trait ApplyField : erased_serde::Serialize {
        fn print(&self, bar : &Bar);
    }

    use FieldTransformation;

    pub struct FieldContainer<T : 'static + fmt::Debug + Serialize> {
        pub field : &'static Field<Bar, T>,
        pub transform : Box<FieldTransformation<T>>,
    }

    impl <T : 'static + fmt::Debug + Serialize> ApplyField for FieldContainer<T> {
        fn print(&self, bar : &Bar) {
            println!("Printing: {:?} -> {:?}", (self.field.getter)(bar), self.set_to)
        }
    }

    impl <T : 'static + Serialize + fmt::Debug> Serialize for FieldContainer<T> {
        fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
            S: Serializer {
            let mut state = serializer.serialize_tuple(3)?;
            state.serialize_element(&self.field.name)?;
            state.serialize_element(&self.transform.name())?;
            state.serialize_element(&self.set_to)?;
            state.end()
        }
    }

    serialize_trait_object!(ApplyField);

    #[derive(Serialize, Deserialize)]
    pub struct Container {
//        pub test_values: Vec<Box<TestTrait>>
        pub field_applications : Vec<Box<ApplyField>>
    }

    use serde::de;
    impl <'de> Deserialize<'de> for Box<ApplyField> {
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
            struct FieldContainerVisitor;
            impl <'de> Visitor<'de> for FieldContainerVisitor {
                type Value = Box<ApplyField>;

                fn expecting(&self, formatter: &mut Formatter) -> Result<(), Error> {
                    write!(formatter, "Something we can turn into a field container")
                }

                fn visit_seq<V>(self, mut seq: V) -> Result<Box<ApplyField>, V::Error> where V: SeqAccess<'de> {
                    let field_name : String = seq.next_element()?.ok_or_else(||panic!("paniced too early"))?;
                    let boxed_res : Box<ApplyField> = match field_name.as_str() {
                        "f" => box FieldContainer { field : &Bar::f, set_to : seq.next_element()?.ok_or_else(||panic!("set_to could not be deserialized"))? },
                        "b" => box FieldContainer { field : &Bar::b, set_to : seq.next_element()?.ok_or_else(||panic!("set_to could not be deserialized"))? },
                        _ => panic!("invalid field")
                    };
                    Ok(boxed_res)
                }
            }
            deserializer.deserialize_tuple(2, FieldContainerVisitor)
        }
    }


    struct Tmp {
        field_transformation_deserializers : Vec<fn(Box<erased_serde::Deserializer>) -> Option<Box<FieldTransformation<T>>>>
    }



    #[test]
    pub fn test_serialization() {
//        use reflect::FieldModifier;
//
        let pretty_config = ser::PrettyConfig {
            ..Default::default()
        };


        let mut test_values: Vec<Box<ApplyField>> = Vec::new();
        test_values.push(box FieldContainer { field : &Bar::f, set_to : 1.0 });
        test_values.push(box FieldContainer { field : &Bar::b, set_to : 7 });

        let value = Container {
            field_applications : test_values
        };

        let serialized_str = ser::to_string_pretty(&value, pretty_config).expect("serialization failed");
        println!("Serialized======================\n{}", serialized_str);
        let deserialized : Container = ron::de::from_str(&serialized_str).unwrap();

        let bar = Bar { f : 0.0, b : -1 };
        for wrapped in deserialized.field_applications {
            wrapped.print(&bar);
        }
    }
}