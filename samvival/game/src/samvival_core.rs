use common::prelude::*;
use game::prelude::*;
use data::entities::*;
use data::events::GameEvent;


pub fn create_world() -> World {
    let mut raw_world = World::new();
    {
        let world = &mut raw_world;
        // -------- entity data --------------
        world.register::<TileData>();
        world.register::<CharacterData>();
        world.register::<CombatData>();
        world.register::<EquipmentData>();
        world.register::<InventoryData>();
        world.register::<SkillData>();
        world.register::<ItemData>();
        world.register::<FactionData>();
        world.register::<PositionData>();
        world.register::<GraphicsData>();
        world.register::<IdentityData>();
        world.register::<ActionData>();
        world.register::<ModifierTrackingData>();
        world.register::<AttributeData>();
        world.register::<AllegianceData>();
        world.register::<ObserverData>();
        world.register::<Attack>();
        world.register::<MovementData>();
        world.register::<DerivedAttackData>();
        world.register::<MovementType>();

        register_custom_ability_data(world);
        // -------- world data ---------------
        world.register::<MapData>();
        world.register::<TurnData>();
        world.register::<TimeData>();
        world.register::<VisibilityData>();

        world.register_index::<AxialCoord>();

        world.register_event_type::<GameEvent>();

        world.attach_world_data(MapData {
            min_tile_bound: AxialCoord::new(-30, -30),
            max_tile_bound: AxialCoord::new(30, 30),
        });
        world.attach_world_data(VisibilityData::default());
    }

    raw_world
}