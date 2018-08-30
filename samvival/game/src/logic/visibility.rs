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


//pub struct

#[derive(Clone,Debug,Default,Serialize, Deserialize, PrintFields)]
pub struct VisibilityComputor {

}
impl EntityData for VisibilityComputor {}


impl VisibilityComputor {
    pub fn new() -> VisibilityComputor {
        VisibilityComputor {

        }
    }


    pub fn register(world: &mut World) {
        world.register::<VisibilityComputor>();
        world.register::<VisibilityData>();

        let world_view = world.view();
        for (faction,_) in world_view.entities_with_data::<FactionData>() {
            let vis = {
                let computor = world.world_data_mut::<VisibilityComputor>();
                computor.recompute_visibility(world_view, *faction, None)
            };
            world.modify_world(VisibilityData::visibility_by_faction.set_key_to(*faction, vis), None);
            world.add_event(CoreEvent::Recomputation);
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
                        computor.recompute_visibility(world_view, faction, None)
                    };

                    world.modify_world(VisibilityData::visibility_by_faction.set_key_to(faction, vis), None);
                    world.add_event(CoreEvent::Recomputation);
                },
                _ => ()
            }
        });
    }

    pub fn recompute_visibility(&self, world : &WorldView, faction : Entity, moved_entities_ : Option<Vec<Entity>>) -> Visibility {
//        let current_visibility = world.world_data::<VisibilityData>().visibility_by_faction.get(faction).unwrap_or_else(|| Visibility::new());

        // for the moment, recompute from scratch every time

        let mut visibility = Visibility::new();
        if let Some(old_visibility) = world.world_data::<VisibilityData>().visibility_by_faction.get(&faction) {
            visibility.revealed_hexes = old_visibility.revealed_hexes.clone();
        }

        for (ent,cdata) in world.entities_with_data::<ObserverData>() {
            let allegiance = world.data::<AllegianceData>(*ent);
            if allegiance.faction == faction {
                self.compute_observer_visibility(world, *ent, &mut visibility);
            }
        }

        visibility
    }


    fn compute_observer_visibility(&self, world : &WorldView, ent : Entity, visibility : &mut Visibility) {
        let center : AxialCoord = world.data::<PositionData>(ent).hex;
        let observer = world.data::<ObserverData>(ent);
        let center_cube : CubeCoord = center.as_cube_coord();
        let center_f = v3(center_cube.x as f32 + 1e-6, center_cube.y as f32 + 2e-6, center_cube.z as f32 - 3e-6);

        visibility.visible_hexes.insert(center);

        let start_elevation = world.tile_opt(center).map(|t| t.elevation).unwrap_or(0);

        let max_r = observer.vision_range_at_time(TimeOfDay::Daylight) + 1;

        for edge in CubeCoord::ring(center_cube, max_r as u32) {
            let edge_f = v3(edge.x as f32 + 1e-6, edge.y as f32 + 2e-6, edge.z as f32 - 3e-6);
            let delta = edge_f - center_f;
            let mut visibility_remaining = max_r;

            let mut max_intervening_elevation = 0;

            let hex_dist = center_cube.distance(&edge);
            visibility.visible_hexes.insert(center);
            for i in 1 .. hex_dist {
                let pcnt = (i as f32) / (hex_dist as f32);
                let point = center_f + delta * pcnt;
                let hex = CubeCoord::rounded(point.x, point.y, point.z).as_axial_coord();

                if let Some(tile) = world.tile_opt(hex) {
                    let elevation = tile.elevation;

                    if max_intervening_elevation > start_elevation && elevation < max_intervening_elevation {
                        visibility_remaining = -1;
                    } else if elevation >= start_elevation {
                        let obstruction = tile.cover;
                        visibility_remaining -= 1 + obstruction as i32;
                    }
                    max_intervening_elevation = max_intervening_elevation.max(elevation);

                    if visibility_remaining >= 0 {
                        visibility.visible_hexes.insert(hex);
                        visibility.revealed_hexes.insert(hex);
                    } else {
                        break;
                    }
                }
            }
        }

//        for r in 1 .. max_r {
//            for hex in CubeCoord::ring(center_cube, r as u32) {
//                visibility.visible_hexes.insert(hex.as_axial_coord());
//            }
//        }
    }
}