use common::prelude::*;

use common::reflect::*;
use game::reflect::*;
use game::modifiers::*;
use game::prelude::*;
use game::EntityData;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use serde::Serialize;
use serde::Serializer;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::Visitor;
use std::fmt::Formatter;
use std::fmt::Error;
use std::ops::Add;
use std::fmt::Debug;
use std::ops::Sub;

pub mod attributes {
    use super::*;

    pub static TrainingWeapon: AttributeType = &AttributeTypeStruct {
        name: "training weapons",
        description: "this weapon is more suitable for practice than actual combat. It doesn't do much damage but it provides an experience bonus when used.",
        value_type: ValueType::BoundedNumeric(None, Some(3)),
        combination: Combination::Additive,
        remove_on_zero: true,
    };

    pub static Sentinel: AttributeType = &AttributeTypeStruct {
        name: "default",
        description: "default attribute, indicates that something couldn't be found",
        value_type: ValueType::Boolean,
        combination: Combination::Replace,
        remove_on_zero: true
    };


    pub static AllAttributes : &[AttributeType] = &[Sentinel, TrainingWeapon];

    pub fn attribute_with_name(name : &str) -> AttributeType {
        AllAttributes.iter().find(|at| at.name == name).unwrap_or(&&Sentinel)
    }
}

#[derive(PartialEq,Clone,Copy,Eq,Debug)]
pub enum ValueType {
    Boolean,
    Numeric,
    BoundedNumeric(Option<i32>, Option<i32>),
}

#[derive(PartialEq,Clone,Copy,Eq,Debug)]
pub enum Combination {
    Replace,
    Additive,
    Maximal,
    Minimal,
}

#[derive(Clone, Copy, Eq)]
pub struct AttributeTypeStruct {
    name: Str,
    // what kind of value is this represented as
    value_type: ValueType,
    // how this attribute should be combined, when multiple of the same key are added together
    combination: Combination,
    // whether the trait should automatically be removed on zero
    remove_on_zero: bool,
    // a description of the attribute
    description: Str,
}
pub type AttributeType = &'static AttributeTypeStruct;

impl Debug for AttributeTypeStruct {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.name)
    }
}
impl PartialEq<AttributeTypeStruct> for AttributeTypeStruct {
    fn eq(&self, other: &AttributeTypeStruct) -> bool { self.name == other.name }
}
impl Hash for AttributeTypeStruct {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
    }
}
impl Serialize for AttributeType {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
        serializer.serialize_str(self.name)
    }
}
struct ATVisitor;
impl <'de> Visitor<'de> for ATVisitor {
    type Value = AttributeType;

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "AttributeType name")
    }

    fn visit_str<E>(self, v: &str) -> Result<<Self as Visitor<'de>>::Value, E> where E: ::serde::de::Error, {
        Ok(attributes::attribute_with_name(v))
    }

    fn visit_string<E>(self, v: String) -> Result<<Self as Visitor<'de>>::Value, E> where E: ::serde::de::Error, {
        Ok(attributes::attribute_with_name(&v))
    }
}
impl <'de> Deserialize<'de> for AttributeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        deserializer.deserialize_str(ATVisitor)
    }
}
impl Add for AttributeValue {
    type Output = AttributeValue;
    fn add(self, rhs: AttributeValue) -> AttributeValue {
        let AttributeValue(attr_type, amnt_a) = self;
        let AttributeValue(other_attr_type, amnt_b) = rhs;

        if attr_type != other_attr_type {
            warn!("Attempting to combine two attributes of different types, {:?} and {:?}", attr_type, other_attr_type);
            self
        } else {
            let new_raw_value = match attr_type.combination {
                Combination::Additive => amnt_a + amnt_b,
                Combination::Maximal => amnt_a.max(amnt_b),
                Combination::Minimal => amnt_a.min(amnt_b),
                Combination::Replace => amnt_b
            };

            let new_effective_value = match attr_type.value_type {
                ValueType::Boolean => if new_raw_value > 0 { 1 } else { 0 },
                ValueType::Numeric => new_raw_value,
                ValueType::BoundedNumeric(min, max) => {
                    let mind = if let Some(min) = min { new_raw_value.min(min) } else { new_raw_value };
                    let maxd = if let Some(max) = max { mind.max(max) } else { mind };
                    maxd
                }
            };

            AttributeValue(attr_type, new_effective_value)
        }
    }
}
impl Sub for AttributeValue {
    type Output = AttributeValue;
    fn sub(self, rhs: AttributeValue) -> AttributeValue {
        let AttributeValue(attr_type, amnt_a) = self;
        let AttributeValue(other_attr_type, amnt_b) = rhs;

        if attr_type != other_attr_type {
            warn!("Attempting to combine two attributes of different types, {:?} and {:?}", attr_type, other_attr_type);
            self
        } else {
            let new_raw_value = match attr_type.combination {
                Combination::Additive => amnt_a - amnt_b,
                Combination::Maximal => amnt_a.min(amnt_b),
                Combination::Minimal => amnt_a.max(amnt_b),
                Combination::Replace => amnt_b
            };

            let new_effective_value = match attr_type.value_type {
                ValueType::Boolean => if new_raw_value > 0 { 1 } else { 0 },
                ValueType::Numeric => new_raw_value,
                ValueType::BoundedNumeric(min, max) => {
                    let mind = if let Some(min) = min { new_raw_value.min(min) } else { new_raw_value };
                    let maxd = if let Some(max) = max { mind.max(max) } else { mind };
                    maxd
                }
            };

            AttributeValue(attr_type, new_effective_value)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
pub struct AttributeValue(AttributeType, i32);
impl Default for AttributeValue {
    fn default() -> Self {
        AttributeValue(attributes::Sentinel, 0)
    }
}


#[derive(Clone, Debug, Fields, Default, Serialize, Deserialize)]
pub struct AttributeData {
//    pub(crate) attributes: Vec<AttributeValue>
    pub(crate) attributes: HashMap<String, AttributeValue>
}

pub fn increase_attribute(world : &mut World, entity : Entity, attr: AttributeType, by : i32) {
    world.modify(entity, AttributeData::attributes.add_to_key(attr.name.to_string(), AttributeValue(attr, by)));
//    let attr_data = world.view().data::<AttributeData>(entity);
//    if let Some(existing) = attr_data.attributes.get(attr.name) {
//
//    } else {
//        world.modify(entity, )
//    }
}
pub fn decrease_attribute(world : &mut World, entity : Entity, attr: AttributeType, by : i32) {
    world.modify(entity, AttributeData::attributes.add_to_key(attr.name.to_string(), AttributeValue(attr, -by)));
}

impl AttributeData {
    pub fn value_for(&self, for_attr: AttributeType) -> Option<i32> {
        self.attributes.get(for_attr.name).map(|attr| attr.1)
    }


    pub fn increase_attribute(&mut self, attr : AttributeType, by : i32) {
        let new_value = if let Some(existing) = self.attributes.get(attr.name) {
            *existing + AttributeValue(attr, by)
        } else {
            AttributeValue(attr, by)
        };
        self.attributes.insert(attr.name.to_string(), new_value);
    }

    pub fn decrease_attribute(&mut self, attr : AttributeType, by : i32) {
        self.increase_attribute(attr, -by);
    }

//    pub fn value_for(&self, for_attr: &AttributeTypeStruct) -> Option<i32> {
//        AttributeData::value_for_intern(&self.attributes, for_attr)
//    }
//    fn value_for_intern(attributes: &Vec<AttributeValue>, for_attr: &AttributeTypeStruct) -> Option<i32> {
//        attributes.iter().find(|attr| attr.0 == for_attr.name).map(|attr| attr.1)
//    }


//    pub fn set_value_for(&mut self, set_attr: &AttributeTypeStruct, value: i32) {
//        AttributeData::set_value_for_intern(&mut self.attributes, set_attr, value);
//    }
//    fn set_value_for_intern(attributes: &mut Vec<AttributeValue>, set_attr: &AttributeTypeStruct, value: i32) {
//        attributes.retain(|attr| attr.0 != set_attr.name);
//        let new_value = value.min(set_attr.maximum_value).min(set_attr.minimum_value);
//        attributes.push(AttributeValue(String::from(set_attr.name), new_value));
//    }
//
//
//    pub fn add_value_to(&mut self, add_attr: &AttributeTypeStruct, value: i32) {
//        AttributeData::add_value_to_intern(&mut self.attributes, add_attr, value);
//    }
//    fn add_value_to_intern(attributes: &mut Vec<AttributeValue>, add_attr: &AttributeTypeStruct, value: i32) {
//        let existing = AttributeData::value_for_intern(attributes, add_attr);
//        if let Some(existing) = existing {
//            if add_attr.additive {
//                // if we have an additive attribute and it already has an existing value, use that + arg
//                let new_value = existing + value;
//                AttributeData::set_value_for_intern(attributes, add_attr, new_value);
//                return;
//            }
//        }
//        // fall back on just setting the value to the given argument
//        AttributeData::set_value_for_intern(attributes, add_attr, value);
//    }
}

impl EntityData for AttributeData {}


impl AttributeData {
//    pub const attributes: Field<AttributeData, Vec<AttributeValue>> = Field::new(stringify!( attributes ), |t| &t.attributes, |t| &mut t.attributes, |t, v| { t.attributes = v; });

//    pub fn set_value(attr : &AttributeType, value : i32) -> Box<FieldModifier<AttributeData, Vec<AttributeValue>>> {
////        FieldModifier::permanent(&AttributeData::attributes, SetAttributeValue { attr : attr.clone(), value })
//    }
}

//struct SetAttributeValue {attr : AttributeType, value : i32}
//impl FieldTransformation<Vec<AttributeValue>> for SetAttributeValue {
//    fn apply(&self, current: &mut Vec<AttributeValue>) {
//        AttributeData::set_value_for_intern(current, &self.attr, self.value)
//    }
//
//    fn description(&self) -> Transformation {
//        Transformation::Custom(format!("Set {} to {}", self.attr.name, self.value))
//    }
//}
//
//
//struct AddAttributeValue {attr : AttributeType, value : i32}
//impl FieldTransformation<Vec<AttributeValue>> for AddAttributeValue {
//    fn apply(&self, current: &mut Vec<AttributeValue>) {
//        AttributeData::add_value_to_intern(current, &self.attr, self.value)
//    }
//
//    fn description(&self) -> Transformation {
//        Transformation::Custom(format!("Added {} to {}", self.value, self.attr.name))
//    }
//}