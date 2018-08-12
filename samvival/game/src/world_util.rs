use game::world::WorldView;
use common::hex::AxialCoord;
use game::Entity;
use entities::CharacterData;
use std::collections::HashSet;
use noisy_float::prelude::R32;
use pathfinding::prelude::astar;
use noisy_float::prelude::r32;
use entities::TileStore;
use entities::CharacterStore;

pub fn character_at(view : &WorldView, coord : AxialCoord) -> Option<(Entity, &CharacterData)> {
    for (cref, cdata) in view.entities_with_data::<CharacterData>() {
        let character = view.character(*cref);
        if character.position.hex == coord && cdata.is_alive() {
            return Some((*cref, cdata));
        }
    }
    None
}



//pub trait GetCommonData {
//    fn character (&self, entity : Entity) -> &CharacterData;
//}
//
//impl GetCommonData for WorldView {
//    fn character(&self, entity: Entity) -> &CharacterData {
//        self.data::<CharacterData>(entity)
//    }
//}