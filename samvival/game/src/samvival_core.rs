use common::prelude::*;
use game::prelude::*;
use data::entities::*;
use data::events::GameEvent;
use logic::visibility::VisibilityComputor;


pub fn create_world() -> World {
    let mut world = World::new();

    taxonomy::register();

    register_world_data(&mut world);

    world.attach_world_data(MapData {
        min_tile_bound: AxialCoord::new(-30, -30),
        max_tile_bound: AxialCoord::new(30, 30),
    });
    world.attach_world_data(VisibilityData::default());

    world
}

pub fn register_world_data(world : &mut World) {
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
    world.register::<VisibilityComputor>();
    world.register::<ToolData>();
    world.register::<StackData>();
    world.register::<Harvestable>();
    world.register::<VegetationData>();
    world.register::<TerrainData>();
    world.register::<Material>();
    world.register::<WorthData>();
    world.register::<ItemArchetype>();
    world.register::<EntityMetadata>();
    world.register::<Recipe>();

    register_custom_ability_data(world);
    // -------- world data ---------------
    world.register::<MapData>();
    world.register::<TurnData>();
    world.register::<TimeData>();
    world.register::<VisibilityData>();
    world.register::<Effects>();
    world.register::<Resources>();
    world.register::<RuntimeTaxonData>();

    println!("Registering axial coord index");
    world.register_index::<AxialCoord>();

    world.register_event_type::<GameEvent>();
}

pub fn initialize_world(world : &mut World) {
    Resources::init_resources(world);
    Effects::init_effects(world);
}