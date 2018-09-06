use entity::EntityData;
use prelude::*;
use common::prelude::*;
use std::marker::PhantomData;
use common::reflect::Field;
//use anymap::any::*;
use std::fmt;
use std::fmt::Formatter;
use std::fmt::Error;
use std::any::TypeId;
use std::any::Any;
use std::collections::VecDeque;
use erased_serde;

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
    Dynamic,
}

//pub trait ConstantModifier<T: EntityData>: Sized + 'static {
//    fn modify(&self, data: &mut T);
//
//    fn apply_to(self, entity: Entity, world: &mut World) {
//        world.add_constant_modifier(entity, self);
//    }
//    fn apply_to_world(self, world: &mut World) {
//        world.add_constant_world_modifier(self);
//    }
//
//    fn wrap(self) -> Box<Modifier<T>> {
//        box ConstantModifierWrapper {
//            inner: self,
//            _ignored: PhantomData,
//        }
//    }
//}

//pub(crate) struct ConstantModifierWrapper<T: EntityData, CM: ConstantModifier<T>> {
//    pub(crate) inner: CM,
//    pub(crate) _ignored: PhantomData<T>,
//}
//
//impl<T: EntityData, CM: ConstantModifier<T>> Modifier<T> for ConstantModifierWrapper<T, CM> {
//    fn modify(&self, data: &mut T, world: &WorldView) {
//        self.inner.modify(data);
//    }
//
//    fn is_active(&self, world: &WorldView) -> bool {
//        true
//    }
//
//    fn modifier_type(&self) -> ModifierType {
//        ModifierType::Permanent
//    }
//}
//
//pub trait LimitedModifier<T: EntityData>: Sized + 'static {
//    fn modify(&self, data: &mut T);
//
//    fn is_active(&self, world: &WorldView) -> bool;
//}
//
//pub(crate) struct LimitedModifierWrapper<T: EntityData, LM: LimitedModifier<T>> {
//    pub(crate) inner: LM,
//    pub(crate) _ignored: PhantomData<T>,
//}
//
//impl<T: EntityData, LM: LimitedModifier<T>> Modifier<T> for LimitedModifierWrapper<T, LM> {
//    fn modify(&self, data: &mut T, world: &WorldView) {
//        self.inner.modify(data);
//    }
//
//    fn is_active(&self, world: &WorldView) -> bool {
//        self.inner.is_active(world)
//    }
//
//    fn modifier_type(&self) -> ModifierType {
//        ModifierType::Limited
//    }
//}
//
//pub trait DynamicModifier<T: EntityData> {
//    fn modify(&self, data: &mut T, world: &WorldView);
//
//    fn is_active(&self, world: &WorldView) -> bool;
//}
//
//pub(crate) struct DynamicModifierWrapper<T: EntityData, DM: DynamicModifier<T>> {
//    pub(crate) inner: DM,
//    pub(crate) _ignored: PhantomData<T>,
//}
//
//impl<T: EntityData, LM: DynamicModifier<T>> Modifier<T> for DynamicModifierWrapper<T, LM> {
//    fn modify(&self, data: &mut T, world: &WorldView) {
//        self.inner.modify(data, world);
//    }
//
//    fn is_active(&self, world: &WorldView) -> bool {
//        self.inner.is_active(world)
//    }
//
//    fn modifier_type(&self) -> ModifierType {
//        ModifierType::Dynamic
//    }
//}

pub enum Transformation {
    Add(Box<Any>),
    Mul(Box<Any>),
    Div(Box<Any>),
    Set(Box<Any>),
    Reduce(Box<Any>),
    Recover(Box<Any>),
    SetKey(Box<Any>, Box<Any>),
    AddToKey(Box<Any>, Box<Any>),
    SubFromKey(Box<Any>, Box<Any>),
    RemoveKey(Box<Any>),
    Append(Box<Any>),
    Remove(Box<Any>),
    ModifyKey(Box<Any>, Box<Transformation>),
    Custom(String),
    Reset
}

//pub trait CloneToAny {
//    fn clone_to_any(&self) -> Box<Any>;
//    fn clone_to_clone_any(&self) -> Box<CloneToAny>;
//}
//impl <T> CloneToAny for T where T : Any + Clone {
//    fn clone_to_any(&self) -> Box<Any> {
//        box self.clone()
//    }
//    fn clone_to_clone_any(&self) -> Box<CloneToAny> {
//        box self.clone()
//    }
//}


//trait Downcastable {
//    fn downcast_ref<T : 'static>(&self) -> Option<&T>;
//}
//
//impl Downcastable for Box<Any> {
//    fn downcast_ref<T : 'static>(&self) -> Option<&T> {
//        let reference : &Any = self;
//        println!("TypeId is : {:?}, Box<Any> is: {:?}", reference.get_type_id(), TypeId::of::<Box<Any>>());
//        if TypeId::of::<T>() == reference.get_type_id() {
//            Some(unsafe { self.downcast_ref_unchecked::<T>() })
////            let raw : &Any = self;
////            unsafe { Some(&*(raw as *const Any as *const T)) }
//        } else {
//            None
//        }
//    }
//}


impl Transformation {
    pub fn as_string(a: &Box<Any>, include_sign: bool, invert: bool) -> String {
        if let Some(a) = a.downcast_ref::<i32>() {
            let a = if invert { -a } else { *a };
            if include_sign { a.to_string_with_sign() } else { a.to_string() }
        } else if let Some(a) = a.downcast_ref::<f32>() {
            let a = if invert { -a } else { *a };
            if include_sign { a.to_string_with_sign() } else { a.to_string() }
        } else if let Some(a) = a.downcast_ref::<f64>() {
            let a = if invert { -a } else { *a };
            if include_sign { a.to_string_with_sign() } else { a.to_string() }
        } else if let Some(a) = a.downcast_ref::<u32>() {
            if invert { format!("don't use u32 -{}", a.to_string()) } else { a.to_string() }
        } else if let Some(s) = a.downcast_ref::<Str>() {
            strf(s)
        } else if let Some(s) = a.downcast_ref::<String>() {
            s.clone()
        } else {
            strf("cannot represent")
        }
    }


    pub fn combine_boxed_values(a: &Box<Any>, b: &Box<Any>, negate: bool) -> Option<Box<Any>> {
        if let Some(a) = a.downcast_ref::<i32>() {
            if let Some(b) = b.downcast_ref::<i32>() {
                if negate { return Some(box (a - b)); } else { return Some(box (a + b)); }
            }
        } else if let Some(a) = a.downcast_ref::<f32>() {
            if let Some(b) = b.downcast_ref::<f32>() {
                if negate { return Some(box (a - b)); } else { return Some(box (a + b)); }
            }
        } else if let Some(a) = a.downcast_ref::<i64>() {
            if let Some(b) = b.downcast_ref::<i64>() {
                if negate { return Some(box (a - b)); } else { return Some(box (a + b)); }
            }
        } else if let Some(a) = a.downcast_ref::<f64>() {
            if let Some(b) = b.downcast_ref::<f64>() {
                if negate { return Some(box (a - b)); } else { return Some(box (a + b)); }
            }
        } else if let Some(a) = a.downcast_ref::<Sext>() {
            if let Some(b) = b.downcast_ref::<Sext>() {
                if negate { return Some(box (*a - *b)); } else { return Some(box (*a + *b)); }
            }
        }

        None
    }

    pub fn can_combine_with(&self, other: &Transformation) -> bool {
        match other {
            Transformation::Add(b) => {
                if let Transformation::Add(a) = self {
                    true
                } else {
                    false
                }
            }
            Transformation::Set(_) => true,
            _ => false
        }
    }

    pub fn is_set(&self) -> bool {
        match self {
            Transformation::Set(_) => true,
            _ => false
        }
    }

    pub fn combine(self, other: Transformation) -> Transformation {
        match &other {
            Transformation::Add(b) => {
                if let Transformation::Add(a) = &self {
                    if let Some(combined) = Transformation::combine_boxed_values(a, b, false) {
                        return Transformation::Add(combined);
                    }
                }
            }
            Transformation::Set(_) => return other,
            Transformation::SetKey(bk, _) => {
                if let Transformation::SetKey(ak, _) = &self {
                    if Transformation::as_string(ak, false, false) == Transformation::as_string(bk, false, false) {
                        return other;
                    }
                }
            }
            _ => ()
        }
        error!("Attempted to combine two transformation types that could not be combined");
        self
    }
}

impl fmt::Display for Transformation {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Transformation::Add(a) => write!(f, "{}", Transformation::as_string(a, true, false)),
            Transformation::Mul(a) => write!(f, "*{}", Transformation::as_string(a, false, false)),
            Transformation::Div(a) => write!(f, "/{}", Transformation::as_string(a, false, false)),
            Transformation::Set(a) => write!(f, "={}", Transformation::as_string(a, false, false)),
            Transformation::Recover(a) => write!(f, "recovered {}", Transformation::as_string(a, false, false)),
            Transformation::Reduce(a) => write!(f, "reduced {}", Transformation::as_string(a, false, false)),
            Transformation::Custom(s) => write!(f, "{}", s),
            Transformation::SetKey(_, v) => write!(f, "{}", Transformation::as_string(v, false, false)),
            Transformation::AddToKey(_, v) => write!(f, "{}", Transformation::as_string(v, true, false)),
            Transformation::SubFromKey(_, v) => write!(f, "-{}", Transformation::as_string(v, true, false)),
            Transformation::RemoveKey(_) => write!(f, "removed"),
            Transformation::Append(a) => write!(f, "appended {}", Transformation::as_string(a, false, false)),
            Transformation::Remove(a) => write!(f, "removed {}", Transformation::as_string(a, false, false)),
            Transformation::ModifyKey(k, tr) => write!(f, "[{}] {}", Transformation::as_string(k, false, false), tr),
            Transformation::Reset => write!(f, "reset"),
        }
    }
}

pub struct FieldModification {
    pub field: Str,
    pub modification: Transformation,
    pub description: Option<String>,
}

impl FieldModification {
    pub fn new<S : Into<Option<Str>>>(field: Str, modification: Transformation, description: S) -> FieldModification {
        FieldModification { field, modification, description : description.into().map(|s| String::from(s)) }
    }
    pub fn squash(mut transformations: VecDeque<FieldModification>) -> VecDeque<FieldModification> {
        let mut new_vec = VecDeque::new();
        while !transformations.is_empty() {
            if let Some(cur_t) = transformations.pop_front() {
                let (mut can_combine, rest): (VecDeque<FieldModification>, VecDeque<FieldModification>) =
                    transformations.into_iter().partition(
                        |ot| ot.field == cur_t.field && ot.description.is_some() && ot.description == cur_t.description && cur_t.modification.can_combine_with(&ot.modification));
                let mut t = cur_t;
                while !can_combine.is_empty() {
                    if let Some(next) = can_combine.pop_front() {
                        t = FieldModification { field: t.field, modification: Transformation::combine(t.modification, next.modification), description: t.description };
                    }
                }

                transformations = rest;
                new_vec.push_back(t);
            }
        }


        new_vec
    }
}

pub trait Modifier<T: EntityData> : erased_serde::Serialize {
    fn modify(&self, data: &mut T, world: &WorldView);

    fn is_active(&self, world: &WorldView) -> bool;

    fn modifier_type(&self) -> ModifierType;

    fn modified_fields(&self) -> Vec<FieldModification> {
        Vec::with_capacity(0)
    }
}
serialize_trait_object!(<T: EntityData> Modifier<T>);


pub struct FieldLogs<T: EntityData> {
    pub base_value: T,
    pub field_modifications: Vec<FieldModification>,
}

impl<E: EntityData> FieldLogs<E> {
    pub fn modifications_for<'a, T: Clone + 'static>(&'a self, field: &'static Field<E, T>) -> impl Iterator<Item=&FieldModification> + 'a {
        let name = field.name;
        self.field_modifications.iter().filter(move |m| m.field == name)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use spectral::prelude::*;

    #[test]
    pub fn test_field_modification_squashing() {
        let field_modifications = vec![FieldModification::new("A", Transformation::Add(box 2i32), Some("A")),
                                       FieldModification::new("B", Transformation::Add(box 1i32), None),
                                       FieldModification::new("A", Transformation::Add(box 3i32), Some("A"))];

        let squashed = FieldModification::squash(VecDeque::from(field_modifications));

        assert_that(&squashed.len()).is_equal_to(&2);

        let value_0 = if let Transformation::Add(a) = &squashed[0].modification {
            a.downcast_ref::<i32>()
        } else {
            None
        };
        assert_that(&value_0).is_equal_to(&Some(&5i32));

        let value_1 = if let Transformation::Add(a) = &squashed[1].modification {
            a.downcast_ref::<i32>()
        } else {
            None
        };
        assert_that(&value_1).is_equal_to(&Some(&1i32));
    }

    #[test]
    pub fn test_field_modification_squashing_with_set() {
        let field_modifications = vec![FieldModification::new("A", Transformation::Add(box 2i32), Some("A")),
                                       FieldModification::new("B", Transformation::Add(box 1i32), None),
                                       FieldModification::new("A", Transformation::Add(box 3i32), Some("A")),
                                       FieldModification::new("A", Transformation::Set(box 9i32), Some("A")),
                                       FieldModification::new("A", Transformation::Add(box 5i32), Some("Different Desc"))];

        let squashed = FieldModification::squash(VecDeque::from(field_modifications));

        assert_that(&squashed.len()).is_equal_to(&3);

        let value_0 = if let Transformation::Set(a) = &squashed[0].modification {
            a.downcast_ref::<i32>()
        } else {
            None
        };
        assert_that(&value_0).is_equal_to(&Some(&9i32));

        let value_1 = if let Transformation::Add(a) = &squashed[1].modification {
            a.downcast_ref::<i32>()
        } else {
            None
        };
        assert_that(&value_1).is_equal_to(&Some(&1i32));

        let value_2 = if let Transformation::Add(a) = &squashed[2].modification {
            a.downcast_ref::<i32>()
        } else {
            None
        };
        assert_that(&value_2).is_equal_to(&Some(&5i32));
    }
}