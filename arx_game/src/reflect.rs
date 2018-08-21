use common::prelude::*;
use prelude::*;
use entity::EntityData;
use modifiers::ConstantModifier;
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


pub trait SettableField<E : EntityData, T: 'static> {
    fn set_to(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn set_to_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> SettableField<E,T> for Field<E, T> where E: EntityData, T: Clone {
    fn set_to(&'static self, new_value: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::SetTo(new_value)) }
    fn set_to_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>> { FieldModifier::limited(self, transformations::SetTo(new_value), condition) }
}

pub trait SetKeyableField<E : EntityData, K : Clone + Hash + Eq + 'static, V : Clone + 'static> {
    fn set_key_to(&'static self, key : K, new_value : V) -> Box<FieldModifier<E, HashMap<K,V>>>;
    fn remove_key(&'static self, key : K) -> Box<FieldModifier<E, HashMap<K,V>>>;
}
impl<E, K : Clone + Hash + Eq + 'static, V : Clone + 'static> SetKeyableField<E,K,V> for Field<E, HashMap<K,V>> where E: EntityData {
    fn set_key_to(&'static self, key : K, new_value : V) -> Box<FieldModifier<E, HashMap<K,V>>> { FieldModifier::permanent(self, transformations::SetKeyTo(key, new_value)) }
    fn remove_key(&'static self, key : K) -> Box<FieldModifier<E, HashMap<K,V>>> { FieldModifier::permanent(self, transformations::RemoveKey(key))}
}

pub trait AddableField<E : EntityData, T: 'static> {
    fn add(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn add_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> AddableField<E,T> for Field<E, T> where E: EntityData, T: Clone + ops::Add<Output=T> {
    fn add(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Add(amount)) }
    fn add_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, T>> { FieldModifier::limited(self, transformations::Add(amount), condition) }
}

pub trait SubableField<E : EntityData, T: 'static> {
    fn sub(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn sub_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> SubableField<E,T> for Field<E, T> where E: EntityData, T: Clone + ops::Sub<Output=T> + ops::Neg<Output=T> {
    fn sub(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Sub(amount)) }
    fn sub_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, T>> { FieldModifier::limited(self, transformations::Sub(amount), condition) }
}

pub trait MulableField<E : EntityData, T: 'static> {
    fn mul(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn mul_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> MulableField<E,T> for Field<E, T> where E: EntityData, T: Clone  + ops::Mul<Output=T> {
    fn mul(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Mul(amount)) }
    fn mul_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, T>> { FieldModifier::limited(self, transformations::Mul(amount), condition) }
}

pub trait DivableField<E : EntityData, T: 'static> {
    fn div(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn div_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> DivableField<E,T> for Field<E, T> where E: EntityData, T: Clone  + ops::Div<Output=T> {
    fn div(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Div(amount)) }
    fn div_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, T>> { FieldModifier::limited(self, transformations::Div(amount), condition) }
}

pub trait ReduceableField<E : EntityData, T: ReduceableType + 'static> {
    fn reduce_by(&'static self, new_value: T) -> Box<FieldModifier<E, Reduceable<T>>>;
    fn reduce_by_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, Reduceable<T>>>;

    fn reduce_to(&'static self, new_value: T) -> Box<FieldModifier<E, Reduceable<T>>>;
    fn reduce_to_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, Reduceable<T>>>;

    fn recover_by(&'static self, new_value: T) -> Box<FieldModifier<E, Reduceable<T>>>;
    fn recover_by_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, Reduceable<T>>>;

    fn increase_by(&'static self, new_value: T) -> Box<FieldModifier<E, Reduceable<T>>>;
}
impl<E, T: 'static> ReduceableField<E,T> for Field<E, Reduceable<T>> where E: EntityData, T: Clone  + ReduceableType {
    fn reduce_by(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::ReduceBy(amount)) }
    fn reduce_by_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::limited(self, transformations::ReduceBy(amount), condition) }

    fn reduce_to(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::ReduceBy(amount)) }
    fn reduce_to_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::limited(self, transformations::ReduceBy(amount), condition) }

    fn recover_by(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::RecoverBy(amount)) }
    fn recover_by_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::limited(self, transformations::RecoverBy(amount), condition) }

    fn increase_by(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::IncreaseBy(amount)) }
}


pub trait VecField<E : EntityData, T: Clone + PartialEq + 'static> {
    fn append(&'static self, new_value: T) -> Box<FieldModifier<E, Vec<T>>>;
    fn remove(&'static self, new_value: T) -> Box<FieldModifier<E, Vec<T>>>;
}
impl<E, T: Clone + PartialEq + 'static> VecField<E,T> for Field<E, Vec<T>> where E: EntityData {
    fn append(&'static self, new_value: T) -> Box<FieldModifier<E, Vec<T>>> { FieldModifier::permanent(self, transformations::Append(new_value)) }
    fn remove(&'static self, new_value: T) -> Box<FieldModifier<E, Vec<T>>> { FieldModifier::permanent(self, transformations::Remove(new_value)) }
}

pub trait GameDisplayable {
    fn to_game_str_full(&self, &WorldView) -> String;
}

impl <T> GameDisplayable for Option<T> where T : GameDisplayable {
    fn to_game_str_full(&self, view: &WorldView) -> String {
        match self {
            Some(inner) => inner.to_game_str_full(view),
            None => strf("none")
        }
    }
}


pub trait ModifierCondition {
    fn is_active(&self, world: &WorldView) -> bool;
}

pub trait FieldTransformation<T> {
    fn apply(&self, current: &mut T);
    fn description(&self) -> Transformation;
}

pub mod transformations {
    use super::*;
    use std::hash::Hash;
    use std::collections::HashMap;
    use std::hash::BuildHasher;

    pub struct SetTo<T: Clone>(pub T);

    impl<T: Clone + 'static> FieldTransformation<T> for SetTo<T> {
        fn apply(&self, current: &mut T) { *current = self.0.clone() }
        fn description(&self) -> Transformation {
            let cloned = self.0.clone();
            Transformation::Set(box self.0.clone())
        }
    }

    pub struct Add<T: Clone + ops::Add<Output=T>>(pub T);

    impl<T: Clone + ops::Add<Output=T> + 'static> FieldTransformation<T> for Add<T>  {
        fn apply(&self, current: &mut T) { *current = current.clone() + self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Add(box self.0.clone()) }
    }

    pub struct Sub<T: Clone + ops::Sub<Output=T>>(pub T);

    impl<T: Clone + ops::Sub<Output=T> + ops::Neg<Output=T> + 'static> FieldTransformation<T> for Sub<T>  {
        fn apply(&self, current: &mut T) { *current = current.clone() - self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Add(box (-self.0.clone())) }
    }

    pub struct Mul<T: Clone + ops::Mul<Output=T>>(pub T);

    impl<T: Clone + ops::Mul<Output=T> + 'static> FieldTransformation<T> for Mul<T>  {
        fn apply(&self, current: &mut T) { *current = current.clone() * self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Mul(box self.0.clone()) }
    }

    pub struct Div<T: Clone + ops::Div<Output=T> + 'static>(pub T);

    impl<T: Clone + ops::Div<Output=T>> FieldTransformation<T> for Div<T>  {
        fn apply(&self, current: &mut T) { *current = current.clone() / self.0.clone() }
        fn description(&self) -> Transformation { Transformation::Div(box self.0.clone()) }
    }

    pub struct ReduceBy<R: ReduceableType + 'static>(pub R);
    impl<R: ReduceableType> FieldTransformation<Reduceable<R>> for ReduceBy<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.reduce_by(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Reduce(box self.0.clone()) }
    }

    pub struct ReduceTo<R: ReduceableType + 'static>(pub R);
    impl<R: ReduceableType> FieldTransformation<Reduceable<R>> for ReduceTo<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.reduce_to(self.0.clone()) }
        fn description(&self) -> Transformation {
            let cloned = self.0.clone();
            Transformation::Set(box self.0.clone())
        }
    }

    pub struct RecoverBy<R: ReduceableType + 'static>(pub R);
    impl<R: ReduceableType> FieldTransformation<Reduceable<R>> for RecoverBy<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.recover_by(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Recover(box self.0.clone()) }
    }

    pub struct IncreaseBy<R: ReduceableType + 'static>(pub R);
    impl<R: ReduceableType> FieldTransformation<Reduceable<R>> for IncreaseBy<R> {
        fn apply(&self, current: &mut Reduceable<R>) { current.increase_by(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Add(box self.0.clone()) }
    }

    pub struct SetKeyTo<K : Clone + Hash + Eq + 'static, V : Clone + 'static> (pub K, pub V);
    impl <K : Clone + Hash + Eq + 'static, V : Clone, S : BuildHasher> FieldTransformation<HashMap<K,V,S>> for SetKeyTo<K,V> {
        fn apply(&self, current: &mut HashMap<K, V, S>) {
            current.insert(self.0.clone(), self.1.clone());
        }

        fn description(&self) -> Transformation {
            Transformation::SetKey(box self.0.clone(), box self.1.clone())
        }
    }

    pub struct RemoveKey<K : Clone + Hash + Eq + 'static> (pub K);
    impl <K : Clone + Hash + Eq + 'static, V : Clone, S : BuildHasher> FieldTransformation<HashMap<K,V,S>> for RemoveKey<K> {
        fn apply(&self, current: &mut HashMap<K, V, S>) {
            current.remove(&self.0);
        }

        fn description(&self) -> Transformation {
            Transformation::RemoveKey(box self.0.clone())
        }
    }


    pub struct Append<R : Clone + 'static>(pub R);
    impl<R: Clone + 'static> FieldTransformation<Vec<R>> for Append<R> {
        fn apply(&self, current: &mut Vec<R>) { current.push(self.0.clone()) }
        fn description(&self) -> Transformation { Transformation::Append(box self.0.clone()) }
    }

    pub struct Remove<R : Clone + PartialEq + 'static>(pub R);
    impl<R: Clone + PartialEq + 'static> FieldTransformation<Vec<R>> for Remove<R> {
        fn apply(&self, current: &mut Vec<R>) { current.remove_item(&self.0); }
        fn description(&self) -> Transformation { Transformation::Remove(box self.0.clone()) }
    }
}


pub struct FieldModifier<E: EntityData, T: 'static> {
    field: &'static Field<E, T>,
    transform: Box<FieldTransformation<T>>,
    condition: Option<Box<ModifierCondition>>,
}

impl<E: EntityData, T: Clone> FieldModifier<E, T> {
    pub fn permanent<TR: FieldTransformation<T> + 'static>(field: &'static Field<E, T>, transform: TR) -> Box<FieldModifier<E, T>> {
        box FieldModifier {
            field,
            transform: box transform,
            condition: None,
        }
    }

    pub fn limited<TR: FieldTransformation<T> + 'static, C: ModifierCondition + 'static>(field: &'static Field<E, T>, transform: TR, condition: C) -> Box<FieldModifier<E, T>> {
        box FieldModifier {
            field,
            transform: box transform,
            condition: Some(box condition),
        }
    }
}

impl<E: EntityData, T: Clone + 'static> Modifier<E> for FieldModifier<E, T> {
    fn modify(&self, data: &mut E, world: &WorldView) {
        let value = (self.field.getter_mut)(data);
        self.transform.apply(value)
    }

    fn is_active(&self, world: &WorldView) -> bool {
        self.condition.as_ref().map(|c| c.is_active(world)).unwrap_or(true)
    }

    fn modifier_type(&self) -> ModifierType {
        if self.condition.is_some() {
            ModifierType::Limited
        } else {
            ModifierType::Permanent
        }
    }

    fn modified_fields(&self) -> Vec<FieldModification> {
        vec![FieldModification { field: self.field.name, modification: self.transform.description(), description : None }]
    }
}