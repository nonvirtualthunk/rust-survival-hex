use game::world::WorldView;
use common::hex::AxialCoord;
use game::Entity;
use data::entities::CharacterData;
use std::collections::HashSet;
use noisy_float::prelude::R32;
use pathfinding::prelude::astar;
use noisy_float::prelude::r32;
use data::entities::TileStore;
use data::entities::CharacterStore;
use data::entities::character::Character;

pub fn character_at(view : &WorldView, coord : AxialCoord) -> Option<(Entity, Character)> {
    for (cref, cdata) in view.entities_with_data::<CharacterData>() {
        let character = view.character(*cref);
        if character.position.hex == coord {
            if cdata.is_alive() {
                return Some((*cref, view.character(*cref)));
            } else {
                info!("Attempted to select dead character");
            }
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