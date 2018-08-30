use game::Entity;
use game::world::WorldView;
use data::entities::CharacterData;
use data::entities::TileData;
use common::AxialCoord;
use common::prelude::*;
use data::entities::*;
use game::core::Sext;


pub fn position_of(entity : Entity, world : &WorldView) -> Option<AxialCoord> {
    if world.has_data::<TileData>(entity) {
        Some(world.data::<TileData>(entity).position)
    } else if world.has_data::<CharacterData>(entity) {
        Some(world.data::<PositionData>(entity).hex)
    } else {
        None
    }
}