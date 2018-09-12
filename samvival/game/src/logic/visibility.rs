use common::prelude::*;
use prelude::*;

use common::hex::CubeCoord;
use game::events::CoreEvent;

use data::entities::Visibility;
use data::entities::VisibilityData;
use data::entities::character::CharacterData;
use data::entities::common_entities::*;
use game::EntityData;
use data::entities::character::ObserverData;
use data::entities::character::AllegianceData;
use data::entities::time::TimeOfDay;
use data::entities::faction::FactionData;
use data::entities::tile::TileStore;
use std::collections::HashMap;
use std::collections::HashSet;
use entities::TileAccessor;


//pub struct

#[derive(Clone,Debug,Default,Serialize, Deserialize, Fields)]
pub struct VisibilityComputor {

}
impl EntityData for VisibilityComputor {}


impl VisibilityComputor {
    pub fn new() -> VisibilityComputor {
        VisibilityComputor {

        }
    }


    pub fn register(world: &mut World) {
        let world_view = world.view();
        for (faction,_) in world_view.entities_with_data::<FactionData>() {
            let has_vis = world.world_data_opt::<VisibilityData>().and_then(|vd| vd.visibility_by_faction.get(&faction)).map(|v| !v.revealed_hexes.is_empty()).unwrap_or(false);
            if ! has_vis {
                let vis = {
                    let computor = world.world_data_mut::<VisibilityComputor>();
                    computor.recompute_visible_hexes(world_view, *faction, None)
                };
                world.modify_world(VisibilityData::visibility_by_faction.set_key_to(*faction, Visibility { revealed_hexes : vis.clone(), visible_hexes : vis }), None);
                world.add_event(CoreEvent::Recomputation);
            }
        }

        world.add_callback(|world,event_w| {
            match event_w.event {
                GameEvent::WorldStart => {
                },
                GameEvent::Move { character, from, to, .. } => {
                    let world_view = world.view();
                    let faction = world_view.character(character).allegiance.faction;
                    let vis = {
                        let computor = world.world_data_mut::<VisibilityComputor>();
                        computor.recompute_visible_hexes(world_view, faction, None)
                    };

                    let mut modified = false;
                    let view = world.view();
                    if let Some(old_visibility) = view.world_data::<VisibilityData>().visibility_by_faction.get(&faction) {
                        let newly_revealed_hexes : HashSet<AxialCoord> = vis.iter().filter(|h| !old_visibility.revealed_hexes.contains(h)).cloned().collect();
                        let newly_visible_hexes : HashSet<AxialCoord> = vis.difference(&old_visibility.visible_hexes).cloned().collect();
                        let no_longer_visible_hexes : HashSet<AxialCoord> = old_visibility.visible_hexes.difference(&vis).cloned().collect();

                        if !newly_revealed_hexes.is_empty() || !newly_visible_hexes.is_empty() {
                            let add_vis = Visibility { visible_hexes : newly_visible_hexes, revealed_hexes : newly_revealed_hexes };
                            world.modify_world(VisibilityData::visibility_by_faction.add_to_key(faction, add_vis), None);
                            modified = true;
                        }
                        if !no_longer_visible_hexes.is_empty() {
                            let sub_vis = Visibility { visible_hexes : no_longer_visible_hexes, revealed_hexes : HashSet::new() };
                            world.modify_world(VisibilityData::visibility_by_faction.sub_from_key(faction, sub_vis), None);
                            modified = true;
                        }
                    } else {
                        let mut visibility = Visibility::new();
                        visibility.visible_hexes = vis.clone();
                        visibility.revealed_hexes = vis;
                        world.modify_world(VisibilityData::visibility_by_faction.set_key_to(faction, visibility), None);
                        modified = true;
                    }
                    if modified {
                        world.add_event(CoreEvent::Recomputation);
                    }
                },
                _ => ()
            }
        });
    }

    pub fn recompute_visible_hexes(&self, world : &WorldView, faction : Entity, moved_entities_ : Option<Vec<Entity>>) -> HashSet<AxialCoord> {
//        let current_visibility = world.world_data::<VisibilityData>().visibility_by_faction.get(faction).unwrap_or_else(|| Visibility::new());

        // for the moment, recompute from scratch every time

        let mut visible_hexes = HashSet::new();

        for (ent,cdata) in world.entities_with_data::<ObserverData>() {
            let allegiance = world.data::<AllegianceData>(*ent);
            if allegiance.faction == faction {
                self.compute_observer_visibility(world, *ent, &mut visible_hexes);
            }
        }

        visible_hexes
    }


    fn compute_observer_visibility(&self, world : &WorldView, ent : Entity, visible_hexes : &mut HashSet<AxialCoord>) {
        let center : AxialCoord = world.data::<PositionData>(ent).hex;
        let observer = world.data::<ObserverData>(ent);
        let center_cube : CubeCoord = center.as_cube_coord();
        let center_f = v3(center_cube.x as f32 + 1e-6, center_cube.y as f32 + 2e-6, center_cube.z as f32 - 3e-6);

        visible_hexes.insert(center);

        let accessor = TileAccessor::new(world);

        let start_elevation = accessor.terrain_at(center).elevation;

        let max_r = observer.vision_range_at_time(TimeOfDay::Daylight) + 1;

        for edge in CubeCoord::ring(center_cube, max_r as u32) {
            let edge_f = v3(edge.x as f32 + 1e-6, edge.y as f32 + 2e-6, edge.z as f32 - 3e-6);
            let delta = edge_f - center_f;
            let mut visibility_remaining = max_r;

            let mut max_intervening_elevation = 0;

            let hex_dist = center_cube.distance(&edge);
            for i in 1 .. hex_dist {
                let pcnt = (i as f32) / (hex_dist as f32);
                let point = center_f + delta * pcnt;
                let hex = CubeCoord::rounded(point.x, point.y, point.z).as_axial_coord();

                if let Some(tile) = accessor.tile_opt(hex) {
                    let terrain = accessor.terrain(&tile);
                    let elevation = terrain.elevation;

                    if max_intervening_elevation > start_elevation && elevation < max_intervening_elevation {
                        visibility_remaining = -1;
                    } else if elevation >= start_elevation {
                        let obstruction = terrain.cover + accessor.vegetation(&tile).cover;
                        visibility_remaining -= 1 + obstruction as i32;
                    }
                    max_intervening_elevation = max_intervening_elevation.max(elevation);

                    if visibility_remaining >= 0 {
                        visible_hexes.insert(hex);
                    } else {
                        break;
                    }
                }
            }
        }
    }
}


pub fn faction_visibility_for_character (view : &WorldView, character : Entity) -> &Visibility {
    let faction = view.data::<AllegianceData>(character).faction;
    let vd = view.world_data::<VisibilityData>();
    vd.visibility_for(faction)
}