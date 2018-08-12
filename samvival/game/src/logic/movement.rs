use common::flood_search;
use common::hex::*;
use game::core::Sext;
use entities::*;
use entities::Attack;
use entities::Skill;
use entities::modifiers::*;
use game::events::*;
use noisy_float;
use noisy_float::prelude::*;
use pathfinding::prelude::astar;
use rand::Rng;
use rand::SeedableRng;
use rand::StdRng;
use std::collections::HashMap;
use game::Entity;
use game::world::World;
use game::world::WorldView;
use logic::movement;
use events::GameEvent;
use game::SettableField;
use std::collections::HashSet;


pub fn max_moves_remaining(world_view : &WorldView, mover : Entity, multiplier: f64) -> Sext {
    let cd = world_view.data::<CharacterData>(mover);
    cd.moves + Sext::of_rounded(cd.move_speed.as_f64() * cd.action_points.cur_value() as f64 * multiplier)
}

pub fn hexes_in_range(world_view : &WorldView, mover : Entity, range : Sext) -> HashMap<AxialCoord, f64> {
    let mover_c = world_view.character(mover);
    let start_position = mover_c.position.hex;
    flood_search(start_position, range.as_f64(), |from, to| move_cost_to(world_view, &mover_c, to) as f64, |&from| from.neighbors_vec())
}

pub fn path_to(world_view: &WorldView, mover : Entity, to : AxialCoord) -> Option<(Vec<AxialCoord>, f64)> {
    let mover_c = world_view.character(mover);
    let from = mover_c.position.hex;
    astar(&from, |c| c.neighbors_vec().into_iter().map(|c| (c, r32(move_cost_to(world_view, &mover_c, &c) as f32))), |c| c.distance(&to), |c| *c == to)
        .map(|(vec, cost)| (vec, cost.raw() as f64))
}


pub fn hex_ap_cost(world : &WorldView, mover : Entity, hex : AxialCoord) -> u32 {
    let mover = world.character(mover);
    let hex_cost = world.tile(hex).move_cost;
    let mut moves = mover.moves;
    let mut ap_cost = 0;
    while moves < hex_cost {
        moves += mover.move_speed;
        ap_cost += 1;
    }
    ap_cost
}


pub fn handle_move(world : &mut World, mover : Entity, path : &[AxialCoord]) {
    let view = world.view();
    let start_pos = view.character(mover).position.hex;
    let mut prev_hex = start_pos;
    let mut prev_hex_ent = view.entity_by_key(&start_pos).expect("hex must exist");
    for hex in path {
        let hex = *hex;
        if hex != start_pos {
            let hex_ent = view.entity_by_key(&hex).expect("hex must exist");
            let hex_cost = view.tile(hex).move_cost;
            // how many ap must be changed to move points in order to enter the given hex
            let ap_required = movement::hex_ap_cost(world.view(), mover, hex);
            if ap_required as i32 <= view.character(mover).action_points.cur_value() {
                let moves_converted = view.character(mover).move_speed * ap_required;
                let net_moves_lost = hex_cost - moves_converted;
                modify(world, mover, ReduceActionsMod(ap_required));
                modify(world, mover, ReduceMoveMod(net_moves_lost));
                world.modify(mover, PositionData::hex.set_to(hex), "movement");
                modify(world, prev_hex_ent, SetHexOccupantMod(None));
                modify(world, hex_ent, SetHexOccupantMod(Some(mover)));

//                modify(world, mover, SkillXPMod(Skill::ForestSurvival, 1));
                // advance the event clock
                world.add_event(GameEvent::Move { character : mover, from : prev_hex, to : hex });

                prev_hex = hex;
                prev_hex_ent = hex_ent;
            } else {
                break;
            }
        }
    }
}

pub fn place_entity_in_world(world: &mut World, character : Entity, pos : AxialCoord) -> bool {
    let view = world.view();
    if let Some(tile) = view.tile_ent_opt(pos) {
        if tile.occupied_by.is_none() {
            modify(world, tile.entity, SetHexOccupantMod(Some(character)));
            world.modify(character, PositionData::hex.set_to(pos), "placement");

            world.add_event(GameEvent::EntityAppears { entity: character, at : pos });

            return true;
        }
    }
    false
}

pub fn remove_entity_from_world(world: &mut World, entity : Entity) {
    let view = world.view();
    if let Some(cur_pos) = view.data_opt::<PositionData>(entity) {
        if let Some(tile) = view.tile_ent_opt(cur_pos.hex) {
            world.modify(tile.entity, TileData::occupied_by.set_to(None), "entity removed");
        }
    }
}



pub fn path(world : &WorldView, mover : Entity, from: AxialCoord, to: AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mover = world.character(mover);
    astar(&from, |c| c.neighbors_vec().into_iter().map(|c| (c, r32(move_cost_to(world, &mover, &c)))), |c| c.distance(&to), |c| *c == to)
}

pub fn path_any_v(world : &WorldView, mover : Entity, from: AxialCoord, to: &Vec<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mut set = HashSet::new();
    set.extend(to.iter());
    path_any(world, mover, from, &set, heuristical_center)
}

pub fn path_any(world : &WorldView, mover : Entity, from: AxialCoord, to: &HashSet<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mover = world.character(mover);
    astar(&from, |c| c.neighbors_vec().into_iter().map(|c| (c, r32(move_cost_to(world, &mover, &c)))), |c| c.distance(&heuristical_center), |c| to.contains(c))
}


pub fn move_cost_to(world: &WorldView, mover : &Character, to: &AxialCoord) -> f32 {
    if let Some(tile) = world.tile_opt(*to) {
        if tile.occupied_by.is_none() {
            tile.move_cost.as_f32()
        } else {
            10000000.0
        }
    } else {
        10000000.0
    }
}
