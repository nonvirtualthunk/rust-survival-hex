use common::flood_search;
use common::hex::*;
use game::core::Sext;
use data::entities::*;
use data::entities::Attack;
use data::entities::Skill;
use data::entities::modifiers::*;
use data::events::*;
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
use data::events::GameEvent;
use game::SettableField;
use game::reflect::*;
use std::collections::HashSet;
use data::entities::movement::MovementData;
use data::entities::movement::MovementTypeRef;
use common::ExtendedCollection;


pub fn max_moves_remaining(world_view : &WorldView, mover : Entity, movement_type : MovementTypeRef) -> Sext {
    let cd = world_view.data::<CharacterData>(mover);
    max_moves_for_ap_expenditure(world_view, mover, movement_type, cd.action_points.cur_value())
}

pub fn max_moves_per_turn(world_view: &WorldView, mover : Entity, movement_type : MovementTypeRef) -> Sext {
    let cd = world_view.data::<CharacterData>(mover);
    max_moves_for_ap_expenditure(world_view, mover, movement_type, cd.action_points.max_value())
}

pub fn max_moves_for_ap_expenditure(world_view: &WorldView, mover : Entity, movement_type : MovementTypeRef, ap : i32) -> Sext {
    if let Some(mt) = movement_type.resolve(world_view) {
        let md = world_view.data::<MovementData>(mover);
        md.moves + md.move_speed * mt.move_multiplier * ap + mt.move_bonus
    } else {
        warn!("max_moves_remaining of unresolveable movement type is always 0");
        Sext::of(0)
    }
}

pub fn hexes_in_range(world_view : &WorldView, mover : Entity, range : Sext) -> HashMap<AxialCoord, f64> {
    use std::time::*;

    let mover_c = world_view.character(mover);
    let start_position = mover_c.position.hex;

//    const cw : usize = 60;
//    const cwi : i32 = cw as i32;
//
//    let start = Instant::now();
//    let mut flat_move_costs : [f32;cw*cw] = [0.0;cw*cw];
//    for (ent, tile) in world_view.entities_with_data::<TileData>() {
//        if tile.position.distance(&start_position) <= range.as_f32() {
//            let dq = tile.position.q - start_position.q;
//            let dr = tile.position.r - start_position.r;
//
//            let cost = if tile.occupied_by.is_none() {
//                tile.move_cost.as_f32()
//            } else {
//                10000000.0
//            };
//
//            flat_move_costs[((dq + (cw as i32)/2) * cwi + (dr + (cw as i32)/2)) as usize] = cost;
//        }
//    }

//    let mut move_costs = HashMap::with_capacity(1000);
//    for (ent, tile) in world_view.entities_with_data::<TileData>() {
//        if tile.position.distance(&start_position) <= range.as_f32() {
//            let cost = if tile.occupied_by.is_none() {
//                tile.move_cost.as_f32()
//            } else {
//                10000000.0
//            };
//            move_costs.insert(tile.position, cost);
//        }
//    }
//    println!("Initial move cost map building took {:?}", Instant::now().duration_since(start));

    let tile_accessor = TileAccessor::new(world_view);
    flood_search(start_position, range.as_f64(), move |from, to| {
        let cost = if let Some(tile) = tile_accessor.tile_opt(*to) {
            if tile.occupied_by.is_none() {
                tile_accessor.terrain(&tile).move_cost.as_f32() + tile_accessor.vegetation(&tile).move_cost.as_f32()
            } else { 10000.0 }
        } else { 10000.0 };
        cost as f64
    }, |&from| from.neighbors_vec())

//    flood_search(start_position, range.as_f64(), move |from, to| *(move_costs.get(&to).unwrap_or(&1000000.0)) as f64, |&from| from.neighbors_vec())

//    flood_search(start_position, range.as_f64(), move |from, to| {
//        let dq = to.q - start_position.q;
//        let dr = to.r - start_position.r;
//        let raw_cost = flat_move_costs[((dq + (cw as i32)/2) * (cw as i32)+ (dr + (cw as i32)/2)) as usize];
//        (if raw_cost > 0.0 {
//            raw_cost
//        } else {
//            10000000.0
//        }) as f64
//    }, |&from| from.neighbors_vec())
}

pub fn hexes_reachable_by_character_this_turn(world_view: &WorldView, mover : Entity, movement_type : MovementTypeRef) -> HashMap<AxialCoord, f64> {
    let moves = max_moves_remaining(world_view, mover, movement_type);
    hexes_in_range(world_view, mover, moves)
}

/// returns all the hexes reachable by the given entity this turn using their default movement type
pub fn hexes_reachable_by_character_this_turn_default(world_view: &WorldView, mover:Entity) -> HashMap<AxialCoord, f64> {
    if let Some(movement_type) = default_movement_type(world_view, mover) {
        hexes_reachable_by_character_this_turn(world_view, mover, movement_type)
    } else {
        HashMap::new()
    }
}

pub fn path_to(world_view: &WorldView, mover : Entity, to : AxialCoord) -> Option<(Vec<AxialCoord>, f64)> {
    let mover_c = world_view.character(mover);
    let from = mover_c.position.hex;
    let accessor = TileAccessor::new(world_view);
    astar(&from, |c| c.neighbors_vec().into_iter().map(|c| (c, r32(move_cost_to_f32(&accessor, &mover_c, &c) as f32))), |c| c.distance(&to), |c| *c == to)
        .map(|(vec, cost)| (vec, cost.raw() as f64))
}

pub fn path_adjacent_to(world_view: &WorldView, mover : Entity, to : Entity) -> Option<(Vec<AxialCoord>, R32)> {
    let mover_c = world_view.character(mover);
    let from = mover_c.position.hex;
    let (possibles, center) = if let Some(pos) = world_view.data_opt::<PositionData>(to) {
        (pos.hex.neighbors_vec(), pos.hex)
    } else {
        warn!("path_adjacent_to called against a non-position-data entity");
        (vec![], AxialCoord::new(0,0))
    };
    path_any(world_view, mover, from, &possibles.into_iter().collect(), center)
}

pub fn portion_of_path_traversable_this_turn(view : &WorldView, mover : Entity, path : &Vec<AxialCoord>) -> Vec<AxialCoord> {
    let character = view.character(mover);
    let mut moves = character.movement.moves;
    let mut ret = Vec::new();
    let mut ap_remaining = character.action_points.cur_value();
    if let Some(start) = path.first() { ret.push(*start) }

    let accessor = TileAccessor::new(view);
    for hex in path.iter().skip(1) {
        let hex_cost = move_cost_to(&accessor, mover, hex);
        while hex_cost > moves && ap_remaining > 0 {
            ap_remaining -= 1;
            moves += character.movement.move_speed;
        }

        if moves >= hex_cost {
            ret.push(*hex);
            moves -= hex_cost;
        } else {
            break;
        }
    }
    ret
}


pub fn hex_ap_cost(world : &WorldView, mover : Entity, hex : AxialCoord) -> u32 {
    let accessor = TileAccessor::new(world);
    let hex_cost = move_cost_to(&accessor, mover, &hex);
    ap_cost_for_move_cost(world, mover, hex_cost)
}

pub fn ap_cost_for_move_cost(world : &WorldView, mover : Entity, move_cost : Sext) -> u32 {
    let mover = world.character(mover);
    let mut moves = mover.movement.moves;
    let mut ap_cost = 0;
    while moves < move_cost {
        moves += mover.movement.move_speed;
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
            let hex_cost = view.terrain(hex).move_cost + view.vegetation(hex).move_cost;
            // how many ap must be changed to move points in order to enter the given hex
            let ap_required = movement::hex_ap_cost(world.view(), mover, hex) as i32;
            if ap_required <= view.character(mover).action_points.cur_value() {
                let moves_converted = view.character(mover).movement.move_speed * ap_required;
                let net_moves_lost : Sext = hex_cost - moves_converted;
                world.modify_with_desc(mover, CharacterData::action_points.reduce_by(ap_required), None);
                world.modify_with_desc(mover, MovementData::moves.sub(net_moves_lost), None);
                world.modify_with_desc(mover, PositionData::hex.set_to(hex), None);
                world.modify_with_desc(prev_hex_ent, TileData::occupied_by.set_to(None), None);
                world.modify_with_desc(hex_ent, TileData::occupied_by.set_to(Some(mover)), None);

//                modify(world, mover, SkillXPMod(Skill::ForestSurvival, 1));
                // advance the event clock
                world.add_event(GameEvent::Move { character : mover, from : prev_hex, to : hex, cost: hex_cost });

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
            world.modify_with_desc(tile.entity, TileData::occupied_by.set_to(Some(character)), None);
            world.modify_with_desc(character, PositionData::hex.set_to(pos), "placement");

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
            world.modify_with_desc(tile.entity, TileData::occupied_by.set_to(None), "entity removed");
        }
    }
}



pub fn path(world : &WorldView, mover : Entity, from: AxialCoord, to: AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mover = world.character(mover);
    let accessor = TileAccessor::new(world);
    astar(&from, |c| c.neighbors_vec().into_iter().map(|c| (c, r32(move_cost_to_f32(&accessor, &mover, &c)))), |c| c.distance(&to), |c| *c == to)
}

pub fn path_any_v(world : &WorldView, mover : Entity, from: AxialCoord, to: &Vec<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mut set = HashSet::new();
    set.extend(to.iter());
    path_any(world, mover, from, &set, heuristical_center)
}

pub fn path_any(world : &WorldView, mover : Entity, from: AxialCoord, to: &HashSet<AxialCoord>, heuristical_center : AxialCoord) -> Option<(Vec<AxialCoord>, R32)> {
    let mover = world.character(mover);
    let accessor = TileAccessor::new(world);
    astar(&from, |c| c.neighbors_vec().into_iter().map(|c| (c, r32(move_cost_to_f32(&accessor, &mover, &c)))), |c| c.distance(&heuristical_center), |c| to.contains(c))
}


pub fn move_cost_to(accessor : &TileAccessor, mover : Entity, to : &AxialCoord) -> Sext {
    if let Some(tile) = accessor.tile_opt(*to) {
        if tile.occupied_by.is_none() {
            accessor.vegetation(&tile).move_cost + accessor.terrain(&tile).move_cost
        } else {
            Sext::of(100000)
        }
    } else {
        Sext::of(1000000)
    }
}

pub fn move_cost_to_f32(accessor : &TileAccessor , mover : &Character, to: &AxialCoord) -> f32 {
    if let Some(tile) = accessor.tile_opt(*to) {
        if tile.occupied_by.is_none() {
            (accessor.vegetation(&tile).move_cost + accessor.terrain(&tile).move_cost).as_f32()
        } else {
            10000000.0
        }
    } else {
        10000000.0
    }
}

pub fn default_movement_type(world: &WorldView, mover : Entity) -> Option<MovementTypeRef> {
    world.data_opt::<MovementData>(mover).and_then(|md| md.active_movement_type)
        .or(movement_types_available(world, mover).first().cloned())
}

pub fn movement_types_available(world: &WorldView, mover : Entity) -> Vec<MovementTypeRef> {
    if let Some(move_data) = world.data_opt::<MovementData>(mover) {
        move_data.movement_types.map(|mt| MovementTypeRef::of_movement_and_mover(*mt, mover))
    } else {
        warn!("attempted to get the movement types available for an entity with no movement data. This may be fine, but worth mentioning");
        Vec::new()
    }
}


pub trait ResolveMovementType {
    fn resolve<'a, 'b>(&'a self, world : &'b WorldView) -> Option<&'b MovementType>;
}

impl ResolveMovementType for MovementTypeRef {
    fn resolve<'a, 'b>(&'a self, world : &'b WorldView) -> Option<&'b MovementType> {
        // check that the mover still has access to this kind of movement
        if movement::movement_types_available(world, self.mover).contains(self) {
            // if they do, attempt to get the actual movement type data
            world.data_opt::<MovementType>(self.movement)
        } else {
            None
        }
    }
}