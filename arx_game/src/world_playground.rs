/*

pub struct World {
    pub entities: Vec<EntityContainer>,
    pub self_entity: Entity,
    pub data: Map<CloneAny>,
    pub modifiers: AnyMap,
    pub total_modifier_count: ModifierClock,
    pub total_dynamic_modifier_count: ModifierClock,
    pub current_time: GameEventClock,
    pub events: Vec<Rc<GameEventWrapper>>,
    pub view: UnsafeCell<WorldView>,
    pub modifier_application_by_type: hash_map::HashMap<TypeId, ModifiersApplication>,
    pub entity_indices: Map<CloneAny>,
    pub index_applications: Vec<IndexApplication>
}

pub struct WorldView {
    entities: Vec<EntityContainer>,
    self_entity : Entity,
    constant_data: Map<CloneAny>,
    effective_data: Map<CloneAny>,
    pub current_time: GameEventClock,
    modifier_cursor: ModifierClock,
    modifier_indices: hash_map::HashMap<TypeId, usize>,
    events: Vec<Rc<GameEventWrapper>>,
    pub entity_indices: Map<CloneAny>
}
*/



mod world_mk3 {
    use world::EntityContainer;
    use entity::Entity;
    use entity::EntityData;
    use anymap::Map;
    use anymap::any::CloneAny;

    struct World {
        pub entities: Vec<EntityContainer>,
        pub self_entity: Entity,
        pub base_data: Map<CloneAny>,
        pub modifiers: Map<CloneAny>,

    }


    struct WorldView {

    }
//    impl WorldView {
//        pub fn data<T : EntityData>(ent : Entity) -> &T {
//            unimplemented()
//        }
//    }
}


#[cfg(test)]
mod test {
    use common::prelude::*;
    use common::reflect::*;
    use prelude::*;
    use entity::EntityData;
    use spectral::assert_that;
    use events::CoreEvent;

    use super::super::entity;
    #[derive(Clone,Default,Debug, Serialize, Deserialize, PrintFields)]
    struct Nested {
        a : f32,
        b : f32
    }

    #[derive(Clone,Default,Debug,Serialize, Deserialize, PrintFields)]
    struct TestData {
        foo : i32,
        bar : Reduceable<i32>,
        name : String,
        nested : Vec<Nested>
    }

    impl EntityData for TestData {}
    
    impl Nested { pub const a : Field < Nested , f32 > = Field :: new ( stringify ! ( a ) , | t | & t . a , | t | & mut t . a , | t , v | { t . a = v ; } ) ; pub const b : Field < Nested , f32 > = Field :: new ( stringify ! ( b ) , | t | & t . b , | t | & mut t . b , | t , v | { t . b = v ; } ) ; }
    impl TestData { pub const foo : Field < TestData , i32 > = Field :: new ( stringify ! ( foo ) , | t | & t . foo , | t | & mut t . foo , | t , v | { t . foo = v ; } ) ; pub const bar : Field < TestData , Reduceable < i32 > > = Field :: new ( stringify ! ( bar ) , | t | & t . bar , | t | & mut t . bar , | t , v | { t . bar = v ; } ) ; pub const name : Field < TestData , String > = Field :: new ( stringify ! ( name ) , | t | & t . name , | t | & mut t . name , | t , v | { t . name = v ; } ) ; pub const nested : Field < TestData , Vec < Nested > > = Field :: new ( stringify ! ( nested ) , | t | & t . nested , | t | & mut t . nested , | t , v | { t . nested = v ; } ) ; }


    #[test]
    pub fn test_tmp () {

        let x = TestData {
            foo : 0,
            bar : Reduceable::new(3),
            ..Default::default()
        };

        println!("Name {}", TestData::foo.name);
        println!("Size of field ref: {}", std::mem::size_of::<Field<TestData, i32>>());

        let mut world = World::new();
        world.register::<TestData>();

        let view = world.view();

        let ent = EntityBuilder::new()
            .with(x)
            .create(&mut world);

        world.add_modifier(ent, FieldModifier::permanent(&TestData::foo, transformations::SetTo(7i32)), "Test 1");
        world.add_event(CoreEvent::TimePassed);
        assert_that(&view.data::<TestData>(ent).foo).is_equal_to(&7);

        world.add_modifier(ent, TestData::foo.set_to(9), "Test 2");
        world.add_event(CoreEvent::TimePassed);
        assert_that(&view.data::<TestData>(ent).foo).is_equal_to(&9);

        world.add_modifier(ent, TestData::foo.add(1), "Test 3");
        world.add_event(CoreEvent::TimePassed);
        assert_that(&view.data::<TestData>(ent).foo).is_equal_to(&10);


        let data = view.data::<TestData>(ent);
        assert_that(&data.bar).is_equal_to(&Reduceable::new(3));
        world.add_modifier(ent, FieldModifier::permanent(&TestData::bar, transformations::ReduceBy(1)), "Test 4");
        world.add_event(CoreEvent::TimePassed);
        assert_that(&data.bar.cur_value()).is_equal_to(&2);

        world.add_modifier(ent, TestData::bar.recover_by(1), "Test 5");
        world.add_event(CoreEvent::TimePassed);
        assert_that(&data.bar.cur_value()).is_equal_to(&3);

        assert_eq!(TestData::foo, TestData::foo);
        assert_ne!(TestData::foo, TestData::bar);

        let modifiers = world.permanent_field_logs_for::<TestData>(ent);
        println!("Modifications for foo");
        modifiers.modifications_for(&TestData::foo).for_each(|fm| { println!("{} {}", fm.field, fm.modification); });

        println!("Modifications for bar");
        modifiers.modifications_for(&TestData::bar).for_each(|fm| { println!("{} {}", fm.field, fm.modification); });
    }
}