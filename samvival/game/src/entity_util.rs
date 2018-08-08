use game::Entity;
use game::world::WorldView;
use entities::CharacterData;
use entities::TileData;
use common::AxialCoord;
use common::prelude::*;
use entities::*;
use game::core::Oct;


pub fn position_of(entity : Entity, world : &WorldView) -> Option<AxialCoord> {
    if world.has_data::<TileData>(entity) {
        Some(world.data::<TileData>(entity).position)
    } else if world.has_data::<CharacterData>(entity) {
        Some(world.data::<CharacterData>(entity).position)
    } else {
        None
    }
}

pub fn max_remaining_move(entity : Entity, world : &WorldView) -> Oct {
    let char = world.character(entity);
    char.moves + char.move_speed * char.action_points.cur_value()
}