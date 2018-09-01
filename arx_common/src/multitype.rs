use serde::Serialize;
use ron;
use bincode;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use anymap::any::CloneAny;
use anymap::Map;
use std::collections::HashMap;
use serde::Deserializer;
use serialize::SerializableError::InvalidDataFormat;
use serde::Serializer;
use serde::de::Visitor;
use std::fmt::Formatter;
use std::fmt;
use serde::de::MapAccess;
use serialize::SerializableError;

pub struct MultiTypeContainer {
    storage : Map<CloneAny>,
    serialized_string_data : HashMap<String,String>,
    serialized_byte_data : HashMap<String,Vec<u8>>,
    serialize_to_string_functions: HashMap<String, fn(&MultiTypeContainer) -> Result<String,SerializableError>>,
    serialize_to_bytes_functions: HashMap<String, fn(&MultiTypeContainer) -> Result<Vec<u8>,SerializableError>>
}

impl MultiTypeContainer {
    pub fn new() -> MultiTypeContainer {
        MultiTypeContainer {
            storage : Map::new(),
            serialized_string_data : HashMap::new(),
            serialized_byte_data : HashMap::new(),
            serialize_to_string_functions : HashMap::new(),
            serialize_to_bytes_functions : HashMap::new(),
        }
    }

    pub fn register<U>(&mut self) where U : Serialize + DeserializeOwned + Clone + Default + 'static {
        let type_name = unsafe {::std::intrinsics::type_name::<U>()};
        if let Some(serialized) = self.serialized_string_data.remove(type_name) {
            let deserialized : U = ron::de::from_str(&serialized).expect(format!("could not deserialize string on register of type {}", type_name).as_str());
            self.storage.insert(deserialized);
        } else if let Some(serialized) = self.serialized_byte_data.remove(type_name) {
            let deserialized : U = bincode::deserialize(serialized.as_slice()).expect(format!("could not deserialize binary on register of type {}", type_name).as_str());
            self.storage.insert(deserialized);
        } else {
            self.storage.insert(U::default());
        }

        self.serialize_to_string_functions.insert(String::from(type_name), |mte: &MultiTypeContainer| {
            let value = mte.storage.get::<U>().expect("registered type must always be present");
            ron::ser::to_string(value).map_err(|_e| SerializableError::Error)
        });
        self.serialize_to_bytes_functions.insert(String::from(type_name), |mte: &MultiTypeContainer| {
            let value = mte.storage.get::<U>().expect("registered type must always be present");
            bincode::serialize(value).map_err(|_e| SerializableError::Error)
        });
    }

    pub fn get<U>(&self) -> &U where U : Clone + 'static {
        self.storage.get::<U>().unwrap_or_else(|| panic!(format!("MultiTypeContainer received request for type {}, but that type was not registered", unsafe {::std::intrinsics::type_name::<U>()})))
    }

    pub fn get_mut<U>(&mut self) -> &mut U where U : Clone + 'static {
        self.storage.get_mut::<U>().unwrap_or_else(|| panic!(format!("MultiTypeContainer received request for type {}, but that type was not registered", unsafe {::std::intrinsics::type_name::<U>()})))
    }
}

impl Serialize for MultiTypeContainer {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
        use serde::ser::SerializeMap;
        use serde::ser::Error;

        let human_readable = serializer.is_human_readable();

        let mut map = serializer.serialize_map(Some(self.storage.len()))?;
        if human_readable {
            for (type_name, func) in &self.serialize_to_string_functions {
                map.serialize_key(type_name)?;
                let string_form = (func)(self).map_err(|_e| S::Error::custom("custom err map over for str"))?;
                map.serialize_value(&string_form)?;
            }
        } else {
            for (type_name, func) in &self.serialize_to_bytes_functions {
                map.serialize_key(type_name)?;
                let byte_form = (func)(self).map_err(|_e| S::Error::custom("custom err map over for bytes"))?;
                map.serialize_value(&byte_form)?;
            }
        }
        map.end()
    }
}

impl <'de> Deserialize<'de> for MultiTypeContainer {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        struct MapVisitor {
            human_readable : bool
        }
        impl <'de2> Visitor<'de2> for MapVisitor {
            type Value = MultiTypeContainer;

            fn expecting(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
                write!(formatter, "MultiTypeContainer")
            }

            fn visit_map<A>(self, mut map: A) -> Result<<Self as Visitor<'de2>>::Value, <A as MapAccess<'de2>>::Error> where A: MapAccess<'de2>, {
                let mut container = MultiTypeContainer::new();
                if self.human_readable {
                    while let Some((key, value)) = map.next_entry()? {
                        container.serialized_string_data.insert(key, value);
                    }
                } else {
                    while let Some((key, value)) = map.next_entry()? {
                        container.serialized_byte_data.insert(key, value);
                    }
                }
                Ok(container)
            }
        }

        let visitor = MapVisitor { human_readable : deserializer.is_human_readable() };
        deserializer.deserialize_map(visitor)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[derive(Serialize,Deserialize,Clone,Debug,PartialEq)]
    struct Foo {
        a : i32
    }

    #[derive(Serialize,Deserialize,Clone,Debug,PartialEq)]
    struct Bar {
        b : String
    }

    #[test]
    pub fn test_multi_type_container_serialization_string() {
        use spectral::prelude::*;

        let mut container = MultiTypeContainer::new();

        container.register::<Vec<Foo>>();
        container.register::<Vec<Bar>>();

        let foo_vec = container.get_mut::<Vec<Foo>>();
        foo_vec.push(Foo { a : 3 });
        foo_vec.push(Foo { a : 2 });

        let bar_vec = container.get_mut::<Vec<Bar>>();
        bar_vec.push(Bar { b : String::from("Hello") });

        let serialized = ron::ser::to_string(&container).expect("container could not serialize to string");
        println!("Serialized multi type container: {}", serialized);


        let mut container_2 : MultiTypeContainer = ron::de::from_str(&serialized).expect("container could not deserialize from string");
        container_2.register::<Vec<Bar>>();
        container_2.register::<Vec<Foo>>();

        let foo_vec = container_2.get::<Vec<Foo>>();
        assert_that(foo_vec).contains(Foo { a : 3});
        assert_that(foo_vec).contains(Foo { a : 2});

        let bar_vec = container_2.get::<Vec<Bar>>();
        assert_that(bar_vec).contains(Bar { b : String::from("Hello")});
    }

    #[test]
    pub fn test_multi_type_container_serialization_binary() {
        use spectral::prelude::*;

        let mut container = MultiTypeContainer::new();

        container.register::<Vec<Foo>>();
        container.register::<Vec<Bar>>();

        let foo_vec = container.get_mut::<Vec<Foo>>();
        foo_vec.push(Foo { a : 3 });
        foo_vec.push(Foo { a : 2 });

        let bar_vec = container.get_mut::<Vec<Bar>>();
        bar_vec.push(Bar { b : String::from("Hello") });

        let serialized = bincode::serialize(&container).expect("container could not serialize to bytes");

        let mut container_2 : MultiTypeContainer = bincode::deserialize(&serialized).expect("container could not deserialize from bytes");
        container_2.register::<Vec<Bar>>();
        container_2.register::<Vec<Foo>>();

        let foo_vec = container_2.get::<Vec<Foo>>();
        assert_that(foo_vec).contains(Foo { a : 3});
        assert_that(foo_vec).contains(Foo { a : 2});

        let bar_vec = container_2.get::<Vec<Bar>>();
        assert_that(bar_vec).contains(Bar { b : String::from("Hello")});
    }
}