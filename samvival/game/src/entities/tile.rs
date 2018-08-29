use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use game::core::*;
use std::ops::Deref;
use common::prelude::*;
use common::hex::*;
use entities::common::Taxon;

#[derive(Clone, Default, Debug, PrintFields)]
pub struct TileData {
    pub main_terrain_name: Str,
    pub secondary_terrain_name: Option<Str>,
    pub position: AxialCoord,
    pub move_cost: Sext,
    pub cover: i8,
    pub occupied_by : Option<Entity>,
    pub elevation: i8
}
impl EntityData for TileData {}


pub trait TileStore {
    fn tile (&self, coord : AxialCoord) -> &TileData;
    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData>;
    fn tile_ent(&self, coord: AxialCoord) -> TileEntity;
    fn tile_ent_opt(&self, coord: AxialCoord) -> Option<TileEntity>;
}

impl TileStore for WorldView {
    fn tile(&self, coord: AxialCoord) -> &TileData {
        let tile_ent = self.entity_by_key(&coord).expect("Tile is expected to exist");
        self.data::<TileData>(tile_ent)
    }

    fn tile_opt(&self, coord: AxialCoord) -> Option<&TileData> {
        self.entity_by_key(&coord).map(|e| self.data::<TileData>(e))
    }

    fn tile_ent(&self, coord: AxialCoord) -> TileEntity {
        self.tile_ent_opt(coord).expect("tile is expected to exist")
    }

    fn tile_ent_opt(&self, coord: AxialCoord) -> Option<TileEntity> {
        if let Some(entity) = self.entity_by_key(&coord) {
            Some(TileEntity { entity, data : self.data::<TileData>(entity) })
        } else {
            None
        }
    }
}

pub struct TileEntity<'a> { pub entity : Entity, pub data : &'a TileData }
impl <'a> Deref for TileEntity<'a> {
    type Target = TileData;

    fn deref(&self) -> &TileData {
        self.data
    }
}



pub struct HarvestableData {
    pub harvestables : Vec<Harvestable>
}



pub struct Harvestable {
    pub non_renewable_amount : Reduceable<Sext>,
    pub renewable_amount : Reduceable<Sext>,
    pub renew_rate : Sext,
    pub resource : Entity,
//    pub on_depletion_effects :
}

#[derive(Clone,Default,Debug,PrintFields)]
pub struct Resources {
    pub straw : Entity,
    pub wood : Entity,
    pub iron : Entity,
    pub stone : Entity,
    pub fruit : Entity,
    pub custom_resources : HashMap<String,Entity>,
}
impl EntityData for Resources {}

#[derive(Clone,Default,Debug,PrintFields)]
pub struct FoodInfo {
    pub satiation : i32
}
impl EntityData for FoodInfo {}

#[derive(Clone,Default,Debug,PrintFields)]
pub struct Material {
    pub edge : i32, // how well it can hold an edge
    pub hardness : i32, 
    pub flammable : bool,
    pub density: i32, 
    pub ductile : bool,
    pub cordable : bool,
    pub magnetic : bool,
    pub stack_size : i32,
    
}
impl EntityData for Material {}


use entities::taxonomy;
use entities::taxon;
use entities::taxon_vec;
use std::collections::HashMap;
use game::World;
use game::EntityBuilder;
use entities::common::IdentityData;


struct InitResourcesModifier {
    resources : Resources
}
use game::Modifier;
use game::ModifierType;
impl Modifier<Resources> for InitResourcesModifier {
    fn modify(&self, data: &mut Resources, world: &WorldView) {
        *data = self.resources.clone()
    }

    fn is_active(&self, world: &WorldView) -> bool { true }
    fn modifier_type(&self) -> ModifierType { ModifierType::Permanent }
}

fn init_resources(world: &mut World) {
    let existing = world.world_data_opt::<Resources>().cloned().unwrap_or(Resources::default());

    let straw = EntityBuilder::new()
        .with(Material {
            hardness : 1,
            density: 1,
            flammable : true,
            cordable : true,
            stack_size : 6,
            ..Default::default()
        })
        .with(IdentityData::of_kind(&taxonomy::resources::Straw));
    let wood = EntityBuilder::new()
        .with(Material {
            hardness : 4,
            density: 3,
            flammable : true,
            stack_size : 3,
            ..Default::default()
        })
        .with(IdentityData::of_kind(&taxonomy::resources::Wood));
    let iron = EntityBuilder::new()
        .with(Material {
            edge : 6,
            hardness : 10,
            density: 6,
            ductile : true,
            magnetic : true,
            stack_size : 3,
            ..Default::default()
        })
        .with(IdentityData::of_kind(&taxonomy::resources::Iron));
    let stone = EntityBuilder::new()
        .with(Material {
            hardness : 6,
            density : 5,
            stack_size : 1,
            ..Default::default()
        })
        .with(IdentityData::of_kind(&taxonomy::resources::Stone));

    let fruit = EntityBuilder::new()
        .with(Material {
            hardness : 1,
            density: 1,
            flammable : true,
            cordable : true,
            stack_size : 6,
            ..Default::default()
        })
        .with(IdentityData::of_kind(&taxonomy::resources::Fruit));
    
    let resources = Resources {
        straw : existing.straw.as_opt().unwrap_or_else(|| straw.create(world)),
        iron : existing.iron.as_opt().unwrap_or_else(|| iron.create(world)),
        wood : existing.wood.as_opt().unwrap_or_else(|| wood.create(world)),
        fruit : existing.fruit.as_opt().unwrap_or_else(|| fruit.create(world)),
        stone : existing.stone.as_opt().unwrap_or_else(|| stone.create(world)),
        custom_resources : existing.custom_resources
    };

    if world.has_world_data::<Resources>() {
        world.modify_world(box InitResourcesModifier { resources }, "resource initialization");
    } else {
        world.attach_world_data(resources);
    }
}