use entity::EntityData;
use prelude::*;
use common::prelude::*;
use std::marker::PhantomData;
use common::reflect::Field;

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
    Dynamic
}

pub trait ConstantModifier<T: EntityData>: Sized + 'static {
    fn modify(&self, data: &mut T);

    fn apply_to(self, entity: Entity, world: &mut World) {
        world.add_constant_modifier(entity, self);
    }
    fn apply_to_world(self, world : &mut World) {
        world.add_constant_world_modifier(self);
    }

    fn wrap(self) -> Box<Modifier<T>> {
        box ConstantModifierWrapper {
            inner: self,
            _ignored: PhantomData
        }
    }
}

pub(crate) struct ConstantModifierWrapper<T: EntityData, CM: ConstantModifier<T>> {
    pub(crate) inner: CM,
    pub(crate) _ignored: PhantomData<T>
}

impl<T: EntityData, CM: ConstantModifier<T>> Modifier<T> for ConstantModifierWrapper<T, CM> {
    fn modify(&self, data: &mut T, world: &WorldView) {
        self.inner.modify(data);
    }

    fn is_active(&self, world: &WorldView) -> bool {
        true
    }

    fn modifier_type(&self) -> ModifierType {
        ModifierType::Permanent
    }
}

pub trait LimitedModifier<T: EntityData>: Sized + 'static {
    fn modify(&self, data: &mut T);

    fn is_active(&self, world: &WorldView) -> bool;
}

pub(crate) struct LimitedModifierWrapper<T: EntityData, LM: LimitedModifier<T>> {
    pub(crate) inner: LM,
    pub(crate) _ignored: PhantomData<T>
}

impl<T: EntityData, LM: LimitedModifier<T>> Modifier<T> for LimitedModifierWrapper<T, LM> {
    fn modify(&self, data: &mut T, world: &WorldView) {
        self.inner.modify(data);
    }

    fn is_active(&self, world: &WorldView) -> bool {
        self.inner.is_active(world)
    }

    fn modifier_type(&self) -> ModifierType {
        ModifierType::Limited
    }
}

pub trait DynamicModifier<T: EntityData> {
    fn modify(&self, data: &mut T, world: &WorldView);

    fn is_active(&self, world: &WorldView) -> bool;
}

pub(crate) struct DynamicModifierWrapper<T: EntityData, DM: DynamicModifier<T>> {
    pub(crate) inner: DM,
    pub(crate) _ignored: PhantomData<T>
}

impl<T: EntityData, LM: DynamicModifier<T>> Modifier<T> for DynamicModifierWrapper<T, LM> {
    fn modify(&self, data: &mut T, world: &WorldView) {
        self.inner.modify(data, world);
    }

    fn is_active(&self, world: &WorldView) -> bool {
        self.inner.is_active(world)
    }

    fn modifier_type(&self) -> ModifierType {
        ModifierType::Dynamic
    }
}


pub struct FieldModification {
    pub field : Str,
    pub modification : String,
    pub description : Option<Str>
}

pub trait Modifier<T: EntityData> {
    fn modify(&self, data: &mut T, world: &WorldView);

    fn is_active(&self, world: &WorldView) -> bool;

    fn modifier_type(&self) -> ModifierType;

    fn modified_fields(&self) -> Vec<FieldModification> {
        Vec::with_capacity(0)
    }
}





pub struct FieldLogs<T : EntityData> {
    pub base_value : T,
    pub field_modifications : Vec<FieldModification>
}
impl <E : EntityData> FieldLogs<E> {
    pub fn modifications_for<'a, T : Clone + 'static>(&'a self, field : &'static Field<E,T>) -> impl Iterator<Item=&FieldModification> + 'a {
        let name = field.name;
        self.field_modifications.iter().filter(move |m| m.field == name)
    }
}

