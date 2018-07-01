use world::WorldView;
use common::hex::AxialCoord;
use world::Entity;
use entities::CharacterData;
use std::collections::HashSet;
use noisy_float::prelude::R32;
use pathfinding::prelude::astar;
use noisy_float::prelude::r32;

pub fn character_at(view : &WorldView, coord : AxialCoord) -> Option<(Entity, &CharacterData)> {
    for (cref, cdata) in view.entities_with_data::<CharacterData>() {
        if cdata.position == coord && cdata.is_alive() {
            return Some((*cref, cdata));
        }
    }
    None
}

pub fn path(world : &WorldView, from: AxialCoord, to: AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    astar(&from, |c| c.neighbors().into_iter().map(|c| (c, r32(1.0))), |c| c.distance(&to), |c| *c == to)
}

pub fn path_any_v(world : &WorldView, from: AxialCoord, to: &Vec<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mut set = HashSet::new();
    set.extend(to.iter());
    path_any(world, from, &set, heuristical_center)
}

pub fn path_any(world : &WorldView, from: AxialCoord, to: &HashSet<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    astar(&from, |c| c.neighbors().into_iter().map(|c| (c, r32(1.0))), |c| c.distance(&heuristical_center), |c| to.contains(c))
}



trait GetCommonData {
    fn character (&self, entity : Entity) -> &CharacterData;
}

impl GetCommonData for WorldView {
    fn character(&self, entity: Entity) -> &CharacterData {
        self.data::<CharacterData>(entity)
    }
}