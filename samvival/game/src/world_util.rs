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


pub fn path(world : &WorldView, mover : Entity, from: AxialCoord, to: AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mover = world.character(mover);
    astar(&from, |c| c.neighbors().into_iter().map(|c| (c, r32(move_cost_to(world, &mover, c)))), |c| c.distance(&to), |c| *c == to)
}

pub fn path_any_v(world : &WorldView, mover : Entity, from: AxialCoord, to: &Vec<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mut set = HashSet::new();
    set.extend(to.iter());
    path_any(world, mover, from, &set, heuristical_center)
}

pub fn path_any(world : &WorldView, mover : Entity, from: AxialCoord, to: &HashSet<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mover = world.character(mover);
    astar(&from, |c| c.neighbors().into_iter().map(|c| (c, r32(move_cost_to(world, &mover, c)))), |c| c.distance(&heuristical_center), |c| to.contains(c))
}


pub fn move_cost_to(world: &WorldView, mover : &CharacterData, to: AxialCoord) -> f32 {
    if let Some(tile) = world.tile_opt(to) {
        if tile.occupied_by.is_none() {
            tile.move_cost.as_f32()
        } else {
            10000000.0
        }
    } else {
        10000000.0
    }
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