use entity::*;
use prelude::*;
use common::hex::*;
use common::reflect::*;
use events::CoreEvent;

#[cfg(test)]
mod test {
    use super::*;
    use modifiers::*;

    #[derive(Clone, Default, PartialEq, Debug, PrintFields)]
    struct FooData {
        a: i32,
        b: Vec<f32>
    }

    #[derive(Clone, Default, PartialEq, Debug, PrintFields)]
    struct BarData {
        x: f32
    }

    impl FooData { pub const a : Field < FooData , i32 > = Field :: new ( stringify ! ( a ) , | t | & t . a , | t , v | { t . a = v ; } ) ; pub const b : Field < FooData , Vec < f32 > > = Field :: new ( stringify ! ( b ) , | t | & t . b , | t , v | { t . b = v ; } ) ; }
    impl BarData { pub const x : Field < BarData , f32 > = Field :: new ( stringify ! ( x ) , | t | & t . x , | t , v | { t . x = v ; } ) ; }


    impl EntityData for FooData {}

    impl EntityData for BarData {}

    struct AddToAModifier {
        delta_a: i32
    }

    impl ConstantModifier<FooData> for AddToAModifier {
        fn modify(&self, data: &mut FooData) {
            data.a += self.delta_a;
        }
    }

    pub struct MultiplyByOtherEntityModifier {
        other_entity: Entity
    }

    impl DynamicModifier<FooData> for MultiplyByOtherEntityModifier {
        fn modify(&self, data: &mut FooData, world: &WorldView) {
            data.a = data.a * world.data::<FooData>(self.other_entity).a;
        }

        fn is_active(&self, world: &WorldView) -> bool {
            true
        }
    }

    pub struct AddBarDataModifier {
        delta: f32
    }

    impl ConstantModifier<BarData> for AddBarDataModifier {
        fn modify(&self, data: &mut BarData) {
            data.x += self.delta;
        }
    }

    use spectral::prelude::*;

    pub struct AddFooBValueModifier {}

    impl ConstantModifier<FooData> for AddFooBValueModifier {
        fn modify(&self, data: &mut FooData) {
            data.b.push(1.0);
        }
    }

    #[test]
    pub fn test_new_world() {
        let mut world = World::new();

        world.register::<FooData>();

        let initial_data = FooData {
            a: 1,
            b: vec![]
        };

        let ent1 = EntityBuilder::new()
            .with(initial_data.clone())
            .create(&mut world);

        let ent2 = EntityBuilder::new()
            .with(FooData {
                a: 4,
                b: vec![]
            }).create(&mut world);

        let view = world.view();

        let test_data_1 = view.data::<FooData>(ent1);
        let test_data_2 = view.data::<FooData>(ent2);

        assert_eq!(*test_data_1, initial_data);
        assert_eq!(test_data_1.a, 1);

        world.add_constant_modifier(ent1, AddToAModifier { delta_a: 4 });
        world.add_event(CoreEvent::TimePassed);

        assert_eq!(test_data_1.a, 5);
        assert_eq!(test_data_2.a, 4);

        world.add_dynamic_modifier(ent1, MultiplyByOtherEntityModifier { other_entity: ent2 });
        world.add_event(CoreEvent::TimePassed);

        assert_eq!(test_data_1.a, 20);

        world.add_constant_modifier(ent2, AddToAModifier { delta_a: 1 });
        world.add_event(CoreEvent::TimePassed);

        assert_that!(&test_data_2.a).is_equal_to(5);
        assert_that!(&test_data_1.a).is_equal_to(25);
    }

    #[test]
    pub fn test_multiple_data_types() {
        let mut world = World::new();

        world.register::<FooData>();
        world.register::<BarData>();

        let ent1 = EntityBuilder::new()
            .with(FooData {
                a: 1,
                b: vec![]
            })
            .with(BarData {
                x: 1.0
            })
            .create(&mut world);

        let ent2 = EntityBuilder::new()
            .with(FooData {
                a: 2,
                b: vec![]
            })
            .with(BarData {
                x: 1.0
            })
            .create(&mut world);

        world.add_entity(ent1);
        world.add_entity(ent2);

        let view = world.view();

        let foo_data_1 = view.data::<FooData>(ent1);
        let foo_data_2 = view.data::<FooData>(ent2);

        let bar_data_1 = view.data::<BarData>(ent1);
        let bar_data_2 = view.data::<BarData>(ent2);

        assert_that(&foo_data_1.a).is_equal_to(1);
        assert_that(&bar_data_1.x).is_equal_to(bar_data_2.x);

        world.add_modifier(ent1, AddBarDataModifier { delta: 2.0 }.wrap(), "test");
        world.add_event(CoreEvent::TimePassed);

        // show up in reverse chronological order, last created first in list
        assert_that(&view.entities.get(0).unwrap().0).is_equal_to(ent2);
        assert_that(&view.entities.get(1).unwrap().0).is_equal_to(ent1);

        // now that it's been modified they should not be the same
        assert_that(&bar_data_1.x).is_not_equal_to(bar_data_2.x);
        assert_that(&bar_data_1.x).is_equal_to(3.0);

        world.add_dynamic_modifier(ent1, MultiplyByOtherEntityModifier { other_entity: ent2 });
        world.add_event(CoreEvent::TimePassed);

        assert_that(&foo_data_1.a).is_equal_to(2);

        world.add_constant_modifier(ent2, AddFooBValueModifier {});
        world.add_event(CoreEvent::TimePassed);

        assert_that(&bar_data_1.x).is_equal_to(3.0);
        assert_that(&foo_data_1.a).is_equal_to(2);
        assert_that(&foo_data_2.b).is_equal_to(vec![1.0]);
    }

    #[test]
    pub fn test_entity_index() {
        let mut world = World::new();

        world.register::<FooData>();
        world.register::<BarData>();
        world.register_index::<AxialCoord>();

        let ent1 = EntityBuilder::new()
            .with(FooData {
                a: 1,
                b: vec![]
            })
            .with(BarData {
                x: 1.0
            })
            .create(&mut world);

        let ent2 = EntityBuilder::new()
            .with(FooData {
                a: 2,
                b: vec![]
            })
            .with(BarData {
                x: 1.0
            })
            .create(&mut world);

        world.add_entity(ent1);
        world.index_entity(ent1, AxialCoord::new(2,2));
        world.add_entity(ent2);
        world.index_entity(ent2, AxialCoord::new(3,4));

        world.add_event(CoreEvent::TimePassed);

        let view = world.view();

        assert_that!(view.entity_by_key(&AxialCoord::new(2,2))).is_equal_to(Some(ent1));
        assert_that!(view.entity_by_key(&AxialCoord::new(3,4))).is_equal_to(Some(ent2));
        assert_that!(view.entity_by_key(&AxialCoord::new(4,5))).is_equal_to(None);
    }


    #[test]
    pub fn test_disabling_modifiers() {
        pretty_env_logger::init();

        let mut world : World = World::new();

        world.register::<FooData>();

        let ent1 = EntityBuilder::new()
            .with(FooData {
                a: 1,
                b: vec![]
            })
            .create(&mut world);

        let ent2 = EntityBuilder::new()
            .with(FooData {
                a: 2,
                b: vec![]
            })
            .create(&mut world);

        world.add_entity(ent1);
        world.add_entity(ent2);

        let view = world.view();

        let foo_data_1 = view.data::<FooData>(ent1);
        let foo_data_2 = view.data::<FooData>(ent2);

        assert_that(&foo_data_1.a).is_equal_to(1);
        assert_that(&foo_data_1.b).is_equal_to(&Vec::new());

        assert_that(&foo_data_2.a).is_equal_to(2);
        assert_that(&foo_data_2.b).is_equal_to(&Vec::new());

        // add a simple modifier to increase a by 4, should now be 5. Keep a reference to the modifier
        let modifier_ref_1 = world.add_modifier(ent1, FooData::a.add(4), "simple addition");
        world.add_event(CoreEvent::WorldInitialized);

        assert_that(&foo_data_1.a).is_equal_to(5);

        // add a modifier on top of that one that multiples by 2, should now be 10
        world.add_modifier(ent1, FooData::a.mul(2), "multiply by 2");
        world.add_event(CoreEvent::WorldInitialized);

        assert_that(&foo_data_1.a).is_equal_to(10);

        // now disable the first modifier, the second modifier should be layer on top of the base data to make a 2
        world.disable_modifier(modifier_ref_1);
        world.add_event(CoreEvent::TimePassed);

        assert_that(&foo_data_1.a).is_equal_to(2);
    }
}