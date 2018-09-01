use common::prelude::*;

use common::reflect::*;
use game::reflect::*;
use game::modifiers::*;
use game::prelude::*;
use game::EntityData;
use std::collections::HashMap;


#[derive(Eq, Clone)]
pub struct AttributeType {
    name: Str,
    additive: bool,
    // whether combining two of these attributes results summed strength
    maximal: bool,
    // whether, if not additive, combining two of these attributes results in the max of the two
    minimal: bool,
    // whether, if not additive, combining two of these attributes results in the min of the two
    remove_on_zero: bool,
    // whether the trait should automatically be removed on zero
    minimum_value: i32,
    // the minimum value this attribute can have
    maximum_value: i32,
    // the maximum value this attribute can have
    description: Str,      // a description of the attribute
}

impl PartialEq<AttributeType> for AttributeType {
    fn eq(&self, other: &AttributeType) -> bool { self.name == other.name }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttributeValue(String, i32);


#[derive(Clone, Debug, PrintFields, Default, Serialize, Deserialize)]
pub struct AttributeData {
    pub(crate) attributes: Vec<AttributeValue>
}

impl AttributeData {
    pub fn value_for(&self, for_attr: &AttributeType) -> Option<i32> {
        AttributeData::value_for_intern(&self.attributes, for_attr)
    }
    fn value_for_intern(attributes : &Vec<AttributeValue>, for_attr: &AttributeType) -> Option<i32> {
        attributes.iter().find(|attr| attr.0 == for_attr.name).map(|attr| attr.1)
    }


    pub fn set_value_for(&mut self, set_attr: &AttributeType, value: i32) {
        AttributeData::set_value_for_intern(&mut self.attributes, set_attr, value);
    }
    fn set_value_for_intern(attributes : &mut Vec<AttributeValue>, set_attr : &AttributeType, value : i32) {
        attributes.retain(|attr| attr.0 != set_attr.name);
        let new_value = value.min(set_attr.maximum_value).min(set_attr.minimum_value);
        attributes.push(AttributeValue(String::from(set_attr.name), new_value));
    }


    pub fn add_value_to(&mut self, add_attr: &AttributeType, value: i32) {
        AttributeData::add_value_to_intern(&mut self.attributes, add_attr, value);
    }
    fn add_value_to_intern(attributes : &mut Vec<AttributeValue>, add_attr: &AttributeType, value: i32) {
        let existing = AttributeData::value_for_intern(attributes, add_attr);
        if let Some(existing) = existing {
            if add_attr.additive {
                // if we have an additive attribute and it already has an existing value, use that + arg
                let new_value = existing + value;
                AttributeData::set_value_for_intern(attributes, add_attr, new_value);
                return;
            }
        }
        // fall back on just setting the value to the given argument
        AttributeData::set_value_for_intern(attributes, add_attr, value);
    }
}

impl EntityData for AttributeData {}


impl AttributeData {
    pub const attributes: Field<AttributeData, Vec<AttributeValue>> = Field::new(stringify!( attributes ), |t| &t.attributes, |t| &mut t.attributes, |t, v| { t.attributes = v; });

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