use common::prelude::*;
use common::reflect::Field;
use entity::EntityData;
use entity::FieldVisitor;
use erased_serde;
use modifiers::FieldModification;
use modifiers::Modifier;
use modifiers::ModifierType;
use modifiers::Transformation;
use prelude::*;
use serde;
use serde::de::DeserializeOwned;
use serde::de::Error as Serror;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::Deserialize;
use serde::Deserializer;
use serde::ser::SerializeStruct;
use serde::ser::SerializeTuple;
use serde::Serialize;
use serde::Serializer;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
//use modifiers::ConstantModifier;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::ops;


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

    fn reset(&'static self) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::Reset) }
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
    fn name(&self) -> Str;
}
serialize_trait_object!(<T> FieldTransformation<T>);


pub mod transformations {
    use super::*;

    #[derive(Serialize, Deserialize, Clone)]
    pub struct SetTo<T: Clone + serde::Serialize>(pub T);

    impl<T: Clone + 'static + serde::Serialize> FieldTransformation<T> for SetTo<T> {
        fn apply(&self, current: &mut T) { *current = self.0.clone() }
        fn description(&self) -> Transformation {
            let cloned = self.0.clone();
            Transformation::Set(box self.0.clone())
        }
        fn name(&self) -> &'static str { "set" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Add<T: Clone + ops::Add<Output=T> + serde::Serialize>(pub T);

    impl<T: Clone + ops::Add<Output=T> + 'static + serde::Serialize> FieldTransformation<T> for Add<T> {
        fn apply(&self, current: &mut T) { *current = current.clone() + self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Add(box self.0.clone()) }
        fn name(&self) -> &'static str { "add" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Sub<T: Clone + ops::Sub<Output=T> + serde::Serialize>(pub T);

    impl<T: Clone + ops::Sub<Output=T> + ops::Neg<Output=T> + 'static + serde::Serialize> FieldTransformation<T> for Sub<T> {
        fn apply(&self, current: &mut T) { *current = current.clone() - self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Add(box (-self.0.clone())) }
        fn name(&self) -> &'static str { "sub" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Mul<T: Clone + ops::Mul<Output=T> + serde::Serialize>(pub T);

    impl<T: Clone + ops::Mul<Output=T> + 'static + serde::Serialize> FieldTransformation<T> for Mul<T> {
        fn apply(&self, current: &mut T) { *current = current.clone() * self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Mul(box self.0.clone()) }
        fn name(&self) -> &'static str { "mul" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Div<T: Clone + ops::Div<Output=T> + serde::Serialize + 'static>(pub T);

    impl<T: Clone + ops::Div<Output=T> + serde::Serialize> FieldTransformation<T> for Div<T> {
        fn apply(&self, current: &mut T) { *current = current.clone() / self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Div(box self.0.clone()) }
        fn name(&self) -> &'static str { "div" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct ReduceBy<R: ReduceableType + serde::Serialize + 'static>(pub R);

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for ReduceBy<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.reduce_by(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Reduce(box self.0.clone()) }
        fn name(&self) -> &'static str { "reduce_by" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct ReduceTo<R: ReduceableType + serde::Serialize + 'static>(pub R);

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for ReduceTo<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.reduce_to(self.0.clone()) }
        fn description(&self) -> Transformation {
            let cloned = self.0.clone();
            Transformation::Set(box self.0.clone())
        }
        fn name(&self) -> &'static str { "reduce_to" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct RecoverBy<R: ReduceableType + serde::Serialize + 'static>(pub R);

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for RecoverBy<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.recover_by(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Recover(box self.0.clone()) }
        fn name(&self) -> &'static str { "recover_by" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct IncreaseBy<R: ReduceableType + 'static + serde::Serialize>(pub R);

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for IncreaseBy<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.increase_by(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Add(box self.0.clone()) }
        fn name(&self) -> &'static str { "increase_by" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Reset;

    impl<R: ReduceableType + serde::Serialize> FieldTransformation<Reduceable<R>> for Reset {
        fn apply(&self, current: &mut Reduceable<R>) { current.reset() }
        fn description(&self) -> Transformation { Transformation::Reset }
        fn name(&self) -> &'static str { "reset" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct AddToKey<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize> (pub K, pub V);

    impl<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + serde::Serialize + ops::Add<Output=V> + Default, S: BuildHasher> FieldTransformation<HashMap<K, V, S>>
    for AddToKey<K, V> {
        fn apply(&self, current: &mut HashMap<K, V, S>) {
            let cur = current.entry(self.0.clone()).or_insert_with(|| V::default());
            *cur = cur.clone() + self.1.clone();
        }

        fn description(&self) -> Transformation {
            Transformation::AddToKey(box self.0.clone(), box self.1.clone())
        }
        fn name(&self) -> &'static str { "add_to_key" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct SetKeyTo<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize> (pub K, pub V);

    impl<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + serde::Serialize, S: BuildHasher> FieldTransformation<HashMap<K, V, S>> for SetKeyTo<K, V> {
        fn apply(&self, current: &mut HashMap<K, V, S>) {
            current.insert(self.0.clone(), self.1.clone());
        }
        fn description(&self) -> Transformation {
            Transformation::SetKey(box self.0.clone(), box self.1.clone())
        }
        fn name(&self) -> &'static str { "set_key_to" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct RemoveKey<K: Clone + Hash + Eq + 'static + serde::Serialize> (pub K);

    impl<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + serde::Serialize, S: BuildHasher> FieldTransformation<HashMap<K, V, S>> for RemoveKey<K> {
        fn apply(&self, current: &mut HashMap<K, V, S>) {
            current.remove(&self.0);
        }
        fn description(&self) -> Transformation {
            Transformation::RemoveKey(box self.0.clone())
        }
        fn name(&self) -> &'static str { "remove_key" }
    }

    pub struct TransformKey<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize> (pub K, pub Box<FieldTransformation<V>>, pub ::std::marker::PhantomData<V>);

    impl<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize + Default> FieldTransformation<HashMap<K, V>> for TransformKey<K, V> {
        fn apply(&self, current: &mut HashMap<K, V>) {
            let value_at_key = current.entry(self.0.clone()).or_insert_with(|| V::default());
            self.1.apply(value_at_key);
        }
        fn description(&self) -> Transformation {
            Transformation::ModifyKey(box self.0.clone(), box self.1.description())
        }
        fn name(&self) -> &'static str { "transform_key" }
    }


    #[derive(Serialize, Deserialize, Clone)]
    pub struct Append<R: Clone + 'static + serde::Serialize>(pub R);

    impl<R: Clone + 'static + serde::Serialize> FieldTransformation<Vec<R>> for Append<R> {
        fn apply(&self, current: &mut Vec<R>) { current.push(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Append(box self.0.clone()) }
        fn name(&self) -> &'static str { "append" }
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Remove<R: Clone + PartialEq + 'static + serde::Serialize>(pub R);

    impl<R: Clone + PartialEq + 'static + serde::Serialize> FieldTransformation<Vec<R>> for Remove<R> {
        fn apply(&self, current: &mut Vec<R>) { current.remove_item(&self.0); }
        fn description(&self) -> Transformation { Transformation::Remove(box self.0.clone()) }
        fn name(&self) -> &'static str { "remove" }
    }
}


pub struct FieldModifier<E: EntityData, T: 'static> {
    pub(crate) field: &'static Field<E, T>,
    pub(crate) transform: Box<FieldTransformation<T>>,
}

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



macro_rules! implement_transform_ser {
    ($base:ident, $func_name:ident, $field_type:path, $transform_type:path, $(+ $bounds:path)*) => {
        pub trait $base<'de, T> {
            fn $func_name<V>(seq : &mut V) -> Result<Box<FieldTransformation<T>>, V::Error> where V : SeqAccess<'de>;
        }

        impl <'de, T> $base <'de, T> for T {
            default fn $func_name<V>(seq: & mut V) -> Result<Box<FieldTransformation<T>>, <V as SeqAccess<'de>>::Error> where V: SeqAccess<'de> {
                Err(V::Error::custom("attempted to deserialize invalid type"))
            }
        }

        impl <'de, T> $base <'de, $field_type> for $field_type where T : Clone + serde::Serialize + serde::Deserialize<'de> + 'static $(+ $bounds)* {
            fn $func_name<V>(seq : &mut V) -> Result<Box<FieldTransformation<$field_type>>, V::Error> where V : SeqAccess<'de>   {
                let tr : $transform_type = seq.next_element()?.ok_or_else(||panic!("deserialize transformation failed"))?;
                Ok(box tr)
            }
        }
    }
}

macro_rules! implement_map_transform_ser {
    ($base:ident, $func_name:ident, $transform_type:path, $(+ $bounds:path)*) => {
        pub trait $base<'de, T> {
            fn $func_name<V>(seq : &mut V) -> Result<Box<FieldTransformation<T>>, V::Error> where V : SeqAccess<'de>;
        }

        impl <'de, T> $base <'de, T> for T {
            default fn $func_name<V>(seq: & mut V) -> Result<Box<FieldTransformation<T>>, <V as SeqAccess<'de>>::Error> where V: SeqAccess<'de> {
                Err(V::Error::custom("attempted to deserialize invalid type"))
            }
        }

        impl <'de, K, V> $base <'de, HashMap<K,V>> for HashMap<K,V>
        where
            K : Clone + Hash + Eq + 'static + serde::Serialize + serde::Deserialize<'de>,
            V : Clone + serde::Serialize + serde::Deserialize<'de> + 'static $(+ $bounds)* {
            fn $func_name<S>(seq : &mut S) -> Result<Box<FieldTransformation<HashMap<K,V>>>, S::Error> where S : SeqAccess<'de>   {
                let tr : $transform_type = seq.next_element()?.ok_or_else(||panic!("deserialize transformation failed"))?;
                Ok(box tr)
            }
        }
    }
}
//<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + serde::Serialize + ops::Add<Output=V> + ops::Sub<Output=V> + Default, S: BuildHasher> FieldTransformation<HashMap<K, V, S>>

implement_transform_ser!(DeserializeAddTransformation, deserialize_add, T, transformations::Add<T>, + ops::Add<Output=T>);
implement_transform_ser!(DeserializeSubTransformation, deserialize_sub, T, transformations::Sub<T>, + ops::Sub<Output=T> + ops::Neg<Output=T>);
implement_transform_ser!(DeserializeMulTransformation, deserialize_mul, T, transformations::Mul<T>, + ops::Mul<Output=T>);
implement_transform_ser!(DeserializeDivTransformation, deserialize_div, T, transformations::Div<T>, + ops::Div<Output=T>);
implement_transform_ser!(DeserializeSetToTransformation, deserialize_set_to, T, transformations::SetTo<T>,);
implement_transform_ser!(DeserializeReduceByTransformation, deserialize_reduce_by, Reduceable<T>, transformations::ReduceBy<T>, + ReduceableType);
implement_transform_ser!(DeserializeReducetoTransformation, deserialize_reduce_to, Reduceable<T>, transformations::ReduceTo<T>, + ReduceableType);
implement_transform_ser!(DeserializeRecoverByTransformation, deserialize_recover_by, Reduceable<T>, transformations::RecoverBy<T>, + ReduceableType);
implement_transform_ser!(DeserializeIncreaseByTransformation, deserialize_increase_by, Reduceable<T>, transformations::IncreaseBy<T>, + ReduceableType);
implement_transform_ser!(DeserializeResetTransformation, deserialize_reset, Reduceable<T>, transformations::Reset, + ReduceableType);

implement_transform_ser!(DeserializeAppendTransformation, deserialize_append, Vec<T>, transformations::Append<T>,);
implement_transform_ser!(DeserializeRemoveTransformation, deserialize_remove, Vec<T>, transformations::Remove<T>, + PartialEq<T>);

implement_map_transform_ser!(DeserializeAddToKeyTransformation, deserialize_add_to_key, transformations::AddToKey<K,V>, + ops::Add<Output=V> + Default);
implement_map_transform_ser!(DeserializeSetKeyToTransformation, deserialize_set_key_to, transformations::SetKeyTo<K,V>,);
implement_map_transform_ser!(DeserializeRemoveKeyTransformation, deserialize_remove_key, transformations::RemoveKey<K>,);


pub trait DeserializeTransformKeyTransformation<'de, T> {
    fn deserialize_transform_key<V>(seq: &mut V) -> Result<Box<FieldTransformation<T>>, V::Error> where V: SeqAccess<'de>;
}

impl<'de, T> DeserializeTransformKeyTransformation<'de, T> for T {
    default fn deserialize_transform_key<V>(seq: &mut V) -> Result<Box<FieldTransformation<T>>, <V as SeqAccess<'de>>::Error> where V: SeqAccess<'de> {
        Err(V::Error::custom("attempted to deserialize transform key for invalid field type"))
    }
}

impl<'de, K, V> DeserializeTransformKeyTransformation<'de, HashMap<K, V>> for HashMap<K, V> where
    K: Clone + Hash + Eq + 'static + serde::Serialize + serde::Deserialize<'de>,
    V: Clone + serde::Serialize + serde::Deserialize<'de> + Default + 'static
{
    fn deserialize_transform_key<S>(seq: &mut S) -> Result<Box<FieldTransformation<HashMap<K, V>>>, S::Error> where S: SeqAccess<'de> {
        let tr : transformations::TransformKey<K,V> = seq.next_element()?.ok_or_else(||panic!("deserialize add transformation failed"))?;
        Ok(box tr)
    }
}


impl<K: Clone + Hash + Eq + 'static + serde::Serialize, V: Clone + 'static + serde::Serialize + Default> Serialize for transformations::TransformKey<K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
        use serde::ser::SerializeSeq;
        // key, tr name, tr
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.0)?;
        seq.serialize_element(&self.1.name())?;
        seq.serialize_element(&self.1)?;
        seq.end()
    }
}



struct TransformKeyVisitor<'de, K: Clone + Hash + Eq + 'static + serde::Serialize + serde::Deserialize<'de>,V: Clone + 'static + serde::Serialize + serde::Deserialize<'de> + Default>
{
    _a : ::std::marker::PhantomData<&'de transformations::TransformKey<K,V>>
}
impl <'de, K: Clone + Hash + Eq + 'static + serde::Serialize + serde::Deserialize<'de>,V: Clone + 'static + serde::Serialize + serde::Deserialize<'de> + Default> Visitor<'de> for TransformKeyVisitor<'de, K,V> {
    type Value = transformations::TransformKey<K,V>;

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "TransformKey deserializer")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<<Self as Visitor<'de>>::Value, <A as SeqAccess<'de>>::Error> where A: SeqAccess<'de>, {
        let k : K = seq.next_element()?.unwrap_or_else(||panic!("transform key deserializer had no key"));
        let tr : Box<FieldTransformation<V>> = deserialize_to_transform::<V,A>(&mut seq)?;
        Ok(transformations::TransformKey(k, tr, ::std::marker::PhantomData::default()))
    }
}


impl<'de, K: Clone + Hash + Eq + 'static + serde::Serialize + serde::Deserialize<'de>, V: Clone + 'static + serde::Serialize + serde::Deserialize<'de> + Default>
Deserialize<'de>
for transformations::TransformKey<K, V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(TransformKeyVisitor::<'de, K,V> { _a : ::std::marker::PhantomData::default() })
    }
}

//pub trait DeserializeAddToKeyTransformation<'de, T> {
//    fn deserialize_add_to_key<V>(&self, seq : &mut V) -> Result<Box<FieldTransformation<T>>, V::Error> where V : SeqAccess<'de> ;
//}
//impl <'de, E:EntityData,T> DeserializeAddToKeyTransformation<'de, T> for Field<E,T> {
//    default fn deserialize_add_to_key<V>(&self, seq: & mut V) -> Result<Box<FieldTransformation<T>>, <V as SeqAccess<'de>>::Error> where V: SeqAccess<'de> {
//        Err(V::Error::custom("attempted to deserialize add for invalid field type"))
//    }
//}
//impl <'de, E : EntityData, K, V> DeserializeAddTransformation<'de, > for Field<E, T> where T: Clone + ops::Add<Output=T> + serde::Serialize + serde::Deserialize<'de> {
//    fn deserialize_add_to_key<V>(&self, seq : &mut V) -> Result<Box<FieldTransformation<T>>, V::Error> where V : SeqAccess<'de>   {
//        let add : transformations::Add<T> = seq.next_element()?.ok_or_else(||panic!("deserialize add transformation failed"))?;
//        Ok(box add)
//    }
//}


//trait CanDeserializeToTransform<T> {
//    fn deserialize_to_transform<'de, V>(&self, seq: &mut V) -> Result<Box<FieldTransformation<T>>, V::Error> where V: SeqAccess<'de>;
//}

pub fn deserialize_to_transform<'de, T, V>(seq: &mut V) -> Result<Box<FieldTransformation<T>>, V::Error> where V: SeqAccess<'de> {
    let transform_name: String = seq.next_element()?.ok_or_else(|| panic!("couldn't get transform name"))?;
    match transform_name.as_str() {
        "transform_key" => T::deserialize_transform_key(seq),
        "add" => T::deserialize_add(seq),
        "sub" => T::deserialize_sub(seq),
        "set" => T::deserialize_set_to(seq),
        "mul" => T::deserialize_mul(seq),
        "div" => T::deserialize_div(seq),
        "reduce_by" => T::deserialize_reduce_by(seq),
        "reduce_to" => T::deserialize_reduce_to(seq),
        "recover_by" => T::deserialize_recover_by(seq),
        "increase_by" => T::deserialize_increase_by(seq),
        "reset" => T::deserialize_reset(seq),
        "add_to_key" => T::deserialize_add_to_key(seq),
        "set_key_to" => T::deserialize_set_key_to(seq),
        "remove_key" => T::deserialize_remove_key(seq),
        "transform_key" => T::deserialize_remove_key(seq),
        "append" => T::deserialize_append(seq),
        "remove" => T::deserialize_remove(seq),
        name => panic!(format!("unsupported transform name {}", name))
    }
}
//impl<E: EntityData, T> CanDeserializeToTransform<T> for Field<E, T> {
//    fn deserialize_to_transform<'de, V>(&self, seq: &mut V) -> Result<Box<FieldTransformation<T>>, V::Error> where V: SeqAccess<'de> {
//        let transform_name: String = seq.next_element()?.ok_or_else(|| panic!("couldn't get transform name"))?;
//        match transform_name.as_str() {
//            "add" => T::deserialize_add(seq),
//            "sub" => T::deserialize_sub(seq),
//            "set" => T::deserialize_set_to(seq),
//            "mul" => T::deserialize_mul(seq),
//            "div" => T::deserialize_div(seq),
//            "reduce_by" => T::deserialize_reduce_by(seq),
//            "reduce_to" => T::deserialize_reduce_to(seq),
//            "recover_by" => T::deserialize_recover_by(seq),
//            "increase_by" => T::deserialize_increase_by(seq),
//            "reset" => T::deserialize_reset(seq),
//            "add_to_key" => T::deserialize_add_to_key(seq),
//            "set_key_to" => T::deserialize_set_key_to(seq),
//            "remove_key" => T::deserialize_remove_key(seq),
//            "transform_key" => T::deserialize_remove_key(seq),
//            "append" => T::deserialize_append(seq),
//            "remove" => T::deserialize_remove(seq),
//            "transform_key" => T::deserialize_transform_key(seq),
//            name => panic!(format!("unsupported transform name {}", name))
//        }
//    }
//}


#[derive(Default)]
struct EFieldVisitor<E> { _phantom: std::marker::PhantomData<E> }

impl<'de, 'a, E: EntityData, V: SeqAccess<'de>> FieldVisitor<E, Result<Box<Modifier<E>>, V::Error>, V> for EFieldVisitor<E> {
    fn visit<T: 'static + Clone + Serialize>(&self, field: &'static Field<E, T>, arg: &mut V) -> Option<Result<Box<Modifier<E>>, V::Error>> {
        Some(deserialize_to_transform::<T, V>(arg).map(|tr| FieldModifier::new_modifier(field, tr)))
    }
}

#[derive(Default)]
struct FieldModifierVisitor<E> { _phantom: std::marker::PhantomData<E> }

impl<'de, E: EntityData> Visitor<'de> for FieldModifierVisitor<E> {
    type Value = Box<Modifier<E>>;

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), Error> {
        write!(formatter, "Something we can turn into a modifier")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Box<Modifier<E>>, V::Error> where V: SeqAccess<'de> {
        let field_name: String = seq.next_element()?.ok_or_else(|| panic!("panicked earlier than expected"))?;

        E::visit_field_named(field_name.as_str(), EFieldVisitor::<E>::default(), &mut seq).unwrap_or_else(|| Err(V::Error::custom("could not identify field")))
    }
}

impl<'de, E: EntityData> Deserialize<'de> for Box<Modifier<E>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        deserializer.deserialize_tuple(3, FieldModifierVisitor::<E>::default())
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

    use super::super::entity;
    use common::reflect::Field;

    #[derive(Clone, Default, PrintFields, Debug)]
    pub struct Bar {
        pub f: f32,
        pub b: i32,
        pub m: HashMap<String, i32>,
    }

    impl Bar {
        pub const f: Field<Bar, f32> = Field::new(stringify!( f ), |t| &t.f, |t| &mut t.f, |t, v| { t.f = v; });
        pub const b: Field<Bar, i32> = Field::new(stringify!( b ), |t| &t.b, |t| &mut t.b, |t, v| { t.b = v; });
        pub const m: Field<Bar, HashMap<String, i32>> = Field::new(stringify!( m ), |t| &t.m, |t| &mut t.m, |t, v| { t.m = v; });
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
        test_values.push(Bar::m.add_to_key(String::from("test"), 1));
        test_values.push(FieldModifier::permanent(&Bar::m, transformations::TransformKey(String::from("test"), box transformations::Add(2), ::std::marker::PhantomData::default())));

        let value = Container {
            modifiers: test_values
        };

        let serialized_str = ron::ser::to_string_pretty(&value, pretty_config).expect("serialization failed");
        println!("Serialized======================\n{}", serialized_str);
//        let deserialized: Container<Bar> = ron::de::from_str(&serialized_str).unwrap();
        let deserialized: Container = ron::de::from_str(&serialized_str).unwrap();

        let mut bar = Bar { f: 0.0, b: -1, m: HashMap::new() };
        let world = World::new();
        for wrapped in deserialized.modifiers {
            wrapped.modify(&mut bar, world.view());
        }
        println!("Bar: {:?}", bar)
    }
}