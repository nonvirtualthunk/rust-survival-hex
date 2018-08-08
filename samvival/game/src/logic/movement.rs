use common::flood_search;
use common::hex::*;
use game::core::Oct;
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

pub struct MovementTarget {
    hex : AxialCoord,
    move_cost : Oct
}


pub fn move_cost(world_view : &WorldView, from : &AxialCoord, to : &AxialCoord) -> f64 {
    world_view.tile_opt(*to).map(|t| t.move_cost.as_f64()).unwrap_or(100000.0)
}

pub fn hexes_in_range(world_view : &WorldView, mover : Entity, range : Oct) -> HashMap<AxialCoord, f64> {
    let start_position = world_view.character(mover).position;
    flood_search(start_position, range.as_f64(), |from, to| move_cost(world_view, from, to), |&from| from.neighbors())
}

pub fn path_to(world_view: &WorldView, mover : Entity, to : AxialCoord) -> Option<(Vec<AxialCoord>, f64)> {
    let from = world_view.character(mover).position;
    astar(&from, |c| c.neighbors().into_iter().map(|c| (c, r32(move_cost(world_view, &c, &c) as f32))), |c| c.distance(&to), |c| *c == to)
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
    let start_pos = view.character(mover).position;
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
                modify(world, mover, ChangePositionMod(hex));
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
            modify(world, character, ChangePositionMod(pos));

            world.add_event(GameEvent::EntityAppears { character, at : pos });

            return true;
        }
    }
    false
}

