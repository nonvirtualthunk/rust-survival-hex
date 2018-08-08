use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::hex::AxialCoord;
use game::core::*;


#[derive(Clone, Default, Debug, PrintFields)]
pub struct MapData {
    pub min_tile_bound : AxialCoord,
    pub max_tile_bound : AxialCoord
}
impl EntityData for MapData {}
