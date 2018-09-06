use entity::*;
use prelude::*;
use common::prelude::*;
use common::hex::*;
use common::reflect::*;
use events::CoreEvent;

#[cfg(test)]
mod test {
    use super::*;
    use modifiers::*;

    use super::super::super::entity;
    #[derive(Clone, Default, PartialEq, Debug, Serialize, Deserialize, Fields)]
    struct FooData {
        pub a: i32,
        pub b: Vec<f32>
    }

    #[derive(Clone, Default, PartialEq, Debug, Serialize, Deserialize, Fields)]
    struct BarData {
        pub x: f32
    }

    impl FooData { pub const a : Field < FooData , i32 > = Field :: new ( stringify ! ( a ) , | t | & t . a , | t | &mut t . a, | t , v | { t . a = v ; } ) ; pub const b : Field < FooData , Vec < f32 > > = Field :: new ( stringify ! ( b ) , | t | & t . b , | t | &mut t . b, | t , v | { t . b = v ; } ) ; }
    impl BarData { pub const x : Field < BarData , f32 > = Field :: new ( stringify ! ( x ) , | t | & t . x , | t | &mut t . x, | t , v | { t . x = v ; } ) ; }

    impl EntityData for FooData {}

    impl EntityData for BarData {}

    #[derive(Serialize)]
    struct AddToAModifier {
        delta_a: i32
    }

    impl Modifier<FooData> for AddToAModifier {
        fn modify(&self, data: &mut FooData, world: &WorldView) {
            data.a += self.delta_a;
        }

        fn is_active(&self, world: &WorldView) -> bool { true }

        fn modifier_type(&self) -> ModifierType { ModifierType::Permanent }
    }

    #[derive(Serialize)]
    pub struct MultiplyByOtherEntityModifier {
        other_entity: Entity
    }

    impl Modifier<FooData> for MultiplyByOtherEntityModifier {
        fn modify(&self, data: &mut FooData, world: &WorldView) {
//            println!("Applying dynamic modifier: {:?}, {:?}", data.a , world.data::<FooData>(self.other_entity).a);
            data.a = data.a * world.data::<FooData>(self.other_entity).a;
        }

        fn is_active(&self, world: &WorldView) -> bool {
            true
        }

        fn modifier_type(&self) -> ModifierType { ModifierType::Dynamic }
    }

    #[derive(Serialize)]
    pub struct AddBarDataModifier {
        delta: f32
    }

    impl Modifier<BarData> for AddBarDataModifier {
        fn modify(&self, data: &mut BarData, world: &WorldView) {
            data.x += self.delta;
        }

        fn is_active(&self, world: &WorldView) -> bool { true }

        fn modifier_type(&self) -> ModifierType { ModifierType::Permanent }
    }

    use spectral::prelude::*;

    #[derive(Serialize)]
    pub struct AddFooBValueModifier {}

    impl Modifier<FooData> for AddFooBValueModifier {
        fn modify(&self, data: &mut FooData, world: &WorldView) {
            data.b.push(1.0);
        }

        fn is_active(&self, world: &WorldView) -> bool { true }

        fn modifier_type(&self) -> ModifierType { ModifierType::Permanent }
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

        world.modify_with_desc(ent1, box AddToAModifier { delta_a: 4 }, None);
        world.add_event(CoreEvent::TimePassed);

        assert_eq!(test_data_1.a, 5);
        assert_eq!(test_data_2.a, 4);

        assert_eq!(view.data::<FooData>(ent1).a, 5);
        assert_eq!(view.data::<FooData>(ent2).a, 4);

        world.modify_with_desc(ent1, box MultiplyByOtherEntityModifier { other_entity: ent2 }, None);
        world.add_event(CoreEvent::TimePassed);

        assert_eq!(test_data_1.a, 20);

        world.modify_with_desc(ent2, box AddToAModifier { delta_a: 1 }, None);
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


        let view = world.view();

        let foo_data_1 = view.data::<FooData>(ent1);
        let foo_data_2 = view.data::<FooData>(ent2);

        let bar_data_1 = view.data::<BarData>(ent1);
        let bar_data_2 = view.data::<BarData>(ent2);

        assert_that(&foo_data_1.a).is_equal_to(1);
        assert_that(&bar_data_1.x).is_equal_to(bar_data_2.x);

        world.modify_with_desc(ent1, box AddBarDataModifier { delta: 2.0 }, "test");
        world.add_event(CoreEvent::TimePassed);

        // show up in chronological order, first created first in list
        assert_that(&view.entities.get(0).unwrap().0).is_equal_to(ent1);
        assert_that(&view.entities.get(1).unwrap().0).is_equal_to(ent2);

        // now that it's been modified they should not be the same
        assert_that(&bar_data_1.x).is_not_equal_to(bar_data_2.x);
        assert_that(&bar_data_1.x).is_equal_to(3.0);

        world.modify_with_desc(ent1, box MultiplyByOtherEntityModifier { other_entity: ent2 }, None);
        world.add_event(CoreEvent::TimePassed);

        assert_that(&foo_data_1.a).is_equal_to(2);

        world.modify_with_desc(ent2, box AddFooBValueModifier {}, None);
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
        rust_init();

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
        let modifier_ref_1 = world.modify_with_desc(ent1, FooData::a.add(4), "simple addition");
        world.add_event(CoreEvent::WorldInitialized);

        let mut view_2 = world.view_at_time(world.next_time);

        assert_that(&view_2.data::<FooData>(ent1).a).is_equal_to(5);
        assert_that(&foo_data_1.a).is_equal_to(5);

        // add a modifier on top of that one that multiples by 2, should now be 10
        world.modify_with_desc(ent1, FooData::a.mul(2), "multiply by 2");
        world.add_event(CoreEvent::WorldInitialized);

        assert_that(&foo_data_1.a).is_equal_to(10);

        // now disable the first modifier, the second modifier should be layer on top of the base data to make a 2
        world.disable_modifier(modifier_ref_1);
        world.add_event(CoreEvent::TimePassed);
        let just_disabled_time = world.current_time();

        assert_that(&foo_data_1.a).is_equal_to(2);

//        println!("Applying modifier to take effect at {:?}", world.next_time);
        world.modify_with_desc(ent1, FooData::a.add(4), None);
        world.add_event(CoreEvent::TimePassed);

//        println!("next time: {:?}, mut_view.cur_time: {:?}, just_disabled_time: {:?}", world.next_time, view.current_time, just_disabled_time);
        // bringing the non-realtime view up to date with just after we disabled the modifier it should have the correct value
        world.update_view_to_time(&mut view_2, just_disabled_time);
        assert_that(&view_2.data::<FooData>(ent1).a).is_equal_to(2);

    }


    #[test]
    pub fn test_registering_new_data_type() {
        rust_init();

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

        let view = world.view();
        let mut view_2 = world.view_at_time(world.next_time);

        let foo_data_1 = view.data::<FooData>(ent1);
        let foo_data_2 = view.data::<FooData>(ent2);

        assert_that(&foo_data_1.a).is_equal_to(1);
        assert_that(&foo_data_1.b).is_equal_to(&Vec::new());

        assert_that(&foo_data_2.a).is_equal_to(2);
        assert_that(&foo_data_2.b).is_equal_to(&Vec::new());

        // add a simple modifier to increase a by 4, should now be 5. Keep a reference to the modifier
        world.modify_with_desc(ent1, FooData::a.add(4), "simple addition");
        world.add_event(CoreEvent::WorldInitialized);

        world.register::<BarData>();

        world.attach_data(ent1, BarData {x : 3.0});
        world.modify_with_desc(ent1, BarData::x.add(1.0), "x addition");
        world.add_event(CoreEvent::TimePassed);

        world.update_view_to_time(&mut view_2, world.next_time);

        assert_that(&view.has_data::<BarData>(ent1)).is_true();
        assert_that(&view.data::<BarData>(ent1).x).is_equal_to(4.0);
        assert_that(&view_2.data::<BarData>(ent1).x).is_equal_to(4.0);
    }

    #[test]
    pub fn test_deserialization() {
        use spectral::prelude::*;
        rust_init();

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

        // add a simple modifier to increase a by 4, should now be 5. Keep a reference to the modifier
        world.modify_with_desc(ent1, FooData::a.add(4), "simple addition");
        world.add_event(CoreEvent::WorldInitialized);

        world.register::<BarData>();

        world.attach_data(ent1, BarData {x : 3.0});
        world.modify_with_desc(ent1, BarData::x.add(1.0), "x addition");
        world.add_event(CoreEvent::TimePassed);

        use ron;

        let serialized_world = ron::ser::to_string(&world).expect("Could not serialize world");

        let mut deserialized_world : World = ron::de::from_str(&serialized_world).expect("Could not deserialize world");
        deserialized_world.initialize_loaded_world();

        deserialized_world.register::<FooData>();
        deserialized_world.register::<BarData>();

        let view = deserialized_world.view();
        let foo1 = view.data::<FooData>(ent1);
        assert_that(&foo1).is_equal_to(&FooData { a : 5 , b : vec![] });
        let bar1 = view.data::<BarData>(ent1);
        assert_that(&bar1).is_equal_to(&BarData { x: 4.0 });

        let core_events = view.events::<CoreEvent>().collect_vec();
        assert_that(&core_events).matching_contains(|w| w.event == CoreEvent::TimePassed);
        assert_that(&core_events).matching_contains(|w| w.event == CoreEvent::WorldInitialized);
    }
}