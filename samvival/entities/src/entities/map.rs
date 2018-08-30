use game::Entity;
use game::entity::EntityData;
use game::world::WorldView;
use common::hex::AxialCoord;
use game::core::*;
use game::entity;

#[derive(Clone, Default, Debug, Serialize, Deserialize, PrintFields)]
pub struct MapData {
    pub min_tile_bound : AxialCoord,
    pub max_tile_bound : AxialCoord
}
impl EntityData for MapData {}
