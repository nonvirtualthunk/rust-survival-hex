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


pub trait SettableField<E : EntityData, T: 'static> {
    fn set_to(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn set_to_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> SettableField<E,T> for Field<E, T> where E: EntityData, T: Clone + Display {
    fn set_to(&'static self, new_value: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::SetTo(new_value)) }
    fn set_to_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>> { FieldModifier::limited(self, transformations::SetTo(new_value), condition) }
}

pub trait AddableField<E : EntityData, T: 'static> {
    fn add(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn add_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> AddableField<E,T> for Field<E, T> where E: EntityData, T: Clone + Display + ops::Add<Output=T> {
    fn add(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Add(amount)) }
    fn add_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, T>> { FieldModifier::limited(self, transformations::Add(amount), condition) }
}

pub trait SubableField<E : EntityData, T: 'static> {
    fn sub(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn sub_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> SubableField<E,T> for Field<E, T> where E: EntityData, T: Clone + Display + ops::Sub<Output=T> {
    fn sub(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Sub(amount)) }
    fn sub_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, T>> { FieldModifier::limited(self, transformations::Sub(amount), condition) }
}

pub trait MulableField<E : EntityData, T: 'static> {
    fn mul(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn mul_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> MulableField<E,T> for Field<E, T> where E: EntityData, T: Clone + Display + ops::Mul<Output=T> {
    fn mul(&'static self, amount: T) -> Box<FieldModifier<E, T>> { FieldModifier::permanent(self, transformations::Mul(amount)) }
    fn mul_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, T>> { FieldModifier::limited(self, transformations::Mul(amount), condition) }
}

pub trait DivableField<E : EntityData, T: 'static> {
    fn div(&'static self, new_value: T) -> Box<FieldModifier<E, T>>;
    fn div_while<C: ModifierCondition + 'static>(&'static self, new_value: T, condition: C) -> Box<FieldModifier<E, T>>;
}
impl<E, T: 'static> DivableField<E,T> for Field<E, T> where E: EntityData, T: Clone + Display + ops::Div<Output=T> {
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
}
impl<E, T: 'static> ReduceableField<E,T> for Field<E, Reduceable<T>> where E: EntityData, T: Clone + Display + ReduceableType {
    fn reduce_by(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::ReduceBy(amount)) }
    fn reduce_by_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::limited(self, transformations::ReduceBy(amount), condition) }

    fn reduce_to(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::ReduceBy(amount)) }
    fn reduce_to_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::limited(self, transformations::ReduceBy(amount), condition) }

    fn recover_by(&'static self, amount: T) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::permanent(self, transformations::RecoverBy(amount)) }
    fn recover_by_while<C: ModifierCondition + 'static>(&'static self, amount: T, condition: C) -> Box<FieldModifier<E, Reduceable<T>>> { FieldModifier::limited(self, transformations::RecoverBy(amount), condition) }
}



pub trait ModifierCondition {
    fn is_active(&self, world: &WorldView) -> bool;
}

pub trait FieldTransformation<T> {
    fn apply(&self, current: &T) -> T;
    fn description(&self) -> String;
}

pub mod transformations {
    use super::*;

    pub struct SetTo<T: Clone>(pub T);

    impl<T: Clone> FieldTransformation<T> for SetTo<T> where T: Display {
        fn apply(&self, current: &T) -> T { self.0.clone() }
        fn description(&self) -> String { format!("= {}", self.0) }
    }

    pub struct Add<T: Clone + ops::Add<Output=T>>(pub T);

    impl<T: Clone + ops::Add<Output=T>> FieldTransformation<T> for Add<T> where T: Display {
        fn apply(&self, current: &T) -> T { current.clone() + self.0.clone() }
        fn description(&self) -> String { format!("+ {}", self.0) }
    }

    pub struct Sub<T: Clone + ops::Sub<Output=T>>(pub T);

    impl<T: Clone + ops::Sub<Output=T>> FieldTransformation<T> for Sub<T> where T: Display {
        fn apply(&self, current: &T) -> T { current.clone() - self.0.clone() }
        fn description(&self) -> String { format!("- {}", self.0) }
    }

    pub struct Mul<T: Clone + ops::Mul<Output=T>>(pub T);

    impl<T: Clone + ops::Mul<Output=T>> FieldTransformation<T> for Mul<T> where T: Display {
        fn apply(&self, current: &T) -> T { current.clone() * self.0.clone() }
        fn description(&self) -> String { format!("* {}", self.0) }
    }

    pub struct Div<T: Clone + ops::Div<Output=T>>(pub T);

    impl<T: Clone + ops::Div<Output=T>> FieldTransformation<T> for Div<T> where T: Display {
        fn apply(&self, current: &T) -> T { current.clone() / self.0.clone() }
        fn description(&self) -> String { format!("/ {}", self.0) }
    }

    pub struct ReduceBy<R: ReduceableType>(pub R);

    impl<R: ReduceableType> FieldTransformation<Reduceable<R>> for ReduceBy<R> where R: Display {
        fn apply(&self, current: &Reduceable<R>) -> Reduceable<R> { current.reduced_by(self.0.clone()) }
        fn description(&self) -> String { format!("reduced by {}", self.0) }
    }

    pub struct ReduceTo<R: ReduceableType>(pub R);

    impl<R: ReduceableType> FieldTransformation<Reduceable<R>> for ReduceTo<R> where R: Display {
        fn apply(&self, current: &Reduceable<R>) -> Reduceable<R> { current.reduced_to(self.0.clone()) }
        fn description(&self) -> String { format!("reduced to {}", self.0) }
    }


    pub struct RecoverBy<R: ReduceableType>(pub R);

    impl<R: ReduceableType> FieldTransformation<Reduceable<R>> for RecoverBy<R> where R: Display {
        fn apply(&self, current: &Reduceable<R>) -> Reduceable<R> { current.recovered_by(self.0.clone()) }
        fn description(&self) -> String { format!("recovered by {}", self.0) }
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

impl<E: EntityData, T: Clone + 'static> Modifier<E> for FieldModifier<E, T> where T: Display {
    fn modify(&self, data: &mut E, world: &WorldView) {
        let new_value = {
            let old_value = (self.field.getter)(data);
            self.transform.apply(old_value)
        };
        (self.field.setter)(data, new_value);
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