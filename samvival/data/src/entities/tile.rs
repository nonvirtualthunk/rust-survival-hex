use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use game::core::*;
use std::ops::Deref;
use common::prelude::*;
use common::hex::*;
use entities::common_entities::Taxon;
use game::prelude::*;
use common::string::IStr;

#[derive(Clone, Default, Debug, Serialize, Deserialize, PrintFields)]
pub struct TileData {
    pub main_terrain_name: String,
    pub secondary_terrain_name: Option<String>,
    pub position: AxialCoord,
    pub move_cost: Sext,
    pub cover: i8,
    pub occupied_by: Option<Entity>,
    pub elevation: i8,
}

impl EntityData for TileData {}


pub trait TileStore {
    fn tile(&self, coord: AxialCoord) -> &TileData;
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
            Some(TileEntity { entity, data: self.data::<TileData>(entity) })
        } else {
            None
        }
    }
}

pub struct TileEntity<'a> { pub entity: Entity, pub data: &'a TileData }

impl<'a> Deref for TileEntity<'a> {
    type Target = TileData;

    fn deref(&self) -> &TileData {
        self.data
    }
}


pub struct HarvestableData {
    pub harvestables: Vec<Harvestable>
}


pub struct Harvestable {
    pub non_renewable_amount: Reduceable<Sext>,
    pub renewable_amount: Reduceable<Sext>,
    pub renew_rate: Sext,
    pub resource: Entity,
//    pub on_depletion_effects :
}


#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ConstantResources {
    pub straw: Entity,
    pub wood: Entity,
    pub iron: Entity,
    pub stone: Entity,
    pub fruit: Entity,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PrintFields)]
pub struct Resources {
    pub main: ConstantResources,
    pub custom_resources: HashMap<String, Entity>,
}

impl EntityData for Resources {}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PrintFields)]
pub struct FoodInfo {
    pub satiation: i32
}

impl EntityData for FoodInfo {}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PrintFields)]
pub struct Material {
    pub edge: i32,
    // how well it can hold an edge
    pub hardness: i32,
    pub flammable: bool,
    pub density: i32,
    pub ductile: bool,
    pub cordable: bool,
    pub magnetic: bool,
    pub stack_size: i32,

}

impl EntityData for Material {}


use entities::common_entities::taxonomy;
use entities::common_entities::taxon;
use entities::common_entities::taxon_vec;
use std::collections::HashMap;
use game::World;
use game::EntityBuilder;
use entities::common_entities::IdentityData;


struct InitResourcesModifier {
    resources: Resources
}

use game::Modifier;
use game::ModifierType;
//impl Modifier<Resources> for InitResourcesModifier {
//    fn modify(&self, data: &mut Resources, world: &WorldView) {
//        *data = self.resources.clone()
//    }
//
//    fn is_active(&self, world: &WorldView) -> bool { true }
//    fn modifier_type(&self) -> ModifierType { ModifierType::Permanent }
//}

fn init_resources(world: &mut World) {
    world.ensure_world_data::<Resources>();
    let mut main = world.world_data::<Resources>().main.clone();

    if main.straw.is_sentinel() {
        main.straw = EntityBuilder::new()
            .with(Material {
                hardness: 1,
                density: 1,
                flammable: true,
                cordable: true,
                stack_size: 6,
                ..Default::default()
            })
            .with(IdentityData::of_kind(&taxonomy::resources::Straw))
            .create(world);
    }

    if main.wood.is_sentinel() {
        main.wood = EntityBuilder::new()
            .with(Material {
                hardness: 4,
                density: 3,
                flammable: true,
                stack_size: 3,
                ..Default::default()
            })
            .with(IdentityData::of_kind(&taxonomy::resources::Wood))
            .create(world);
    }
    if main.iron.is_sentinel() {
        main.iron = EntityBuilder::new()
            .with(Material {
                edge: 6,
                hardness: 10,
                density: 6,
                ductile: true,
                magnetic: true,
                stack_size: 3,
                ..Default::default()
            })
            .with(IdentityData::of_kind(&taxonomy::resources::Iron))
            .create(world);
    }

    if main.stone.is_sentinel() {
        main.stone = EntityBuilder::new()
            .with(Material {
                hardness: 6,
                density: 5,
                stack_size: 1,
                ..Default::default()
            })
            .with(IdentityData::of_kind(&taxonomy::resources::Stone))
            .create(world);
    }

    if main.fruit.is_sentinel() {
        let fruit = EntityBuilder::new()
            .with(Material {
                hardness: 1,
                density: 1,
                flammable: true,
                cordable: true,
                stack_size: 6,
                ..Default::default()
            })
            .with(IdentityData::of_kind(&taxonomy::resources::Fruit))
            .create(world);
    }

    world.modify_world(Resources::main.set_to(main), "resource initialization");
}