use world::Entity;
use world::EntityData;
use world::WorldView;
use common::hex::AxialCoord;
use core::*;


#[derive(Clone, Default, Debug, PrintFields)]
pub struct MapData {
    pub min_tile_bound : AxialCoord,
    pub max_tile_bound : AxialCoord
}
impl EntityData for MapData {}
