use common::prelude::*;
use prelude::*;
use entity::EntityData;
//use modifiers::ConstantModifier;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use modifiers::Modifier;
use modifiers::ModifierType;
use modifiers::FieldModification;
use std::fmt::Display;
use std::ops;
use common::reflect::Field;
use modifiers::Transformation;
use std::hash::Hash;
use std::collections::HashMap;
use serde;
use erased_serde;
use serde::Serializer;
use serde::Serialize;


pub trait SettableField<E: EntityData, T: 'static> {
    fn set_to(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
}

impl<E, T: 'static> SettableField<E, T> for Field<E, T> where E: EntityData, T: Clone + serde::Serialize {
    fn set_to(&'static self, new_value: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::SetTo(new_value)) }
}

pub trait SetKeyableField<E: EntityData, K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize> {
    fn set_key_to(&'static self, key: K, new_value: V) -> Box<FieldModifier<E, HashMap<K, V>>>;
    fn remove_key(&'static self, key: K) -> Box<FieldModifier<E, HashMap<K, V>>>;
}

impl<E, K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize> SetKeyableField<E, K, V> for Field<E, HashMap<K, V>> where E: EntityData {
    fn set_key_to(&'static self, key: K, new_value: V) -> Box<FieldModifier<E, HashMap<K, V>>> { FieldModifier::permanent(self, transformations::SetKeyTo(key, new_value)) }
    fn remove_key(&'static self, key: K) -> Box<FieldModifier<E, HashMap<K, V>>> { FieldModifier::permanent(self, transformations::RemoveKey(key)) }
}

pub trait AddToKeyableField<E: EntityData, K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize + ops::Add<Output=V> + ops::Sub<Output=V>> {
    fn add_to_key(&'static self, key: K, new_value: V) -> Box<FieldModifier<E, HashMap<K, V>>>;
    fn sub_from_key(&'static self, key: K, new_value: V) -> Box<FieldModifier<E, HashMap<K, V>>>;
}

impl<E, K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize + ops::Add<Output=V> + ops::Sub<Output=V> + Default + ops::Neg<Output=V>> AddToKeyableField<E, K, V> for Field<E, HashMap<K, V>> where E: EntityData {
    fn add_to_key(&'static self, key: K, new_value: V) -> Box<FieldModifier<E, HashMap<K, V>>> { FieldModifier::permanent(self, transformations::AddToKey(key, new_value)) }
    fn sub_from_key(&'static self, key: K, new_value: V) -> Box<FieldModifier<E, HashMap<K, V>>> { FieldModifier::permanent(self, transformations::AddToKey(key, -new_value)) }
}

pub trait AddableField<E: EntityData, T: 'static> {
    fn add(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
}

impl<E, T: 'static> AddableField<E, T> for Field<E, T> where E: EntityData, T: Clone + ops::Add<Output=T> + serde::Serialize {
    fn add(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Add(amount)) }
}

pub trait SubableField<E: EntityData, T: 'static> {
    fn sub(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
}

impl<E, T: 'static> SubableField<E, T> for Field<E, T> where E: EntityData, T: Clone + ops::Sub<Output=T> + ops::Neg<Output=T> + serde::Serialize {
    fn sub(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Sub(amount)) }
}

pub trait MulableField<E: EntityData, T: 'static> {
    fn mul(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
}

impl<E, T: 'static> MulableField<E, T> for Field<E, T> where E: EntityData, T: Clone + ops::Mul<Output=T> + serde::Serialize {
    fn mul(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Mul(amount)) }
}

pub trait DivableField<E: EntityData, T: 'static> {
    fn div(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
}

impl<E, T: 'static> DivableField<E, T> for Field<E, T> where E: EntityData, T: Clone + ops::Div<Output=T> + serde::Serialize {
    fn div(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Div(amount)) }
}

pub trait ReduceableField<E: EntityData, T: ReduceableType + 'static> {
    fn reduce_by(&'static self, new_value: T) -> Box<FieldModifier<E, Reduceable<T>>>;

    fn reduce_to(&'static self, new_value: T) -> Box<FieldModifier<E, Reduceable<T>>>;

    fn recover_by(&'static self, new_value: T) -> Box<FieldModifier<E, Reduceable<T>>>;

    fn increase_by(&'static self, new_value: T) -> Box<FieldModifier<E, Reduceable<T>>>;

    fn reset(&'static self) -> Box<FieldModifier<E, Reduceable<T>>>;
}

impl<E, T: 'static> ReduceableField<E, T> for Field<E, Reduceable<T>> where E: EntityData, T: Clone + ReduceableType + serde::Serialize {
    fn reduce_by(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::ReduceBy(amount)) }

    fn reduce_to(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::ReduceBy(amount)) }

    fn recover_by(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::RecoverBy(amount)) }

    fn increase_by(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::IncreaseBy(amount)) }

    fn reset(&'static self) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::Reset)}
}


pub trait VecField<E: EntityData, T: Clone + PartialEq + 'static> {
    fn append(&'static self, new_value: T) -> Box<FieldModifier<E, Vec<T>>>;
    fn remove(&'static self, new_value: T) -> Box<FieldModifier<E, Vec<T>>>;
}

impl<E, T: Clone + PartialEq + 'static + serde::Serialize> VecField<E, T> for Field<E, Vec<T>> where E: EntityData {
    fn append(&'static self, new_value: T) -> Box<FieldModifier<E, Vec<T>>> { FieldModifier::permanent(self, transformations::Append(new_value)) }
    fn remove(&'static self, new_value: T) -> Box<FieldModifier<E, Vec<T>>> { FieldModifier::permanent(self, transformations::Remove(new_value)) }
}

pub trait GameDisplayable {
    fn to_game_str_full(&self, &WorldView) -> String;
}

impl<T> GameDisplayable for Option<T> where T: GameDisplayable {
    fn to_game_str_full(&self, view: &WorldView) -> String {
        match self {
            Some(inner) => inner.to_game_str_full(view),
            None => strf("none")
        }
    }
}


pub trait FieldTransformation<T>: erased_serde::Serialize where T: serde::Serialize {
    fn apply(&self, current: &mut T);
    fn description(&self) -> Transformation;
    fn name(&self) -> Str {
        "default"
    }
}
serialize_trait_object!(<T> FieldTransformation<T>);

pub mod transformations {
    use super::*;
    use std::hash::Hash;
    use std::collections::HashMap;
    use std::hash::BuildHasher;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct SetTo<T: Clone + serde::Serialize>(pub T);

    impl<T: Clone + 'static + serde::Serialize> FieldTransformation<T> for SetTo<T> {
        fn apply(&self, current: &mut T) { *current = self.0.clone() }
        fn description(&self) -> Transformation {
            let cloned = self.0.clone();
            Transformation::Set(box self.0.clone())
        }
        fn name(&self) -> Str { "set" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Add<T: Clone + ops::Add<Output=T> + serde::Serialize>(pub T);

    impl<T: Clone + ops::Add<Output=T> + 'static + serde::Serialize> FieldTransformation<T> for Add<T> {
        fn apply(&self, current: &mut T) { *current = current.clone() + self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Add(box self.0.clone()) }
        fn name(&self) -> Str { "add" }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Sub<T: Clone + ops::Sub<Output=T> + serde::Serialize>(pub T);

    impl<T: Clone + ops::Sub<Output=T> + ops::Neg<Output=T> + 'static + serde::Serialize> FieldTransformation<T> for Sub<T> {
        fn apply(&self, current: &mut T) { *current = current.clone() - self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Add(box (-self.0.clone())) }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Mul<T: Clone + ops::Mul<Output=T> + serde::Serialize>(pub T);

    impl<T: Clone + ops::Mul<Output=T> + 'static + serde::Serialize> FieldTransformation<T> for Mul<T> {
        fn apply(&self, current: &mut T) { *current = current.clone() * self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Mul(box self.0.clone()) }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Div<T: Clone + ops::Div<Output=T> + serde::Serialize + 'static>(pub T);

    impl<T: Clone + ops::Div<Output=T> + serde::Serialize> FieldTransformation<T> for Div<T> {
        fn apply(&self, current: &mut T) { *current = current.clone() / self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Div(box self.0.clone()) }
    }

    #[derive(Serialize, Deserialize)]
    pub struct ReduceBy<R: ReduceableType + serde::Serialize + 'static>(pub R);

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for ReduceBy<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.reduce_by(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Reduce(box self.0.clone()) }
    }

    #[derive(Serialize, Deserialize)]
    pub struct ReduceTo<R: ReduceableType + serde::Serialize + 'static>(pub R);

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for ReduceTo<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.reduce_to(self.0.clone()) }
        fn description(&self) -> Transformation {
            let cloned = self.0.clone();
            Transformation::Set(box self.0.clone())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct RecoverBy<R: ReduceableType + serde::Serialize + 'static>(pub R);

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for RecoverBy<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.recover_by(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Recover(box self.0.clone()) }
    }

    #[derive(Serialize, Deserialize)]
    pub struct IncreaseBy<R: ReduceableType + 'static + serde::Serialize>(pub R);

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for IncreaseBy<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.increase_by(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Add(box self.0.clone()) }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Reset;

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for Reset {
        fn apply(&self, current: &mut Reduceable<R>) { current.reset() }
        fn description(&self) -> Transformation { Transformation::Reset }
    }

    #[derive(Serialize, Deserialize)]
    pub struct AddToKey<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize> (pub K, pub V);

    impl<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + serde::Serialize + ops::Add<Output=V> + ops::Sub<Output=V> + Default, S: BuildHasher> FieldTransformation<HashMap<K, V, S>>
    for AddToKey<K, V> {
        fn apply(&self, current: &mut HashMap<K, V, S>) {
            let cur = current.entry(self.0.clone()).or_insert_with(|| V::default());
            *cur = cur.clone() + self.1.clone();
        }

        fn description(&self) -> Transformation {
            Transformation::AddToKey(box self.0.clone(), box self.1.clone())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct SetKeyTo<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize> (pub K, pub V);

    impl<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + serde::Serialize, S: BuildHasher> FieldTransformation<HashMap<K, V, S>> for SetKeyTo<K, V> {
        fn apply(&self, current: &mut HashMap<K, V, S>) {
            current.insert(self.0.clone(), self.1.clone());
        }

        fn description(&self) -> Transformation {
            Transformation::SetKey(box self.0.clone(), box self.1.clone())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct RemoveKey<K: Clone + Hash + Eq + 'static + serde::Serialize> (pub K);

    impl<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + serde::Serialize, S: BuildHasher> FieldTransformation<HashMap<K, V, S>> for RemoveKey<K> {
        fn apply(&self, current: &mut HashMap<K, V, S>) {
            current.remove(&self.0);
        }

        fn description(&self) -> Transformation {
            Transformation::RemoveKey(box self.0.clone())
        }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Append<R: Clone + 'static + serde::Serialize>(pub R);

    impl<R: Clone + 'static + serde::Serialize> FieldTransformation<Vec<R>> for Append<R> {
        fn apply(&self, current: &mut Vec<R>) { current.push(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Append(box self.0.clone()) }
    }

    #[derive(Serialize, Deserialize)]
    pub struct Remove<R: Clone + PartialEq + 'static + serde::Serialize>(pub R);

    impl<R: Clone + PartialEq + 'static + serde::Serialize> FieldTransformation<Vec<R>> for Remove<R> {
        fn apply(&self, current: &mut Vec<R>) { current.remove_item(&self.0); }
        fn description(&self) -> Transformation { Transformation::Remove(box self.0.clone()) }
    }
}


pub struct FieldModifier<E: EntityData, T: 'static> {
    pub(crate) field: &'static Field<E, T>,
    pub(crate) transform: Box<FieldTransformation<T>>,
}

use serde::ser::SerializeTuple;

impl<E: EntityData, T> Serialize for FieldModifier<E, T> where T: Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
        let mut tuple_state = serializer.serialize_tuple(3)?;
        tuple_state.serialize_element(&self.field.name)?;
        tuple_state.serialize_element(&self.transform.name())?;
        tuple_state.serialize_element(&self.transform)?;
        tuple_state.end()
    }
}

impl<E: EntityData, T: Clone + serde::Serialize> FieldModifier<E, T> {
    pub fn new(field: &'static Field<E, T>, transform: Box<FieldTransformation<T>>) -> Box<FieldModifier<E, T>> {
        box FieldModifier {
            field,
            transform,
        }
    }
    pub fn new_modifier(field: &'static Field<E, T>, transform: Box<FieldTransformation<T>>) -> Box<Modifier<E>> {
        box FieldModifier {
            field,
            transform,
        }
    }
    pub fn permanent<TR: FieldTransformation<T> + 'static>(field: &'static Field<E, T>, transform: TR) -> Box<FieldModifier<E, T>> {
        box FieldModifier {
            field,
            transform: box transform,
        }
    }
}

impl<E: EntityData, T: Clone + 'static + serde::Serialize> Modifier<E> for FieldModifier<E, T> {
    fn modify(&self, data: &mut E, world: &WorldView) {
        let value = (self.field.getter_mut)(data);
        self.transform.apply(value)
    }

    fn is_active(&self, world: &WorldView) -> bool {
        true
    }

    fn modifier_type(&self) -> ModifierType {
        ModifierType::Permanent
    }

    fn modified_fields(&self) -> Vec<FieldModification> {
        vec![FieldModification { field: self.field.name, modification: self.transform.description(), description: None }]
    }
}